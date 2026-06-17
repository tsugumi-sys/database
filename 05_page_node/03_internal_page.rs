// Step 5-3: Store separator keys and child page ids in an internal page.
//
// Run:
// rustc --edition=2021 --test 03_internal_page.rs && ./03_internal_page

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const NODE_TYPE_OFFSET: usize = 0;
const KEY_COUNT_OFFSET: usize = 1;
const NODE_HEADER_SIZE: usize = 11;
const INTERNAL_NODE_TYPE: u8 = 2;
const INTERNAL_CELL_SIZE: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InternalEntry {
    key: i64,
    right_child: PageId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new_internal(leftmost_child: PageId) -> Self {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };
        page.data[NODE_TYPE_OFFSET] = INTERNAL_NODE_TYPE;
        page.set_leftmost_child(leftmost_child);
        page
    }

    fn key_count(&self) -> usize {
        let bytes: [u8; 8] = self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 8]
            .try_into()
            .unwrap();
        usize::from_le_bytes(bytes)
    }

    fn set_key_count(&mut self, count: usize) {
        self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 8].copy_from_slice(&count.to_le_bytes());
    }

    fn leftmost_child(&self) -> PageId {
        let bytes: [u8; 4] = self.data[NODE_HEADER_SIZE..NODE_HEADER_SIZE + 4]
            .try_into()
            .unwrap();
        PageId::new(u32::from_le_bytes(bytes))
    }

    fn set_leftmost_child(&mut self, child: PageId) {
        self.data[NODE_HEADER_SIZE..NODE_HEADER_SIZE + 4].copy_from_slice(&child.0.to_le_bytes());
    }

    fn cell_offset(index: usize) -> usize {
        NODE_HEADER_SIZE + 4 + index * INTERNAL_CELL_SIZE
    }

    fn read_entry(&self, index: usize) -> InternalEntry {
        let offset = Page::cell_offset(index);
        let offset_end = offset
            .checked_add(INTERNAL_CELL_SIZE)
            .expect("offset overflow");
        let key_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
        let pageid_bytes: [u8; 4] = self.data[offset + 8..offset_end].try_into().unwrap();
        InternalEntry {
            key: i64::from_le_bytes(key_bytes),
            right_child: PageId::new(u32::from_le_bytes(pageid_bytes)),
        }
    }

    fn write_entry(&mut self, index: usize, entry: InternalEntry) {
        let offset = Page::cell_offset(index);
        let offset_end = offset
            .checked_add(INTERNAL_CELL_SIZE)
            .expect("offset overflow");

        self.data[offset..offset + 8].copy_from_slice(&entry.key.to_le_bytes());
        self.data[offset + 8..offset_end].copy_from_slice(&entry.right_child.0.to_le_bytes());
    }

    fn insert_separator(&mut self, key: i64, right_child: PageId) -> Result<(), String> {
        let count = self.key_count();
        if count >= (PAGE_SIZE - NODE_HEADER_SIZE) / INTERNAL_CELL_SIZE {
            return Err("this page is full".to_string());
        }
        let mut left = 0;
        let mut right = count;

        while left < right {
            let mid = (left + right) / 2;
            let entry = self.read_entry(mid);
            if entry.key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        for i in (left..count).rev() {
            let entry = self.read_entry(i);
            self.write_entry(i + 1, entry);
        }

        self.write_entry(left, InternalEntry { key, right_child });
        self.set_key_count(count + 1);
        Ok(())
    }

    fn child_for_key(&self, search_key: i64) -> PageId {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(mid).key <= search_key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left == 0 {
            return self.leftmost_child();
        }
        self.read_entry(left - 1).right_child
    }
}

fn main() {
    println!("internal cell size = {}", INTERNAL_CELL_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_internal_page() {
        let page = Page::new_internal(PageId::new(3));

        assert_eq!(page.data[NODE_TYPE_OFFSET], INTERNAL_NODE_TYPE);
        assert_eq!(page.key_count(), 0);
        assert_eq!(page.leftmost_child(), PageId::new(3));
    }

    #[test]
    fn writes_and_reads_internal_entry() {
        let mut page = Page::new_internal(PageId::new(1));

        page.write_entry(
            0,
            InternalEntry {
                key: 50,
                right_child: PageId::new(2),
            },
        );
        page.set_key_count(1);

        assert_eq!(
            page.read_entry(0),
            InternalEntry {
                key: 50,
                right_child: PageId::new(2),
            }
        );
    }

    #[test]
    fn insert_separator_keeps_keys_sorted() {
        let mut page = Page::new_internal(PageId::new(1));

        page.insert_separator(30, PageId::new(4)).unwrap();
        page.insert_separator(10, PageId::new(2)).unwrap();
        page.insert_separator(20, PageId::new(3)).unwrap();

        assert_eq!(page.key_count(), 3);
        assert_eq!(page.read_entry(0).key, 10);
        assert_eq!(page.read_entry(1).key, 20);
        assert_eq!(page.read_entry(2).key, 30);
    }

    #[test]
    fn chooses_child_for_search_key() {
        let mut page = Page::new_internal(PageId::new(1));
        page.insert_separator(10, PageId::new(2)).unwrap();
        page.insert_separator(20, PageId::new(3)).unwrap();
        page.insert_separator(30, PageId::new(4)).unwrap();

        assert_eq!(page.child_for_key(5), PageId::new(1));
        assert_eq!(page.child_for_key(10), PageId::new(2));
        assert_eq!(page.child_for_key(19), PageId::new(2));
        assert_eq!(page.child_for_key(20), PageId::new(3));
        assert_eq!(page.child_for_key(99), PageId::new(4));
    }
}
