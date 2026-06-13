// Step 4-1: Map page ids to file offsets.
//
// Run:
// rustc --edition=2021 --test 01_page_offset.rs && ./01_page_offset

#![allow(unused)]

const PAGE_SIZE: u64 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }

    fn as_u32(self) -> u32 {
        self.0
    }

    fn file_offset(self) -> u64 {
        u64::from(self.as_u32()) * PAGE_SIZE
    }
}

fn main() {
    let page_id = PageId::new(2);
    println!(
        "page {} starts at {}",
        page_id.as_u32(),
        page_id.file_offset()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_page_starts_at_zero() {
        assert_eq!(PageId::new(0).file_offset(), 0);
    }

    #[test]
    fn computes_offset_from_page_size() {
        assert_eq!(PageId::new(3).file_offset(), 3 * PAGE_SIZE);
    }

    #[test]
    fn adjacent_pages_have_non_overlapping_ranges() {
        let first_start = PageId::new(7).file_offset();
        let second_start = PageId::new(8).file_offset();

        assert_eq!(first_start + PAGE_SIZE, second_start);
    }

    #[test]
    fn large_page_ids_fit_file_offset_type() {
        assert_eq!(
            PageId::new(u32::MAX).file_offset(),
            u64::from(u32::MAX) * PAGE_SIZE
        );
    }
}
