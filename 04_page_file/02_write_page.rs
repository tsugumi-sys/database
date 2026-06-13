// Step 4-2: Write fixed-size pages to a file.
//
// Run:
// rustc --edition=2021 --test 02_write_page.rs && ./02_write_page

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
        self.file.write_all(&page.data)?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()?;
        Ok(())
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
            "database_step4_{}_{}_write.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn writes_exactly_one_page() {
        let path = temp_path("one_page");
        let mut page_file = PageFile::create(&path).unwrap();

        page_file
            .write_page(PageId::new(0), &Page::filled(0xab))
            .unwrap();
        page_file.flush().unwrap();

        assert_eq!(fs::metadata(&path).unwrap().len(), PAGE_SIZE as u64);
        let mut bytes = Vec::new();
        File::open(&path).unwrap().read_to_end(&mut bytes).unwrap();
        assert!(bytes.iter().all(|byte| *byte == 0xab));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn writes_at_the_requested_page_offset() {
        let path = temp_path("offset");
        let mut page_file = PageFile::create(&path).unwrap();

        page_file
            .write_page(PageId::new(1), &Page::filled(0xcd))
            .unwrap();
        page_file.flush().unwrap();

        let mut bytes = Vec::new();
        File::open(&path).unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes.len(), PAGE_SIZE * 2);
        assert!(bytes[..PAGE_SIZE].iter().all(|byte| *byte == 0));
        assert!(bytes[PAGE_SIZE..].iter().all(|byte| *byte == 0xcd));
        fs::remove_file(path).unwrap();
    }
}
