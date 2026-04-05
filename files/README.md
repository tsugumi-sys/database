# files

Very small implementations of three classic data file organizations:

- `heap file`
- `primary file`
- `hash file`

The project stores fixed-size records in fixed-size pages so the storage layout is easy to inspect and benchmark.

See [architecture.md](/Users/akira.noda/dev/personal/database/files/architecture.md) for the storage model and design notes.

## Commands

Run a functional smoke check:

```bash
python3 main.py verify
```

Run a simple benchmark:

```bash
python3 main.py benchmark --records 5000 --lookups 2000
```

## Example Results

Example output from `python3 main.py benchmark --records 5000 --lookups 2000`:

Benchmark configuration:

- records: `5000`
- lookups: `2000`
- range scan: `[1666, 2166]`

### Timings

| Operation     |      Heap |    Primary |      Hash |
| :------------ | --------: | ---------: | --------: |
| Bulk insert   | 0.133740s | 23.765753s | 0.146181s |
| Random get    | 1.946388s |  0.052599s | 0.094988s |
| Random update | 1.965967s |  4.103032s | 0.117489s |
| Range scan    | 0.005230s |  0.002206s | 0.005035s |
| Full scan     | 0.003795s |  0.003833s | 0.004840s |
| Random delete | 0.501972s |  4.530982s | 0.029870s |

### Throughput

| Operation     |            Heap |         Primary |           Hash |
| :------------ | --------------: | --------------: | -------------: |
| Bulk insert   |   37385.86 ops/s |     210.39 ops/s |  34204.13 ops/s |
| Random get    |    1027.54 ops/s |   38023.30 ops/s |  21055.32 ops/s |
| Random update |    1017.31 ops/s |     487.44 ops/s |  17022.94 ops/s |
| Range scan    |   95801.12 ops/s |  227099.35 ops/s |  99505.12 ops/s |
| Full scan     | 1317537.64 ops/s | 1304447.28 ops/s | 1033013.26 ops/s |
| Random delete |     996.07 ops/s |     110.35 ops/s |  16738.95 ops/s |

### Storage Stats

| Metric             |   Heap | Primary |    Hash |
| :----------------- | -----: | ------: | ------: |
| Page reads         | 374188 |     146 |   25816 |
| Page writes        |   7662 |     292 |    7884 |
| File size bytes    | 663552 |  598016 | 1048576 |
| Page count         |    162 |     146 |     256 |
| Active records     |   4500 |    4500 |    4500 |
| Deleted slots      |    500 |       0 |     500 |
| Overflow pages     |      0 |       0 |     128 |
| Page splits        |      0 |     161 |       0 |
| Bucket count       |      0 |       0 |     128 |
| Max overflow chain |      0 |       0 |       2 |

## How To Read The Results

The broad pattern is correct:

- `heap` and `hash` have much cheaper inserts than `primary`
- `hash` is strongest for equality lookups, updates, and deletes
- `primary` is strongest for ordered access and range scans
- `full_scan` is similar across all three

One result looks surprising at first:

- `primary random_get` is faster than `hash random_get` in this benchmark

That happens because the current `hash file` is a static hash file with too few buckets for the dataset, so many lookups must traverse overflow pages. In the sample run:

- `bucket_count: 128`
- `overflow_pages: 128`
- `max_overflow_chain: 2`

So a hash lookup is often not just "hash once and read one page". It can become:

1. hash to bucket
2. read the primary bucket page
3. follow `overflow_page_id`
4. read one or more overflow pages

By contrast, the current `primary file` uses cached page key ranges and then binary-searches inside the target page, which makes `get` very efficient for this workload.

These are the meaningful structural takeaways:

- `heap file`: cheap append-style inserts, but expensive point lookup because `get`, `update`, and `delete` do full scans
- `primary file`: records are stored in key order, so `get` and `range_scan` benefit from page key ranges and binary search
- `hash file`: equality search goes directly to a bucket, so point operations are fast, but range queries are weak

## Important Caveat

This project is intentionally a small educational implementation, so some benchmark numbers are influenced by simplifications in the code.

The main caveat is the current `primary file` write path:

- `insert` rewrites the sorted file instead of performing true local page splits
- `delete` also rewrites the sorted file

Because of that, `primary` write-side timings in this project are worse than you would expect from a more realistic primary-file implementation.

So:

- trust the `primary` advantage for ordered reads and range scans
- trust the `hash` advantage for equality search
- treat the extreme `primary` insert/delete penalty as partly an artifact of the simplified implementation

The `hash` file also pays a space cost in the sample run:

- `bucket_count: 128`
- `overflow_pages: 128`
- `max_overflow_chain: 2`

That is why point lookups are still fast, but scans and range queries are not especially strong.
