// Step 6-1: Store B+Tree metadata in page 0.
//
// Run:
// rustc --edition=2021 --test 01_meta_page.rs && ./01_meta_page

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const MAGIC_OFFSET: usize = 0;
const ROOT_PAGE_ID_OFFSET: usize = 4;
const NEXT_PAGE_ID_OFFSET: usize = 8;
const MAGIC: &[u8; 4] = b"DB01";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MetaPage {
    data: [u8; PAGE_SIZE],
}

impl MetaPage {
    fn new(root_page_id: PageId, next_page_id: PageId) -> Self {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };
        page.data[MAGIC_OFFSET..MAGIC_OFFSET + 4].copy_from_slice(MAGIC);
        page.data[ROOT_PAGE_ID_OFFSET..ROOT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&root_page_id.0.to_le_bytes());
        page.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&next_page_id.0.to_le_bytes());
        page
    }

    fn validate(&self) -> Result<(), String> {
        let magic_bytes: [u8; 4] = self.data[MAGIC_OFFSET..MAGIC_OFFSET + 4]
            .try_into()
            .unwrap();
        if &magic_bytes != MAGIC {
            return Err("invalid magic bytes".to_string());
        }
        Ok(())
    }

    fn root_page_id(&self) -> PageId {
        let root_pid_bytes: [u8; 4] = self.data[ROOT_PAGE_ID_OFFSET..ROOT_PAGE_ID_OFFSET + 4]
            .try_into()
            .unwrap();
        PageId::new(u32::from_le_bytes(root_pid_bytes))
    }

    fn set_root_page_id(&mut self, page_id: PageId) {
        self.data[ROOT_PAGE_ID_OFFSET..ROOT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&page_id.0.to_le_bytes());
    }

    fn next_page_id(&self) -> PageId {
        let next_pid_bytes: [u8; 4] = self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
            .try_into()
            .unwrap();
        PageId::new(u32::from_le_bytes(next_pid_bytes))
    }

    fn set_next_page_id(&mut self, page_id: PageId) {
        self.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4]
            .copy_from_slice(&page_id.0.to_le_bytes());
    }
}

fn main() {
    println!("metadata page size = {}", PAGE_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_metadata_page() {
        let meta = MetaPage::new(PageId::new(1), PageId::new(2));

        assert!(meta.validate().is_ok());
        assert_eq!(meta.root_page_id(), PageId::new(1));
        assert_eq!(meta.next_page_id(), PageId::new(2));
    }

    #[test]
    fn stores_fields_as_little_endian() {
        let meta = MetaPage::new(PageId::new(0x0102_0304), PageId::new(0x0506_0708));

        assert_eq!(&meta.data[MAGIC_OFFSET..MAGIC_OFFSET + 4], MAGIC);
        assert_eq!(
            &meta.data[ROOT_PAGE_ID_OFFSET..ROOT_PAGE_ID_OFFSET + 4],
            &[0x04, 0x03, 0x02, 0x01]
        );
        assert_eq!(
            &meta.data[NEXT_PAGE_ID_OFFSET..NEXT_PAGE_ID_OFFSET + 4],
            &[0x08, 0x07, 0x06, 0x05]
        );
    }

    #[test]
    fn updates_root_and_next_page_id() {
        let mut meta = MetaPage::new(PageId::new(1), PageId::new(2));

        meta.set_root_page_id(PageId::new(9));
        meta.set_next_page_id(PageId::new(10));

        assert_eq!(meta.root_page_id(), PageId::new(9));
        assert_eq!(meta.next_page_id(), PageId::new(10));
    }

    #[test]
    fn rejects_invalid_magic() {
        let mut meta = MetaPage::new(PageId::new(1), PageId::new(2));
        meta.data[0] = b'X';

        assert!(meta.validate().is_err());
    }
}
