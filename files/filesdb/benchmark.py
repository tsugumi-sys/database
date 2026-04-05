from __future__ import annotations

import random
import shutil
import time
from dataclasses import dataclass
from pathlib import Path

from filesdb.engines import HashFile, HeapFile, PrimaryFile


@dataclass(slots=True)
class BenchmarkResult:
    engine: str
    operation: str
    seconds: float
    ops: int

    @property
    def ops_per_second(self) -> float:
        if self.seconds == 0:
            return float("inf")
        return self.ops / self.seconds


def _payload_for_key(key: int) -> bytes:
    return f"value-{key:08d}".encode("utf-8")


def _measure(operation: str, func, ops: int, engine: str) -> BenchmarkResult:
    start = time.perf_counter()
    func()
    elapsed = time.perf_counter() - start
    return BenchmarkResult(engine=engine, operation=operation, seconds=elapsed, ops=ops)


def _engine_factories(base_dir: Path):
    return {
        "heap": lambda: HeapFile(base_dir / "heap.dat"),
        "primary": lambda: PrimaryFile(base_dir / "primary.dat"),
        "hash": lambda: HashFile(base_dir / "hash.dat"),
    }


def run_benchmarks(
    base_dir: Path, record_count: int = 5000, lookup_count: int = 2000, seed: int = 7
) -> dict[str, object]:
    if base_dir.exists():
        shutil.rmtree(base_dir)
    base_dir.mkdir(parents=True, exist_ok=True)

    rng = random.Random(seed)
    keys = list(range(record_count))
    insert_order = keys[:]
    rng.shuffle(insert_order)
    lookup_keys = rng.sample(keys, k=min(lookup_count, record_count))
    update_keys = lookup_keys[:]
    delete_keys = lookup_keys[: max(1, len(lookup_keys) // 4)]
    range_start = record_count // 3
    range_end = range_start + max(1, record_count // 10)

    benchmark_rows: list[BenchmarkResult] = []
    engine_stats: dict[str, dict[str, int]] = {}

    for engine_name, factory in _engine_factories(base_dir).items():
        engine = factory()
        try:
            benchmark_rows.append(
                _measure(
                    operation="bulk_insert",
                    engine=engine_name,
                    ops=len(insert_order),
                    func=lambda: [
                        engine.insert(key, _payload_for_key(key))
                        for key in insert_order
                    ],
                )
            )
            benchmark_rows.append(
                _measure(
                    operation="random_get",
                    engine=engine_name,
                    ops=len(lookup_keys),
                    func=lambda: [engine.get(key) for key in lookup_keys],
                )
            )
            benchmark_rows.append(
                _measure(
                    operation="random_update",
                    engine=engine_name,
                    ops=len(update_keys),
                    func=lambda: [
                        engine.update(key, f"updated-{key:08d}".encode("utf-8"))
                        for key in update_keys
                    ],
                )
            )
            benchmark_rows.append(
                _measure(
                    operation="range_scan",
                    engine=engine_name,
                    ops=range_end - range_start + 1,
                    func=lambda: engine.range_scan(range_start, range_end),
                )
            )
            benchmark_rows.append(
                _measure(
                    operation="full_scan",
                    engine=engine_name,
                    ops=record_count,
                    func=engine.scan,
                )
            )
            benchmark_rows.append(
                _measure(
                    operation="random_delete",
                    engine=engine_name,
                    ops=len(delete_keys),
                    func=lambda: [engine.delete(key) for key in delete_keys],
                )
            )
            engine_stats[engine_name] = engine.stats()
        finally:
            engine.close()

    return {
        "results": benchmark_rows,
        "stats": engine_stats,
        "record_count": record_count,
        "range_start": range_start,
        "range_end": range_end,
    }


def format_benchmark_report(report: dict[str, object]) -> str:
    lines = []
    lines.append(f"records: {report['record_count']}")
    lines.append(f"range_scan: [{report['range_start']}, {report['range_end']}]")
    lines.append("")
    lines.append("timings")
    for row in report["results"]:
        assert isinstance(row, BenchmarkResult)
        lines.append(
            f"- {row.engine:7s} {row.operation:14s} "
            f"{row.seconds:9.6f}s  {row.ops_per_second:10.2f} ops/s"
        )
    lines.append("")
    lines.append("engine stats")
    for engine_name, stats in report["stats"].items():
        lines.append(f"- {engine_name}")
        for key, value in stats.items():
            lines.append(f"  {key}: {value}")
    return "\n".join(lines)


def smoke_check(base_dir: Path) -> list[str]:
    if base_dir.exists():
        shutil.rmtree(base_dir)
    base_dir.mkdir(parents=True, exist_ok=True)

    failures: list[str] = []

    for engine_name, factory in _engine_factories(base_dir).items():
        engine = factory()
        try:
            for key in [10, 3, 7, 20, 15]:
                engine.insert(key, _payload_for_key(key))

            if engine.get(7) != _payload_for_key(7):
                failures.append(f"{engine_name}: get failed")

            if not engine.update(7, b"changed"):
                failures.append(f"{engine_name}: update returned false")

            if engine.get(7) != b"changed":
                failures.append(f"{engine_name}: updated value mismatch")

            scan_keys = [record.key for record in engine.scan()]
            if set(scan_keys) != {3, 7, 10, 15, 20}:
                failures.append(f"{engine_name}: scan keys mismatch {scan_keys}")

            range_keys = [record.key for record in engine.range_scan(7, 15)]
            if set(range_keys) != {7, 10, 15}:
                failures.append(f"{engine_name}: range scan mismatch {range_keys}")

            if not engine.delete(10):
                failures.append(f"{engine_name}: delete returned false")

            if engine.get(10) is not None:
                failures.append(f"{engine_name}: deleted key still visible")
        finally:
            engine.close()

    return failures
