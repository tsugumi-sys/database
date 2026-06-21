// Step 5-5: Exercise leaf and internal node pages together.
//
// Run:
// rustc --edition=2021 --test 05_node_page_smoke.rs && ./05_node_page_smoke

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const NODE_TYPE_OFFSET: usize = 0;
const KEY_COUNT_OFFSET: usize = 1;
const PARENT_PAGE_ID_OFFSET: usize = 3;
const NEXT_PAGE_ID_OFFSET: usize = 7;
const NODE_HEADER_SIZE: usize = 11;
const LEAF_CELL_SIZE: usize = 16;
const NO_PAGE: u32 = u32::MAX;
const INTERNAL_CELL_SIZE: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeType {
    Leaf,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new_node(node_type: NodeType) -> Self {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };
        match node_type {
            NodeType::Leaf => {
                page.data[NODE_TYPE_OFFSET] = 1;
            }
            NodeType::Internal => {
                page.data[NODE_TYPE_OFFSET] = 2;
            }
        }
        page
    }

    fn key_count(&self) -> usize {
        let bytes: [u8; 2] = self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 2]
            .try_into()
            .unwrap();
        u16::from_le_bytes(bytes).into()
    }

    fn set_key_count(&mut self, count: usize) {
        self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 2]
            .copy_from_slice(&(count as u16).to_le_bytes());
    }

    fn set_next_page_id(&mut self, page_id: Option<PageId>) {
        if let Some(page_id) = page_id {
            self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
                .copy_from_slice(&page_id.0.to_le_bytes());
        }
    }

    fn next_page_id(&self) -> Option<PageId> {
        let bytes: [u8; 4] = self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
            .try_into()
            .unwrap();
        Some(PageId::new(u32::from_le_bytes(bytes)))
    }

    fn leaf_cell_offset(index: usize) -> usize {
        NODE_HEADER_SIZE + index * LEAF_CELL_SIZE
    }

    fn leaf_insert(&mut self, key: i64, value: i64) -> Result<(), String> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;

        while left < right {
            let mid = (left + right) / 2;
            // let mid_key = self.leaf_get(mid as i64).unwrap();
            let offset = Page::leaf_cell_offset(mid);
            let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
            let mid_key = i64::from_le_bytes(bytes);
            if mid_key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        if left < count && self.leaf_get(left as i64).unwrap() == key {
            let offset = Page::leaf_cell_offset(left);
            self.data[offset + 8..offset + LEAF_CELL_SIZE].copy_from_slice(&value.to_le_bytes());
            return Ok(());
        }

        if count >= (PAGE_SIZE - NODE_HEADER_SIZE) / LEAF_CELL_SIZE {
            return Err("page is full".to_string());
        }

        // shift all entries
        for i in (left..count).rev() {
            let offset = Page::leaf_cell_offset(i);
            let new_offset = Page::leaf_cell_offset(i + 1);
            self.data.copy_within(offset..offset + 16, new_offset);
        }

        let new_offset = Page::leaf_cell_offset(left);
        self.data[new_offset..new_offset + 8].copy_from_slice(&key.to_le_bytes());
        self.data[new_offset + 8..new_offset + 16].copy_from_slice(&value.to_le_bytes());
        self.set_key_count(count + 1);
        Ok(())
    }

    fn leaf_get(&self, key: i64) -> Option<i64> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            let offset = Page::leaf_cell_offset(mid);
            let key_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
            let k = i64::from_le_bytes(key_bytes);
            if k < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left < count {
            let offset = Page::leaf_cell_offset(left);
            let key_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
            let k = i64::from_le_bytes(key_bytes);
            if k == key {
                let value_bytes: [u8; 8] = self.data[offset + 8..offset + 16].try_into().unwrap();
                return Some(i64::from_le_bytes(value_bytes));
            }
        }
        None
    }

    fn init_internal(&mut self, leftmost_child: PageId) {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };
        page.data[NODE_TYPE_OFFSET] = 2;
        self.data[NODE_HEADER_SIZE..NODE_HEADER_SIZE + 4]
            .copy_from_slice(&leftmost_child.0.to_le_bytes());
    }

    fn internal_cell_offset(index: usize) -> usize {
        NODE_HEADER_SIZE + 4 + index * INTERNAL_CELL_SIZE
    }

    fn internal_insert(&mut self, key: i64, right_child: PageId) -> Result<(), String> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            let offset = Page::internal_cell_offset(mid);
            let key_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
            let k = i64::from_le_bytes(key_bytes);
            if k < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        if count >= (PAGE_SIZE - NODE_HEADER_SIZE) / INTERNAL_CELL_SIZE {
            return Err("this page is full".to_string());
        }

        if left < count {
            let offset = Page::internal_cell_offset(left);
            let k_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
            let k = i64::from_le_bytes(k_bytes);
            if k == key {
                self.data[offset + 8..offset + 12].copy_from_slice(&right_child.0.to_le_bytes());
                return Ok(());
            }
        }

        // shift all entries
        for i in (left..count).rev() {
            let offset = Page::internal_cell_offset(i);
            let new_offset = Page::internal_cell_offset(i + 1);
            self.data.copy_within(offset..offset + 12, new_offset);
        }

        let new_offset = Page::internal_cell_offset(left);
        self.data[new_offset..new_offset + 8].copy_from_slice(&key.to_le_bytes());
        self.data[new_offset + 8..new_offset + 12].copy_from_slice(&right_child.0.to_le_bytes());
        self.set_key_count(count + 1);
        Ok(())
    }

    fn child_for_key(&self, key: i64) -> PageId {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            let offset = Page::internal_cell_offset(mid);
            let k_bytes: [u8; 8] = self.data[offset..offset + 8].try_into().unwrap();
            let k = i64::from_le_bytes(k_bytes);
            if k <= key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left == 0 {
            let pid_bytes: [u8; 4] = self.data[NODE_HEADER_SIZE..NODE_HEADER_SIZE + 4]
                .try_into()
                .unwrap();
            return PageId::new(u32::from_le_bytes(pid_bytes));
        }
        let offset = Page::internal_cell_offset(left - 1);
        let pid_bytes: [u8; 4] = self.data[offset + 8..offset + 12].try_into().unwrap();
        PageId::new(u32::from_le_bytes(pid_bytes))
    }
}

fn main() {
    println!("node page smoke test");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_pages_store_values_and_siblings() {
        let mut left = Page::new_node(NodeType::Leaf);
        let mut right = Page::new_node(NodeType::Leaf);

        left.leaf_insert(10, 100).unwrap();
        left.leaf_insert(20, 200).unwrap();
        right.leaf_insert(30, 300).unwrap();
        left.set_next_page_id(Some(PageId::new(2)));

        assert_eq!(left.leaf_get(10), Some(100));
        assert_eq!(left.leaf_get(30), None);
        assert_eq!(right.leaf_get(30), Some(300));
        assert_eq!(left.next_page_id(), Some(PageId::new(2)));
    }

    #[test]
    fn internal_page_routes_to_leaf_page_ids() {
        let mut root = Page::new_node(NodeType::Internal);
        root.init_internal(PageId::new(1));
        root.internal_insert(30, PageId::new(2)).unwrap();
        root.internal_insert(60, PageId::new(3)).unwrap();

        assert_eq!(root.child_for_key(10), PageId::new(1));
        assert_eq!(root.child_for_key(30), PageId::new(2));
        assert_eq!(root.child_for_key(59), PageId::new(2));
        assert_eq!(root.child_for_key(60), PageId::new(3));
    }
}
