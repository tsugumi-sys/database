// Step 6-3: Split a full leaf page into two leaf pages.
//
// Run:
// rustc --edition=2021 --test 03_leaf_split.rs && ./03_leaf_split

#![allow(unused)]

const LEAF_MAX_KEYS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    key: i64,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LeafPage {
    page_id: PageId,
    entries: Vec<Entry>,
    next: Option<PageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SplitResult {
    separator_key: i64,
    right: LeafPage,
}

impl LeafPage {
    fn new(page_id: PageId) -> Self {
        Self {
            page_id,
            entries: Vec::new(),
            next: None,
        }
    }

    fn insert_sorted(&mut self, key: i64, value: i64) {
        let count = self.entries.len();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            if self.entries[mid].key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left < count && self.entries[left].key == key {
            self.entries[left].value = value;
            return;
        } else {
            self.entries.insert(left, Entry { key, value });
        }
    }

    fn split(&mut self, right_page_id: PageId) -> SplitResult {
        let mut new_page = LeafPage::new(right_page_id);
        new_page.next = self.next;
        self.next = Some(right_page_id);
        let split_at = self.entries.len() / 2;
        let right_entries = self.entries.split_off(split_at);
        new_page.entries = right_entries;
        SplitResult {
            separator_key: new_page.entries[0].key,
            right: new_page,
        }
    }
}

fn main() {
    println!("leaf max keys = {}", LEAF_MAX_KEYS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_moves_upper_half_to_right_leaf() {
        let mut left = LeafPage::new(PageId::new(1));
        for key in [10, 20, 30, 40, 50] {
            left.insert_sorted(key, key * 10);
        }

        let split = left.split(PageId::new(2));

        assert_eq!(
            left.entries,
            vec![
                Entry {
                    key: 10,
                    value: 100
                },
                Entry {
                    key: 20,
                    value: 200
                },
            ]
        );
        assert_eq!(
            split.right.entries,
            vec![
                Entry {
                    key: 30,
                    value: 300
                },
                Entry {
                    key: 40,
                    value: 400
                },
                Entry {
                    key: 50,
                    value: 500
                },
            ]
        );
        assert_eq!(split.separator_key, 30);
    }

    #[test]
    fn split_updates_sibling_pointers() {
        let mut left = LeafPage::new(PageId::new(1));
        left.next = Some(PageId::new(9));
        for key in [10, 20, 30, 40, 50] {
            left.insert_sorted(key, key);
        }

        let split = left.split(PageId::new(2));

        assert_eq!(left.next, Some(PageId::new(2)));
        assert_eq!(split.right.next, Some(PageId::new(9)));
    }
}
