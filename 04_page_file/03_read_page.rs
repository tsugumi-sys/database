// Step 4-3: Read fixed-size pages from a file.
//
// Run:
// rustc --edition=2021 --test 03_read_page.rs && ./03_read_page

#![allow(unused)]

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
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
    fn filled(byte: u8) -> Self {
        Self {
            data: [byte; PAGE_SIZE],
        }
    }
}

struct PageFile {
    file: File,
}

impl PageFile {
    fn create(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(Self { file })
    }

    fn write_page(&mut self, page_id: PageId, page: &Page) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(page_id.file_offset()))?;
        self.file.write_all(&page.data)
    }

    fn read_page(&mut self, page_id: PageId) -> io::Result<Page> {
        let mut page = Page {
            data: [0; PAGE_SIZE],
        };
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
            "database_step4_{}_{}_read.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn reads_page_by_id() {
        let path = temp_path("by_id");
        let mut page_file = PageFile::create(&path).unwrap();
        page_file
            .write_page(PageId::new(0), &Page::filled(0x11))
            .unwrap();
        page_file
            .write_page(PageId::new(1), &Page::filled(0x22))
            .unwrap();

        assert_eq!(
            page_file.read_page(PageId::new(0)).unwrap(),
            Page::filled(0x11)
        );
        assert_eq!(
            page_file.read_page(PageId::new(1)).unwrap(),
            Page::filled(0x22)
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_page_beyond_end_of_file() {
        let path = temp_path("missing");
        let mut page_file = PageFile::create(&path).unwrap();
        page_file
            .write_page(PageId::new(0), &Page::filled(0x11))
            .unwrap();

        assert!(page_file.read_page(PageId::new(1)).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_partial_page() {
        let path = temp_path("partial");
        let mut page_file = PageFile::create(&path).unwrap();
        page_file.file.write_all(&[1, 2, 3]).unwrap();

        assert!(page_file.read_page(PageId::new(0)).is_err());
        fs::remove_file(path).unwrap();
    }
}
