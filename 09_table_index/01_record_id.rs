// Step 9-1: Encode and decode record ids.
//
// Run:
// rustc --edition=2021 --test 01_record_id.rs && ./01_record_id

#![allow(unused)]

const RECORD_ID_SIZE: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordId {
    page_id: PageId,
    slot_id: SlotId,
}

fn encode_record_id(record_id: RecordId) -> [u8; RECORD_ID_SIZE] {
    let mut bytes = [0; RECORD_ID_SIZE];
    bytes[0..4].copy_from_slice(&record_id.page_id.0.to_le_bytes());
    bytes[4..6].copy_from_slice(&record_id.slot_id.0.to_le_bytes());
    bytes
}

fn decode_record_id(bytes: &[u8]) -> Result<RecordId, String> {
    if bytes.len() < 6 {
        return Err("bytes too short for record id.".to_string());
    }
    let page_id = PageId(u32::from_le_bytes(bytes[0..4].try_into().unwrap()));
    let slot_id = SlotId(u16::from_le_bytes(bytes[4..6].try_into().unwrap()));
    Ok(RecordId { page_id, slot_id })
}

fn main() {
    println!("record id size = {}", RECORD_ID_SIZE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_id_has_fixed_size() {
        assert_eq!(RECORD_ID_SIZE, 6);
    }

    #[test]
    fn encodes_as_little_endian() {
        let record_id = RecordId {
            page_id: PageId(0x0102_0304),
            slot_id: SlotId(0x0506),
        };

        assert_eq!(
            encode_record_id(record_id),
            [0x04, 0x03, 0x02, 0x01, 0x06, 0x05]
        );
    }

    #[test]
    fn decodes_record_id() {
        let bytes = [1, 0, 0, 0, 2, 0];

        assert_eq!(
            decode_record_id(&bytes).unwrap(),
            RecordId {
                page_id: PageId(1),
                slot_id: SlotId(2),
            }
        );
    }

    #[test]
    fn rejects_short_input() {
        assert!(decode_record_id(&[1, 2, 3]).is_err());
    }
}
