// Step 7-3: Read cells by slot id and mark deleted cells.
//
// Run:
// rustc --edition=2021 --test 03_read_delete_cell.rs && ./03_read_delete_cell

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

impl SlottedPage {
    fn new() -> Self {
        let mut page = [0; PAGE_SIZE];
        page[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        page[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
        Self { data: page }
    }

    fn insert_cell(&mut self, bytes: &[u8]) -> Result<SlotId, String> {
        let free_start = u16::from_le_bytes(
            self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
                .try_into()
                .unwrap(),
        ) as usize;
        let free_end = u16::from_le_bytes(
            self.data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
                .try_into()
                .unwrap(),
        ) as usize;
        let cell_size = bytes.len() as usize;
        if free_end - free_start < cell_size {
            return Err("no enough space".to_string());
        }
        // write slot entry
        let cell_offset = free_end - bytes.len() as usize;
        let slot_offset = free_start;
        self.data[slot_offset..slot_offset + 2]
            .copy_from_slice(&(cell_offset as u16).to_le_bytes());
        self.data[slot_offset + 2..slot_offset + 4]
            .copy_from_slice(&(cell_size as u16).to_le_bytes());
        self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(slot_offset as u16 + 4).to_le_bytes());

        // write cell
        self.data[cell_offset..free_end].copy_from_slice(&bytes);
        self.data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(cell_offset as u16).to_le_bytes());

        let slot_id = SlotId(u16::from_le_bytes(
            self.data[SLOT_COUNT_OFFSET..SLOT_COUNT_OFFSET + 2]
                .try_into()
                .unwrap(),
        ));
        self.data[SLOT_COUNT_OFFSET..SLOT_COUNT_OFFSET + 2]
            .copy_from_slice(&(slot_id.0 + 1).to_le_bytes());

        Ok(slot_id)
    }

    fn read_cell(&self, slot_id: SlotId) -> Result<Option<&[u8]>, String> {
        let slot_offset = HEADER_SIZE + SLOT_ENTRY_SIZE * slot_id.0 as usize;
        let free_start = u16::from_le_bytes(
            self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
                .try_into()
                .unwrap(),
        ) as usize;
        if slot_offset > free_start {
            return Err("invalid offset".to_string());
        }
        let offset =
            u16::from_le_bytes(self.data[slot_offset..slot_offset + 2].try_into().unwrap());
        let size = u16::from_le_bytes(
            self.data[slot_offset + 2..slot_offset + 4]
                .try_into()
                .unwrap(),
        );
        if offset == TOMBSTONE_OFFSET {
            return Ok(None);
        }
        Ok(Some(&self.data[offset as usize..(offset + size) as usize]))
    }

    fn delete_cell(&mut self, slot_id: SlotId) -> Result<(), String> {
        let slot_offset = HEADER_SIZE + SLOT_ENTRY_SIZE * slot_id.0 as usize;
        let free_start = u16::from_le_bytes(
            self.data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
                .try_into()
                .unwrap(),
        ) as usize;

        if slot_offset > free_start {
            return Err("invalid offset".to_string());
        }
        self.data[slot_offset..slot_offset + 2].copy_from_slice(&TOMBSTONE_OFFSET.to_le_bytes());
        Ok(())
    }
}

fn main() {
    println!("read and delete slotted cells");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_existing_cells() {
        let mut page = SlottedPage::new();
        let first = page.insert_cell(b"first").unwrap();
        let second = page.insert_cell(b"second").unwrap();

        assert_eq!(page.read_cell(first).unwrap(), Some(&b"first"[..]));
        assert_eq!(page.read_cell(second).unwrap(), Some(&b"second"[..]));
    }

    #[test]
    fn delete_marks_cell_as_absent() {
        let mut page = SlottedPage::new();
        let slot = page.insert_cell(b"dead").unwrap();

        page.delete_cell(slot).unwrap();

        assert_eq!(page.read_cell(slot).unwrap(), None);
    }

    #[test]
    fn delete_does_not_affect_other_slots() {
        let mut page = SlottedPage::new();
        let first = page.insert_cell(b"first").unwrap();
        let second = page.insert_cell(b"second").unwrap();
        let third = page.insert_cell(b"third").unwrap();

        page.delete_cell(second).unwrap();

        assert_eq!(page.read_cell(first).unwrap(), Some(&b"first"[..]));
        assert_eq!(page.read_cell(second).unwrap(), None);
        assert_eq!(page.read_cell(third).unwrap(), Some(&b"third"[..]));
    }

    #[test]
    fn invalid_slot_is_an_error() {
        let page = SlottedPage::new();

        assert!(page.read_cell(SlotId(7)).is_err());
    }

    #[test]
    fn deleting_invalid_slot_is_an_error() {
        let mut page = SlottedPage::new();

        assert!(page.delete_cell(SlotId(7)).is_err());
    }
}
