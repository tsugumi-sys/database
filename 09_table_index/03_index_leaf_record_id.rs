// Step 9-3: Store key -> RecordId entries in an index leaf.
//
// Run:
// rustc --edition=2021 --test 03_index_leaf_record_id.rs && ./03_index_leaf_record_id

#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordId {
    page_id: PageId,
    slot_id: SlotId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IndexEntry {
    key: i64,
    record_id: RecordId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexLeaf {
    entries: Vec<IndexEntry>,
}

impl IndexLeaf {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, key: i64, record_id: RecordId) {
        let position = self.leftmost_position(key);
        if position < self.entries.len() && self.entries[position].key == key {
            self.entries[position].record_id = record_id;
            return;
        }
        self.entries.push(IndexEntry { key, record_id });
        let tail = self.entries.len() - 1;
        if position < tail {
            let mut current = tail;
            while current > position {
                self.entries.swap(current, current - 1);
                current -= 1;
            }
        }
    }

    fn get(&self, key: i64) -> Option<RecordId> {
        let position = self.leftmost_position(key);
        if position < self.entries.len() && self.entries[position].key == key {
            return Some(self.entries[position].record_id);
        }
        None
    }

    fn leftmost_position(&self, key: i64) -> usize {
        let count = self.entries.len();

        let mut lo = 0;
        let mut hi = count;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.entries[mid].key < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        lo
    }
}

fn main() {
    println!("index leaf record ids");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rid(page: u32, slot: u16) -> RecordId {
        RecordId {
            page_id: PageId(page),
            slot_id: SlotId(slot),
        }
    }

    #[test]
    fn inserts_record_ids_sorted_by_key() {
        let mut leaf = IndexLeaf::new();

        leaf.insert(20, rid(1, 2));
        leaf.insert(10, rid(1, 1));
        leaf.insert(30, rid(1, 3));

        assert_eq!(leaf.entries[0].key, 10);
        assert_eq!(leaf.entries[1].key, 20);
        assert_eq!(leaf.entries[2].key, 30);
    }

    #[test]
    fn gets_record_id_by_key() {
        let mut leaf = IndexLeaf::new();
        leaf.insert(10, rid(7, 1));

        assert_eq!(leaf.get(10), Some(rid(7, 1)));
        assert_eq!(leaf.get(99), None);
    }

    #[test]
    fn duplicate_key_updates_record_id() {
        let mut leaf = IndexLeaf::new();
        leaf.insert(10, rid(7, 1));
        leaf.insert(10, rid(8, 2));

        assert_eq!(leaf.entries.len(), 1);
        assert_eq!(leaf.get(10), Some(rid(8, 2)));
    }
}
