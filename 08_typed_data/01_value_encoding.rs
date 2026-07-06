// Step 8-1: Encode and decode typed values.
//
// Run:
// rustc --edition=2021 --test 01_value_encoding.rs && ./01_value_encoding

#![allow(unused)]

const TAG_INT: u8 = 1;
const TAG_STRING: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Int(i64),
    String(String),
}

fn encode_value(value: &Value) -> Vec<u8> {
    match value {
        Value::Int(value) => {
            let mut bytes = vec![0u8; 9];
            bytes[0] = TAG_INT;
            bytes[1..9].copy_from_slice(&value.to_le_bytes());
            bytes
        }
        Value::String(value) => {
            let string_bytes = value.as_bytes();
            let mut bytes = Vec::with_capacity(1 + string_bytes.len());
            bytes.push(TAG_STRING);
            bytes.extend_from_slice(&(string_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(string_bytes);
            bytes
        }
    }
}

fn decode_value(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.is_empty() {
        return Err("bytes is empty".to_string());
    }
    match bytes[0] {
        TAG_INT => {
            if bytes.len() < 9 {
                return Err("not enough bytes for int".to_string());
            }
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes[1..9]);
            Ok((Value::Int(i64::from_le_bytes(arr)), 9))
        }
        TAG_STRING => {
            if bytes.len() < 5 {
                // at least, tag and length (4) should be included.
                return Err("not enough bytes for string".to_string());
            }
            let length = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as usize;
            if bytes.len() < 5 + length {
                return Err("not enough bytes for the length".to_string());
            }
            let text = std::str::from_utf8(&bytes[5..5 + length]).map_err(|e| e.to_string())?;
            Ok((Value::String(text.to_string()), 5 + length))
        }

        tag => Err(format!("unknown tag: {}", tag)),
    }
}

fn main() {
    println!("typed value encoding");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_int_as_tag_and_little_endian_i64() {
        assert_eq!(
            encode_value(&Value::Int(0x0102_0304_0506_0708)),
            vec![TAG_INT, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn decodes_int_and_consumed_size() {
        let bytes = [TAG_INT, 1, 0, 0, 0, 0, 0, 0, 0, 99];

        assert_eq!(decode_value(&bytes).unwrap(), (Value::Int(1), 9));
    }

    #[test]
    fn encodes_and_decodes_string() {
        let encoded = encode_value(&Value::String("alice".to_string()));

        assert_eq!(
            decode_value(&encoded).unwrap(),
            (Value::String("alice".to_string()), encoded.len())
        );
    }

    #[test]
    fn rejects_truncated_value() {
        assert!(decode_value(&[TAG_INT, 1, 2]).is_err());
        assert!(decode_value(&[TAG_STRING, 5, 0, 0, 0, b'a']).is_err());
    }
}
