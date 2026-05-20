// Step 1-4: Insert into a node that is known to be non-full.
//
// Run:
// rustc --edition=2021 --test 04_insert_non_full.rs && ./04_insert_non_full

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

    fn leaf(keys: Vec<i32>, values: Vec<&str>) -> Self {
        Self {
            keys,
            values: values.into_iter().map(String::from).collect(),
            children: Vec::new(),
            leaf: true,
        }
    }

    fn internal(keys: Vec<i32>, values: Vec<&str>, children: Vec<Node>) -> Self {
        Self {
            keys,
            values: values.into_iter().map(String::from).collect(),
            children,
            leaf: false,
        }
    }

    fn is_full(&self) -> bool {
        self.keys.len() == MAX_KEYS
    }
}

fn split_child(parent: &mut Node, child_index: usize) {
    let child = parent.children.remove(child_index);
    let mid = child.keys.len() / 2;
    let mid_val = child.values[mid].clone();

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

// Insert into node, assuming node itself is not full.
//
// Important:
// - if node is a leaf, insert key/value directly
// - if node is internal, choose the child to descend into
// - before descending, split the child if it is full
// - after splitting, choose the correct child again
fn insert_non_full(node: &mut Node, key: i32, value: String) {
    // find insersion position
    let mut lo = 0;
    let mut hi = node.keys.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if node.keys[mid] < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < node.keys.len() && node.keys[lo] == key {
        node.values[lo] = value;
        return;
    }
    if node.leaf {
        node.keys.insert(lo, key);
        node.values.insert(lo, value);
        return;
    }
    // check proper child pointer.
    let child = &mut node.children[lo];
    if child.is_full() {
        split_child(node, lo);
        if node.keys[lo] < key {
            insert_non_full(&mut node.children[lo + 1], key, value);
        } else {
            insert_non_full(&mut node.children[lo], key, value);
        }
    } else {
        insert_non_full(child, key, value);
    }
}

fn main() {
    let mut root = Node::empty_leaf();
    insert_non_full(&mut root, 10, "ten".to_string());
    println!("{:?}", root);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(node: &Node, key: i32) -> Option<&str> {
        let mut i = 0;
        while i < node.keys.len() && key > node.keys[i] {
            i += 1;
        }
        if i < node.keys.len() && key == node.keys[i] {
            return Some(&node.values[i]);
        }
        if node.leaf {
            return None;
        }
        get(&node.children[i], key)
    }

    fn assert_sorted(node: &Node) {
        assert!(node.keys.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(node.keys.len(), node.values.len());
        for child in &node.children {
            assert_sorted(child);
        }
    }

    #[test]
    fn inserts_into_non_full_leaf() {
        let mut root = Node::leaf(vec![10], vec!["ten"]);

        insert_non_full(&mut root, 5, "five".to_string());
        insert_non_full(&mut root, 20, "twenty".to_string());

        assert_eq!(root.keys, vec![5, 10, 20]);
        assert_eq!(root.values, vec!["five", "ten", "twenty"]);
    }

    #[test]
    fn descends_into_child() {
        let left = Node::leaf(vec![5], vec!["five"]);
        let right = Node::leaf(vec![20], vec!["twenty"]);
        let mut root = Node::internal(vec![10], vec!["ten"], vec![left, right]);

        insert_non_full(&mut root, 30, "thirty".to_string());

        assert_eq!(get(&root, 30), Some("thirty"));
        assert_sorted(&root);
    }

    #[test]
    fn splits_full_child_before_descending() {
        let left = Node::leaf(vec![1, 2, 3], vec!["one", "two", "three"]);
        let right = Node::leaf(vec![20], vec!["twenty"]);
        let mut root = Node::internal(vec![10], vec!["ten"], vec![left, right]);

        insert_non_full(&mut root, 4, "four".to_string());

        assert_eq!(root.keys, vec![2, 10]);
        assert_eq!(root.values, vec!["two", "ten"]);
        assert_eq!(root.children.len(), 3);
        assert_eq!(get(&root, 1), Some("one"));
        assert_eq!(get(&root, 2), Some("two"));
        assert_eq!(get(&root, 3), Some("three"));
        assert_eq!(get(&root, 4), Some("four"));
        assert_sorted(&root);
    }

    #[test]
    fn overwrites_existing_leaf_key() {
        let mut root = Node::leaf(vec![10], vec!["old"]);

        insert_non_full(&mut root, 10, "new".to_string());

        assert_eq!(get(&root, 10), Some("new"));
        assert_eq!(root.keys, vec![10]);
    }
}
