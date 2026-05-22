// Step 2-1: Point lookup in an in-memory B+Tree.
//
// Run:
// rustc --edition=2021 --test 01_point_lookup.rs && ./01_point_lookup

#![allow(unused)]

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Internal { keys: Vec<i32>, children: Vec<Node> },
    Leaf { keys: Vec<i32>, values: Vec<String> },
}

impl Node {
    fn leaf(keys: Vec<i32>, values: Vec<&str>) -> Self {
        Self::Leaf {
            keys,
            values: values.into_iter().map(String::from).collect(),
        }
    }

    fn internal(keys: Vec<i32>, children: Vec<Node>) -> Self {
        Self::Internal { keys, children }
    }
}

// Return the value for key.
//
// In a B+Tree, internal keys are separators only. Values are stored in leaves.
// When key is equal to a separator key, descend to the right child.
fn get(node: &Node, key: i32) -> Option<&str> {
    let mut current = node;
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

fn main() {
    let tree = Node::internal(
        vec![10],
        vec![
            Node::leaf(vec![1, 5], vec!["one", "five"]),
            Node::leaf(vec![10, 20], vec!["ten", "twenty"]),
        ],
    );
    println!("{:?}", get(&tree, 10));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_value_from_leaf() {
        let tree = Node::leaf(vec![10, 20, 30], vec!["ten", "twenty", "thirty"]);

        assert_eq!(get(&tree, 20), Some("twenty"));
    }

    #[test]
    fn returns_none_when_missing_from_leaf() {
        let tree = Node::leaf(vec![10, 20, 30], vec!["ten", "twenty", "thirty"]);

        assert_eq!(get(&tree, 25), None);
    }

    #[test]
    fn descends_through_internal_node() {
        let tree = Node::internal(
            vec![10, 30],
            vec![
                Node::leaf(vec![1, 5], vec!["one", "five"]),
                Node::leaf(vec![10, 20], vec!["ten", "twenty"]),
                Node::leaf(vec![30, 40], vec!["thirty", "forty"]),
            ],
        );

        assert_eq!(get(&tree, 1), Some("one"));
        assert_eq!(get(&tree, 20), Some("twenty"));
        assert_eq!(get(&tree, 40), Some("forty"));
        assert_eq!(get(&tree, 99), None);
    }

    #[test]
    fn equal_separator_goes_right() {
        let tree = Node::internal(
            vec![10],
            vec![
                Node::leaf(vec![1, 5], vec!["one", "five"]),
                Node::leaf(vec![10, 20], vec!["ten", "twenty"]),
            ],
        );

        assert_eq!(get(&tree, 10), Some("ten"));
    }
}
