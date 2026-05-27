// Step 3-5: Serialize records into a page.
//
// Run:
// rustc --edition=2021 --test 05_page_serialization.rs && ./05_page_serialization

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const RECORD_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    key: u32,
    value: i32,
}

impl Record {
    fn new(key: u32, value: i32) -> Self {
        Self { key, value }
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

    fn write_record(&mut self, offset: usize, record: &Record) -> Result<(), String> {
        let value_end = offset.checked_add(RECORD_SIZE).ok_or("overflow")?;
        if value_end > self.data.len() {
            return Err("out of bounds".to_string());
        }
        let value_start = offset + RECORD_SIZE / 2;
        let value_bytes = record.value.to_le_bytes();
        self.data[value_start..value_end].copy_from_slice(&value_bytes);

        let key_bytes = record.key.to_le_bytes();
        self.data[offset..value_start].copy_from_slice(&key_bytes);

        Ok(())
    }

    fn read_record(&self, offset: usize) -> Result<Record, String> {
        let value_end = offset.checked_add(RECORD_SIZE).ok_or("overflow")?;
        if value_end > self.data.len() {
            return Err("out of bounds".to_string());
        }
        let value_start = offset + RECORD_SIZE / 2;
        let value_bytes: [u8; RECORD_SIZE / 2] =
            self.data[value_start..value_end].try_into().unwrap();
        let key_bytes: [u8; RECORD_SIZE / 2] = self.data[offset..value_start].try_into().unwrap();

        Ok(Record {
            key: u32::from_le_bytes(key_bytes),
            value: i32::from_le_bytes(value_bytes),
        })
    }
}

fn main() {
    let record = Record::new(1, 100);
    println!("{:?}", record);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_has_fixed_disk_size() {
        assert_eq!(RECORD_SIZE, 8);
    }

    #[test]
    fn writes_record_as_little_endian_bytes() {
        let mut page = Page::new();
        let record = Record::new(0x0102_0304, -2);

        page.write_record(10, &record).unwrap();

        assert_eq!(
            &page.data[10..18],
            &[0x04, 0x03, 0x02, 0x01, 0xfe, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn reads_record_from_page() {
        let mut page = Page::new();

        page.write_record(0, &Record::new(10, 200)).unwrap();

        assert_eq!(page.read_record(0).unwrap(), Record::new(10, 200));
    }

    #[test]
    fn writes_multiple_records() {
        let mut page = Page::new();

        page.write_record(0, &Record::new(1, 10)).unwrap();
        page.write_record(RECORD_SIZE, &Record::new(2, 20)).unwrap();

        assert_eq!(page.read_record(0).unwrap(), Record::new(1, 10));
        assert_eq!(page.read_record(RECORD_SIZE).unwrap(), Record::new(2, 20));
    }

    #[test]
    fn rejects_record_past_end() {
        let mut page = Page::new();

        assert!(page
            .write_record(PAGE_SIZE - 1, &Record::new(1, 1))
            .is_err());
        assert!(page.read_record(PAGE_SIZE - 1).is_err());
    }
}
