// Step 1-1: Search inside one B-Tree node.
//
// Run:
// rustc --edition=2021 --test 01_search_node.rs && ./01_search_node

#![allow(unused)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    keys: Vec<i32>,
}

impl Node {
    fn new(keys: Vec<i32>) -> Self {
        Self { keys }
    }
}

// Return:
// - Ok(index) when key exists in node.keys
// - Err(child_index) when key does not exist and search should descend to that child
//
// Example:
// keys = [10, 20, 30]
// key 20 -> Ok(1)
// key 25 -> Err(2)
// key  5 -> Err(0)
// key 40 -> Err(3)
fn search_node(node: &Node, key: i32) -> Result<usize, usize> {
    let n = node.keys.len();
    if n == 0 {
        return Err(0);
    }
    let mut lo = 0;
    let mut hi = n;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if node.keys[mid] == key {
            return Ok(mid);
        } else if node.keys[mid] < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < n && node.keys[lo] == key {
        return Ok(lo);
    }
    Err(lo)
}

fn main() {
    let node = Node::new(vec![10, 20, 30]);
    println!("{:?}", search_node(&node, 20));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_existing_key() {
        let node = Node::new(vec![10, 20, 30]);

        assert_eq!(search_node(&node, 20), Ok(1));
    }

    #[test]
    fn returns_left_child_index() {
        let node = Node::new(vec![10, 20, 30]);

        assert_eq!(search_node(&node, 5), Err(0));
    }

    #[test]
    fn returns_middle_child_index() {
        let node = Node::new(vec![10, 20, 30]);

        assert_eq!(search_node(&node, 25), Err(2));
    }

    #[test]
    fn returns_rightmost_child_index() {
        let node = Node::new(vec![10, 20, 30]);

        assert_eq!(search_node(&node, 40), Err(3));
    }

    #[test]
    fn handles_empty_node() {
        let node = Node::new(vec![]);

        assert_eq!(search_node(&node, 10), Err(0));
    }
}
