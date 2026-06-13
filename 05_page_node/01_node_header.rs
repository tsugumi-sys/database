// Step 5-1: Store a B+Tree node header in page bytes.
//
// Run:
// rustc --edition=2021 --test 01_node_header.rs && ./01_node_header

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const NODE_TYPE_OFFSET: usize = 0;
const KEY_COUNT_OFFSET: usize = 1;
const PARENT_PAGE_ID_OFFSET: usize = 3;
const NEXT_PAGE_ID_OFFSET: usize = 7;
const NODE_HEADER_SIZE: usize = 11;
const NO_PAGE: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeType {
    Leaf,
    Internal,
}

impl NodeType {
    fn to_u8(self) -> u8 {
        match self {
            NodeType::Leaf => 0,
            NodeType::Internal => 1,
        }
    }

    fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(NodeType::Leaf),
            1 => Ok(NodeType::Internal),
            _ => Err(format!("Invalid node type {}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new() -> Self {
        Self {
            data: [0; PAGE_SIZE],
        }
    }

    fn init_node(&mut self, node_type: NodeType) {
        self.data[NODE_TYPE_OFFSET] = node_type.to_u8();
        self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 2].copy_from_slice(&0u16.to_le_bytes());
        self.data[PARENT_PAGE_ID_OFFSET..PARENT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&NO_PAGE.to_le_bytes());
        self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&NO_PAGE.to_le_bytes());
    }

    fn node_type(&self) -> NodeType {
        NodeType::from_u8(self.data[NODE_TYPE_OFFSET]).expect("invalid node type")
    }

    fn key_count(&self) -> u16 {
        let bytes: [u8; 2] = self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 2]
            .try_into()
            .unwrap();
        u16::from_le_bytes(bytes)
    }

    fn set_key_count(&mut self, count: u16) {
        self.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 2].copy_from_slice(&count.to_le_bytes());
    }

    fn parent_page_id(&self) -> Option<PageId> {
        let bytes: [u8; 4] = self.data[PARENT_PAGE_ID_OFFSET..PARENT_PAGE_ID_OFFSET + 4]
            .try_into()
            .unwrap();
        match u32::from_le_bytes(bytes) {
            NO_PAGE => None,
            v => Some(PageId::new(v)),
        }
    }

    fn set_parent_page_id(&mut self, page_id: Option<PageId>) {
        let raw = match page_id {
            Some(page_id) => page_id.0,
            _ => NO_PAGE,
        };
        self.data[PARENT_PAGE_ID_OFFSET..PARENT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&raw.to_le_bytes());
    }

    fn next_page_id(&self) -> Option<PageId> {
        let bytes: [u8; 4] = self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
            .try_into()
            .unwrap();
        match u32::from_le_bytes(bytes) {
            NO_PAGE => None,
            v => Some(PageId::new(v)),
        }
    }

    fn set_next_page_id(&mut self, page_id: Option<PageId>) {
        let raw = match page_id {
            Some(page_id) => page_id.0,
            _ => NO_PAGE,
        };
        self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4].copy_from_slice(&raw.to_le_bytes())
    }
}

fn main() {
    println!("node header size = {}", NODE_HEADER_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_leaf_header() {
        let mut page = Page::new();

        page.init_node(NodeType::Leaf);

        assert_eq!(page.node_type(), NodeType::Leaf);
        assert_eq!(page.key_count(), 0);
        assert_eq!(page.parent_page_id(), None);
        assert_eq!(page.next_page_id(), None);
    }

    #[test]
    fn initializes_internal_header() {
        let mut page = Page::new();

        page.init_node(NodeType::Internal);

        assert_eq!(page.node_type(), NodeType::Internal);
        assert_eq!(page.key_count(), 0);
    }

    #[test]
    fn stores_key_count_little_endian() {
        let mut page = Page::new();
        page.init_node(NodeType::Leaf);

        page.set_key_count(0x0102);

        assert_eq!(
            &page.data[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 2],
            &[0x02, 0x01]
        );
        assert_eq!(page.key_count(), 0x0102);
    }

    #[test]
    fn stores_optional_page_ids() {
        let mut page = Page::new();
        page.init_node(NodeType::Leaf);

        page.set_parent_page_id(Some(PageId::new(7)));
        page.set_next_page_id(Some(PageId::new(9)));

        assert_eq!(page.parent_page_id(), Some(PageId::new(7)));
        assert_eq!(page.next_page_id(), Some(PageId::new(9)));

        page.set_parent_page_id(None);
        page.set_next_page_id(None);

        assert_eq!(page.parent_page_id(), None);
        assert_eq!(page.next_page_id(), None);
    }
}
