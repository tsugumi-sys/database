// Step 1-2: Insert key-value pairs into a leaf node.
//
// Run:
// rustc --edition=2021 --test 02_insert_into_leaf.rs && ./02_insert_into_leaf

#![allow(unused)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct LeafNode {
    keys: Vec<i32>,
    values: Vec<String>,
}

impl LeafNode {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
}

// Insert key and value into the leaf.
//
// Requirements:
// - keep keys sorted
// - keep values aligned with keys
// - overwrite value when key already exists
// - do not split in this exercise
fn insert_into_leaf(leaf: &mut LeafNode, key: i32, value: String) {
    // Find insert position
    let mut lo = 0;
    let mut hi = leaf.keys.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if leaf.keys[mid] == key {
            lo = mid;
            break;
        } else if leaf.keys[mid] < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < leaf.keys.len() && leaf.keys[lo] == key {
        leaf.values[lo] = value;
    } else {
        leaf.values.insert(lo, value);
        leaf.keys.insert(lo, key);
    }
}

fn main() {
    let mut leaf = LeafNode::new();
    insert_into_leaf(&mut leaf, 10, "ten".to_string());
    println!("{:?}", leaf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_into_empty_leaf() {
        let mut leaf = LeafNode::new();

        insert_into_leaf(&mut leaf, 10, "ten".to_string());

        assert_eq!(leaf.keys, vec![10]);
        assert_eq!(leaf.values, vec!["ten"]);
    }

    #[test]
    fn keeps_keys_sorted() {
        let mut leaf = LeafNode::new();

        insert_into_leaf(&mut leaf, 20, "twenty".to_string());
        insert_into_leaf(&mut leaf, 10, "ten".to_string());
        insert_into_leaf(&mut leaf, 30, "thirty".to_string());

        assert_eq!(leaf.keys, vec![10, 20, 30]);
        assert_eq!(leaf.values, vec!["ten", "twenty", "thirty"]);
    }

    #[test]
    fn overwrites_existing_key() {
        let mut leaf = LeafNode::new();

        insert_into_leaf(&mut leaf, 10, "old".to_string());
        insert_into_leaf(&mut leaf, 10, "new".to_string());

        assert_eq!(leaf.keys, vec![10]);
        assert_eq!(leaf.values, vec!["new"]);
    }
}
