# Step 4: ページファイルを作る

この step では、固定サイズの page をメモリ上の byte 配列だけでなく、ファイル上に保存する。

`page_id` から決まる file offset に seek して読み書きすることで、必要な page だけをランダムアクセスできるようにする。まだ buffer pool、WAL、クラッシュ安全性は扱わない。

各ファイルは独立した問題になっている。`todo!()` を埋めて、テストが通るようにする。

実行例:

```sh
rustc --edition=2021 --test 01_page_offset.rs && ./01_page_offset
```

まとめて確認する場合:

```sh
for f in *.rs; do rustc --edition=2021 --test "$f" && "./${f%.rs}"; done
```

`rustc --test` は `.rs` と同じ名前の実行バイナリを生成する。生成されたバイナリは `.gitignore` で無視する。

## 1. page id と file offset

File: `01_page_offset.rs`

固定サイズ page がファイル上のどこに置かれるかを計算する。

Goal:

- `PageId` を raw な数値と分けて扱う
- offset は `page_id * PAGE_SIZE` で決まる
- page 同士の領域は重ならない
- file API で使える `u64` offset を返す

## 2. page の書き込み

File: `02_write_page.rs`

固定サイズの `Page` を、指定した `PageId` の位置へ書き込む。

Goal:

- `seek` で書き込み位置を選ぶ
- 常に `PAGE_SIZE` bytes を書き込む
- 異なる page id の書き込みを混同しない
- `flush` により書き込んだデータをファイルへ反映する

## 3. page の読み込み

File: `03_read_page.rs`

指定した `PageId` から固定サイズの `Page` を読み込む。

Goal:

- 書き込んだ page を page id で復元する
- 複数 page を個別に読める
- 存在しない、または途中までしかない page の読み込みを失敗させる

## 4. page allocation

File: `04_allocate_page.rs`

ファイル末尾へ空 page を追加し、新しい `PageId` を割り当てる。

Goal:

- 空のファイルでは最初の page id は `0`
- allocate ごとに page id が増える
- 新しい page は 0 で初期化される
- ファイルを再オープンしても次の page id を判断できる

## 5. PageFile smoke test

File: `05_page_file_smoke.rs`

allocate、write、flush、再オープン、read を一連の操作として確認する。

Goal:

- 複数 page を allocate して内容を保存する
- close / reopen 後に同じ page id から内容を読める
- 再オープン後の allocate が既存 page を上書きしない
- page file の最小限の永続化フローを確認する

## この step で意識すること

- `PageId` はファイル内の page の位置を特定するための番号
- 固定サイズ page なら offset 計算だけでランダムアクセスできる
- allocate はまず append-only とし、解放済み page の再利用は後の設計課題に残す
- `flush` は必要だが、単独ではクラッシュ安全性や transaction を保証しない
- 次の step では、保存できる page の bytes に B+Tree node を表現する
