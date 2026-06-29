// Step 7-1: Define a slotted page header and slot directory.
//
// Run:
// rustc --edition=2021 --test 01_slot_directory.rs && ./01_slot_directory

#![allow(unused)]

const PAGE_SIZE: usize = 4096;
const SLOT_COUNT_OFFSET: usize = 0;
const FREE_START_OFFSET: usize = 2;
const FREE_END_OFFSET: usize = 4;
const HEADER_SIZE: usize = 6;
const SLOT_ENTRY_SIZE: usize = 4;
const TOMBSTONE_OFFSET: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotEntry {
    offset: u16,
    size: u16,
}

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
}

fn main() {
    println!("slot entry size = {}", SLOT_ENTRY_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_header() {
        let page = SlottedPage::new();

        assert_eq!(page.slot_count(), 0);
        assert_eq!(page.free_start(), HEADER_SIZE as u16);
        assert_eq!(page.free_end(), PAGE_SIZE as u16);
    }

    #[test]
    fn writes_and_reads_slot_entry() {
        let mut page = SlottedPage::new();

        page.set_slot(
            SlotId(0),
            SlotEntry {
                offset: 4000,
                size: 12,
            },
        );

        assert_eq!(
            page.slot(SlotId(0)),
            Some(SlotEntry {
                offset: 4000,
                size: 12
            })
        );
    }

    #[test]
    fn writing_slot_updates_header_space() {
        let mut page = SlottedPage::new();

        page.set_slot(
            SlotId(0),
            SlotEntry {
                offset: 4000,
                size: 12,
            },
        );

        assert_eq!(page.slot_count(), 1);
        assert_eq!(page.free_start(), (HEADER_SIZE + SLOT_ENTRY_SIZE) as u16);
        assert_eq!(page.free_end(), PAGE_SIZE as u16);
    }

    #[test]
    fn missing_slot_returns_none() {
        let page = SlottedPage::new();

        assert_eq!(page.slot(SlotId(0)), None);
    }
}
