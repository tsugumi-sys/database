# Step 9: B+Tree と Slotted Page を統合する

この step では、record 本体を table page の slotted page に保存し、B+Tree leaf には `key -> RecordId` だけを保存する。

これにより、index と table storage の役割を分けて考えられるようになる。ここでは primary key から record を取得する最小構成だけを扱う。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_record_id.rs && ./01_record_id
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. RecordId

File: `01_record_id.rs`

record の場所を `page_id + slot_id` として表現し、bytes に encode / decode する。

Goal:

- `PageId` と `SlotId` を raw な数値と分ける
- `RecordId` を固定長 6 bytes で保存する
- little endian で encode / decode する
- B+Tree leaf に保存しやすい固定長 pointer にする

## 2. table page

File: `02_table_page.rs`

row bytes を table page に保存し、`RecordId` を返す。

Goal:

- table page は slotted page として row bytes を保存する
- insert は `RecordId` を返す
- `RecordId` から row bytes を読む
- 削除済み record は `None` として扱う

## 3. index leaf に RecordId を保存する

File: `03_index_leaf_record_id.rs`

B+Tree leaf の value として `RecordId` を保存する。

Goal:

- leaf entry を `key -> RecordId` にする
- key を昇順に保存する
- duplicate key は RecordId を更新する
- key から RecordId を取得する

## 4. key から record を取得する

File: `04_table_lookup.rs`

index で `RecordId` を探し、table page から record を読む。

Goal:

- index lookup と table read を分ける
- missing key は `None` を返す
- deleted record は `None` を返す
- string value を含む row を取得できる

## 5. Table + Index smoke test

File: `05_table_index_smoke.rs`

table insert、index insert、lookup を一連の流れで確認する。

Goal:

- row を table page に保存する
- primary key から RecordId を引ける
- RecordId から row を取得する
- 複数 record を key で個別に読める

## この step で意識すること

- index は検索用、table page は record 保存用
- `RecordId` は index と table storage の接点
- clustered index では leaf に record 本体を置く設計もある
- secondary index では leaf から primary key や RecordId をたどる
- 次の step では、単発 lookup だけでなく scan と filter を扱う
