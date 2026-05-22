// Step 2-4: Split a full B+Tree internal node.
//
// Run:
// rustc --edition=2021 --test 04_split_internal.rs && ./04_split_internal

#![allow(unused)]

const MAX_KEYS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Internal {
    keys: Vec<i32>,
    children: Vec<usize>,
}

impl Internal {
    fn new(keys: Vec<i32>, children: Vec<usize>) -> Self {
        Self { keys, children }
    }
}

// Split an internal node and return:
// - separator key to move up into the parent
// - right internal node
//
// Unlike a B+Tree leaf split, the separator key is removed from the internal
// node level and moved into the parent.
fn split_internal(node: &mut Internal) -> (i32, Internal) {
    let sep_idx = node.keys.len() / 2;
    let right_keys = node.keys.split_off(sep_idx + 1);
    let sep_key = node.keys.pop().unwrap(); // unwrap -> match { OK(v) => {..}, Err(_) => panic! }, つまり即クラッシュ
    let right_children = node.children.split_off(sep_idx + 1);
    (
        sep_key,
        Internal {
            keys: right_keys,
            children: right_children,
        },
    )
}

fn main() {
    let mut node = Internal::new(vec![10, 20, 30, 40], vec![0, 1, 2, 3, 4]);
    println!("{:?}", split_internal(&mut node));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_internal_node() {
        let mut node = Internal::new(vec![10, 20, 30, 40], vec![0, 1, 2, 3, 4]);

        let (separator, right) = split_internal(&mut node);

        assert_eq!(separator, 30);
        assert_eq!(node.keys, vec![10, 20]);
        assert_eq!(node.children, vec![0, 1, 2]);
        assert_eq!(right.keys, vec![40]);
        assert_eq!(right.children, vec![3, 4]);
    }

    #[test]
    fn separator_does_not_remain_in_children() {
        let mut node = Internal::new(vec![10, 20, 30, 40], vec![0, 1, 2, 3, 4]);

        let (separator, right) = split_internal(&mut node);

        assert!(!node.keys.contains(&separator));
        assert!(!right.keys.contains(&separator));
    }

    #[test]
    fn both_sides_keep_child_count_invariant() {
        let mut node = Internal::new(vec![10, 20, 30, 40], vec![0, 1, 2, 3, 4]);

        let (_, right) = split_internal(&mut node);

        assert_eq!(node.children.len(), node.keys.len() + 1);
        assert_eq!(right.children.len(), right.keys.len() + 1);
    }
}
