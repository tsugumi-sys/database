// Step 7-4: Compact a slotted page after deletions.
//
// Run:
// rustc --edition=2021 --test 04_compaction.rs && ./04_compaction

#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlottedPage {
    data: [u8; PAGE_SIZE],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlotEntry {
    offset: u16,
    size: u16,
}

const PAGE_SIZE: usize = 256;
const SLOT_COUNT_OFFSET: usize = 0;
const FREE_START_OFFSET: usize = 2;
const FREE_END_OFFSET: usize = 4;
const HEADER_SIZE: usize = 6;
const SLOT_ENTRY_SIZE: usize = 4;
const TOMBSTONE_OFFSET: u16 = u16::MAX;

impl SlottedPage {
    fn new() -> Self {
        let mut page = [0; PAGE_SIZE];
        page[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        page[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
        Self { data: page }
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
        self.set_free_start(next_free_start);
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
    fn insert_cell(&mut self, bytes: &[u8]) -> Result<SlotId, String> {
        let free_start = self.free_start();
        let free_end = self.free_end();
        let cell_size = bytes.len() as usize;
        if free_end - free_start < cell_size + SLOT_ENTRY_SIZE {
            return Err("no enough space".to_string());
        }
        let slot_id = self.allocate_slot_id()?;
        let slot_offset = free_start;
        let cell_offset = free_end - cell_size;

        // write slot entry
        self.write_slot(
            slot_offset,
            SlotEntry {
                offset: cell_offset as u16,
                size: cell_size as u16,
            },
        )?;

        // write cell
        self.data[cell_offset..free_end].copy_from_slice(&bytes);
        self.set_free_end(cell_offset);

        Ok(slot_id)
    }

    fn slot_offset(&self, slot_id: SlotId) -> usize {
        HEADER_SIZE + SLOT_ENTRY_SIZE * slot_id.0 as usize
    }
    fn read_slot(&self, slot_id: SlotId) -> Result<SlotEntry, String> {
        let slot_offset = self.slot_offset(slot_id);
        let offset =
            u16::from_le_bytes(self.data[slot_offset..slot_offset + 2].try_into().unwrap());
        let size = u16::from_le_bytes(
            self.data[slot_offset + 2..slot_offset + 4]
                .try_into()
                .unwrap(),
        );
        Ok(SlotEntry { offset, size })
    }
    fn read_cell(&self, slot_id: SlotId) -> Result<Option<&[u8]>, String> {
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

    fn delete_cell(&mut self, slot_id: SlotId) -> Result<(), String> {
        let slot_offset = self.slot_offset(slot_id);
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
    fn compact(&mut self) {
        let mut new_data = self.data;
        let mut new_free_end = PAGE_SIZE;
        for sid in 0..self.slot_count() {
            let slot_id = SlotId(sid);
            let slot = self.read_slot(slot_id).unwrap();
            if slot.offset == TOMBSTONE_OFFSET {
                continue;
            }
            let old_start = slot.offset as usize;
            let cell_size = slot.size as usize;
            let old_end = old_start + cell_size;

            new_free_end -= cell_size;
            new_data[new_free_end..new_free_end + cell_size]
                .copy_from_slice(&self.data[old_start..old_end]);
            let slot_offset = self.slot_offset(slot_id);
            new_data[slot_offset..slot_offset + 2]
                .copy_from_slice(&(new_free_end as u16).to_le_bytes());
            new_data[slot_offset + 2..slot_offset + 4].copy_from_slice(&slot.size.to_le_bytes());
        }
        self.data = new_data;
        self.set_free_end(new_free_end);
    }
}

fn main() {
    println!("compact slotted page");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_preserves_live_slot_ids() {
        let mut page = SlottedPage::new();
        let a = page.insert_cell(b"alpha").unwrap();
        let b = page.insert_cell(b"bravo bravo").unwrap();
        let c = page.insert_cell(b"charlie").unwrap();

        page.delete_cell(b).unwrap();
        page.compact();

        assert_eq!(page.read_cell(a).unwrap(), Some(&b"alpha"[..]));
        assert_eq!(page.read_cell(b).unwrap(), None);
        assert_eq!(page.read_cell(c).unwrap(), Some(&b"charlie"[..]));
    }

    #[test]
    fn compact_keeps_deleted_slots_absent() {
        let mut page = SlottedPage::new();
        let deleted = page.insert_cell(b"deleted").unwrap();
        let live = page.insert_cell(b"live").unwrap();

        page.delete_cell(deleted).unwrap();
        page.compact();

        assert_eq!(page.read_cell(deleted).unwrap(), None);
        assert_eq!(page.read_cell(live).unwrap(), Some(&b"live"[..]));
    }

    #[test]
    fn compact_increases_contiguous_free_space() {
        let mut page = SlottedPage::new();
        let a = page.insert_cell(&[1; 20]).unwrap();
        let b = page.insert_cell(&[2; 40]).unwrap();
        let c = page.insert_cell(&[3; 20]).unwrap();
        page.delete_cell(b).unwrap();
        let before = page.free_space();

        page.compact();

        assert!(page.free_space() > before);
        assert_eq!(page.read_cell(a).unwrap(), Some(&[1; 20][..]));
        assert_eq!(page.read_cell(c).unwrap(), Some(&[3; 20][..]));
    }

    #[test]
    fn compact_makes_reclaimed_space_usable() {
        let mut page = SlottedPage::new();
        let deleted = page.insert_cell(&[1; 120]).unwrap();
        let live = page.insert_cell(&[2; 40]).unwrap();

        page.delete_cell(deleted).unwrap();
        assert!(page.insert_cell(&[3; 80]).is_err());

        page.compact();
        let inserted = page.insert_cell(&[3; 80]).unwrap();

        assert_eq!(page.read_cell(deleted).unwrap(), None);
        assert_eq!(page.read_cell(live).unwrap(), Some(&[2; 40][..]));
        assert_eq!(page.read_cell(inserted).unwrap(), Some(&[3; 80][..]));
    }

    #[test]
    fn compact_updates_live_slot_offsets() {
        let mut page = SlottedPage::new();
        let deleted = page.insert_cell(&[1; 120]).unwrap();
        let live = page.insert_cell(&[2; 40]).unwrap();
        let before = page.read_slot(live).unwrap();

        page.delete_cell(deleted).unwrap();
        page.compact();

        let after = page.read_slot(live).unwrap();
        assert_ne!(after.offset, before.offset);
        assert_eq!(after.offset, (PAGE_SIZE - 40) as u16);
        assert_eq!(after.size, 40);
        assert_eq!(page.read_cell(live).unwrap(), Some(&[2; 40][..]));
    }
}
