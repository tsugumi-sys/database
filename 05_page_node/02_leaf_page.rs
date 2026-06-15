// Step 5-2: Store sorted key-value cells in a leaf page.
//
// Run:
// rustc --edition=2021 --test 02_leaf_page.rs && ./02_leaf_page

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const NODE_TYPE_OFFSET: usize = 0;
const KEY_COUNT_OFFSET: usize = 1;
const NODE_HEADER_SIZE: usize = 11;
const LEAF_NODE_TYPE: u8 = 1;
const LEAF_CELL_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LeafEntry {
    key: i64,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new_leaf() -> Self {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };
        page.data[NODE_TYPE_OFFSET] = LEAF_NODE_TYPE;
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

    fn cell_offset(index: usize) -> usize {
        NODE_HEADER_SIZE + index * LEAF_CELL_SIZE
    }

    fn read_entry(&self, index: usize) -> LeafEntry {
        let offset = Page::cell_offset(index);
        let offset_end = offset.checked_add(LEAF_CELL_SIZE).expect("overflow");
        let key_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
        let value_bytes: [u8; 8] = self.data[offset + 8..offset_end].try_into().unwrap();
        LeafEntry {
            key: i64::from_le_bytes(key_bytes),
            value: i64::from_le_bytes(value_bytes),
        }
    }

    fn write_entry(&mut self, index: usize, entry: LeafEntry) {
        let offset = Page::cell_offset(index);
        let offset_end = offset.checked_add(LEAF_CELL_SIZE).expect("overflow");
        self.data[offset..offset + 8].copy_from_slice(&entry.key.to_le_bytes());
        self.data[offset + 8..offset_end].copy_from_slice(&entry.value.to_le_bytes());
    }

    fn insert(&mut self, key: i64, value: i64) -> Result<(), String> {
        let n_entries = self.key_count();
        let mut left = 0;
        let mut right = n_entries;

        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(mid).key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        let insert_idx = left;

        // if item is alredy there, just update it.
        if insert_idx < n_entries && self.read_entry(insert_idx).key == key {
            self.write_entry(insert_idx, LeafEntry { key, value });
            return Ok(());
        }

        let max_cells = (PAGE_SIZE - NODE_HEADER_SIZE) / LEAF_CELL_SIZE;
        if n_entries >= max_cells {
            return Err("page is full".to_string());
        }

        // shift existing entries
        for i in (insert_idx..n_entries).rev() {
            let entry = self.read_entry(i);
            self.write_entry(i + 1, entry);
        }

        self.write_entry(insert_idx, LeafEntry { key, value });
        self.set_key_count(n_entries + 1);
        Ok(())
    }

    fn get(&self, key: i64) -> Option<i64> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count; // you cannot use count - 1, if the page is empty, this causes underflow.
        while left < right {
            let mid = (left + right) / 2;
            let e = self.read_entry(mid);

            if e.key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left < count {
            let e = self.read_entry(left);
            if e.key == key {
                return Some(e.value);
            }
        }
        None
    }
}

fn main() {
    println!("leaf cell size = {}", LEAF_CELL_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_leaf_page() {
        let page = Page::new_leaf();

        assert_eq!(page.data[NODE_TYPE_OFFSET], LEAF_NODE_TYPE);
        assert_eq!(page.key_count(), 0);
    }

    #[test]
    fn writes_and_reads_leaf_entry() {
        let mut page = Page::new_leaf();

        page.write_entry(
            0,
            LeafEntry {
                key: 10,
                value: 100,
            },
        );
        page.set_key_count(1);

        assert_eq!(
            page.read_entry(0),
            LeafEntry {
                key: 10,
                value: 100
            }
        );
    }

    #[test]
    fn insert_keeps_keys_sorted() {
        let mut page = Page::new_leaf();

        page.insert(30, 300).unwrap();
        page.insert(10, 100).unwrap();
        page.insert(20, 200).unwrap();

        assert_eq!(page.key_count(), 3);
        assert_eq!(page.read_entry(0).key, 10);
        assert_eq!(page.read_entry(1).key, 20);
        assert_eq!(page.read_entry(2).key, 30);
    }

    #[test]
    fn get_returns_matching_value() {
        let mut page = Page::new_leaf();
        page.insert(10, 100).unwrap();
        page.insert(20, 200).unwrap();

        assert_eq!(page.get(10), Some(100));
        assert_eq!(page.get(20), Some(200));
        assert_eq!(page.get(30), None);
    }

    #[test]
    fn duplicate_key_updates_value() {
        let mut page = Page::new_leaf();

        page.insert(10, 100).unwrap();
        page.insert(10, 999).unwrap();

        assert_eq!(page.key_count(), 1);
        assert_eq!(page.get(10), Some(999));
    }
}
