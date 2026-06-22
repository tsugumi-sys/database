// Step 6-2: Persist inserts in a single root leaf page.
//
// Run:
// rustc --edition=2021 --test 02_persistent_leaf_root.rs && ./02_persistent_leaf_root

#![allow(unused)]

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

const PAGE_SIZE: usize = 4096;
const MAGIC_OFFSET: usize = 0;

const META_ROOT_PAGE_ID_OFFSET: usize = 4;
const META_NEXT_PAGE_ID_OFFSET: usize = 8;
const MAGIC: &[u8; 4] = b"DB01";

const LEAF_NODE_TYPE_OFFSET: usize = 0;
const KEY_COUNT_OFFSET: usize = 1;
const LEAF_NEXT_PAGE_ID_OFFSET: usize = 9;
const NODE_HEADER_SIZE: usize = 13; // <1. type | 8. key count | 4. next page id>
const NO_PAGE: u32 = u32::MAX;
const LEAF_CELL_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

#[derive(Debug)]
struct PersistentBPlusTree {
    file: File,
    root_page_id: PageId,
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
        let meta_page_id = PageId(0);
        let root_page_id = PageId(1);
        if is_new {
            let mut meta = [0; PAGE_SIZE];
            meta[MAGIC_OFFSET..MAGIC_OFFSET + 4].copy_from_slice(MAGIC);
            meta[META_ROOT_PAGE_ID_OFFSET..META_ROOT_PAGE_ID_OFFSET + 4]
                .copy_from_slice(&root_page_id.0.to_le_bytes());
            meta[META_NEXT_PAGE_ID_OFFSET..META_NEXT_PAGE_ID_OFFSET + 4]
                .copy_from_slice(&2_i32.to_le_bytes());

            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            file.write_all(&meta).map_err(|e| e.to_string())?;

            let mut leaf = [0; PAGE_SIZE];
            leaf[LEAF_NODE_TYPE_OFFSET] = 1; // 1=leaf, 2=internal
            leaf[LEAF_NEXT_PAGE_ID_OFFSET..LEAF_NEXT_PAGE_ID_OFFSET + 4]
                .copy_from_slice(&NO_PAGE.to_le_bytes());
            file.seek(SeekFrom::Start(PAGE_SIZE as u64))
                .map_err(|e| e.to_string())?;
            file.write_all(&leaf).map_err(|e| e.to_string())?;
        } else {
            let mut meta_page = [0; PAGE_SIZE];
            file.seek(SeekFrom::Start(meta_page_id.0 as u64 * PAGE_SIZE as u64))
                .map_err(|e| e.to_string())?;
            file.read_exact(&mut meta_page).map_err(|e| e.to_string())?;

            let magic_bytes: [u8; 4] = meta_page[MAGIC_OFFSET..MAGIC_OFFSET + 4]
                .try_into()
                .unwrap();
            if &magic_bytes != MAGIC {
                return Err("invalid magic bytes".to_string());
            }
        }
        Ok(Self { file, root_page_id })
    }

    fn insert(&mut self, key: i64, value: i64) -> Result<(), String> {
        let mut page = self.read_page(self.root_page_id)?;

        let count = self.key_count(&page);
        let left = self.lower_bound(&page, key, count);

        if left < count && self.read_key(&page, left) == key {
            let offset = self.leaf_offset(left);
            page[offset + LEAF_CELL_SIZE / 2..offset + LEAF_CELL_SIZE]
                .copy_from_slice(&value.to_le_bytes());
            self.write_root_page(&page)?;
            return Ok(());
        }

        if count >= (PAGE_SIZE - NODE_HEADER_SIZE) / LEAF_CELL_SIZE {
            return Err("page is full".to_string());
        }

        for i in (left..count).rev() {
            let offset = self.leaf_offset(i);
            let new_offset = self.leaf_offset(i + 1);
            page.copy_within(offset..offset + LEAF_CELL_SIZE, new_offset);
        }
        let offset = self.leaf_offset(left);
        page[offset..offset + LEAF_CELL_SIZE / 2].copy_from_slice(&key.to_le_bytes());
        page[offset + LEAF_CELL_SIZE / 2..offset + LEAF_CELL_SIZE]
            .copy_from_slice(&value.to_le_bytes());
        self.set_key_count(&mut page, count + 1);
        self.write_root_page(&page)?;
        Ok(())
    }

    fn get(&mut self, key: i64) -> Result<Option<i64>, String> {
        let mut page = self.read_page(self.root_page_id)?;

        let count = self.key_count(&page);
        if count == 0 {
            return Ok(None);
        }
        let left = self.lower_bound(&page, key, count);

        if left < count && self.read_key(&page, left) == key {
            Ok(Some(self.read_value(&page, left)))
        } else {
            Ok(None)
        }
    }

    fn lower_bound(&self, page: &[u8; PAGE_SIZE], key: i64, count: usize) -> usize {
        let mut left = 0;
        let mut right = count;

        while left < right {
            let mid = (left + right) / 2;
            if self.read_key(&page, mid) < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        left
    }

    fn key_count(&self, page: &[u8; PAGE_SIZE]) -> usize {
        let cnt_b: [u8; 8] = page[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 8]
            .try_into()
            .unwrap();
        u64::from_le_bytes(cnt_b) as usize
    }

    fn set_key_count(&self, page: &mut [u8; PAGE_SIZE], count: usize) {
        page[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 8].copy_from_slice(&count.to_le_bytes());
    }

    fn read_page(&mut self, page_id: PageId) -> Result<[u8; PAGE_SIZE], String> {
        self.file
            .seek(SeekFrom::Start(page_id.0 as u64 * PAGE_SIZE as u64))
            .map_err(|e| e.to_string())?;
        let mut page = [0; PAGE_SIZE];
        self.file.read_exact(&mut page).map_err(|e| e.to_string())?;
        Ok(page)
    }

    fn write_root_page(&mut self, page: &[u8; PAGE_SIZE]) -> Result<(), String> {
        let offset = self.root_page_id.0 as u64 * PAGE_SIZE as u64;
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        self.file.write_all(page).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn leaf_offset(&self, index: usize) -> usize {
        NODE_HEADER_SIZE + index * LEAF_CELL_SIZE
    }

    fn read_key(&self, page: &[u8; PAGE_SIZE], index: usize) -> i64 {
        let offset = self.leaf_offset(index);
        let key_b: [u8; 8] = page[offset..offset + 8].try_into().unwrap();
        i64::from_le_bytes(key_b)
    }
    fn read_value(&self, page: &[u8; PAGE_SIZE], index: usize) -> i64 {
        let offset = self.leaf_offset(index);
        let value_b: [u8; 8] = page[offset + 8..offset + 16].try_into().unwrap();
        i64::from_le_bytes(value_b)
    }

    fn flush(&mut self) -> Result<(), String> {
        self.file.flush().map_err(|e| e.to_string());
        Ok(())
    }
}

fn main() {
    println!("persistent root leaf");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "database_step6_{}_{}_leaf_root.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn inserts_and_gets_from_root_leaf() {
        let path = temp_path("insert_get");
        let mut tree = PersistentBPlusTree::open(&path).unwrap();

        tree.insert(10, 100).unwrap();
        tree.insert(20, 200).unwrap();

        assert_eq!(tree.get(10).unwrap(), Some(100));
        assert_eq!(tree.get(20).unwrap(), Some(200));
        assert_eq!(tree.get(30).unwrap(), None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn duplicate_insert_updates_value() {
        let path = temp_path("update");
        let mut tree = PersistentBPlusTree::open(&path).unwrap();

        tree.insert(10, 100).unwrap();
        tree.insert(10, 999).unwrap();

        assert_eq!(tree.get(10).unwrap(), Some(999));
        fs::remove_file(path).unwrap();
    }
}
