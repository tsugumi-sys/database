// Step 3-2: Fixed-size page.
//
// Run:
// rustc --edition=2021 --test 02_fixed_size_page.rs && ./02_fixed_size_page

#![allow(unused)]

const PAGE_SIZE: usize = 4096;

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

    fn write_bytes(&mut self, offset: usize, bytes: &[u8]) -> Result<(), String> {
        if offset + bytes.len() > self.data.len() {
            return Err("out of bounds".to_string());
        }
        for i in 0..bytes.len() {
            self.data[offset + i] = bytes[i];
        }
        Ok(())
    }

    fn read_bytes(&self, offset: usize, len: usize) -> Result<&[u8], String> {
        if offset + len > self.data.len() {
            return Err("out of bounds".to_string());
        }
        Ok(&self.data[offset..offset + len])
    }
}

fn main() {
    let page = Page::new();
    println!("page size = {}", page.data.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_page_is_zero_filled() {
        let page = Page::new();

        assert_eq!(page.data.len(), PAGE_SIZE);
        assert!(page.data.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn writes_and_reads_bytes() {
        let mut page = Page::new();

        page.write_bytes(10, &[1, 2, 3]).unwrap();

        assert_eq!(page.read_bytes(10, 3).unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn write_does_not_touch_other_offsets() {
        let mut page = Page::new();

        page.write_bytes(100, &[9, 8]).unwrap();

        assert_eq!(page.read_bytes(99, 4).unwrap(), &[0, 9, 8, 0]);
    }

    #[test]
    fn rejects_write_past_end() {
        let mut page = Page::new();

        assert!(page.write_bytes(PAGE_SIZE - 1, &[1, 2]).is_err());
    }

    #[test]
    fn rejects_read_past_end() {
        let page = Page::new();

        assert!(page.read_bytes(PAGE_SIZE - 1, 2).is_err());
    }
}
