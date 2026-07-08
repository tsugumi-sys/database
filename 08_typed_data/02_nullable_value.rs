// Step 8-2: Add NULL to the value encoding.
//
// Run:
// rustc --edition=2021 --test 02_nullable_value.rs && ./02_nullable_value

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

fn main() {
    println!("nullable value encoding");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_has_tag_only() {
        assert_eq!(encode_value(&Value::Null), vec![TAG_NULL]);
        assert_eq!(decode_value(&[TAG_NULL, 99]).unwrap(), (Value::Null, 1));
    }

    #[test]
    fn decodes_nullable_values() {
        assert_eq!(
            decode_value(&encode_value(&Value::Int(-10))).unwrap().0,
            Value::Int(-10)
        );
        assert_eq!(
            decode_value(&encode_value(&Value::String("db".to_string())))
                .unwrap()
                .0,
            Value::String("db".to_string())
        );
    }

    #[test]
    fn invalid_tag_is_error() {
        assert!(decode_value(&[99]).is_err());
    }
}
