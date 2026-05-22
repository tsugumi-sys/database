// Step 2-5: Implement insert/get/range scan for an in-memory B+Tree.
//
// Run:
// rustc --edition=2021 --test 05_bplus_tree_insert_range.rs && ./05_bplus_tree_insert_range

#![allow(unused)]

const MAX_KEYS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Internal { keys: Vec<i32>, children: Vec<Node> },
    Leaf { keys: Vec<i32>, values: Vec<String> },
}

impl Node {
    fn empty_leaf() -> Self {
        Self::Leaf {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    fn key_count(&self) -> usize {
        match self {
            Node::Internal { keys, .. } | Node::Leaf { keys, .. } => keys.len(),
        }
    }

    fn is_full(&self) -> bool {
        self.key_count() > MAX_KEYS
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BPlusTree {
    root: Node,
}

impl BPlusTree {
    fn new() -> Self {
        Self {
            root: Node::empty_leaf(),
        }
    }

    fn get(&self, key: i32) -> Option<&str> {
        let mut current = &self.root;
        loop {
            match current {
                Node::Internal { keys, children } => match keys.binary_search(&key) {
                    Ok(idx) => {
                        current = &children[idx + 1];
                    }
                    Err(idx) => {
                        current = &children[idx];
                    }
                },
                Node::Leaf { keys, values } => match keys.binary_search(&key) {
                    Ok(idx) => {
                        return Some(&values[idx]);
                    }
                    Err(_) => {
                        return None;
                    }
                },
            }
        }
    }

    fn insert(&mut self, key: i32, value: String) {
        if let Some((separator, right)) = insert_into(&mut self.root, key, value) {
            let old_root = std::mem::replace(&mut self.root, Node::empty_leaf());
            self.root = Node::Internal {
                keys: vec![separator],
                children: vec![old_root, right],
            };
        }
    }

    fn range_scan(&self, start: i32, end: i32) -> Vec<(i32, String)> {
        let mut result = Vec::new();
        collect_range(&self.root, start, end, &mut result);
        result
    }
}

fn insert_into(node: &mut Node, key: i32, value: String) -> Option<(i32, Node)> {
    match node {
        Node::Leaf { keys, values } => {
            match keys.binary_search(&key) {
                Ok(idx) => {
                    values[idx] = value;
                    None
                }
                Err(idx) => {
                    values.insert(idx, value);
                    keys.insert(idx, key);

                    if keys.len() > MAX_KEYS {
                        let mid_idx = keys.len() / 2;
                        let right_keys = keys.split_off(mid_idx);
                        let right_values = values.split_off(mid_idx);
                        let separator = right_keys[0];
                        let right = Node::Leaf {
                            keys: right_keys,
                            values: right_values,
                        };

                        Some((separator, right))
                    } else {
                        None
                    }
                }
            }
        }
        Node::Internal { keys, children } => {
            let child_idx = match keys.binary_search(&key) {
                Ok(idx) => idx + 1,
                Err(idx) => idx,
            };

            if let Some((separator, right)) = insert_into(&mut children[child_idx], key, value) {
                keys.insert(child_idx, separator);
                children.insert(child_idx + 1, right);

                if keys.len() > MAX_KEYS {
                    let sep_idx = keys.len() / 2;
                    let right_keys = keys.split_off(sep_idx + 1);
                    let sep_key = keys
                        .pop()
                        .expect("internal split requires at least one separator key");
                    let right_children = children.split_off(sep_idx + 1);
                    let right = Node::Internal {
                        keys: right_keys,
                        children: right_children,
                    };

                    Some((sep_key, right))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

fn collect_range(node: &Node, start: i32, end: i32, result: &mut Vec<(i32, String)>) {
    match node {
        Node::Leaf { keys, values } => {
            for (key, value) in keys.iter().zip(values) {
                if start <= *key && *key < end {
                    result.push((*key, value.clone()));
                }
            }
        }
        Node::Internal { children, .. } => {
            for child in children {
                collect_range(child, start, end, result);
            }
        }
    }
}

fn main() {
    let mut tree = BPlusTree::new();
    tree.insert(10, "ten".to_string());
    println!("{:?}", tree.get(10));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_shape(node: &Node, is_root: bool) {
        match node {
            Node::Leaf { keys, values } => {
                assert!(keys.windows(2).all(|w| w[0] < w[1]));
                assert_eq!(keys.len(), values.len());
                assert!(keys.len() <= MAX_KEYS);
                if !is_root {
                    assert!(!keys.is_empty());
                }
            }
            Node::Internal { keys, children } => {
                assert!(keys.windows(2).all(|w| w[0] < w[1]));
                assert!(keys.len() <= MAX_KEYS);
                assert_eq!(children.len(), keys.len() + 1);
                if !is_root {
                    assert!(!keys.is_empty());
                }
                for child in children {
                    assert_valid_shape(child, false);
                }
            }
        }
    }

    #[test]
    fn inserts_and_gets_single_key() {
        let mut tree = BPlusTree::new();

        tree.insert(10, "ten".to_string());

        assert_eq!(tree.get(10), Some("ten"));
        assert_eq!(tree.get(99), None);
    }

    #[test]
    fn inserts_many_keys_and_keeps_tree_valid() {
        let mut tree = BPlusTree::new();

        for key in [10, 20, 5, 6, 12, 30, 7, 17, 3, 25] {
            tree.insert(key, format!("v{key}"));
        }

        for key in [3, 5, 6, 7, 10, 12, 17, 20, 25, 30] {
            assert_eq!(tree.get(key), Some(format!("v{key}").as_str()));
        }
        assert_eq!(tree.get(999), None);
        assert_valid_shape(&tree.root, true);
    }

    #[test]
    fn overwrites_existing_key() {
        let mut tree = BPlusTree::new();

        tree.insert(10, "ten".to_string());
        tree.insert(10, "TEN".to_string());

        assert_eq!(tree.get(10), Some("TEN"));
        assert_valid_shape(&tree.root, true);
    }

    #[test]
    fn range_scan_returns_sorted_pairs() {
        let mut tree = BPlusTree::new();

        for key in [10, 20, 5, 6, 12, 30, 7, 17, 3, 25] {
            tree.insert(key, format!("v{key}"));
        }

        assert_eq!(
            tree.range_scan(6, 21),
            vec![
                (6, "v6".to_string()),
                (7, "v7".to_string()),
                (10, "v10".to_string()),
                (12, "v12".to_string()),
                (17, "v17".to_string()),
                (20, "v20".to_string()),
            ]
        );
    }
}
