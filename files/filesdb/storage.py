from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct
from typing import Iterator

PAGE_SIZE = 4096
VALUE_SIZE = 119
RECORD_SIZE = 128
HEADER_SIZE = 32
SLOTS_PER_PAGE = (PAGE_SIZE - HEADER_SIZE) // RECORD_SIZE
NO_PAGE = 0xFFFFFFFF

STATUS_EMPTY = 0
STATUS_ACTIVE = 1
STATUS_DELETED = 2

HEADER_STRUCT = struct.Struct("<IHHQQII")
RECORD_STRUCT = struct.Struct(f"<BQ{VALUE_SIZE}s")


@dataclass(slots=True)
class Record:
    key: int
    value: bytes

    @classmethod
    def from_text(cls, key: int, value: str) -> "Record":
        return cls(key=key, value=value.encode("utf-8"))

    def normalized_value(self) -> bytes:
        if len(self.value) > VALUE_SIZE:
            raise ValueError(f"value exceeds {VALUE_SIZE} bytes")
        return self.value.ljust(VALUE_SIZE, b"\x00")

    def value_text(self) -> str:
        return self.value.rstrip(b"\x00").decode("utf-8", errors="replace")


@dataclass(slots=True)
class Slot:
    status: int = STATUS_EMPTY
    key: int = 0
    value: bytes = b""

    @classmethod
    def empty(cls) -> "Slot":
        return cls()

    @classmethod
    def active(cls, record: Record) -> "Slot":
        return cls(status=STATUS_ACTIVE, key=record.key, value=record.normalized_value())

    def is_active(self) -> bool:
        return self.status == STATUS_ACTIVE

    def is_empty(self) -> bool:
        return self.status == STATUS_EMPTY

    def is_deleted(self) -> bool:
        return self.status == STATUS_DELETED

    def to_record(self) -> Record:
        return Record(key=self.key, value=self.value.rstrip(b"\x00"))


@dataclass(slots=True)
class Page:
    page_id: int
    slots: list[Slot]
    overflow_page_id: int = NO_PAGE
    min_key: int = 0
    max_key: int = 0

    @classmethod
    def empty(cls, page_id: int) -> "Page":
        return cls(page_id=page_id, slots=[Slot.empty() for _ in range(SLOTS_PER_PAGE)])

    @property
    def record_count(self) -> int:
        return sum(1 for slot in self.slots if slot.is_active())

    @property
    def free_slots(self) -> int:
        return sum(1 for slot in self.slots if slot.is_empty() or slot.is_deleted())

    def first_free_slot(self) -> int | None:
        for index, slot in enumerate(self.slots):
            if slot.is_empty() or slot.is_deleted():
                return index
        return None

    def active_records(self) -> list[Record]:
        return [slot.to_record() for slot in self.slots if slot.is_active()]

    def set_active_records(self, records: list[Record]) -> None:
        if len(records) > SLOTS_PER_PAGE:
            raise ValueError("too many records for page")
        self.slots = [Slot.active(record) for record in records]
        self.slots.extend(Slot.empty() for _ in range(SLOTS_PER_PAGE - len(records)))
        self.refresh_key_range()

    def refresh_key_range(self) -> None:
        keys = [slot.key for slot in self.slots if slot.is_active()]
        if keys:
            self.min_key = min(keys)
            self.max_key = max(keys)
        else:
            self.min_key = 0
            self.max_key = 0

    def insert_record(self, record: Record) -> int:
        slot_index = self.first_free_slot()
        if slot_index is None:
            raise ValueError("page is full")
        self.slots[slot_index] = Slot.active(record)
        self.refresh_key_range()
        return slot_index

    def encode(self) -> bytes:
        self.refresh_key_range()
        header = HEADER_STRUCT.pack(
            self.page_id,
            self.record_count,
            self.free_slots,
            self.min_key,
            self.max_key,
            self.overflow_page_id,
            0,
        )
        body = bytearray()
        for slot in self.slots:
            body.extend(RECORD_STRUCT.pack(slot.status, slot.key, slot.value.ljust(VALUE_SIZE, b"\x00")))
        encoded = header + body
        return encoded.ljust(PAGE_SIZE, b"\x00")

    @classmethod
    def decode(cls, raw: bytes) -> "Page":
        if len(raw) != PAGE_SIZE:
            raise ValueError("invalid page size")
        header = raw[:HEADER_SIZE]
        page_id, _record_count, _free_slots, min_key, max_key, overflow_page_id, _reserved = HEADER_STRUCT.unpack(header)
        slots: list[Slot] = []
        offset = HEADER_SIZE
        for _ in range(SLOTS_PER_PAGE):
            status, key, value = RECORD_STRUCT.unpack(raw[offset : offset + RECORD_SIZE])
            slots.append(Slot(status=status, key=key, value=value))
            offset += RECORD_SIZE
        page = cls(page_id=page_id, slots=slots, overflow_page_id=overflow_page_id, min_key=min_key, max_key=max_key)
        page.refresh_key_range()
        return page


@dataclass(slots=True)
class IOStats:
    page_reads: int = 0
    page_writes: int = 0


class Pager:
    def __init__(self, path: Path | str):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        if not self.path.exists():
            self.path.touch()
        self._fh = self.path.open("r+b")
        self.stats = IOStats()

    def close(self) -> None:
        self._fh.close()

    def reset_file(self) -> None:
        self._fh.seek(0)
        self._fh.truncate(0)
        self._fh.flush()
        self.stats = IOStats()

    def page_count(self) -> int:
        self._fh.seek(0, 2)
        return self._fh.tell() // PAGE_SIZE

    def read_page(self, page_id: int) -> Page:
        self._fh.seek(page_id * PAGE_SIZE)
        raw = self._fh.read(PAGE_SIZE)
        if len(raw) != PAGE_SIZE:
            raise IndexError(f"page {page_id} does not exist")
        self.stats.page_reads += 1
        return Page.decode(raw)

    def write_page(self, page: Page) -> None:
        self._fh.seek(page.page_id * PAGE_SIZE)
        self._fh.write(page.encode())
        self._fh.flush()
        self.stats.page_writes += 1

    def append_page(self, page: Page | None = None) -> Page:
        page_id = self.page_count()
        page = page or Page.empty(page_id)
        page.page_id = page_id
        self.write_page(page)
        return page

    def iter_pages(self) -> Iterator[Page]:
        for page_id in range(self.page_count()):
            yield self.read_page(page_id)

    def file_size_bytes(self) -> int:
        self._fh.seek(0, 2)
        return self._fh.tell()

