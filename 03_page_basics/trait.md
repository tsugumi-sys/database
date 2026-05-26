# Rust trait notes

`trait` は「この型はこういう操作ができます」という約束。

Java / TypeScript / Go でいう `interface` にかなり近い。

ただし Rust の trait は、既存の型にあとから能力を実装したり、ジェネリクスの制約に使ったりできる。

## struct / impl / trait

ざっくり分けるとこう。

```text
struct = データの形
impl   = 実装を書く場所
trait  = 複数の型に共通する能力の名前
```

例:

```rust
struct PageId(u32);
```

これは「`PageId` は `u32` を1個持つ型」というデータ定義。

```rust
impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}
```

これは `PageId` 専用のメソッド定義。

## trait は interface に近い

```rust
trait Encode {
    fn encode(&self, dst: &mut [u8]);
}
```

これは「`Encode` できる型は `encode` メソッドを持っている」という約束。

Java っぽく言うと、次の interface に近い。

```java
interface Encode {
    void encode(byte[] dst);
}
```

## trait を struct に実装する

```rust
struct PageId(u32);

impl Encode for PageId {
    fn encode(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.0.to_le_bytes());
    }
}
```

これは「`PageId` は `Encode` という能力を持つ」と実装している。

つまり:

```text
PageId implements Encode
```

という意味。

## impl には2種類ある

型そのものにメソッドを生やす `impl`。

```rust
impl PageId {
    fn as_u32(self) -> u32 {
        self.0
    }
}
```

trait を型に実装する `impl`。

```rust
impl Encode for PageId {
    fn encode(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.0.to_le_bytes());
    }
}
```

同じ `impl` でも意味が違う。

```text
impl PageId
  -> PageId 専用のメソッドを書く

impl Encode for PageId
  -> PageId に Encode という trait を実装する
```

## derive との関係

`derive` は、よくある trait の実装をコンパイラに自動生成してもらう仕組み。

```rust
#[derive(Debug)]
struct PageId(u32);
```

これは概念的には、次のような実装をコンパイラに作ってもらっている。

```rust
impl std::fmt::Debug for PageId {
    // {:?} で表示するための実装
}
```

つまり:

```text
Debug        = trait
PageId       = struct
derive(Debug)= PageId に Debug trait の impl を自動生成
```

`PartialEq` も trait。

```rust
#[derive(PartialEq)]
struct PageId(u32);
```

これにより、概念的には次のような実装が作られる。

```rust
impl PartialEq for PageId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
```

だから `==` が使える。

```rust
PageId(1) == PageId(1)
```

## 既存の型にも実装できる

Rust の trait は、既存の型にあとから実装できる。

```rust
trait DebugName {
    fn debug_name(&self) -> &'static str;
}

impl DebugName for u32 {
    fn debug_name(&self) -> &'static str {
        "u32"
    }
}
```

`u32` は自分で定義した型ではないが、自分で作った trait なら実装できる。

## ジェネリクスの制約に使う

trait は「この能力を持つ型だけ受け取る」という制約にも使う。

```rust
fn write_to_page<T: Encode>(value: &T, dst: &mut [u8]) {
    value.encode(dst);
}
```

これは:

```text
T は Encode を実装している型なら何でもよい
```

という意味。

`PageId` でも `Record` でも `PageHeader` でも、`Encode` を実装していれば同じ関数に渡せる。

## DB 実装で出てきそうな trait

```rust
trait Encode {
    fn encode(&self, dst: &mut [u8]);
}

trait Decode: Sized {
    fn decode(src: &[u8]) -> Self;
}
```

`PageId` に実装する例:

```rust
impl Encode for PageId {
    fn encode(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.0.to_le_bytes());
    }
}

impl Decode for PageId {
    fn decode(src: &[u8]) -> Self {
        let bytes = [src[0], src[1], src[2], src[3]];
        PageId(u32::from_le_bytes(bytes))
    }
}
```

## まとめ

```text
struct = 何を持っているか
impl 型 = その型に何ができるか
trait = どういう能力があるべきか
impl Trait for 型 = その型でその能力を実現する
derive = よくある trait impl を自動生成する
```

`struct` は名詞、`trait` は能力、`impl` は実装を書く場所。
