// Step 1-3: Split a full child node.
//
// Run:
// rustc --edition=2021 --test 03_split_child.rs && ./03_split_child

#![allow(unused)]

const T: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    keys: Vec<i32>,
    values: Vec<String>,
    children: Vec<Node>,
    leaf: bool,
}

impl Node {
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
}

// Split parent.children[child_index].
//
// For T = 2, a full child has 3 keys:
// [10, 20, 30] becomes:
// - left child:  [10]
// - parent key:  20
// - right child: [30]
//
// If the child is internal, its children must also be split.
fn split_child(parent: &mut Node, child_index: usize) {
    let child = parent.children.remove(child_index);

    let mid = child.keys.len() / 2;
    let mid_key = child.keys[mid];
    let mid_val = child.values[mid].clone(); // String type does not have Copy, so we need to clone it.

    let mut left = Node {
        keys: Vec::new(),
        values: Vec::new(),
        children: Vec::new(),
        leaf: child.leaf, // childがleafかどうかはわからない。だから引き継げばいい。
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
    // childrenのsplitも忘れずに!
    if !child.leaf {
        for i in 0..=mid {
            left.children.push(child.children[i].clone());
        }

        for i in mid + 1..child.children.len() {
            right.children.push(child.children[i].clone());
        }
    }

    parent.keys.insert(child_index, mid_key);
    parent.values.insert(child_index, mid_val);
    parent.children.insert(child_index, right);
    parent.children.insert(child_index, left);
}

fn main() {
    let child = Node::leaf(vec![10, 20, 30], vec!["ten", "twenty", "thirty"]);
    let mut parent = Node::internal(vec![], vec![], vec![child]);
    split_child(&mut parent, 0);
    println!("{:?}", parent);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_leaf_child() {
        let child = Node::leaf(vec![10, 20, 30], vec!["ten", "twenty", "thirty"]);
        let mut parent = Node::internal(vec![], vec![], vec![child]);

        split_child(&mut parent, 0);

        assert_eq!(parent.keys, vec![20]);
        assert_eq!(parent.values, vec!["twenty"]);
        assert_eq!(parent.children.len(), 2);

        assert_eq!(parent.children[0].keys, vec![10]);
        assert_eq!(parent.children[0].values, vec!["ten"]);

        assert_eq!(parent.children[1].keys, vec![30]);
        assert_eq!(parent.children[1].values, vec!["thirty"]);
    }

    #[test]
    fn splits_internal_child() {
        let c0 = Node::leaf(vec![1], vec!["one"]);
        let c1 = Node::leaf(vec![11], vec!["eleven"]);
        let c2 = Node::leaf(vec![21], vec!["twenty-one"]);
        let c3 = Node::leaf(vec![31], vec!["thirty-one"]);
        let child = Node::internal(
            vec![10, 20, 30],
            vec!["ten", "twenty", "thirty"],
            vec![c0, c1, c2, c3],
        );
        let mut parent = Node::internal(vec![], vec![], vec![child]);

        split_child(&mut parent, 0);

        assert_eq!(parent.keys, vec![20]);
        assert_eq!(parent.values, vec!["twenty"]);
        assert_eq!(parent.children.len(), 2);

        assert_eq!(parent.children[0].keys, vec![10]);
        assert_eq!(parent.children[0].values, vec!["ten"]);
        assert_eq!(parent.children[0].children.len(), 2);
        assert_eq!(parent.children[0].children[0].keys, vec![1]);
        assert_eq!(parent.children[0].children[1].keys, vec![11]);

        assert_eq!(parent.children[1].keys, vec![30]);
        assert_eq!(parent.children[1].values, vec!["thirty"]);
        assert_eq!(parent.children[1].children.len(), 2);
        assert_eq!(parent.children[1].children[0].keys, vec![21]);
        assert_eq!(parent.children[1].children[1].keys, vec![31]);
    }

    #[test]
    fn inserts_median_at_requested_child_index() {
        let left = Node::leaf(vec![1], vec!["one"]);
        let right = Node::leaf(vec![30, 40, 50], vec!["thirty", "forty", "fifty"]);
        let mut parent = Node::internal(vec![20], vec!["twenty"], vec![left, right]);

        split_child(&mut parent, 1);

        assert_eq!(parent.keys, vec![20, 40]);
        assert_eq!(parent.values, vec!["twenty", "forty"]);
        assert_eq!(parent.children.len(), 3);
        assert_eq!(parent.children[1].keys, vec![30]);
        assert_eq!(parent.children[2].keys, vec![50]);
    }
}
