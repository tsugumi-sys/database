// Step 4-5: Exercise a persistent page file end to end.
//
// Run:
// rustc --edition=2021 --test 05_page_file_smoke.rs && ./05_page_file_smoke

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

    fn write_bytes(&mut self, offset: usize, bytes: &[u8]) -> io::Result<()> {
        let end = offset
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "offset overflow"))?;
        if end > PAGE_SIZE {
            return Err(io::Error::new(ErrorKind::InvalidInput, "out of bounds"));
        }
        self.data[offset..end].copy_from_slice(bytes);
        Ok(())
    }

    fn read_bytes(&self, offset: usize, len: usize) -> io::Result<&[u8]> {
        let end = offset
            .checked_add(len)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "offset overflow"))?;
        if end > PAGE_SIZE {
            return Err(io::Error::new(ErrorKind::InvalidInput, "out of bounds"));
        }
        Ok(&self.data[offset..end])
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
        self.file.seek(SeekFrom::Start(pageid.file_offset()))?;
        self.file.write_all(&Page::empty().data)?;
        self.next_page_id += 1;
        Ok(pageid)
    }

    fn ensure_allocated(&self, page_id: PageId) -> io::Result<()> {
        if page_id.0 >= self.next_page_id {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "page is not allocated",
            ));
        }
        Ok(())
    }

    fn write_page(&mut self, page_id: PageId, page: &Page) -> io::Result<()> {
        self.ensure_allocated(page_id)?;
        self.file.seek(SeekFrom::Start(page_id.file_offset()))?;
        self.file.write_all(&page.data)
    }

    fn read_page(&mut self, page_id: PageId) -> io::Result<Page> {
        self.ensure_allocated(page_id)?;
        self.file.seek(SeekFrom::Start(page_id.file_offset()))?;
        let mut page = Page::empty();
        self.file.read_exact(&mut page.data)?;
        Ok(page)
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
            "database_step4_{}_{}_smoke.db",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn persists_pages_across_reopen() {
        let path = temp_path("persist");
        let (users_page_id, index_page_id);
        {
            let mut page_file = PageFile::open(&path).unwrap();
            users_page_id = page_file.allocate_page().unwrap();
            index_page_id = page_file.allocate_page().unwrap();

            let mut users_page = Page::empty();
            users_page.write_bytes(0, b"users").unwrap();
            let mut index_page = Page::empty();
            index_page.write_bytes(100, b"root").unwrap();
            page_file.write_page(users_page_id, &users_page).unwrap();
            page_file.write_page(index_page_id, &index_page).unwrap();
            page_file.flush().unwrap();
        }

        let mut reopened = PageFile::open(&path).unwrap();
        assert_eq!(
            reopened
                .read_page(users_page_id)
                .unwrap()
                .read_bytes(0, 5)
                .unwrap(),
            b"users"
        );
        assert_eq!(
            reopened
                .read_page(index_page_id)
                .unwrap()
                .read_bytes(100, 4)
                .unwrap(),
            b"root"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn allocate_after_reopen_preserves_existing_pages() {
        let path = temp_path("append");
        {
            let mut page_file = PageFile::open(&path).unwrap();
            let first = page_file.allocate_page().unwrap();
            let mut page = Page::empty();
            page.write_bytes(0, b"first").unwrap();
            page_file.write_page(first, &page).unwrap();
            page_file.flush().unwrap();
        }

        let mut reopened = PageFile::open(&path).unwrap();
        let next = reopened.allocate_page().unwrap();

        assert_eq!(next, PageId::new(1));
        assert_eq!(
            reopened
                .read_page(PageId::new(0))
                .unwrap()
                .read_bytes(0, 5)
                .unwrap(),
            b"first"
        );
        assert_eq!(reopened.read_page(next).unwrap(), Page::empty());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_io_for_unallocated_pages() {
        let path = temp_path("unallocated");
        let mut page_file = PageFile::open(&path).unwrap();

        assert!(page_file.read_page(PageId::new(0)).is_err());
        assert!(page_file
            .write_page(PageId::new(0), &Page::empty())
            .is_err());
        fs::remove_file(path).unwrap();
    }
}
