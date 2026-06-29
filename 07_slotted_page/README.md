# Step 7: Slotted Page を学ぶ

この step では、可変長 bytes を page 内に保存するための slotted page 形式を実装する。

slotted page は、可変長 record を 1 page の中で管理するための page
layout 全体を指す。

slot directory は slotted page の中にある index 部分で、slot id から
cell body の offset と size を引けるようにする。つまり slotted page が
page 全体の形式で、slot directory はその中の「目次」にあたる。

slot directory は page の先頭側から増え、cell body は page の末尾側から詰める。slot id は固定したまま cell の場所を動かせるため、可変長 record の管理に向いている。

page layout のイメージ:

```text
one slotted page

+--------------------------------------------------------------+
| page header                                                  |
| - slot_count                                                 |
| - free_start                                                 |
| - free_end                                                   |
|                                                              |
| slot directory                                               |
| - slot 0 -> offset 240, size 11                              |
| - slot 1 -> offset 210, size 24                              |
| - slot 2 -> offset 195, size 12                              |
|                                                              |
| free space                                                   |
|                                                              |
| cell 2: "note: active"                                       |
| cell 1: "email: alice@example.com"                           |
| cell 0: "name: alice"                                        |
+--------------------------------------------------------------+
```

実際の page は 1 つの byte array で、header と slot directory は先頭側から
増える。cell body は末尾側から前方向に詰める。両者の間が、今すぐ
連続して使える free space になる。

この step では、1 つの cell は 1 page の中に収まる前提にする。page に
収まらない大きな値は overflow page や外部 storage で扱うこともできるが、
ここでは対象外にする。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_slot_directory.rs && ./01_slot_directory
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. slot directory

File: `01_slot_directory.rs`

page header と slot entry の layout を決める。

Goal:

- slot count を保存する
- free space start / end を保存する
- slot entry に cell offset と cell size を保存する
- slot id から slot entry を読める

## 2. cell 追加

File: `02_insert_cell.rs`

可変長 bytes を page 末尾側に保存し、slot directory へ位置を登録する。

Goal:

- cell body を page 末尾から前方向に詰める
- 追加した cell の slot id を返す
- free space が足りない場合は失敗する
- 異なる長さの cell を同じ page に保存できる

## 3. cell 読み取りと削除マーク

File: `03_read_delete_cell.rs`

slot id から cell を読み、削除した cell は再利用候補としてマークする。

Goal:

- slot id から bytes を読む
- 存在しない slot id は失敗させる
- delete は cell body をすぐ消さず slot を tombstone にする
- 削除済み slot の read は `None` を返す

## 4. compaction

File: `04_compaction.rs`

削除済み cell による断片化を解消し、生きている cell を詰め直す。

Goal:

- live cell だけを page 末尾側へ詰め直す
- slot id は変えない
- free space end を更新する
- compaction 後も live cell を読める

## 5. Slotted Page smoke test

File: `05_slotted_page_smoke.rs`

insert、read、delete、compact を一連の操作として確認する。

Goal:

- 可変長 bytes を複数保存する
- slot id で record を安定して参照する
- delete 後に read が `None` になる
- compact 後も残った record を読める

## この step で意識すること

- slot id は record の論理的な参照先
- cell offset は page 内の物理的な位置
- 可変長 record は削除により fragmentation が起きる
- compaction では slot id を保ったまま cell body だけを動かす
- 次の step では、保存する bytes の中身として型付き value と row format を扱う
