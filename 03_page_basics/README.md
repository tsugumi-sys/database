# Step 3: ページという単位を導入する

この step では、DB エンジンがデータを管理する基本単位として page を扱う。

まだ B+Tree とは結合しない。Rust の構造体をそのまま保存するのではなく、固定サイズの byte 配列に値を読み書きする感覚をつかむ。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_page_id.rs && ./01_page_id
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. PageId

File: `01_page_id.rs`

ページを一意に識別する `PageId` を作る。

Goal:

- `PageId` を raw な `u32` と分けて扱う
- `PageId::new` で作成する
- `as_u32` で保存用の値に戻せる
- `next` で次の page id を作れる

## 2. 固定サイズ Page

File: `02_fixed_size_page.rs`

固定長 byte 配列を持つ `Page` を作り、offset 指定で読み書きする。

Goal:

- page size は `PAGE_SIZE = 4096`
- 新しい page は 0 で初期化される
- 指定 offset に byte 列を書ける
- 指定 offset から byte 列を読める
- page の範囲外アクセスは失敗させる

## 3. Page Header

File: `03_page_header.rs`

page の先頭に header を置き、page type や page id を読み書きする。

Goal:

- header の field ごとに固定 offset を決める
- page type を 1 byte で保存する
- page id を 4 byte で保存する
- free space offset を 2 byte で保存する
- 読み書き後も同じ値が復元できる

## 4. endian と int の保存

File: `04_endian_int.rs`

整数を byte buffer に保存し、byte buffer から復元する。

Goal:

- little endian で `u16`, `u32`, `i32` を保存する
- 保存された byte の並びを確認する
- 復元時に同じ値が返る
- buffer が足りない場合は失敗させる

## 5. 構造体と disk format の分離

File: `05_page_serialization.rs`

メモリ上の構造体を固定長の disk format に変換し、page に保存する。

Goal:

- `Record` を固定長 byte 列として保存する
- page 上の offset を指定して record を読み書きする
- memory layout に依存せず field ごとに明示的に serialize する
- 複数 record を同じ page に並べられる

## この step で意識すること

- page は「構造体」ではなく「byte 配列」
- offset と size を常に意識する
- disk format は明示的に決める
- endian を揃えないと別環境で同じ値として読めない
- B+Tree の node は、次の step 以降で page 上に載せていく

