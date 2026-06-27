# Step 6: 固定長 key-value の B+Tree を永続化する

この step では、page file と page 上の node 表現を組み合わせて、固定長 `i64 -> i64` の B+Tree をファイルに保存する。

最初は root が leaf の小さな tree から始め、page split と root page id の更新を通して、ディスク上の tree を再オープンできる形にする。まだ range scan や buffer pool は扱わない。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_meta_page.rs && ./01_meta_page
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. metadata page

File: `01_meta_page.rs`

page 0 を tree metadata として使い、root page id と次に使う page id を保存する。

Goal:

- metadata page と node page を区別する
- root page id を保存、復元する
- next page id を保存、復元する
- 空ファイルを初期化する時の metadata を決める

## 2. root leaf の永続化

File: `02_persistent_leaf_root.rs`

root が leaf だけの状態で、insert と get を page file に保存する。

Goal:

- `open(path)` で新規 DB を初期化する
- root leaf page を作る
- `insert(i64, i64)` で root leaf に保存する
- `get(i64)` で root leaf から読む

## 3. leaf split

File: `03_leaf_split.rs`

leaf がいっぱいになった時に 2 つの leaf page へ分割する。

Goal:

- sorted entries を左右に分ける
- right leaf の最小 key を separator key として返す
- sibling pointer を更新する
- split 後もすべての key を取得できる

## 4. reopen

File: `04_reopen_tree.rs`

DB を閉じて再オープンした後も metadata と root page を読めるようにする。

Goal:

- metadata page から root page id を復元する
- 既存 DB を truncate しない
- 再オープン後に `get` できる
- 再オープン後の insert が既存データを壊さない

## 5. Persistent B+Tree smoke test

File: `05_persistent_tree_smoke.rs`

insert、split、flush、reopen、get を一連の操作として確認する。

Goal:

- 複数 key-value を永続化する
- root split 後も探索できる
- close / reopen 後も値を読める
- 更新 insert で既存 key の value を変更できる

## この step で意識すること

- root page id は固定値にせず metadata から読む
- page allocation と tree metadata は必ず整合させる
- separator key は右側 page の最小 key として扱うと考えやすい
- flush は永続化の入口だが、クラッシュ安全性はまだ保証しない
- 次の step では、可変長データを置くための slotted page を学ぶ
