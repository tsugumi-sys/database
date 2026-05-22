// Step 2-2: Range scan across linked B+Tree leaves.
//
// Run:
// rustc --edition=2021 --test 02_range_scan.rs && ./02_range_scan

#![allow(unused)]

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BPlusTree {
    // For this exercise, leaves are stored in a Vec and linked by index.
    leaves: Vec<Leaf>,
    first_leaf: Option<usize>,
}

impl BPlusTree {
    fn new(leaves: Vec<Leaf>, first_leaf: Option<usize>) -> Self {
        Self { leaves, first_leaf }
    }

    // Return all key-value pairs where start <= key < end.
    fn range_scan(&self, start: i32, end: i32) -> Vec<(i32, String)> {
        let mut range_items = Vec::new();
        let mut current_index = self.first_leaf;
        while let Some(idx) = current_index {
            let leaf = &self.leaves[idx];
            for i in 0..leaf.keys.len() {
                let key = leaf.keys[i];
                if key < start {
                    continue;
                }
                if key >= end {
                    return range_items;
                }
                range_items.push((key, leaf.values[i].clone()));
            }
            current_index = leaf.next;
        }
        range_items
    }
}

fn main() {
    let tree = BPlusTree::new(
        vec![
            Leaf::new(vec![1, 5], vec!["one", "five"], Some(1)),
            Leaf::new(vec![10, 20], vec!["ten", "twenty"], None),
        ],
        Some(0),
    );
    println!("{:?}", tree.range_scan(5, 21));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> BPlusTree {
        BPlusTree::new(
            vec![
                Leaf::new(vec![1, 5, 9], vec!["one", "five", "nine"], Some(1)),
                Leaf::new(vec![10, 20], vec!["ten", "twenty"], Some(2)),
                Leaf::new(vec![30, 40], vec!["thirty", "forty"], None),
            ],
            Some(0),
        )
    }

    #[test]
    fn scans_within_one_leaf() {
        let tree = sample_tree();

        assert_eq!(
            tree.range_scan(2, 10),
            vec![(5, "five".to_string()), (9, "nine".to_string())]
        );
    }

    #[test]
    fn scans_across_leaves() {
        let tree = sample_tree();

        assert_eq!(
            tree.range_scan(5, 31),
            vec![
                (5, "five".to_string()),
                (9, "nine".to_string()),
                (10, "ten".to_string()),
                (20, "twenty".to_string()),
                (30, "thirty".to_string()),
            ]
        );
    }

    #[test]
    fn excludes_end_key() {
        let tree = sample_tree();

        assert_eq!(tree.range_scan(10, 20), vec![(10, "ten".to_string())]);
    }

    #[test]
    fn empty_range_returns_empty_vec() {
        let tree = sample_tree();

        assert_eq!(tree.range_scan(20, 20), Vec::<(i32, String)>::new());
    }
}
