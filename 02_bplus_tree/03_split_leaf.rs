// Step 2-3: Split a full B+Tree leaf node.
//
// Run:
// rustc --edition=2021 --test 03_split_leaf.rs && ./03_split_leaf

#![allow(unused)]

const MAX_KEYS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Leaf {
    keys: Vec<i32>,
    values: Vec<String>,
    next: Option<usize>,
}

impl Leaf {
    fn new(keys: Vec<i32>, values: Vec<&str>, next: Option<usize>) -> Self {
        Self {
            keys,
            values: values.into_iter().map(String::from).collect(),
            next,
        }
    }
}

// Split leaves[leaf_index] and append the new right leaf to leaves.
//
// Return:
// - separator key to insert into the parent
// - index of the new right leaf
//
// In a B+Tree leaf split, the separator key is copied from the right leaf's
// first key. It is not removed from the leaf.
fn split_leaf(leaves: &mut Vec<Leaf>, leaf_index: usize) -> (i32, usize) {
    let leaf = leaves[leaf_index].clone();
    let old_next = leaf.next;
    let right_index = leaves.len();

    let n = leaf.keys.len();
    let mid_idx = n / 2;
    let mid_key = leaf.keys[mid_idx];

    leaves[leaf_index] = Leaf {
        keys: leaf.keys[..mid_idx].to_vec(),
        values: leaf.values[..mid_idx].to_vec(),
        next: Some(right_index),
    };

    leaves.push(Leaf {
        keys: leaf.keys[mid_idx..].to_vec(),
        values: leaf.values[mid_idx..].to_vec(),
        next: old_next,
    });

    (mid_key, right_index)
}

fn main() {
    let mut leaves = vec![Leaf::new(
        vec![10, 20, 30, 40],
        vec!["ten", "twenty", "thirty", "forty"],
        None,
    )];
    println!("{:?}", split_leaf(&mut leaves, 0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_leaf_and_returns_right_first_key() {
        let mut leaves = vec![Leaf::new(
            vec![10, 20, 30, 40],
            vec!["ten", "twenty", "thirty", "forty"],
            None,
        )];

        let (separator, right_index) = split_leaf(&mut leaves, 0);

        assert_eq!(separator, 30);
        assert_eq!(right_index, 1);
        assert_eq!(leaves[0].keys, vec![10, 20]);
        assert_eq!(leaves[0].values, vec!["ten", "twenty"]);
        assert_eq!(leaves[1].keys, vec![30, 40]);
        assert_eq!(leaves[1].values, vec!["thirty", "forty"]);
    }

    #[test]
    fn links_left_leaf_to_new_right_leaf() {
        let mut leaves = vec![Leaf::new(
            vec![10, 20, 30, 40],
            vec!["ten", "twenty", "thirty", "forty"],
            None,
        )];

        let (_, right_index) = split_leaf(&mut leaves, 0);

        assert_eq!(leaves[0].next, Some(right_index));
        assert_eq!(leaves[1].next, None);
    }

    #[test]
    fn preserves_old_next_link_on_new_right_leaf() {
        let mut leaves = vec![
            Leaf::new(
                vec![10, 20, 30, 40],
                vec!["ten", "twenty", "thirty", "forty"],
                Some(1),
            ),
            Leaf::new(vec![50, 60], vec!["fifty", "sixty"], None),
        ];

        let (_, right_index) = split_leaf(&mut leaves, 0);

        assert_eq!(leaves[0].next, Some(right_index));
        assert_eq!(leaves[right_index].next, Some(1));
    }
}
