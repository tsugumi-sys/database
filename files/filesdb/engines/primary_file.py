from __future__ import annotations

from bisect import bisect_left
from pathlib import Path

from filesdb.engines.base import FileEngine
from filesdb.storage import NO_PAGE, Pager, Record, SLOTS_PER_PAGE


class PrimaryFile(FileEngine):
    def __init__(self, path: Path | str):
        super().__init__(path)
        self.pager = Pager(self.path)
        self.page_splits = 0
        self._cached_ranges: list[tuple[int, int, int]] | None = None

    def close(self) -> None:
        self.pager.close()

    def _all_records(self) -> list[Record]:
        records: list[Record] = []
        for page in self.pager.iter_pages():
            records.extend(page.active_records())
        records.sort(key=lambda record: record.key)
        return records

    def _rewrite_from_records(self, records: list[Record]) -> None:
        page_count_before = self.pager.page_count()
        self.pager.reset_file()
        needed_pages = (len(records) + SLOTS_PER_PAGE - 1) // SLOTS_PER_PAGE
        if page_count_before and needed_pages > page_count_before:
            self.page_splits += needed_pages - page_count_before
        for page_id in range(needed_pages):
            page = self.pager.append_page()
            chunk = records[page_id * SLOTS_PER_PAGE : (page_id + 1) * SLOTS_PER_PAGE]
            page.set_active_records(chunk)
            self.pager.write_page(page)
        self._cached_ranges = None

    def _page_ranges(self) -> list[tuple[int, int, int]]:
        if self._cached_ranges is None:
            ranges: list[tuple[int, int, int]] = []
            for page in self.pager.iter_pages():
                if page.record_count > 0:
                    ranges.append((page.min_key, page.max_key, page.page_id))
            self._cached_ranges = ranges
        return self._cached_ranges

    def _find_page_for_key(self, key: int) -> int | None:
        ranges = self._page_ranges()
        if not ranges:
            return None
        maxes = [entry[1] for entry in ranges]
        index = bisect_left(maxes, key)
        if index < len(ranges):
            min_key, max_key, page_id = ranges[index]
            if min_key <= key <= max_key:
                return page_id
        return None

    def insert(self, key: int, value: bytes) -> None:
        records = self._all_records()
        index = bisect_left([record.key for record in records], key)
        if index < len(records) and records[index].key == key:
            raise ValueError(f"duplicate key {key}")
        records.insert(index, Record(key=key, value=value))
        self._rewrite_from_records(records)

    def get(self, key: int) -> bytes | None:
        page_id = self._find_page_for_key(key)
        if page_id is None:
            return None
        page = self.pager.read_page(page_id)
        records = page.active_records()
        keys = [record.key for record in records]
        index = bisect_left(keys, key)
        if index < len(records) and records[index].key == key:
            return records[index].value
        return None

    def update(self, key: int, value: bytes) -> bool:
        page_id = self._find_page_for_key(key)
        if page_id is None:
            return False
        page = self.pager.read_page(page_id)
        records = page.active_records()
        keys = [record.key for record in records]
        index = bisect_left(keys, key)
        if index >= len(records) or records[index].key != key:
            return False
        records[index] = Record(key=key, value=value)
        page.set_active_records(records)
        self.pager.write_page(page)
        self._cached_ranges = None
        return True

    def delete(self, key: int) -> bool:
        records = self._all_records()
        keys = [record.key for record in records]
        index = bisect_left(keys, key)
        if index >= len(records) or records[index].key != key:
            return False
        del records[index]
        self._rewrite_from_records(records)
        return True

    def scan(self) -> list[Record]:
        return self._all_records()

    def range_scan(self, start: int, end: int) -> list[Record]:
        records: list[Record] = []
        for page in self.pager.iter_pages():
            if page.record_count == 0:
                continue
            if page.max_key < start or page.min_key > end:
                continue
            for record in page.active_records():
                if start <= record.key <= end:
                    records.append(record)
        return records

    def stats(self) -> dict[str, int]:
        records = self._all_records()
        return {
            "page_reads": self.pager.stats.page_reads,
            "page_writes": self.pager.stats.page_writes,
            "file_size_bytes": self.pager.file_size_bytes(),
            "page_count": self.pager.page_count(),
            "active_records": len(records),
            "deleted_slots": 0,
            "overflow_pages": 0,
            "page_splits": self.page_splits,
            "bucket_count": 0,
            "max_overflow_chain": 0,
            "last_overflow_page_id": NO_PAGE,
        }
