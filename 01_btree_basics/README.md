# Step 1: メモリ上の B-Tree

この step では、ディスク、page、serialization は扱わず、メモリ上のデータ構造として B-Tree を実装する。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_search_node.rs && ./01_search_node
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. node 内探索

File: `01_search_node.rs`

B-Tree の node 内で、key がどの位置にあるか、またはどの child に進むべきかを探す。

Goal:

- node 内の key は昇順で保持されることを前提にする
- 見つかった場合は `Ok(index)` を返す
- 見つからない場合は `Err(child_index)` を返す
- 空の node でも動く

## 2. leaf node への挿入

File: `02_insert_into_leaf.rs`

split が不要な leaf node に key-value を挿入する。

Goal:

- key の昇順を維持する
- key と value の位置を対応させる
- 既存 key には value を上書きする
- まだ split は扱わない

## 3. child split

File: `03_split_child.rs`

full になった child node を 2 つに分割し、中央値 key を parent に上げる。

Goal:

- 最小次数 `T = 2` の B-Tree を扱う
- full child は `2 * T - 1` 個の key を持つ
- split 後、left child と right child はそれぞれ `T - 1` 個の key を持つ
- median key を parent に挿入する
- leaf child と internal child の両方を扱う

## 4. non-full node への挿入

File: `04_insert_non_full.rs`

root が full ではない前提で、適切な leaf まで降りながら挿入する。

Goal:

- leaf ならその場で挿入する
- internal node なら進む child を選ぶ
- 降りる前に full child を split する
- split 後に進む child を選び直す

## 5. B-Tree の insert / get

File: `05_btree_insert_get.rs`

root split を含む B-Tree 全体の `insert` と `get` を実装する。

Goal:

- 空の tree に挿入できる
- root が full の場合に root split する
- 複数段の tree でも `get` できる
- 重複 key は value を上書きする
- 挿入後も各 node の key が昇順である

## この step で意識すること

- B-Tree は「node 内は配列、node 間は tree」という構造で考える
- split は B-Tree 実装で一番重要な境界操作
- insert では、full child に降りる前に split すると実装しやすい
- 削除はこの step では扱わない
