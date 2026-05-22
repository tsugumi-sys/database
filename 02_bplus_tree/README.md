# Step 2: メモリ上の B+Tree

この step では、まだディスク、page、serialization は扱わず、メモリ上のデータ構造として B+Tree を実装する。

B-Tree との大きな違いは、実データを leaf node に集め、internal node は探索のための separator key と child pointer だけを持つこと。leaf node 同士をリンクすると range scan がしやすくなる。

B+Tree がこの構造を取る主な理由は、internal node に value を置かないことで、同じ node/page の中により多くの separator key と child pointer を入れられるようにするため。1つの node から分岐できる child 数を fanout と呼ぶ。value の分だけ領域を使わずに済むので fanout が大きくなり、木の高さを低くできる。DB では node を page 単位で読むため、木が浅いほど lookup で読む page 数を減らせる。これは単純なメモリ使用量削減というより、主に I/O 回数と cache miss を減らすための設計。

point lookup では、internal node を道案内として読みながら leaf node まで降り、最後に leaf node 上の value を読む。つまり探索時に internal node だけで完結するわけではなく、実データを読むために leaf node も必ず読む。

value を leaf node に集めることで、range scan も単純になる。start key のある leaf node まで降りたあとは、leaf node の next pointer をたどるだけで、key の昇順に連続した値を取得できる。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_point_lookup.rs && ./01_point_lookup
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

## 1. point lookup

File: `01_point_lookup.rs`

B+Tree の root から leaf まで降り、leaf node 上で key を探す。

Goal:

- internal node は separator key と children だけを持つ
- separator key と等しい key は右側 child に進む
- value は leaf node だけに保存する
- 見つからない key は `None` を返す

## 2. range scan

File: `02_range_scan.rs`

leaf node 同士のリンクをたどりながら、指定した範囲の key-value を順番に取得する。

Goal:

- start key を含む leaf から scan を始める
- `start <= key < end` の範囲だけ返す
- leaf の next pointer を使って隣の leaf に進む
- 結果は key の昇順になる

## 3. leaf split

File: `03_split_leaf.rs`

full になった leaf node を左右に分割し、親に挿入する separator key を返す。

Goal:

- leaf の key/value を左右に分ける
- right leaf の最初の key を separator key にする
- leaf の next pointer をつなぎ替える
- B-Tree と違い、separator key も right leaf に残す

この問題では、新しく作った right leaf は `leaves` の末尾に追加する。既存の leaf の index は変えず、leaf の論理的な順序は `next` pointer で表す。right leaf の `next` には、split 前の leaf が持っていた `next` をそのまま引き継ぐ。

## 4. internal split

File: `04_split_internal.rs`

full になった internal node を分割し、親に上げる separator key と right internal node を作る。

Goal:

- internal node は values を持たない
- median key は親に上げ、左右の internal node には残さない
- children も key の分割位置に合わせて左右に分ける
- split 後も `children.len() == keys.len() + 1` を保つ

## 5. B+Tree の insert / get / range scan

File: `05_bplus_tree_insert_range.rs`

root split を含む B+Tree 全体の `insert`、`get`、`range_scan` を実装する。

Goal:

- 空の tree に挿入できる
- leaf split と internal split を使って木を成長させる
- value は leaf node だけに保存する
- 重複 key は value を上書きする
- leaf link を使って range scan できる

## この step で意識すること

- B+Tree の internal node は「実データ」ではなく「道案内」を持つ
- leaf node にだけ value があるため、point lookup は必ず leaf まで降りる
- separator key の扱いは B-Tree と B+Tree で違う
- range scan は B+Tree が DB index に向いている理由のひとつ
