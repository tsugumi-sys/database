// Step 4-4: Allocate pages at the end of a page file.
//
// Run:
// rustc --edition=2021 --test 04_allocate_page.rs && ./04_allocate_page

#![allow(unused)]

use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

impl PageId {
    fn new(value: u32) -> Self {
        Self(value)
    }

    fn file_offset(self) -> u64 {
        u64::from(self.0) * PAGE_SIZE as u64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn empty() -> Self {
        Self {
            data: [0; PAGE_SIZE],
        }
    }

    fn filled(byte: u8) -> Self {
        Self {
            data: [byte; PAGE_SIZE],
        }
    }
}

struct PageFile {
    file: File,
    next_page_id: u32,
}

impl PageFile {
    fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let len = file.metadata()?.len();
        if len % PAGE_SIZE as u64 != 0 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "File is corrupted",
            ));
        }
        let next_page_id = (len / PAGE_SIZE as u64) as u32;
        Ok(Self { file, next_page_id })
    }

    fn allocate_page(&mut self) -> io::Result<PageId> {
        let pageid = PageId::new(self.next_page_id);
        self.write_page(
            PageId::new(self.next_page_id),
            &Page {
                data: [0; PAGE_SIZE],
            },
        );
        self.next_page_id += 1;
        Ok(pageid)
    }

    fn write_page(&mut self, page_id: PageId, page: &Page) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(page_id.file_offset()))?;
        self.file.write_all(&page.data)
    }

    fn read_page(&mut self, page_id: PageId) -> io::Result<Page> {
        let mut page = Page::empty();
        self.file.seek(SeekFrom::Start(page_id.file_offset()))?;
        self.file.read_exact(&mut page.data)?;
        Ok(page)
    }
}

fn main() {
    println!("page size = {}", PAGE_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "database_step4_{}_{}_allocate.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn allocates_zero_filled_pages_in_order() {
        let path = temp_path("ordered");
        let mut page_file = PageFile::open(&path).unwrap();

        let first = page_file.allocate_page().unwrap();
        let second = page_file.allocate_page().unwrap();

        assert_eq!(first, PageId::new(0));
        assert_eq!(second, PageId::new(1));
        assert_eq!(page_file.read_page(first).unwrap(), Page::empty());
        assert_eq!(fs::metadata(&path).unwrap().len(), (PAGE_SIZE * 2) as u64);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn continues_allocation_after_reopen() {
        let path = temp_path("reopen");
        {
            let mut page_file = PageFile::open(&path).unwrap();
            let first = page_file.allocate_page().unwrap();
            page_file.write_page(first, &Page::filled(0x44)).unwrap();
        }

        let mut reopened = PageFile::open(&path).unwrap();
        let second = reopened.allocate_page().unwrap();

        assert_eq!(second, PageId::new(1));
        assert_eq!(
            reopened.read_page(PageId::new(0)).unwrap(),
            Page::filled(0x44)
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unaligned_existing_file() {
        let path = temp_path("unaligned");
        fs::write(&path, [1, 2, 3]).unwrap();

        assert!(PageFile::open(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
