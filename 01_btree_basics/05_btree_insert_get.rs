// Step 1-5: Implement insert/get for an in-memory B-Tree.
//
// Run:
// rustc --edition=2021 --test 05_btree_insert_get.rs && ./05_btree_insert_get

#![allow(unused)]

const T: usize = 2;
const MAX_KEYS: usize = 2 * T - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    keys: Vec<i32>,
    values: Vec<String>,
    children: Vec<Node>,
    leaf: bool,
}

impl Node {
    fn empty_leaf() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            leaf: true,
        }
    }

    fn empty_internal() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            leaf: false,
        }
    }

    fn is_full(&self) -> bool {
        self.keys.len() == MAX_KEYS
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BTree {
    root: Node,
}

impl BTree {
    fn new() -> Self {
        Self {
            root: Node::empty_leaf(),
        }
    }

    fn get(&self, key: i32) -> Option<&str> {
        let mut current = &self.root;
        loop {
            match current.keys.binary_search(&key) {
                Ok(index) => {
                    return Some(&current.values[index]); //  String は str に deref できるので、&String は &str に自動変換される。
                }
                Err(child_index) => {
                    if current.leaf {
                        return None;
                    }
                    current = &current.children[child_index];
                }
            }
        }
    }

    fn insert(&mut self, key: i32, value: String) {
        if self.root.is_full() {
            let old_root = std::mem::replace(&mut self.root, Node::empty_internal());
            self.root.children.push(old_root);
            split_child(&mut self.root, 0);
        }
        insert_non_full(&mut self.root, key, value);
    }
}

fn split_child(parent: &mut Node, child_index: usize) {
    let child = parent.children.remove(child_index);
    let mid = child.keys.len() / 2;

    let mut left = Node {
        keys: Vec::new(),
        values: Vec::new(),
        children: Vec::new(),
        leaf: child.leaf,
    };
    let mut right = Node {
        keys: Vec::new(),
        values: Vec::new(),
        children: Vec::new(),
        leaf: child.leaf,
    };
    for i in 0..mid {
        left.keys.push(child.keys[i]);
        left.values.push(child.values[i].clone());
    }
    for i in mid + 1..child.keys.len() {
        right.keys.push(child.keys[i]);
        right.values.push(child.values[i].clone());
    }
    if !child.leaf {
        for i in 0..=mid {
            left.children.push(child.children[i].clone());
        }
        for i in mid + 1..child.children.len() {
            right.children.push(child.children[i].clone());
        }
    }
    parent.keys.insert(child_index, child.keys[mid]);
    parent.values.insert(child_index, child.values[mid].clone());
    parent.children.insert(child_index, right);
    parent.children.insert(child_index, left);
}

fn insert_non_full(node: &mut Node, key: i32, value: String) {
    let mut child_index = match node.keys.binary_search(&key) {
        Ok(index) => {
            node.values[index] = value;
            return;
        }
        Err(index) => index,
    };
    if node.leaf {
        node.keys.insert(child_index, key);
        node.values.insert(child_index, value);
        return;
    }
    // check proper child pointer.
    if node.children[child_index].is_full() {
        split_child(node, child_index);
        match node.keys[child_index].cmp(&key) {
            std::cmp::Ordering::Less => {
                child_index += 1;
            }
            std::cmp::Ordering::Equal => {
                node.values[child_index] = value;
                return;
            }
            std::cmp::Ordering::Greater => {}
        }
    }

    insert_non_full(&mut node.children[child_index], key, value);
}

fn main() {
    let mut tree = BTree::new();
    tree.insert(10, "ten".to_string());
    println!("{:?}", tree.get(10));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_sorted_and_bounded(node: &Node, is_root: bool) {
        assert!(node.keys.windows(2).all(|w| w[0] < w[1]));
        assert!(node.keys.len() <= MAX_KEYS);
        if !is_root {
            assert!(node.keys.len() >= T - 1);
        }
        if node.leaf {
            assert!(node.children.is_empty());
            assert_eq!(node.keys.len(), node.values.len());
        } else {
            assert_eq!(node.keys.len(), node.values.len());
            assert_eq!(node.children.len(), node.keys.len() + 1);
            for child in &node.children {
                assert_sorted_and_bounded(child, false);
            }
        }
    }

    #[test]
    fn inserts_and_gets_single_key() {
        let mut tree = BTree::new();

        tree.insert(10, "ten".to_string());

        assert_eq!(tree.get(10), Some("ten"));
        assert_eq!(tree.get(99), None);
    }

    #[test]
    fn inserts_many_keys_and_keeps_tree_valid() {
        let mut tree = BTree::new();

        for key in [10, 20, 5, 6, 12, 30, 7, 17] {
            tree.insert(key, format!("v{key}"));
        }

        for key in [5, 6, 7, 10, 12, 17, 20, 30] {
            assert_eq!(tree.get(key), Some(format!("v{key}").as_str()));
        }
        assert_eq!(tree.get(999), None);
        assert_sorted_and_bounded(&tree.root, true);
    }

    #[test]
    fn splits_root() {
        let mut tree = BTree::new();

        tree.insert(10, "ten".to_string());
        tree.insert(20, "twenty".to_string());
        tree.insert(30, "thirty".to_string());
        tree.insert(40, "forty".to_string());

        assert!(!tree.root.leaf);
        assert_eq!(tree.root.children.len(), tree.root.keys.len() + 1);
        assert_eq!(tree.get(10), Some("ten"));
        assert_eq!(tree.get(20), Some("twenty"));
        assert_eq!(tree.get(30), Some("thirty"));
        assert_eq!(tree.get(40), Some("forty"));
        assert_sorted_and_bounded(&tree.root, true);
    }

    #[test]
    fn overwrites_existing_key() {
        let mut tree = BTree::new();

        tree.insert(10, "old".to_string());
        tree.insert(10, "new".to_string());

        assert_eq!(tree.get(10), Some("new"));
        assert_sorted_and_bounded(&tree.root, true);
    }
}
