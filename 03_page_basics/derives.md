# Rust derive notes

`derive` は、その型に対してよく使う trait の実装をコンパイラに自動生成してもらうための指定。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PageId(u32);
```

この例では、`PageId` に対して表示、コピー、比較、hash 計算などの機能を自動で付けている。

## Debug

`Debug` は `{:?}` で表示できるようにする。

```rust
let id = PageId(3);
println!("{:?}", id);
```

普通の `{}` で表示したい場合は `Display` が必要。`Display` は `derive` できないので、自分で実装する。

## Clone

`Clone` は、明示的に値を複製できるようにする。

```rust
let a = PageId(1);
let b = a.clone();
```

`.clone()` は「同じ値をもう一個作る」操作。

## Copy

`Copy` は、代入や関数呼び出しで move せず、自動でコピーしてよい型に付ける。

```rust
let a = PageId(1);
let b = a;

println!("{:?}", a);
```

`Copy` がない型だと、`let b = a;` の時点で `a` は move され、以後 `a` は使えない。

違い:

```rust
Clone = 明示的に .clone() すれば複製できる
Copy  = 代入や引数渡しで暗黙にコピーされる
```

`Copy` を付けられるのは、`u32`, `usize`, `bool` のような軽くて単純な値を中身に持つ型。`Vec` や `String` のようにヒープ領域を持つ型には基本的に付けられない。

`PageId(u32)` はただの `u32` ラッパーなので、`Copy` が自然。

## PartialEq

`PartialEq` は `==` と `!=` を使えるようにする。

```rust
PageId(1) == PageId(1)
PageId(1) != PageId(2)
```

`Partial` という名前なのは、型によっては「完全な等価比較」にならないケースがあるため。

代表例は浮動小数点の `NaN`。

```rust
f64::NAN == f64::NAN // false
```

## Eq

`Eq` は「この型の `==` は完全な等価関係です」という印。

`Eq` 自体に新しいメソッドはない。`PartialEq` より強い約束を表す。

`PageId(u32)` は普通の ID 値なので、同じ値同士は常に等しい。

```rust
PageId(1) == PageId(1) // true
PageId(1) == PageId(2) // false
```

なので `Eq` を付けてよい。

ざっくり言うと:

```rust
PartialEq = == が使える
Eq        = == がまともな等価比較だと保証する
```

## PartialOrd

`PartialOrd` は `<`, `<=`, `>`, `>=` を使えるようにする。

```rust
PageId(1) < PageId(2)
```

これも `Partial` なのは、浮動小数点の `NaN` のように順序が決められない値があるため。

## Ord

`Ord` は「すべての値同士で必ず順序が決まる」型に付ける。

`PageId(u32)` は `u32` と同じく、必ず順序付けできる。

```rust
PageId(1) < PageId(2)
PageId(10) > PageId(3)
```

`Ord` があると、sort や `BTreeMap` の key に使える。

```rust
let mut ids = vec![PageId(3), PageId(1), PageId(2)];
ids.sort();
```

## Hash

`Hash` は、`HashMap` や `HashSet` の key に使えるようにする。

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert(PageId(1), "root page");
```

`HashMap` の key には、基本的に `Eq + Hash` が必要。

## PageId ではどこまで必要か

最小ならこれで十分。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);
```

ただし、将来的に `PageId` を sort したり、`BTreeMap` / `HashMap` の key にしたくなる可能性が高い。

そのため、最初から次の形にしておくのも自然。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PageId(u32);
```

`PageId` は小さい ID 値なので、`Copy` で扱うのが自然。
