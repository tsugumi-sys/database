# B-Tree ベース DB エンジン学習ロードマップ

Rust で B-Tree を実装する練習から始めて、最終的に B-Tree ベースの小さな DB エンジンを自作するための学習ステップを整理する。

この README は、最初から完成形のコード設計を決めるものではない。各ステップで何を理解し、何を作れるようになるかを明確にするためのロードマップとして使う。

## ゴール

- B-Tree / B+Tree の基本アルゴリズムを自分で説明できる
- ページ単位のストレージ管理を理解する
- internal node と leaf node をページ上に表現できる
- slotted page による可変長データ管理を理解する
- int / string などの基本的な値をページへ保存できる
- B-Tree を使った単純な key-value DB の読み書きができる
- 将来的に transaction, WAL, buffer pool へ進める土台を作る

## For LLM

各ステップのディレクトリを作る時は必ず.gitignoreを作り、バイナリファイルを無視するようにすること。rustcでコンパイルされたバイナリファイルをコミットしたくないので。

## Step 1: メモリ上の B-Tree を理解する

まずはディスクやページを考えず、メモリ上のデータ構造として B-Tree を実装する。

Exercises: [01_btree_basics](./01_btree_basics)

学ぶこと:

- B-Tree の次数、key 数、child 数の関係
- search の流れ
- insert の流れ
- node split
- root split
- 木の高さが増えるタイミング
- B-Tree が平衡木である理由

作るもの:

- `insert(key, value)`
- `get(key)`
- node split を含む基本的な B-Tree
- int key / int value から始める

この段階では、削除や永続化は扱わない。

## Step 2: B+Tree へ進む

DB のインデックスでは B-Tree より B+Tree がよく使われるため、leaf node に値を集める構造を学ぶ。

Exercises: [02_bplus_tree](./02_bplus_tree)

学ぶこと:

- B-Tree と B+Tree の違い
- internal node は探索用 key と child pointer を持つ
- leaf node は実データ、または record pointer を持つ
- leaf node 同士をリンクする理由
- range scan が効率的になる理由

作るもの:

- internal node / leaf node を分けたメモリ上 B+Tree
- point lookup
- range scan
- leaf split
- internal split

この段階でもまだディスク永続化は扱わない。

## Step 3: ページという単位を導入する

DB エンジンではデータを byte 配列のページとして管理する。B+Tree の node を Rust の構造体として直接保存するのではなく、固定サイズページ上のデータとして扱う準備をする。

Exercises: [03_page_basics](./03_page_basics)

学ぶこと:

- page size
- page id
- page header
- byte array としての page
- ページ内 offset
- endian
- serialization / deserialization
- メモリ上の構造体とディスク上の表現の違い

作るもの:

- 固定サイズの `Page`
- `PageId`
- page header の読み書き
- byte buffer への int 保存
- byte buffer からの int 復元

この段階では、まだ B+Tree と結合しなくてよい。

## Step 4: ページファイルを作る

ページをメモリだけでなくファイルに保存し、page id で読み書きできるようにする。

Exercises: [04_page_file](./04_page_file)

学ぶこと:

- file offset と page id の対応
- fixed-size page の read / write
- page allocation
- free page の扱いの入口
- flush の考え方
- Rust の `std::fs` によるランダムアクセス

作るもの:

- page をファイルへ書き込む処理
- page id から page を読み込む処理
- 新しい page を allocate する処理
- 簡単な smoke test

この段階では、クラッシュ安全性や WAL は扱わない。

## Step 5: node を page 上に表現する

B+Tree の internal node と leaf node を、Rust の構造体ではなく page 内の bytes として表現する。

Exercises: [05_page_node](./05_page_node)

学ぶこと:

- node type
- key count
- internal node の key / child pointer 配列
- leaf node の key / value 配列
- parent pointer を持つかどうかの判断
- sibling pointer
- page header と node header の境界

作るもの:

- leaf page の作成
- internal page の作成
- leaf page への key-value 挿入
- internal page への key-child 挿入
- page 内の二分探索

最初は固定長 key / 固定長 value でよい。

## Step 6: 固定長 key-value の B+Tree を永続化する

ページファイルと page 上の node 表現を使って、最初の永続化 B+Tree を作る。

Exercises: [06_persistent_bplus_tree](./06_persistent_bplus_tree)

学ぶこと:

- root page id の管理
- tree metadata page
- insert 時の page split
- split 後の separator key の扱い
- child page id による探索
- ディスク上の tree を再オープンする流れ

作るもの:

- `open(path)`
- `insert(i64, i64)`
- `get(i64) -> Option<i64>`
- DB を閉じて再オープンしても値が読める状態

この段階では range scan は後回しでもよい。

## Step 7: Slotted Page を学ぶ

文字列や可変長 value を扱うため、ページ内に slotted page 形式を導入する。

Exercises: [07_slotted_page](./07_slotted_page)

学ぶこと:

- fixed-length record と variable-length record の違い
- slot directory
- cell offset
- cell size
- free space pointer
- fragmentation
- compaction
- record id / slot id

作るもの:

- slotted page への cell 追加
- slot id から cell を読む処理
- cell 削除のマーク
- page compaction
- 可変長 bytes の保存

この段階では、B+Tree に統合せず slotted page 単体で練習する。

## Step 8: 文字列と型付きデータを保存する

int だけでなく string を保存できるようにし、DB の record 表現に近づける。

Exercises: [08_typed_data](./08_typed_data)

学ぶこと:

- value encoding
- type tag
- length prefix
- nullable value
- row format
- schema を持つ場合と持たない場合
- key encoding と比較順序

作るもの:

- `Value::Int`
- `Value::String`
- value の encode / decode
- row の encode / decode
- slotted page への row 保存

この段階では SQL は扱わない。

## Step 9: B+Tree と Slotted Page を統合する

leaf node に直接 value を詰めるのではなく、record を slotted page に置き、B+Tree から record location を指す構造を学ぶ。

Exercises: [09_table_index](./09_table_index)

学ぶこと:

- index と table storage の分離
- record pointer
- page id + slot id
- primary index
- clustered index と secondary index の違い
- leaf に value を持つ設計との比較

作るもの:

- table page に record を保存
- B+Tree leaf に `key -> RecordId` を保存
- key から record を取得
- string value を含む record の保存と取得

この段階で、単純な key-value DB から小さな table storage へ進む。

## Step 10: Scan と簡単なクエリ操作

DB らしい操作として、範囲取得や全件走査を実装する。

Exercises: [10_scan_query](./10_scan_query)

学ぶこと:

- leaf sibling pointer
- range scan
- iterator
- table scan
- predicate filtering
- record visibility の入口

作るもの:

- `range(start, end)`
- leaf chain を使った順序付き scan
- table scan
- 簡単な filter

この段階では query planner は扱わない。

## Step 11: Buffer Pool の入口

毎回ファイルから page を読むのではなく、メモリ上に page cache を持つ。

学ぶこと:

- buffer pool
- page frame
- dirty page
- pin / unpin
- eviction
- LRU の基本
- flush policy

作るもの:

- page cache
- dirty page の flush
- 小さな LRU eviction
- B+Tree から buffer pool 経由で page を読む構成

この段階で、storage manager と index の境界を意識する。

## Step 12: 削除と空き領域管理

insert / get だけでなく delete を扱い、ページや node の空き領域を管理する。

学ぶこと:

- B+Tree delete
- merge
- redistribution
- underflow
- tombstone
- free list
- page reuse
- slotted page の空き slot 再利用

作るもの:

- leaf からの key 削除
- 必要に応じた merge / redistribution
- free page list
- slotted page の slot 再利用

削除は複雑なので、最初は tombstone 方式から始めてもよい。

## Step 13: クラッシュ安全性の入口

DB エンジンとしての信頼性を学ぶため、WAL や recovery の基本へ進む。

学ぶこと:

- atomicity
- durability
- write-ahead logging
- log record
- pageLSN
- checkpoint
- redo / undo の考え方
- fsync の役割

作るもの:

- append-only WAL
- insert の log record
- 起動時の簡単な redo
- metadata page の安全な更新

この段階では本格的な transaction isolation は扱わない。

## Step 14: Transaction の入口

最後に、複数操作をまとめる transaction の基礎を学ぶ。

学ぶこと:

- transaction id
- commit / abort
- lock
- latch と lock の違い
- isolation level
- simple 2PL
- MVCC の入口

作るもの:

- single-writer transaction
- commit
- abort
- 簡単な lock manager

ここまで進むと、小さな DBMS の主要な構成要素を一通り触れる。

## 推奨する進め方

各 step では、以下の順番で進める。

1. まず紙やメモにデータ構造を書く
2. Rust の型で最小の表現を作る
3. 小さいケースのテストを書く
4. 失敗しやすい境界条件を追加する
5. 実装後に README やメモへ理解したことを書く

特に B+Tree では、次のケースを毎回テストする。

- 空の tree
- root だけの tree
- leaf split
- internal split
- root split
- 重複 key
- 存在しない key
- page size ぎりぎりの挿入

## 最初のマイルストーン

最初の実装目標は、以下に絞る。

1. メモリ上の B-Tree
2. メモリ上の B+Tree
3. 固定サイズ page
4. page file
5. page 上の leaf / internal node
6. 永続化された `insert(i64, i64)` / `get(i64)`

ここまでできれば、B-Tree ベース DB エンジンの土台として十分に次へ進める。
