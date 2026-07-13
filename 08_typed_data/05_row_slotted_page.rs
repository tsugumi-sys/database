// Step 8-5: Store encoded rows in a slotted page.
//
// Run:
// rustc --edition=2021 --test 05_row_slotted_page.rs && ./05_row_slotted_page

#![allow(unused)]

const TAG_NULL: u8 = 0;
const TAG_INT: u8 = 1;
const TAG_STRING: u8 = 2;

const SLOT_COUNT_OFFSET: usize = 0;
const FREE_START_OFFSET: usize = 2;
const FREE_END_OFFSET: usize = 4;
const HEADER_SIZE: usize = 6;
const SLOT_ENTRY_SIZE: usize = 4;
const TOMBSTONE_OFFSET: u16 = u16::MAX;

const PAGE_SIZE: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Null,
    Int(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    values: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlottedPage {
    data: [u8; PAGE_SIZE],
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct SlotEntry {
    offset: u16,
    size: u16,
}

fn encode_value(value: &Value) -> Vec<u8> {
    match value {
        Value::Int(value) => {
            let mut bytes = vec![0u8; 9]; // tag + int bytes
            bytes[0] = TAG_INT;
            bytes[1..9].copy_from_slice(&value.to_le_bytes());
            bytes
        }
        Value::String(value) => {
            let string_bytes = value.as_bytes();
            let mut bytes = Vec::with_capacity(1 + string_bytes.len()); // vec![v; size] はcompile時にsizeが確定している必要があるため、Stringなどには使えない。
            bytes.push(TAG_STRING);
            bytes.extend_from_slice(&(string_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(string_bytes);
            bytes
        }
        Value::Null => {
            vec![TAG_NULL; 1]
        }
    }
}
fn decode_value(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.is_empty() {
        return Err("bytes cannot be empty".to_string());
    }
    match bytes[0] {
        TAG_NULL => Ok((Value::Null, 1)),
        TAG_INT => {
            if bytes.len() < 5 {
                return Err("not enough bytes for int".to_string());
            }
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes[1..9]);
            Ok((Value::Int(i64::from_le_bytes(arr)), 9))
        }
        TAG_STRING => {
            if bytes.len() < 5 {
                return Err("not enough bytes for string".to_string());
            }
            let length = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as usize;
            if bytes.len() < 5 + length {
                return Err("not enough bytes for string".to_string());
            }
            let text = std::str::from_utf8(&bytes[5..5 + length]).map_err(|e| e.to_string())?;
            Ok((Value::String(text.to_string()), 5 + length))
        }
        tag => Err(format!("unknwon tag: {}", tag)),
    }
}

fn encode_row(row: &Row) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend((row.values.len() as u16).to_le_bytes());
    for item in &row.values {
        bytes.extend(encode_value(&item));
    }
    bytes
}

fn decode_row(bytes: &[u8]) -> Result<Row, String> {
    let mut offset = 2; // consume row counts
    let mut values: Vec<Value> = Vec::new();
    while offset < bytes.len() {
        let (value, consumed_size) = decode_value(&bytes[offset..])?;
        values.push(value);
        offset += consumed_size;
    }
    Ok(Row { values })
}

impl SlottedPage {
    fn new() -> Self {
        let mut data = [0; PAGE_SIZE];
        data[FREE_START_OFFSET..FREE_START_OFFSET + 2]
            .copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        data[FREE_END_OFFSET..FREE_END_OFFSET + 2]
            .copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
        Self { data }
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

    fn insert_cell(&mut self, bytes: &[u8]) -> Result<SlotId, String> {
        let cell_size = bytes.len() as usize;
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
        self.data[cell_offset..cell_offset + cell_size].copy_from_slice(&bytes);
        self.set_free_end(cell_offset);
        Ok(new_slot_id)
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

    fn insert_row(&mut self, row: &Row) -> Result<SlotId, String> {
        let bytes = encode_row(row);
        self.insert_cell(&bytes)
    }

    fn read_row(&self, slot_id: SlotId) -> Result<Option<Row>, String> {
        let bytes = self.read_cell(slot_id)?;
        if let Some(row_bytes) = bytes {
            let decoded = decode_row(row_bytes)?;
            Ok(Some(decoded))
        } else {
            Ok(None)
        }
    }
}

fn main() {
    println!("row slotted page");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_reads_row_with_string() {
        let mut page = SlottedPage::new();
        let row = Row {
            values: vec![Value::Int(1), Value::String("alice".to_string())],
        };

        let slot = page.insert_row(&row).unwrap();

        assert_eq!(page.read_row(slot).unwrap(), Some(row));
    }

    #[test]
    fn stores_multiple_rows() {
        let mut page = SlottedPage::new();
        let first = Row {
            values: vec![Value::Int(1), Value::String("alice".to_string())],
        };
        let second = Row {
            values: vec![Value::Int(2), Value::Null],
        };

        let first_slot = page.insert_row(&first).unwrap();
        let second_slot = page.insert_row(&second).unwrap();

        assert_eq!(page.read_row(first_slot).unwrap(), Some(first));
        assert_eq!(page.read_row(second_slot).unwrap(), Some(second));
    }
}
