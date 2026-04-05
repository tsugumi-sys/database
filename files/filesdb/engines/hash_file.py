from __future__ import annotations

from pathlib import Path

from filesdb.engines.base import FileEngine
from filesdb.storage import NO_PAGE, PAGE_SIZE, Pager, Record, STATUS_ACTIVE, STATUS_DELETED


class HashFile(FileEngine):
    def __init__(self, path: Path | str, bucket_count: int = 128):
        super().__init__(path)
        self.bucket_count = bucket_count
        self.pager = Pager(self.path)
        self._ensure_buckets()

    def close(self) -> None:
        self.pager.close()

    def _ensure_buckets(self) -> None:
        while self.pager.page_count() < self.bucket_count:
            self.pager.append_page()

    def _bucket_page_id(self, key: int) -> int:
        return key % self.bucket_count

    def _page_chain(self, bucket_page_id: int) -> list[int]:
        page_ids = []
        current_page_id = bucket_page_id
        while current_page_id != NO_PAGE:
            page_ids.append(current_page_id)
            page = self.pager.read_page(current_page_id)
            current_page_id = page.overflow_page_id
        return page_ids

    def insert(self, key: int, value: bytes) -> None:
        record = Record(key=key, value=value)
        current_page_id = self._bucket_page_id(key)
        while True:
            page = self.pager.read_page(current_page_id)
            for slot in page.slots:
                if slot.status == STATUS_ACTIVE and slot.key == key:
                    raise ValueError(f"duplicate key {key}")
            if page.free_slots > 0:
                page.insert_record(record)
                self.pager.write_page(page)
                return
            if page.overflow_page_id == NO_PAGE:
                overflow_page = self.pager.append_page()
                page.overflow_page_id = overflow_page.page_id
                self.pager.write_page(page)
                overflow_page.insert_record(record)
                self.pager.write_page(overflow_page)
                return
            current_page_id = page.overflow_page_id

    def _locate(self, key: int) -> tuple[int, int] | None:
        for page_id in self._page_chain(self._bucket_page_id(key)):
            page = self.pager.read_page(page_id)
            for slot_index, slot in enumerate(page.slots):
                if slot.status == STATUS_ACTIVE and slot.key == key:
                    return page_id, slot_index
        return None

    def get(self, key: int) -> bytes | None:
        location = self._locate(key)
        if location is None:
            return None
        page_id, slot_index = location
        page = self.pager.read_page(page_id)
        return page.slots[slot_index].value.rstrip(b"\x00")

    def update(self, key: int, value: bytes) -> bool:
        location = self._locate(key)
        if location is None:
            return False
        page_id, slot_index = location
        page = self.pager.read_page(page_id)
        page.slots[slot_index].value = Record(key=key, value=value).normalized_value()
        self.pager.write_page(page)
        return True

    def delete(self, key: int) -> bool:
        location = self._locate(key)
        if location is None:
            return False
        page_id, slot_index = location
        page = self.pager.read_page(page_id)
        page.slots[slot_index].status = STATUS_DELETED
        page.refresh_key_range()
        self.pager.write_page(page)
        return True

    def scan(self) -> list[Record]:
        records: list[Record] = []
        for page in self.pager.iter_pages():
            records.extend(page.active_records())
        return records

    def range_scan(self, start: int, end: int) -> list[Record]:
        return [record for record in self.scan() if start <= record.key <= end]

    def stats(self) -> dict[str, int]:
        deleted_slots = 0
        active_records = 0
        overflow_pages = max(0, self.pager.page_count() - self.bucket_count)
        max_chain = 0
        for bucket in range(self.bucket_count):
            chain_length = len(self._page_chain(bucket))
            if chain_length > max_chain:
                max_chain = chain_length
        for page in self.pager.iter_pages():
            for slot in page.slots:
                if slot.status == STATUS_DELETED:
                    deleted_slots += 1
                if slot.status == STATUS_ACTIVE:
                    active_records += 1
        return {
            "page_reads": self.pager.stats.page_reads,
            "page_writes": self.pager.stats.page_writes,
            "file_size_bytes": self.pager.file_size_bytes(),
            "page_count": self.pager.page_count(),
            "active_records": active_records,
            "deleted_slots": deleted_slots,
            "overflow_pages": overflow_pages,
            "page_splits": 0,
            "bucket_count": self.bucket_count,
            "max_overflow_chain": max_chain,
            "last_overflow_page_id": self.pager.page_count() - 1 if overflow_pages else NO_PAGE,
        }

