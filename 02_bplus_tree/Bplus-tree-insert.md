# B+Tree insert

`leaf split` と `internal split` は別物だが、親に返す情報は同じ形にできる。

```rust
(separator_key, new_right_node)
```

意味は次の通り。

```text
自分を split したら、右側 node が新しくできた。
親は separator_key と new_right_node を child 配列に差し込んでほしい。
```

## 再帰関数の形

insert は、再帰関数を使うと考えやすい。

```rust
fn insert_into(node: &mut Node, key: i32, value: String) -> Option<(i32, Node)>
```

戻り値の意味。

```text
None
  この node 以下で insert は完了した。
  親に伝える split はない。

Some((separator, right))
  この node が split した。
  親は separator と right node を取り込む必要がある。
```

## leaf に insert する場合

```text
insert_into(leaf)
  key がすでにあれば value を overwrite する
  key がなければ sorted insert する

  leaf の key 数が MAX_KEYS を超えたら split する
    -> Some((right leaf の最初の key, right leaf))

  超えていなければ
    -> None
```

overwrite の場合は key 数が増えないので、基本的に split は不要。

この課題の `is_full()` は次の定義になっている。

```rust
fn is_full(&self) -> bool {
    self.key_count() > MAX_KEYS
}
```

そのため、この課題では「insert する前に full なら split」ではなく、次の流れが自然。

```text
insert / overwrite する
その結果 MAX_KEYS を超えたら split する
```

## internal に insert する場合

```text
insert_into(internal)
  key が入る child を選ぶ
  child に insert_into する

  child が split しなかったら
    -> None

  child が split したら
    separator と right child を自分の keys/children に挿入する

  その結果、自分の key 数が MAX_KEYS を超えたら internal split する
    -> Some((middle key, right internal))

  超えていなければ
    -> None
```

## split の伝搬

leaf が split すると、親 internal に separator を挿入する。

```text
leaf split
  -> 親 internal に separator を挿入
```

親 internal も key 数の上限を超えると、親 internal も split する。

```text
leaf split
  -> 親 internal に separator を挿入
  -> 親 internal も split
  -> さらに上の internal に separator を挿入
```

これが root まで続くことがある。

```text
leaf が split
  -> 親 internal が split
  -> さらに上の internal が split
  -> root が split
```

## root split

root だけは親がいないので特別扱いする。

`insert_into(&mut self.root, key, value)` が `Some((separator, right))` を返したら、root 自身が split したという意味。

その場合は新しい root を作る。

```text
old_root        new_right
   \              /
    \            /
     new root: [separator]
               children = [old_root, new_right]
```

つまり `insert` 本体の役割はこうなる。

```rust
fn insert(&mut self, key: i32, value: String) {
    if let Some((separator, right)) = insert_into(&mut self.root, key, value) {
        // root が split したので、新しい root を作る
    }
}
```

## まとめ

leaf split も internal split も、上から見ると同じ。

```text
下の node が split した
  -> separator key と right node が返ってくる
  -> 自分に差し込む
  -> 自分もあふれたら、さらに split して上に返す
```

これが B+Tree insert の split 伝搬。

## 答えの実装

```rust
fn insert(&mut self, key: i32, value: String) {
    if let Some((separator, right)) = insert_into(&mut self.root, key, value) {
        let old_root = std::mem::replace(&mut self.root, Node::empty_leaf());
        self.root = Node::Internal {
            keys: vec![separator],
            children: vec![old_root, right],
        };
    }
}

fn insert_into(node: &mut Node, key: i32, value: String) -> Option<(i32, Node)> {
    match node {
        Node::Leaf { keys, values } => {
            match keys.binary_search(&key) {
                Ok(idx) => {
                    values[idx] = value;
                    None
                }
                Err(idx) => {
                    values.insert(idx, value);
                    keys.insert(idx, key);

                    if keys.len() > MAX_KEYS {
                        let mid_idx = keys.len() / 2;
                        let right_keys = keys.split_off(mid_idx);
                        let right_values = values.split_off(mid_idx);
                        let separator = right_keys[0];
                        let right = Node::Leaf {
                            keys: right_keys,
                            values: right_values,
                        };

                        Some((separator, right))
                    } else {
                        None
                    }
                }
            }
        }
        Node::Internal { keys, children } => {
            let child_idx = match keys.binary_search(&key) {
                Ok(idx) => idx + 1,
                Err(idx) => idx,
            };

            if let Some((separator, right)) = insert_into(&mut children[child_idx], key, value) {
                keys.insert(child_idx, separator);
                children.insert(child_idx + 1, right);

                if keys.len() > MAX_KEYS {
                    let sep_idx = keys.len() / 2;
                    let right_keys = keys.split_off(sep_idx + 1);
                    let sep_key = keys
                        .pop()
                        .expect("internal split requires at least one separator key");
                    let right_children = children.split_off(sep_idx + 1);
                    let right = Node::Internal {
                        keys: right_keys,
                        children: right_children,
                    };

                    Some((sep_key, right))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
```
