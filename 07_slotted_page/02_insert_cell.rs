// Step 7-2: Insert variable-length cells into a slotted page.
//
// Run:
// rustc --edition=2021 --test 02_insert_cell.rs && ./02_insert_cell

#![allow(unused)]

const PAGE_SIZE: usize = 256;
const SLOT_COUNT_OFFSET: usize = 0;
const FREE_START_OFFSET: usize = 2;
const FREE_END_OFFSET: usize = 4;
const HEADER_SIZE: usize = 6;
const SLOT_ENTRY_SIZE: usize = 4;
const TOMBSTONE_OFFSET: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlottedPage {
    data: [u8; PAGE_SIZE],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotEntry {
    offset: u16,
    size: u16,
}

impl SlottedPage {
    fn new() -> Self {
        let mut page = [0; PAGE_SIZE];
        page[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        page[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
        Self { data: page }
    }

    fn slot_count(&self) -> u16 {
        let bytes: [u8; 2] = self.data[SLOT_COUNT_OFFSET..SLOT_COUNT_OFFSET + 2]
            .try_into()
            .unwrap();
        u16::from_le_bytes(bytes)
    }
    fn set_slot_count(&mut self, count: u16) {
        self.data[SLOT_COUNT_OFFSET..SLOT_COUNT_OFFSET + 2].copy_from_slice(&count.to_le_bytes());
    }

    fn free_start(&self) -> u16 {
        let bytes: [u8; 2] = self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .try_into()
            .unwrap();
        u16::from_le_bytes(bytes)
    }
    fn free_end(&self) -> u16 {
        let bytes: [u8; 2] = self.data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .try_into()
            .unwrap();
        u16::from_le_bytes(bytes)
    }
    fn free_space(&self) -> usize {
        (self.free_end() - self.free_start()) as usize
    }

    fn set_slot(&mut self, slot_id: SlotId, entry: SlotEntry) {
        let old_count = self.slot_count();
        let new_count = self.slot_count().max(slot_id.0 + 1);
        for empty_slot in old_count..new_count {
            let offset = HEADER_SIZE + empty_slot as usize * SLOT_ENTRY_SIZE;
            self.data[offset..offset + 2].copy_from_slice(&(TOMBSTONE_OFFSET as u16).to_le_bytes());
            self.data[offset + 2..offset + 4].copy_from_slice(&0u16.to_le_bytes());
        }
        let offset = HEADER_SIZE + slot_id.0 as usize * SLOT_ENTRY_SIZE;
        self.data[offset..offset + 2].copy_from_slice(&entry.offset.to_le_bytes());
        self.data[offset + 2..offset + 4].copy_from_slice(&entry.size.to_le_bytes());
        if offset + 4 > self.free_start().into() {
            self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
                .copy_from_slice(&(offset as u16 + 4).to_le_bytes());
        }
        self.set_slot_count(new_count);
    }
    fn insert_cell(&mut self, bytes: &[u8]) -> Result<SlotId, String> {
        if self.free_space() < bytes.len() + SLOT_ENTRY_SIZE {
            return Err("free space is not enough".to_string());
        }
        let count = self.slot_count();
        let bytes_len = bytes.len() as usize;
        let slot_id = SlotId(count);
        let cell_offset = self.free_end() as usize - bytes_len;
        self.data[cell_offset..cell_offset + bytes_len].copy_from_slice(bytes);
        self.data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(cell_offset as u16).to_le_bytes());

        self.set_slot(
            slot_id,
            SlotEntry {
                offset: cell_offset as u16,
                size: bytes_len as u16,
            },
        );

        Ok(slot_id)
    }

    fn slot(&self, slot_id: SlotId) -> Option<SlotEntry> {
        let slot_offset = HEADER_SIZE + slot_id.0 as usize * SLOT_ENTRY_SIZE;
        if slot_offset >= self.free_start().into() {
            return None;
        }
        let offset_b: [u8; 2] = self.data[slot_offset..slot_offset + 2].try_into().unwrap();
        let size_b: [u8; 2] = self.data[slot_offset + 2..slot_offset + 4]
            .try_into()
            .unwrap();

        let entry = SlotEntry {
            offset: u16::from_le_bytes(offset_b),
            size: u16::from_le_bytes(size_b),
        };
        if entry.offset == TOMBSTONE_OFFSET {
            None
        } else {
            Some(entry)
        }
    }
    fn read_cell(&self, slot_id: SlotId) -> Option<&[u8]> {
        if let Some(slot) = self.slot(slot_id) {
            return Some(&self.data[slot.offset as usize..(slot.offset + slot.size).into()]);
        } else {
            return None;
        }
    }
}

fn main() {
    println!("slotted page size = {}", PAGE_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_cell_and_returns_slot_id() {
        let mut page = SlottedPage::new();

        let slot = page.insert_cell(b"hello").unwrap();

        assert_eq!(slot, SlotId(0));
        assert_eq!(page.slot_count(), 1);
        assert_eq!(page.read_cell(slot), Some(&b"hello"[..]));
    }

    #[test]
    fn inserts_cells_with_different_lengths() {
        let mut page = SlottedPage::new();

        let a = page.insert_cell(b"a").unwrap();
        let b = page.insert_cell(b"longer value").unwrap();

        assert_eq!(a, SlotId(0));
        assert_eq!(b, SlotId(1));
        assert_eq!(page.slot_count(), 2);
        assert_eq!(page.read_cell(a), Some(&b"a"[..]));
        assert_eq!(page.read_cell(b), Some(&b"longer value"[..]));
    }

    #[test]
    fn insert_reduces_free_space() {
        let mut page = SlottedPage::new();
        let before = page.free_space();

        page.insert_cell(b"hello").unwrap();

        assert!(page.free_space() < before);
    }

    #[test]
    fn rejects_cell_when_space_is_insufficient() {
        let mut page = SlottedPage::new();
        let too_large = vec![0xab; PAGE_SIZE];

        assert!(page.insert_cell(&too_large).is_err());
    }
}
