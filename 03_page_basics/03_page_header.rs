// Step 3-3: Page header.
//
// Run:
// rustc --edition=2021 --test 03_page_header.rs && ./03_page_header

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const PAGE_TYPE_OFFSET: usize = 0;
const PAGE_ID_OFFSET: usize = 1;
const FREE_SPACE_OFFSET_OFFSET: usize = 5;
const HEADER_SIZE: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageType {
    Empty,
    Leaf,
    Internal,
}

impl PageType {
    fn to_u8(self) -> u8 {
        match self {
            PageType::Empty => 0,
            PageType::Leaf => 1,
            PageType::Internal => 2,
        }
    }

    fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(PageType::Empty),
            1 => Ok(PageType::Leaf),
            2 => Ok(PageType::Internal),
            _ => Err(format!("Invalid page type {}", value)),
        }
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

    fn set_page_type(&mut self, page_type: PageType) {
        self.data[PAGE_TYPE_OFFSET] = page_type.to_u8();
    }

    fn page_type(&self) -> Result<PageType, String> {
        PageType::from_u8(self.data[PAGE_TYPE_OFFSET])
    }

    fn set_page_id(&mut self, page_id: u32) {
        let bytes = page_id.to_le_bytes();
        self.data[PAGE_ID_OFFSET..PAGE_ID_OFFSET + 4].copy_from_slice(&bytes);
    }

    fn page_id(&self) -> u32 {
        let bytes: [u8; 4] = self.data[PAGE_ID_OFFSET..PAGE_ID_OFFSET + 4]
            .try_into()
            .unwrap();
        u32::from_le_bytes(bytes)
    }

    fn set_free_space_offset(&mut self, offset: u16) {
        let bytes = offset.to_le_bytes();
        self.data[FREE_SPACE_OFFSET_OFFSET..FREE_SPACE_OFFSET_OFFSET + 2].copy_from_slice(&bytes);
    }

    fn free_space_offset(&self) -> u16 {
        let bytes: [u8; 2] = self.data[FREE_SPACE_OFFSET_OFFSET..FREE_SPACE_OFFSET_OFFSET + 2]
            .try_into()
            .unwrap();
        u16::from_le_bytes(bytes)
    }
}

fn main() {
    let page = Page::new();
    println!("header size = {}", HEADER_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_decodes_page_type() {
        assert_eq!(PageType::Empty.to_u8(), 0);
        assert_eq!(PageType::Leaf.to_u8(), 1);
        assert_eq!(PageType::Internal.to_u8(), 2);

        assert_eq!(PageType::from_u8(0).unwrap(), PageType::Empty);
        assert_eq!(PageType::from_u8(1).unwrap(), PageType::Leaf);
        assert_eq!(PageType::from_u8(2).unwrap(), PageType::Internal);
        assert!(PageType::from_u8(99).is_err());
    }

    #[test]
    fn writes_and_reads_page_type() {
        let mut page = Page::new();

        page.set_page_type(PageType::Leaf);

        assert_eq!(page.page_type().unwrap(), PageType::Leaf);
    }

    #[test]
    fn writes_and_reads_page_id() {
        let mut page = Page::new();

        page.set_page_id(0x0102_0304);

        assert_eq!(page.page_id(), 0x0102_0304);
        assert_eq!(
            &page.data[PAGE_ID_OFFSET..PAGE_ID_OFFSET + 4],
            &[0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn writes_and_reads_free_space_offset() {
        let mut page = Page::new();

        page.set_free_space_offset(HEADER_SIZE as u16);

        assert_eq!(page.free_space_offset(), HEADER_SIZE as u16);
    }

    #[test]
    fn header_fields_do_not_overlap() {
        let mut page = Page::new();

        page.set_page_type(PageType::Internal);
        page.set_page_id(123);
        page.set_free_space_offset(456);

        assert_eq!(page.page_type().unwrap(), PageType::Internal);
        assert_eq!(page.page_id(), 123);
        assert_eq!(page.free_space_offset(), 456);
    }
}
