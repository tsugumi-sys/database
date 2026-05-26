// Step 3-1: PageId.
//
// Run:
// rustc --edition=2021 --test 01_page_id.rs && ./01_page_id

#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }

    fn as_u32(self) -> u32 {
        self.0
    }

    fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

fn main() {
    let page_id = PageId::new(1);
    println!("{:?}", page_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_page_id() {
        let page_id = PageId::new(42);

        assert_eq!(page_id.as_u32(), 42);
    }

    #[test]
    fn next_returns_next_page_id() {
        let page_id = PageId::new(10);

        assert_eq!(page_id.next(), PageId::new(11));
    }

    #[test]
    fn page_id_is_not_plain_u32_at_call_site() {
        fn takes_page_id(page_id: PageId) -> u32 {
            page_id.as_u32()
        }

        assert_eq!(takes_page_id(PageId::new(7)), 7);
    }
}
