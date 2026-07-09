// Step 8-4: Encode keys and define comparison behavior.
//
// Run:
// rustc --edition=2021 --test 04_key_encoding.rs && ./04_key_encoding

#![allow(unused)]

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Key {
    Int(i64),
    String(String),
}

fn encode_key(key: &Key) -> Vec<u8> {
    match key {
        Key::Int(value) => {
            let sortable_integer = (*value as u64) ^ 0x8000_0000_0000_0000;
            sortable_integer.to_be_bytes().to_vec()
        }
        Key::String(string) => string.as_bytes().to_vec(),
    }
}

fn compare_keys(left: &Key, right: &Key) -> Result<Ordering, String> {
    match (left, right) {
        (Key::Int(a), Key::Int(b)) => Ok(a.cmp(b)),
        (Key::String(a), Key::String(b)) => Ok(a.cmp(b)),
        (_, _) => Err("mixed key types are not comparable".to_string()),
    }
}

fn main() {
    println!("key encoding");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_key_comparison_matches_numeric_order() {
        assert_eq!(
            compare_keys(&Key::Int(-1), &Key::Int(2)).unwrap(),
            Ordering::Less
        );
        assert_eq!(
            compare_keys(&Key::Int(5), &Key::Int(5)).unwrap(),
            Ordering::Equal
        );
    }

    #[test]
    fn string_key_comparison_matches_lexicographic_order() {
        assert_eq!(
            compare_keys(
                &Key::String("alice".to_string()),
                &Key::String("bob".to_string())
            )
            .unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn mixed_key_types_are_not_comparable() {
        assert!(compare_keys(&Key::Int(1), &Key::String("1".to_string())).is_err());
    }

    #[test]
    fn int_key_comparison_reports_greater() {
        assert_eq!(
            compare_keys(&Key::Int(10), &Key::Int(2)).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn string_key_comparison_handles_prefixes() {
        assert_eq!(
            compare_keys(
                &Key::String("a".to_string()),
                &Key::String("aa".to_string())
            )
            .unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn encoded_int_is_eight_bytes() {
        assert_eq!(encode_key(&Key::Int(1)).len(), 8);
    }

    #[test]
    fn encoded_int_preserves_order_for_positive_values() {
        assert!(encode_key(&Key::Int(1)) < encode_key(&Key::Int(2)));
    }

    #[test]
    fn encoded_int_preserves_order_for_negative_values() {
        assert!(encode_key(&Key::Int(-2)) < encode_key(&Key::Int(-1)));
        assert!(encode_key(&Key::Int(-1)) < encode_key(&Key::Int(0)));
    }

    #[test]
    fn encoded_string_matches_utf8_bytes() {
        assert_eq!(
            encode_key(&Key::String("alice".to_string())),
            b"alice".to_vec()
        );
    }
}
