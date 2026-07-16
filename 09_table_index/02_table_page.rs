// Step 9-2: Store row bytes in a table page and return RecordId.
//
// Run:
// rustc --edition=2021 --test 02_table_page.rs && ./02_table_page

#![allow(unused)]

const PAGE_SIZE: usize = 512;
const SLOT_COUNT_OFFSET: usize = 0;
const FREE_START_OFFSET: usize = 2;
const FREE_END_OFFSET: usize = 4;
const HEADER_SIZE: usize = 6;
const SLOT_ENTRY_SIZE: usize = 4;
const TOMBSTONE_OFFSET: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordId {
    page_id: PageId,
    slot_id: SlotId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TablePage {
    page_id: PageId,
    data: [u8; PAGE_SIZE],
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct SlotEntry {
    offset: u16,
    size: u16,
}

impl TablePage {
    fn new(page_id: PageId) -> Self {
        let mut page = [0; PAGE_SIZE];
        page[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        page[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
        Self {
            page_id,
            data: page,
        }
    }

    fn free_space(&self) -> usize {
        self.free_end() - self.free_start()
    }
    fn free_start(&self) -> usize {
        u16::from_le_bytes(
            self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
                .try_into()
                .unwrap(),
        )
        .into()
    }
    fn free_end(&self) -> usize {
        u16::from_le_bytes(
            self.data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
                .try_into()
                .unwrap(),
        )
        .into()
    }
    fn set_free_start(&mut self, start: usize) {
        self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(start as u16).to_le_bytes());
    }
    fn set_free_end(&mut self, end: usize) {
        self.data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(end as u16).to_le_bytes());
    }
    fn slot_count(&self) -> u16 {
        u16::from_le_bytes(
            self.data[SLOT_COUNT_OFFSET..SLOT_COUNT_OFFSET + 2]
                .try_into()
                .unwrap(),
        )
    }
    fn set_slot_count(&mut self, count: u16) {
        self.data[SLOT_COUNT_OFFSET..SLOT_COUNT_OFFSET + 2].copy_from_slice(&count.to_le_bytes());
    }
    fn allocate_slot_id(&mut self) -> Result<SlotId, String> {
        let slot_id = SlotId(self.slot_count());
        let next_free_start = self.free_start() + SLOT_ENTRY_SIZE;
        if next_free_start > self.free_end() {
            return Err("no enough space".to_string());
        }
        self.set_slot_count(slot_id.0 as u16 + 1);
        Ok(slot_id)
    }
    fn write_slot(&mut self, slot_offset: usize, slot_entry: SlotEntry) -> Result<(), String> {
        if self.free_space() < SLOT_ENTRY_SIZE {
            return Err("no enough space".to_string());
        }
        self.data[slot_offset..slot_offset + 2].copy_from_slice(&(slot_entry.offset).to_le_bytes());
        self.data[slot_offset + 2..slot_offset + 4]
            .copy_from_slice(&(slot_entry.size).to_le_bytes());
        Ok(())
    }

    fn insert_record(&mut self, row_bytes: &[u8]) -> Result<RecordId, String> {
        let cell_size = row_bytes.len() as usize;
        if self.free_space() < cell_size + SLOT_ENTRY_SIZE {
            return Err("no enough space".to_string());
        }
        let new_slot_id = self.allocate_slot_id()?;
        let slot_offset = self.free_start();
        self.set_free_start(slot_offset + SLOT_ENTRY_SIZE);
        let cell_offset = self.free_end() - cell_size;

        self.write_slot(
            slot_offset,
            SlotEntry {
                offset: cell_offset as u16,
                size: cell_size as u16,
            },
        )?;
        self.data[cell_offset..cell_offset + cell_size].copy_from_slice(&row_bytes);
        self.set_free_end(cell_offset);
        Ok(RecordId {
            page_id: self.page_id,
            slot_id: new_slot_id,
        })
    }

    fn slot_offset(&self, slot_id: SlotId) -> usize {
        HEADER_SIZE + SLOT_ENTRY_SIZE * slot_id.0 as usize
    }
    fn read_slot(&self, slot_id: SlotId) -> Result<SlotEntry, String> {
        let slot_offset = self.slot_offset(slot_id);
        if slot_offset > self.free_start() {
            return Err("this slot is not allocated".to_string());
        }
        let offset =
            u16::from_le_bytes(self.data[slot_offset..slot_offset + 2].try_into().unwrap());
        let size = u16::from_le_bytes(
            self.data[slot_offset + 2..slot_offset + 4]
                .try_into()
                .unwrap(),
        );
        Ok(SlotEntry { offset, size })
    }

    fn read_record(&self, record_id: RecordId) -> Result<Option<&[u8]>, String> {
        let slot_id = record_id.slot_id;
        let slot_offset = self.slot_offset(slot_id);
        let free_start = self.free_start();
        if slot_offset > free_start {
            return Err("invalid offset".to_string());
        }
        let slot_entry = self.read_slot(slot_id)?;
        if slot_entry.offset == TOMBSTONE_OFFSET {
            return Ok(None);
        }
        Ok(Some(
            &self.data[slot_entry.offset as usize..(slot_entry.offset + slot_entry.size) as usize],
        ))
    }

    fn delete_record(&mut self, record_id: RecordId) -> Result<(), String> {
        let slot_offset = self.slot_offset(record_id.slot_id);
        let free_start = self.free_start();

        if slot_offset > free_start {
            return Err("invalid offset".to_string());
        }
        self.write_slot(
            slot_offset,
            SlotEntry {
                offset: TOMBSTONE_OFFSET,
                size: 0,
            },
        )?;
        Ok(())
    }
}

fn main() {
    println!("table page");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_record_returns_record_id() {
        let mut page = TablePage::new(PageId(7));

        let record_id = page.insert_record(b"row one").unwrap();

        assert_eq!(record_id.page_id, PageId(7));
        assert_eq!(record_id.slot_id, SlotId(0));
        assert_eq!(page.read_record(record_id).unwrap(), Some(&b"row one"[..]));
    }

    #[test]
    fn reads_multiple_records() {
        let mut page = TablePage::new(PageId(7));
        let first = page.insert_record(b"first").unwrap();
        let second = page.insert_record(b"second").unwrap();

        assert_eq!(page.read_record(first).unwrap(), Some(&b"first"[..]));
        assert_eq!(page.read_record(second).unwrap(), Some(&b"second"[..]));
    }

    #[test]
    fn deleted_record_reads_as_none() {
        let mut page = TablePage::new(PageId(7));
        let record_id = page.insert_record(b"dead").unwrap();

        page.delete_record(record_id).unwrap();

        assert_eq!(page.read_record(record_id).unwrap(), None);
    }
}
