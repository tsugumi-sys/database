from __future__ import annotations

from pathlib import Path

from filesdb.engines.base import FileEngine
from filesdb.storage import NO_PAGE, Pager, Record, STATUS_ACTIVE, STATUS_DELETED


class HeapFile(FileEngine):
    def __init__(self, path: Path | str):
        super().__init__(path)
        self.pager = Pager(self.path)

    def close(self) -> None:
        self.pager.close()

    def insert(self, key: int, value: bytes) -> None:
        record = Record(key=key, value=value)
        page_count = self.pager.page_count()
        if page_count == 0:
            page = self.pager.append_page()
            page.insert_record(record)
            self.pager.write_page(page)
            return

        page = self.pager.read_page(page_count - 1)
        if page.free_slots == 0:
            page = self.pager.append_page()
        page.insert_record(record)
        self.pager.write_page(page)

    def _locate(self, key: int) -> tuple[int, int] | None:
        for page in self.pager.iter_pages():
            for slot_index, slot in enumerate(page.slots):
                if slot.status == STATUS_ACTIVE and slot.key == key:
                    return page.page_id, slot_index
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
            "overflow_pages": 0,
            "page_splits": 0,
            "bucket_count": 0,
            "max_overflow_chain": 0,
            "last_overflow_page_id": NO_PAGE,
        }
