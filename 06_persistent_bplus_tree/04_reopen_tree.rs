// Step 6-4: Reopen a persisted tree without truncating existing data.
//
// Run:
// rustc --edition=2021 --test 04_reopen_tree.rs && ./04_reopen_tree

#![allow(unused)]

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug)]
struct PersistentBPlusTree {
    file: File,
    root_page_id: u32,
}

struct LeafEntry {
    key: i64,
    value: i64,
}

const PAGE_SIZE: usize = 4096;
const META_ROOT_PAGE_ID_OFFSET: usize = 4;
const META_NEXT_PAGE_ID_OFFSET: usize = 8;

const LEAF_NODE_TYPE_OFFSET: usize = 0;
const KEY_COUNT_OFFSET: usize = 1;
const LEAF_NEXT_PAGE_ID_OFFSET: usize = 9;
const NODE_HEADER_SIZE: usize = 13; // <1. type | 8. key count | 4. next page id>
const NO_PAGE: u32 = u32::MAX;
const LEAF_CELL_SIZE: usize = 16;

impl PersistentBPlusTree {
    fn open(path: &Path) -> Result<Self, String> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let is_new = file.metadata().map_err(|e| e.to_string())?.len() == 0;
        let meta_page_id = 0;
        if is_new {
            let root_page_id: u32 = 1;
            let mut metapage = [0; PAGE_SIZE];
            metapage[META_ROOT_PAGE_ID_OFFSET..META_ROOT_PAGE_ID_OFFSET + 4]
                .copy_from_slice(&root_page_id.to_le_bytes());
            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            file.write_all(&metapage).map_err(|e| e.to_string())?;

            let mut leaf = [0; PAGE_SIZE];
            leaf[LEAF_NODE_TYPE_OFFSET] = 1; // 1=leaf, 2=internal
            leaf[LEAF_NEXT_PAGE_ID_OFFSET..LEAF_NEXT_PAGE_ID_OFFSET + 4]
                .copy_from_slice(&NO_PAGE.to_le_bytes());
            file.seek(SeekFrom::Start(PAGE_SIZE as u64))
                .map_err(|e| e.to_string())?;
            file.write_all(&leaf).map_err(|e| e.to_string())?;
            return Ok(Self { file, root_page_id });
        } else {
            let mut metapage = [0; PAGE_SIZE];
            file.seek(SeekFrom::Start(meta_page_id as u64))
                .map_err(|e| e.to_string());
            file.read_exact(&mut metapage).map_err(|e| e.to_string())?;
            let root_page_id_bytes: [u8; 4] = metapage
                [META_ROOT_PAGE_ID_OFFSET..META_ROOT_PAGE_ID_OFFSET + 4]
                .try_into()
                .unwrap();
            let root_page_id = u32::from_le_bytes(root_page_id_bytes);
            return Ok(Self { file, root_page_id });
        }
    }

    fn insert(&mut self, key: i64, value: i64) -> Result<(), String> {
        let mut page = self.read_page(self.root_page_id)?;
        let left = self.lower_bound(&page, key);
        let count = self.key_count(&page);
        if left < count && self.read_entry(&page, left).key == key {
            let offset = self.entry_offset(left);
            page[offset + LEAF_CELL_SIZE / 2..offset + LEAF_CELL_SIZE]
                .copy_from_slice(&value.to_le_bytes());
            self.write_page(&page, self.root_page_id)?;
            return Ok(());
        }

        if count >= (PAGE_SIZE - NODE_HEADER_SIZE) / LEAF_CELL_SIZE {
            return Err("page is full".to_string());
        }

        for i in (left..count).rev() {
            let offset = self.entry_offset(i);
            let new_offset = self.entry_offset(i + 1);
            page.copy_within(offset..offset + LEAF_CELL_SIZE, new_offset);
        }
        let offset = self.entry_offset(left);
        page[offset..offset + LEAF_CELL_SIZE / 2].copy_from_slice(&key.to_le_bytes());
        page[offset + LEAF_CELL_SIZE / 2..offset + LEAF_CELL_SIZE]
            .copy_from_slice(&value.to_le_bytes());
        self.set_key_count(&mut page, count + 1);
        self.write_page(&page, self.root_page_id)?;
        Ok(())
    }

    fn entry_offset(&self, index: usize) -> usize {
        NODE_HEADER_SIZE + index * LEAF_CELL_SIZE
    }

    fn key_count(&self, page: &[u8; PAGE_SIZE]) -> usize {
        let counts_b: [u8; 4] = page[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 4]
            .try_into()
            .unwrap();
        u32::from_le_bytes(counts_b) as usize
    }

    fn set_key_count(&self, page: &mut [u8; PAGE_SIZE], count: usize) {
        page[KEY_COUNT_OFFSET..KEY_COUNT_OFFSET + 8].copy_from_slice(&count.to_le_bytes());
    }

    fn read_page(&mut self, index: u32) -> Result<[u8; PAGE_SIZE], String> {
        let mut page = [0; PAGE_SIZE];
        self.file
            .seek(SeekFrom::Start(index as u64 * PAGE_SIZE as u64))
            .map_err(|e| e.to_string());
        self.file.read_exact(&mut page).map_err(|e| e.to_string())?;
        Ok(page)
    }

    fn write_page(&mut self, page: &[u8; PAGE_SIZE], index: u32) -> Result<(), String> {
        self.file
            .seek(SeekFrom::Start(index as u64 * PAGE_SIZE as u64))
            .map_err(|e| e.to_string());
        self.file.write_all(page).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn read_entry(&self, page: &[u8; PAGE_SIZE], index: usize) -> LeafEntry {
        let offset = self.entry_offset(index);
        let key_b: [u8; 8] = page[offset..offset + 8].try_into().unwrap();
        let value_b: [u8; 8] = page[offset + 8..offset + 16].try_into().unwrap();
        LeafEntry {
            key: i64::from_le_bytes(key_b),
            value: i64::from_le_bytes(value_b),
        }
    }

    fn lower_bound(&self, page: &[u8; PAGE_SIZE], key: i64) -> usize {
        let mut left = 0;
        let mut right = self.key_count(&page);
        while left < right {
            let mid = (left + right) / 2;
            if self.read_entry(&page, mid).key < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        left
    }

    fn get(&mut self, key: i64) -> Result<Option<i64>, String> {
        let page = self.read_page(self.root_page_id)?;
        let left = self.lower_bound(&page, key);
        if left < self.key_count(&page) && self.read_entry(&page, left).key == key {
            return Ok(Some(self.read_entry(&page, left).value));
        }
        Err("not found".to_string())
    }

    fn flush(&mut self) -> Result<(), String> {
        self.file.flush().map_err(|e| e.to_string());
        Ok(())
    }
}

fn main() {
    println!("reopen persistent tree");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "database_step6_{}_{}_reopen.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn reopen_preserves_inserted_values() {
        let path = temp_path("preserve");
        {
            let mut tree = PersistentBPlusTree::open(&path).unwrap();
            tree.insert(10, 100).unwrap();
            tree.insert(20, 200).unwrap();
            tree.flush().unwrap();
        }

        let mut reopened = PersistentBPlusTree::open(&path).unwrap();

        assert_eq!(reopened.get(10).unwrap(), Some(100));
        assert_eq!(reopened.get(20).unwrap(), Some(200));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn insert_after_reopen_keeps_old_values() {
        let path = temp_path("append");
        {
            let mut tree = PersistentBPlusTree::open(&path).unwrap();
            tree.insert(10, 100).unwrap();
            tree.flush().unwrap();
        }

        let mut reopened = PersistentBPlusTree::open(&path).unwrap();
        reopened.insert(20, 200).unwrap();
        reopened.flush().unwrap();

        let mut again = PersistentBPlusTree::open(&path).unwrap();
        assert_eq!(again.get(10).unwrap(), Some(100));
        assert_eq!(again.get(20).unwrap(), Some(200));
        fs::remove_file(path).unwrap();
    }
}
