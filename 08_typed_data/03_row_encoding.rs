// Step 8-3: Encode and decode rows made of typed values.
//
// Run:
// rustc --edition=2021 --test 03_row_encoding.rs && ./03_row_encoding

#![allow(unused)]

const TAG_NULL: u8 = 0;
const TAG_INT: u8 = 1;
const TAG_STRING: u8 = 2;

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

fn main() {
    println!("row encoding");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_decodes_mixed_row() {
        let row = Row {
            values: vec![
                Value::Int(1),
                Value::String("alice".to_string()),
                Value::Null,
            ],
        };

        let encoded = encode_row(&row);

        assert_eq!(decode_row(&encoded).unwrap(), row);
    }

    #[test]
    fn stores_column_count_first() {
        let row = Row {
            values: vec![Value::Int(1), Value::Null],
        };

        let encoded = encode_row(&row);

        assert_eq!(&encoded[0..2], &2u16.to_le_bytes());
    }

    #[test]
    fn rejects_truncated_row() {
        let row = Row {
            values: vec![Value::String("abc".to_string())],
        };
        let mut encoded = encode_row(&row);
        encoded.pop();

        assert!(decode_row(&encoded).is_err());
    }
}
