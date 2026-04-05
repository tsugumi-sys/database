from __future__ import annotations

import argparse
from pathlib import Path

from filesdb.benchmark import format_benchmark_report, run_benchmarks, smoke_check


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Simple data file structure playground")
    subparsers = parser.add_subparsers(dest="command", required=True)

    benchmark_parser = subparsers.add_parser("benchmark", help="Run a simple benchmark across all engines")
    benchmark_parser.add_argument("--records", type=int, default=5000, help="Number of records to insert")
    benchmark_parser.add_argument("--lookups", type=int, default=2000, help="Number of random point reads/updates")
    benchmark_parser.add_argument("--seed", type=int, default=7, help="Random seed")
    benchmark_parser.add_argument("--data-dir", type=Path, default=Path("data"), help="Directory for benchmark files")

    verify_parser = subparsers.add_parser("verify", help="Run a small functional smoke check")
    verify_parser.add_argument("--data-dir", type=Path, default=Path("tmp/smoke"), help="Directory for temporary files")

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    if args.command == "benchmark":
        report = run_benchmarks(
            base_dir=args.data_dir,
            record_count=args.records,
            lookup_count=args.lookups,
            seed=args.seed,
        )
        print(format_benchmark_report(report))
        return

    if args.command == "verify":
        failures = smoke_check(args.data_dir)
        if failures:
            print("verification failed")
            for failure in failures:
                print(f"- {failure}")
            raise SystemExit(1)
        print("verification passed")
        return

    raise SystemExit(f"unknown command: {args.command}")


if __name__ == "__main__":
    main()
