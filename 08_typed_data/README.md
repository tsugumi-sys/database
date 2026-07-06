# Step 8: 文字列と型付きデータを保存する

この step では、page に保存する bytes の中身として、int、string、null を持つ typed value と row format を設計する。

Rust の enum をそのまま保存するのではなく、type tag、length prefix、little endian integer を使って明示的な disk format に変換する。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_value_encoding.rs && ./01_value_encoding
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. Value encode / decode

File: `01_value_encoding.rs`

`Value::Int` と `Value::String` を bytes に変換し、bytes から復元する。

Goal:

- type tag を 1 byte で保存する
- `i64` は little endian で保存する
- string は length prefix と UTF-8 bytes で保存する
- decode では消費した byte 数も返す

## 2. nullable value

File: `02_nullable_value.rs`

`NULL` を保存できる value format を追加する。

Goal:

- null 用の type tag を決める
- null は payload を持たない
- nullable column を row 内で表現できる
- 不正な tag は decode error にする

## 3. row encoding

File: `03_row_encoding.rs`

複数の value を 1 つの row として encode / decode する。

Goal:

- row の先頭に column count を保存する
- value を順番に encode する
- decode 後に同じ column 数と値が復元できる
- bytes が途中で切れている場合は失敗させる

## 4. key encoding と比較順序

File: `04_key_encoding.rs`

B+Tree key として使う値を byte 列にして、比較順序を確認する。

Goal:

- int key を比較可能な bytes に encode する
- string key は UTF-8 bytes として比較する
- 同じ型同士の比較を実装する
- 型が違う key の扱いを明示する

## 5. row を slotted page に保存する

File: `05_row_slotted_page.rs`

encode した row bytes を slotted page に保存し、slot id から row として復元する。

Goal:

- row を bytes に encode する
- slotted page に row bytes を保存する
- slot id から row を decode する
- string を含む row を保存、取得できる

## この step で意識すること

- memory layout と disk format は分けて考える
- type tag と length prefix があると可変長 value を復元できる
- decode は「値」と「何 bytes 読んだか」を返すと扱いやすい
- key encoding は B+Tree の比較順序に直結する
- 次の step では、row の保存先と B+Tree index を分離する
