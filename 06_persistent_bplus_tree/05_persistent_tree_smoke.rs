// Step 6-5: Exercise a small persistent B+Tree end to end.
//
// Run:
// rustc --edition=2021 --test 05_persistent_tree_smoke.rs && ./05_persistent_tree_smoke

#![allow(unused)]

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PageId(u32);

#[derive(Debug)]
struct PersistentBPlusTree {
    file: File,
    root_page_id: PageId,
}

const PAGE_SIZE: usize = 4096;

const MAGIC_OFFSET: usize = 0;
const MAGIC: &[u8; 4] = b"DB01";

const META_ROOT_PAGE_ID_OFFSET: usize = 4;
const META_NEXT_PAGE_ID_OFFSET: usize = 8; // next page id which the new page should be use.

const NODE_TYPE_OFFSET: usize = 0;
const NODE_LEAF: u8 = 1;
const NODE_INTERNAL: u8 = 2;
const KEY_COUNT_OFFSET: usize = 1;

const INTERNAL_LEFTMOST_PAGE_ID_OFFSET: usize = 9;
const INTERNAL_CELL_SIZE: usize = 12; // key(8) + pageid(4)

const LEAF_NEXT_PAGE_ID_OFFSET: usize = 9;
const NODE_HEADER_SIZE: usize = 13;
const LEAF_CELL_SIZE: usize = 16; // key(8) + value(8)

const NO_PAGE: u32 = u32::MAX;

struct Page([u8; PAGE_SIZE]);

impl Page {
    fn new() -> Self {
        Self([0; PAGE_SIZE])
    }
    fn read_u32(&self, offset: usize) -> u32 {
        let b: [u8; 4] = self.0[offset..offset + 4].try_into().unwrap();
        u32::from_le_bytes(b)
    }
    fn write_u32(&mut self, offset: usize, a: u32) {
        self.0[offset..offset + 4].copy_from_slice(&a.to_le_bytes());
    }
    fn read_bytes(&self, offset: usize, size: usize) -> &[u8] {
        &self.0[offset..offset + size]
    }
    fn write_bytes(&mut self, offset: usize, bytes: &[u8]) {
        self.0[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
    fn read_i64(&self, offset: usize) -> i64 {
        let b: [u8; 8] = self.0[offset..offset + 8].try_into().unwrap();
        i64::from_le_bytes(b)
    }
    fn write_i64(&mut self, offset: usize, a: i64) {
        self.0[offset..offset + 8].copy_from_slice(&a.to_le_bytes());
    }
    fn read_u8(&self, offset: usize) -> u8 {
        self.0[offset]
    }
    fn write_u8(&mut self, offset: usize, a: u8) {
        self.0[offset] = a;
    }
}

struct MetaPage {
    data: Page,
}

impl MetaPage {
    fn new(root: PageId, next: PageId) -> Self {
        let mut data = Page::new();
        data.write_bytes(MAGIC_OFFSET, MAGIC);
        data.write_u32(META_ROOT_PAGE_ID_OFFSET, root.0);
        data.write_u32(META_NEXT_PAGE_ID_OFFSET, next.0);
        Self { data }
    }
    fn from_page(page: Page) -> Result<Self, String> {
        if page.read_bytes(MAGIC_OFFSET, 4) != MAGIC {
            return Err("invalid magic".to_string());
        }
        Ok(Self { data: page })
    }
    fn root_page_id(&self) -> PageId {
        PageId(self.data.read_u32(META_ROOT_PAGE_ID_OFFSET))
    }

    fn set_root_page_id(&mut self, page_id: PageId) {
        self.data.write_u32(META_ROOT_PAGE_ID_OFFSET, page_id.0);
    }

    fn next_page_id(&self) -> PageId {
        PageId(self.data.read_u32(META_NEXT_PAGE_ID_OFFSET))
    }

    fn set_next_page_id(&mut self, page_id: PageId) {
        self.data.write_u32(META_NEXT_PAGE_ID_OFFSET, page_id.0);
    }

    fn allocate_page(&mut self) -> PageId {
        let page = self.next_page_id();
        self.set_next_page_id(PageId(page.0 + 1));
        page
    }
}

struct LeafEntry {
    key: i64,
    value: i64,
}
struct LeafSplitResult {
    separator_key: i64,
    right_page_id: PageId,
    right_page: LeafPage,
}
struct LeafPage {
    data: Page,
}
impl LeafPage {
    fn new() -> Self {
        let mut page = Page::new();
        page.write_u8(NODE_TYPE_OFFSET, NODE_LEAF);
        page.write_u32(KEY_COUNT_OFFSET, 0);
        page.write_u32(LEAF_NEXT_PAGE_ID_OFFSET, NO_PAGE);
        Self { data: page }
    }
    fn from_page(page: Page) -> Result<Self, String> {
        if page.read_u8(NODE_TYPE_OFFSET) != NODE_LEAF {
            return Err("invalid node type for leaf".to_string());
        }
        Ok(Self { data: page })
    }
    fn key_count(&self) -> usize {
        self.data.read_u32(KEY_COUNT_OFFSET) as usize
    }
    fn set_key_count(&mut self, count: usize) {
        self.data.write_u32(KEY_COUNT_OFFSET, count as u32);
    }
    fn next_page_id(&self) -> PageId {
        let page_id = self.data.read_u32(LEAF_NEXT_PAGE_ID_OFFSET);
        PageId(page_id)
    }
    fn set_next_page_id(&mut self, page_id: PageId) {
        self.data.write_u32(LEAF_NEXT_PAGE_ID_OFFSET, page_id.0);
    }
    fn write_entry(&mut self, i: usize, key: i64, value: i64) {
        let offset = NODE_HEADER_SIZE + i * LEAF_CELL_SIZE;
        self.data.write_i64(offset, key);
        self.data.write_i64(offset + 8, value);
    }
    fn read_entry(&self, i: usize) -> LeafEntry {
        let offset = NODE_HEADER_SIZE + i * LEAF_CELL_SIZE;
        let key = self.data.read_i64(offset);
        let value = self.data.read_i64(offset + 8);
        LeafEntry { key, value }
    }
    fn get(&self, key: i64) -> Option<i64> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(mid).key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left < count && self.read_entry(left).key == key {
            return Some(self.read_entry(left).value);
        }
        None
    }
    fn is_full(&self) -> bool {
        self.key_count() >= (PAGE_SIZE - NODE_HEADER_SIZE) / LEAF_CELL_SIZE
    }
    fn insert_or_update(&mut self, key: i64, value: i64) -> Result<(), String> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(mid).key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left < count && self.read_entry(left).key == key {
            self.write_entry(left, key, value);
            return Ok(());
        }
        if self.is_full() {
            return Err("page is full".to_string());
        }
        for i in (left..count).rev() {
            let e = self.read_entry(i);
            self.write_entry(i + 1, e.key, e.value);
        }
        self.write_entry(left, key, value);
        self.set_key_count(count + 1);
        Ok(())
    }
    fn clear_entry(&mut self, i: usize) {
        let offset = NODE_HEADER_SIZE + i * LEAF_CELL_SIZE;
        self.data.write_i64(offset, 0);
        self.data.write_i64(offset + 8, 0);
    }
    fn split(&mut self, right_page_id: PageId) -> Result<LeafSplitResult, String> {
        let count = self.key_count();
        if count < 2 {
            return Err("too small to split the leaf page".to_string());
        }

        let mut new_page = LeafPage::new();
        let mid = count / 2;

        new_page.set_next_page_id(self.next_page_id());
        self.set_next_page_id(right_page_id);

        for i in mid..count {
            let e = self.read_entry(i);
            new_page.write_entry(i - mid, e.key, e.value);
            self.clear_entry(i);
        }
        self.set_key_count(mid);
        new_page.set_key_count(count - mid);

        Ok(LeafSplitResult {
            separator_key: new_page.read_entry(0).key,
            right_page_id,
            right_page: new_page,
        })
    }
}
struct InternalPage {
    data: Page,
}
struct InternalEntry {
    key: i64,
    child_page_id: PageId,
}
struct InternalSplitResult {
    separator_key: i64,
    right_page_id: PageId,
    right_page: InternalPage,
}
impl InternalPage {
    fn new(leftmost: PageId) -> Self {
        let mut page = Page([0; PAGE_SIZE]);
        page.write_u8(NODE_TYPE_OFFSET, NODE_INTERNAL);
        page.write_u32(KEY_COUNT_OFFSET, 0);
        page.write_u32(INTERNAL_LEFTMOST_PAGE_ID_OFFSET, leftmost.0);
        Self { data: page }
    }
    fn from_page(page: Page) -> Result<Self, String> {
        if page.read_u8(NODE_TYPE_OFFSET) != NODE_INTERNAL {
            return Err("invalid node type for internal".to_string());
        }
        Ok(Self { data: page })
    }
    fn key_count(&self) -> usize {
        self.data.read_u32(KEY_COUNT_OFFSET) as usize
    }
    fn set_key_count(&mut self, count: usize) {
        self.data.write_u32(KEY_COUNT_OFFSET, count as u32);
    }

    fn leftmost_page_id(&self) -> PageId {
        PageId(self.data.read_u32(INTERNAL_LEFTMOST_PAGE_ID_OFFSET))
    }
    fn set_leftmost_page_id(&mut self, id: PageId) {
        self.data.write_u32(INTERNAL_LEFTMOST_PAGE_ID_OFFSET, id.0);
    }

    fn read_entry(&self, i: usize) -> InternalEntry {
        let offset = NODE_HEADER_SIZE + i * INTERNAL_CELL_SIZE;
        let key = self.data.read_i64(offset);
        let child_page_id = PageId(self.data.read_u32(offset + 8));
        InternalEntry { key, child_page_id }
    }
    fn write_entry(&mut self, i: usize, key: i64, child_page_id: PageId) {
        let offset = NODE_HEADER_SIZE + i * INTERNAL_CELL_SIZE;
        self.data.write_i64(offset, key);
        self.data.write_u32(offset + 8, child_page_id.0);
    }
    fn clear_entry(&mut self, i: usize) {
        let offset = NODE_HEADER_SIZE + i * INTERNAL_CELL_SIZE;
        self.data.write_i64(offset, 0);
        self.data.write_u32(offset + 8, 0);
    }

    fn child_for_key(&self, key: i64) -> PageId {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(mid).key <= key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left == 0 {
            self.leftmost_page_id()
        } else {
            self.read_entry(left - 1).child_page_id
        }
    }
    fn insert(&mut self, separator_key: i64, right_child: PageId) -> Result<(), String> {
        let count = self.key_count();
        let mut left = 0;
        let mut right = count;
        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(mid).key < separator_key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        if left < count && self.read_entry(left).key == separator_key {
            return Err("duplicated separator key".to_string());
        }

        if self.is_full() {
            return Err("page is full".to_string());
        }

        for i in (left..count).rev() {
            let e = self.read_entry(i);
            self.write_entry(i + 1, e.key, e.child_page_id);
        }
        self.write_entry(left, separator_key, right_child);
        self.set_key_count(count + 1);
        Ok(())
    }
    fn is_full(&self) -> bool {
        self.key_count() >= (PAGE_SIZE - NODE_HEADER_SIZE) / INTERNAL_CELL_SIZE
    }
    fn split(&mut self, right_page_id: PageId) -> Result<InternalSplitResult, String> {
        let count = self.key_count();
        if count < 2 {
            return Err("too small to split internal page".to_string());
        }

        let mid = count / 2;
        let middle_entry = self.read_entry(mid);
        let mut new_page = InternalPage::new(middle_entry.child_page_id);
        self.clear_entry(mid);

        for i in (mid + 1..count) {
            let e = self.read_entry(i);
            new_page.write_entry(i - mid - 1, e.key, e.child_page_id);
            self.clear_entry(i);
        }

        self.set_key_count(mid);
        new_page.set_key_count(count - mid - 1);
        Ok(InternalSplitResult {
            separator_key: middle_entry.key,
            right_page_id: right_page_id,
            right_page: new_page,
        })
    }
}

struct ChildSplit {
    separator_key: i64,
    right_page_id: PageId,
}

impl PersistentBPlusTree {
    fn open(path: &Path) -> Result<Self, String> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let is_new = file.metadata().map_err(|e| e.to_string())?.len() == 0;
        if is_new {
            let root_page_id = PageId(1);
            let meta = MetaPage::new(root_page_id, PageId(2));
            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            file.write_all(&meta.data.0).map_err(|e| e.to_string())?;

            let leaf = LeafPage::new(); // As the initial root.
            file.seek(SeekFrom::Start(root_page_id.0 as u64 * PAGE_SIZE as u64))
                .map_err(|e| e.to_string())?;
            file.write_all(&leaf.data.0).map_err(|e| e.to_string())?;
            return Ok(Self { file, root_page_id });
        } else {
            let mut meta_bytes = [0; PAGE_SIZE];
            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            file.read_exact(&mut meta_bytes)
                .map_err(|e| e.to_string())?;
            let meta = MetaPage::from_page(Page(meta_bytes))?;
            return Ok(Self {
                file,
                root_page_id: meta.root_page_id(),
            });
        }
    }

    fn read_page(&mut self, page_id: PageId) -> Result<Page, String> {
        let mut page = Page::new();

        self.file
            .seek(SeekFrom::Start(page_id.0 as u64 * PAGE_SIZE as u64))
            .map_err(|e| e.to_string())?;
        self.file
            .read_exact(&mut page.0)
            .map_err(|e| e.to_string())?;
        Ok(page)
    }

    fn write_page(&mut self, page_id: PageId, page: &Page) -> Result<(), String> {
        self.file
            .seek(SeekFrom::Start(page_id.0 as u64 * PAGE_SIZE as u64))
            .map_err(|e| e.to_string())?;
        self.file.write_all(&page.0).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn insert_and_split(
        &mut self,
        page_id: PageId,
        key: i64,
        value: i64,
    ) -> Result<Option<ChildSplit>, String> {
        let page = self.read_page(page_id)?;
        match page.read_u8(NODE_TYPE_OFFSET) {
            NODE_LEAF => {
                let mut leafpage = LeafPage::from_page(page)?;
                if !leafpage.is_full() {
                    leafpage.insert_or_update(key, value)?;
                    self.write_page(page_id, &leafpage.data)?;
                    return Ok(None);
                }
                let mut meta_page = MetaPage::from_page(self.read_page(PageId(0))?)?;
                let new_right_page_id = meta_page.allocate_page();

                let split_result = leafpage.split(new_right_page_id)?;
                let separator_key = split_result.separator_key;
                let mut right_page = split_result.right_page;

                if key < split_result.separator_key {
                    leafpage.insert_or_update(key, value)?;
                } else {
                    right_page.insert_or_update(key, value)?;
                }

                self.write_page(page_id, &leafpage.data)?;
                self.write_page(new_right_page_id, &right_page.data)?;
                self.write_page(PageId(0), &meta_page.data)?;
                return Ok(Some(ChildSplit {
                    separator_key: split_result.separator_key,
                    right_page_id: new_right_page_id,
                }));
            }
            NODE_INTERNAL => {
                let mut internal_page = InternalPage::from_page(page)?;
                let child_page_id = internal_page.child_for_key(key);
                let Some(child_split) = self.insert_and_split(child_page_id, key, value)? else {
                    return Ok(None);
                };
                let separator_key = child_split.separator_key;
                let right_child_id = child_split.right_page_id;
                if !internal_page.is_full() {
                    internal_page.insert(separator_key, right_child_id)?;
                    self.write_page(page_id, &internal_page.data)?;
                    return Ok(None);
                }

                let mut meta_page = MetaPage::from_page(self.read_page(PageId(0))?)?;
                let new_internal_page_id = meta_page.allocate_page();
                let split_result = internal_page.split(new_internal_page_id)?;
                let internal_separator_key = split_result.separator_key;
                let mut new_internal_page = split_result.right_page;

                if split_result.separator_key < separator_key {
                    internal_page.insert(separator_key, right_child_id)?;
                } else {
                    new_internal_page.insert(separator_key, right_child_id)?;
                }

                self.write_page(page_id, &internal_page.data)?;
                self.write_page(new_internal_page_id, &new_internal_page.data)?;
                self.write_page(PageId(0), &meta_page.data)?;

                Ok(Some(ChildSplit {
                    separator_key: internal_separator_key,
                    right_page_id: new_internal_page_id,
                }))
            }
            _ => return Err("invalid node type".to_string()),
        }
    }

    fn insert(&mut self, key: i64, value: i64) -> Result<(), String> {
        let child_split = self.insert_and_split(self.root_page_id, key, value)?;
        if let Some(child_split) = child_split {
            let old_root_page_id = self.root_page_id;

            let mut meta_page = MetaPage::from_page(self.read_page(PageId(0))?)?;
            let new_root_page_id = meta_page.allocate_page();

            let mut new_root = InternalPage::new(old_root_page_id);
            new_root.insert(child_split.separator_key, child_split.right_page_id)?;

            self.write_page(new_root_page_id, &new_root.data)?;

            meta_page.set_root_page_id(new_root_page_id);
            self.write_page(PageId(0), &meta_page.data)?;
            self.root_page_id = new_root_page_id;
        }
        Ok(())
    }

    fn get(&mut self, key: i64) -> Result<Option<i64>, String> {
        let mut page_id = self.root_page_id;
        loop {
            let page = self.read_page(page_id)?;
            match page.read_u8(NODE_TYPE_OFFSET) {
                NODE_INTERNAL => {
                    let ipage = InternalPage::from_page(page)?;
                    page_id = ipage.child_for_key(key);
                }
                NODE_LEAF => {
                    let lpage = LeafPage::from_page(page)?;
                    return Ok(lpage.get(key));
                }
                _ => return Err("invalid node type".to_string()),
            }
        }
    }

    fn flush(&mut self) -> Result<(), String> {
        self.file.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn main() {
    println!("persistent B+Tree smoke");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "database_step6_{}_{}_smoke.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn persists_values_after_leaf_and_root_split() {
        let path = temp_path("split");
        {
            let mut tree = PersistentBPlusTree::open(&path).unwrap();
            for key in [50, 10, 40, 20, 30, 60, 70] {
                tree.insert(key, key * 10).unwrap();
            }
            tree.flush().unwrap();
        }

        let mut reopened = PersistentBPlusTree::open(&path).unwrap();
        for key in [10, 20, 30, 40, 50, 60, 70] {
            assert_eq!(reopened.get(key).unwrap(), Some(key * 10));
        }
        assert_eq!(reopened.get(99).unwrap(), None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn root_split_persists_many_values_after_reopen() {
        let path = temp_path("root_split");
        {
            let mut tree = PersistentBPlusTree::open(&path).unwrap();
            for key in 0..300 {
                tree.insert(key, key * 10).unwrap();
            }
            assert_ne!(tree.root_page_id, PageId(1));
            tree.flush().unwrap();
        }

        let mut reopened = PersistentBPlusTree::open(&path).unwrap();
        assert_ne!(reopened.root_page_id, PageId(1));
        for key in [0, 1, 127, 128, 254, 255, 299] {
            assert_eq!(reopened.get(key).unwrap(), Some(key * 10));
        }
        assert_eq!(reopened.get(999).unwrap(), None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn internal_split_persists_many_values_after_reopen() {
        let path = temp_path("internal_split");
        {
            let mut tree = PersistentBPlusTree::open(&path).unwrap();
            for key in 0..90_000 {
                tree.insert(key, key * 10).unwrap();
            }
            tree.flush().unwrap();
        }

        let mut reopened = PersistentBPlusTree::open(&path).unwrap();
        for key in [0, 1, 127, 128, 255, 256, 43_520, 89_999] {
            assert_eq!(reopened.get(key).unwrap(), Some(key * 10));
        }
        assert_eq!(reopened.get(100_000).unwrap(), None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn updates_existing_key_across_reopen() {
        let path = temp_path("update");
        {
            let mut tree = PersistentBPlusTree::open(&path).unwrap();
            tree.insert(10, 100).unwrap();
            tree.flush().unwrap();
        }

        let mut reopened = PersistentBPlusTree::open(&path).unwrap();
        reopened.insert(10, 999).unwrap();
        reopened.flush().unwrap();

        let mut again = PersistentBPlusTree::open(&path).unwrap();
        assert_eq!(again.get(10).unwrap(), Some(999));
        fs::remove_file(path).unwrap();
    }
}
