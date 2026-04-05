# Simple Data File Structures

## Goal

Build three very small storage engines in the same codebase and compare their performance under the same workload:

- `primary file`
- `heap file`
- `hash file`

The point is not to build a full DBMS. The point is to isolate the storage layout and access path differences so read/write behavior is easy to understand and benchmark.

## Scope

Keep the first version intentionally small:

- fixed-size records
- page-based storage
- single-process only
- append/update/delete support
- no transactions
- no WAL
- no concurrency control
- no buffer pool beyond a tiny in-memory page cache if needed

This keeps implementation cost low and makes the benchmark results easier to interpret.

## Shared Model

All three file organizations should use the same logical record and page format. Only the placement and lookup strategy should differ.

### Record

Use a fixed-size record so page math stays simple.

Example:

```text
Record {
  key: uint64
  deleted: bool
  value: bytes[119]
}
```

Suggested serialized size:

- `8 bytes` for key
- `1 byte` for deleted flag
- `119 bytes` for payload
- total record size: `128 bytes`

Why fixed-size first:

- easy slot addressing
- easy page occupancy math
- easier fair comparison across file types

Here, "round size" means choosing field widths so every record occupies an easy-to-work-with fixed width on disk. `128 bytes` is convenient because slot offsets become simple:

```text
slot_offset = page_header_size + slot_index * 128
```

If the field sizes did not add up cleanly, you could also add unused padding bytes to reach the chosen fixed width.

### Page

All data files should be page-oriented.

Suggested page size:

- `4096 bytes`

Suggested page layout:

```text
PageHeader {
  page_id: uint32
  record_count: uint16
  free_slots: uint16
}

slots...
```

With `128-byte` records and a small header, each page will hold roughly `31` records.

Concrete example:

```text
Page 0 (4096 bytes)

+-------------------------------+  offset 0
| PageHeader                    |
| - page_id = 0                 |
| - record_count = 3            |
| - free_slots = 28             |
|                               |
| size = 8 bytes                |
+-------------------------------+  offset 8
| Record slot 0                 |
| - key = 1001                  |
| - deleted = 0                 |
| - value = "Alice..."          |
|                               |
| size = 128 bytes              |
+-------------------------------+  offset 136
| Record slot 1                 |
| - key = 1002                  |
| - deleted = 0                 |
| - value = "Bob..."            |
|                               |
| size = 128 bytes              |
+-------------------------------+  offset 264
| Record slot 2                 |
| - key = 1003                  |
| - deleted = 0                 |
| - value = "Carol..."          |
|                               |
| size = 128 bytes              |
+-------------------------------+  offset 392
| Record slot 3                 |
| - empty                       |
|                               |
| size = 128 bytes              |
+-------------------------------+
| ...                           |
+-------------------------------+
| Record slot 30                |
| - empty                       |
|                               |
| size = 128 bytes              |
+-------------------------------+  offset 3976
| Unused tail space             |
| - 120 bytes                   |
+-------------------------------+  offset 4096
```

The offset of slot `n` is:

```text
slot_offset = 8 + n * 128
```

So:

- slot `0` starts at offset `8`
- slot `1` starts at offset `136`
- slot `2` starts at offset `264`

A single record slot looks like this:

```text
Record slot (128 bytes)

+---------------------------+  offset +0
| key                       | 8 bytes
+---------------------------+  offset +8
| deleted                   | 1 byte
+---------------------------+  offset +9
| value                     | 119 bytes
+---------------------------+  offset +128
```

### Common Operations

Every file type should expose the same interface:

```text
insert(record) -> location
get(key) -> record | None
update(key, new_value) -> bool
delete(key) -> bool
scan() -> iterator[record]
stats() -> StorageStats
```

This common interface is what makes the benchmark harness straightforward.

## File Types

## 1. Heap File

### Idea

Records are stored wherever space is available. No ordering by key.

### Layout

- one data file
- pages appended as needed
- free space tracked in memory or a tiny free-page list

### Access Path

- `insert`: append to the last page if it has room, else append a new page
- `get`: full scan unless you add a separate index later
- `update`: full scan to find matching key, then rewrite slot
- `delete`: full scan, then mark deleted

### Why It Matters

Heap file is the simplest baseline:

- best for cheap inserts
- poor point lookup without an index
- full scan performance is easy to reason about

In this project, the heap file uses tail-page append semantics instead of reusing holes in older pages. That keeps `insert` cheap and the implementation small, at the cost of weaker space reuse after deletes.

### Expected Performance

- inserts: good
- point reads by key: poor
- full scan: straightforward, often good

## 2. Primary File

### Idea

Store records physically ordered by primary key.

For the simple version, this is a sorted file on disk. You can binary search pages by page key range, then binary search within a page.

### Layout

- one sorted data file
- each page stores records sorted by key
- page header may store `min_key` and `max_key`

Suggested page header extension:

```text
PageHeader {
  page_id: uint32
  record_count: uint16
  free_slots: uint16
  min_key: uint64
  max_key: uint64
}
```

### Access Path

- `get`: binary search across pages, then within page
- `scan`: sequential page scan
- `insert`: find target page, insert in sorted order
- if page full: split or overflow handling
- `update`: locate by key, rewrite value in place
- `delete`: mark deleted or compact page

### Simplest Practical Insert Strategy

Use page split instead of overflow chains for the first version:

1. find target page by key range
2. insert in sorted order if there is room
3. if full, split page into two sorted pages
4. rewrite affected pages

This is not a B-tree. It is just a sorted file with page splits. That is enough for a basic comparison.

### Why It Matters

Primary file shows the tradeoff:

- better point lookup than heap
- good range scan behavior
- insert cost higher because order must be maintained

### Expected Performance

- inserts: moderate to poor as data grows
- point reads by key: good
- range scan: very good

## 3. Hash File

### Idea

Use a hash function on the key to place records into buckets.

For the first version, use static hashing. Avoid extensible or linear hashing initially.

### Layout

- one metadata file
- one bucket file or one data file logically divided into buckets
- each bucket points to one primary page
- overflow pages allowed for collisions

Metadata example:

```text
HashMeta {
  bucket_count: uint32
  page_size: uint32
}
```

### Access Path

- `bucket = hash(key) % bucket_count`
- `insert`: place into bucket page, use overflow page if full
- `get`: go directly to bucket, then scan bucket chain
- `update`: same as `get`, then rewrite
- `delete`: same as `get`, then mark deleted
- `scan`: scan all buckets and overflow pages

### Why It Matters

Hash file is the classic structure for fast point lookup:

- direct bucket access for equality search
- weak range scan behavior
- performance depends on load factor and overflow growth

### Expected Performance

- inserts: good until overflow chains get long
- point reads by key: very good
- range scan: poor

## On-Disk Files

Suggested directory layout:

```text
data/
  heap.dat
  primary.dat
  hash.dat
  hash.meta
```

Suggested source layout:

```text
main.py
src/
  common/
    record.py
    page.py
    codec.py
    stats.py
  engines/
    heap_file.py
    primary_file.py
    hash_file.py
  benchmark/
    workloads.py
    runner.py
```

If you want to stay minimal, this can also live in a few top-level Python files at first.

## Serialization

Use Python standard library only for the first pass:

- `struct` for binary packing/unpacking
- `pathlib` for files
- `time.perf_counter()` for timing
- `random` for workload generation

Suggested binary encoding:

- fixed-width page header
- fixed-width record slots
- write entire pages back to disk

This is simpler and more comparable than variable-length encoding.

## Benchmark Design

To compare fairly, use the same dataset, same record shape, and same workload mix.

### Dataset Sizes

Start with a few scales:

- `10_000`
- `100_000`
- `1_000_000` if runtime stays acceptable

### Workloads

Measure these separately:

1. Bulk insert of new keys
2. Random point lookup by existing key
3. Random update by existing key
4. Random delete by existing key
5. Full scan
6. Range scan

Range scan is especially important because:

- primary file should perform well
- hash file should perform poorly
- heap file will likely need full scanning

### Metrics

Collect at least:

- wall-clock time
- operations per second
- pages read
- pages written
- file size
- overflow page count for hash file
- split count for primary file
- deleted-slot count

Do not only compare elapsed time. Structural stats will explain the results.

### Benchmark Rules

To keep numbers meaningful:

- use a fresh file for each run
- warm-up once if needed
- repeat each benchmark multiple times
- report median or average
- use the same random seed across engines

## Suggested Interface

Python protocol or base class:

```python
class FileEngine:
    def insert(self, key: int, value: bytes) -> None: ...
    def get(self, key: int) -> bytes | None: ...
    def update(self, key: int, value: bytes) -> bool: ...
    def delete(self, key: int) -> bool: ...
    def scan(self): ...
    def range_scan(self, start: int, end: int): ...
    def stats(self) -> dict: ...
```

`range_scan` is optional for heap and hash in terms of optimization, but it should exist so the benchmark can call the same method on all engines.

## Implementation Order

Build in this order:

1. Shared record/page serializer
2. Heap file
3. Benchmark harness
4. Primary file
5. Hash file
6. Comparative benchmark report

Why this order:

- heap file validates the page format with the least logic
- benchmark harness gives immediate feedback early
- primary and hash can then reuse the same storage primitives

## Recommended Simplifications

To keep the project realistic for a first version:

- fixed-size keys and values only
- single file per engine where possible
- mark deletes instead of page compaction at first
- no crash recovery
- no background reorganization

Avoid adding indexes on top of heap file in the first comparison. That would change the question from "file organization" to "file organization plus index".

## What We Expect to Learn

The benchmark should reveal these tradeoffs:

- heap file favors simple writes and full scans, but equality lookups are expensive
- primary file improves ordered access and point search, but inserts become more expensive
- hash file gives the best equality search, but range access is weak and overflow hurts over time

That gives you a clean educational comparison between the three classic organizations.

## Next Step

After this document, the best next implementation step is:

1. create a shared page/record codec
2. implement `HeapFile`
3. add a tiny benchmark runner that compares insert, get, and scan

That will give you a working baseline before adding the more complex `PrimaryFile` and `HashFile`.
