from __future__ import annotations

from abc import ABC, abstractmethod
from pathlib import Path

from filesdb.storage import Record


class FileEngine(ABC):
    def __init__(self, path: Path | str):
        self.path = Path(path)

    @abstractmethod
    def insert(self, key: int, value: bytes) -> None:
        raise NotImplementedError

    @abstractmethod
    def get(self, key: int) -> bytes | None:
        raise NotImplementedError

    @abstractmethod
    def update(self, key: int, value: bytes) -> bool:
        raise NotImplementedError

    @abstractmethod
    def delete(self, key: int) -> bool:
        raise NotImplementedError

    @abstractmethod
    def scan(self) -> list[Record]:
        raise NotImplementedError

    @abstractmethod
    def range_scan(self, start: int, end: int) -> list[Record]:
        raise NotImplementedError

    @abstractmethod
    def stats(self) -> dict[str, int]:
        raise NotImplementedError

    @abstractmethod
    def close(self) -> None:
        raise NotImplementedError

