# Step 5: node を page 上に表現する

この step では、B+Tree の node を Rust の構造体として持つのではなく、固定サイズ page の byte 配列上に表現する。

page の先頭には node header を置き、その後ろに key/value や key/child pointer の配列を並べる。まずは固定長 key / 固定長 value だけを扱い、可変長データは後の slotted page で扱う。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_node_header.rs && ./01_node_header
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. node header

File: `01_node_header.rs`

page の先頭に node type、key count、parent page id、sibling page id を保存する。

Goal:

- leaf node と internal node を区別する
- key count を header に保存する
- parent pointer を optional な page id として扱う
- leaf の next sibling pointer を optional な page id として扱う
- header field の offset を明示的に決める

## 2. leaf page

File: `02_leaf_page.rs`

leaf page に固定長の `key -> value` を sorted order で保存する。

Goal:

- leaf page を初期化する
- key count から cell offset を計算する
- `i64` key と `i64` value を little endian で保存する
- insert 後も key が昇順に並ぶ
- 同じ key の insert は value を更新する

## 3. internal page

File: `03_internal_page.rs`

internal page に探索用 key と child page id を保存する。

Goal:

- internal page を初期化する
- child pointer は key より 1 つ多いことを意識する
- key と child page id を little endian で保存する
- separator key を昇順に挿入する
- 探索 key から次に読む child page id を選べる

## 4. page 内の二分探索

File: `04_page_binary_search.rs`

page 上の sorted key 配列に対して二分探索を行う。

Goal:

- key が存在する場合はその index を返す
- key が存在しない場合は挿入位置を返す
- leaf と internal の両方で使える key accessor を意識する
- 線形探索ではなく範囲を半分に絞る考え方を確認する

## 5. Node page smoke test

File: `05_node_page_smoke.rs`

leaf page と internal page の最低限の操作を一連の流れで確認する。

Goal:

- leaf page に複数 key-value を保存する
- internal page から探索先 child page id を選ぶ
- sibling pointer で leaf page 同士をつなぐ
- page bytes だけから node の情報を復元する

## この step で意識すること

- disk format は構造体の memory layout に依存させない
- offset、size、endian を毎回明示する
- page header と node header の責務を分けて考える
- fixed-length layout は単純だが、可変長 value には向かない
- 次の step では、この node page を page file に保存して永続化する
