// Step 5-4: Binary search sorted keys stored in a page.
//
// Run:
// rustc --edition=2021 --test 04_page_binary_search.rs && ./04_page_binary_search

#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchResult {
    Found(usize),
    InsertAt(usize),
}

trait PageKeys {
    fn key_count(&self) -> usize;
    fn key_at(&self, index: usize) -> i64;
}

fn binary_search_page<K: PageKeys>(keys: &K, target: i64) -> SearchResult {
    let count = keys.key_count();
    let mut left = 0;
    let mut right = count;
    while left < right {
        let mid = (left + right) / 2;
        if keys.key_at(mid) < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    if left < count && keys.key_at(left) == target {
        return SearchResult::Found(left);
    }
    SearchResult::InsertAt(left)
}

#[derive(Debug, Clone)]
struct TestKeys {
    keys: Vec<i64>,
}

impl PageKeys for TestKeys {
    fn key_count(&self) -> usize {
        self.keys.len()
    }

    fn key_at(&self, index: usize) -> i64 {
        self.keys[index]
    }
}

fn main() {
    println!("binary search page keys");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_existing_keys() {
        let keys = TestKeys {
            keys: vec![10, 20, 30, 40],
        };

        assert_eq!(binary_search_page(&keys, 10), SearchResult::Found(0));
        assert_eq!(binary_search_page(&keys, 30), SearchResult::Found(2));
        assert_eq!(binary_search_page(&keys, 40), SearchResult::Found(3));
    }

    #[test]
    fn returns_insert_position_for_missing_keys() {
        let keys = TestKeys {
            keys: vec![10, 20, 30, 40],
        };

        assert_eq!(binary_search_page(&keys, 5), SearchResult::InsertAt(0));
        assert_eq!(binary_search_page(&keys, 25), SearchResult::InsertAt(2));
        assert_eq!(binary_search_page(&keys, 99), SearchResult::InsertAt(4));
    }

    #[test]
    fn handles_empty_page() {
        let keys = TestKeys { keys: vec![] };

        assert_eq!(binary_search_page(&keys, 10), SearchResult::InsertAt(0));
    }

    #[test]
    fn handles_one_key_page() {
        let keys = TestKeys { keys: vec![10] };

        assert_eq!(binary_search_page(&keys, 10), SearchResult::Found(0));
        assert_eq!(binary_search_page(&keys, 9), SearchResult::InsertAt(0));
        assert_eq!(binary_search_page(&keys, 11), SearchResult::InsertAt(1));
    }
}
