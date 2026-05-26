// Step 3-4: Endian-aware integer encoding.
//
// Run:
// rustc --edition=2021 --test 04_endian_int.rs && ./04_endian_int

#![allow(unused)]

fn write_u16_le(buf: &mut [u8], offset: usize, value: u16) -> Result<(), String> {
    let bytes = value.to_le_bytes();
    let end = offset.checked_add(bytes.len()).ok_or("overflow")?;
    if end > buf.len() {
        return Err("out of bounds".to_string());
    }
    buf[offset..end].copy_from_slice(&bytes);
    Ok(())
}

fn read_u16_le(buf: &[u8], offset: usize) -> Result<u16, String> {
    let end = offset.checked_add(2).ok_or("overflow")?;
    if end > buf.len() {
        return Err("out of bounds".to_string());
    }
    let bytes: [u8; 2] = buf[offset..end].try_into().unwrap();
    Ok(u16::from_le_bytes(bytes))
}

fn write_u32_le(buf: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    let bytes = value.to_le_bytes();
    let end = offset.checked_add(bytes.len()).ok_or("overflow")?;
    if end > buf.len() {
        return Err("out of bounds".to_string());
    }
    buf[offset..end].copy_from_slice(&bytes);
    Ok(())
}

fn read_u32_le(buf: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset.checked_add(4).ok_or("overflow")?;
    if end > buf.len() {
        return Err("out of bounds".to_string());
    }
    let bytes: [u8; 4] = buf[offset..end].try_into().unwrap();
    Ok(u32::from_le_bytes(bytes))
}

fn write_i32_le(buf: &mut [u8], offset: usize, value: i32) -> Result<(), String> {
    let bytes = value.to_le_bytes();
    let end = offset.checked_add(bytes.len()).ok_or("overflow")?;
    if end > buf.len() {
        return Err("out of bounds".to_string());
    }
    buf[offset..end].copy_from_slice(&bytes);
    Ok(())
}

fn read_i32_le(buf: &[u8], offset: usize) -> Result<i32, String> {
    let end = offset.checked_add(4).ok_or("overflow")?;
    if end > buf.len() {
        return Err("out of bounds".to_string());
    }
    let bytes: [u8; 4] = buf[offset..end].try_into().unwrap();
    Ok(i32::from_le_bytes(bytes))
}

fn main() {
    let mut buf = [0; 8];
    println!("{:?}", buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_u16_in_little_endian_order() {
        let mut buf = [0; 4];

        write_u16_le(&mut buf, 1, 0x1234).unwrap();

        assert_eq!(buf, [0x00, 0x34, 0x12, 0x00]);
    }

    #[test]
    fn reads_u16_from_little_endian_order() {
        let buf = [0x00, 0x34, 0x12, 0x00];

        assert_eq!(read_u16_le(&buf, 1).unwrap(), 0x1234);
    }

    #[test]
    fn writes_and_reads_u32() {
        let mut buf = [0; 8];

        write_u32_le(&mut buf, 2, 0x0102_0304).unwrap();

        assert_eq!(&buf[2..6], &[0x04, 0x03, 0x02, 0x01]);
        assert_eq!(read_u32_le(&buf, 2).unwrap(), 0x0102_0304);
    }

    #[test]
    fn writes_and_reads_i32() {
        let mut buf = [0; 4];

        write_i32_le(&mut buf, 0, -12345).unwrap();

        assert_eq!(read_i32_le(&buf, 0).unwrap(), -12345);
    }

    #[test]
    fn rejects_write_past_end() {
        let mut buf = [0; 3];

        assert!(write_u32_le(&mut buf, 0, 1).is_err());
        assert!(write_u16_le(&mut buf, 2, 1).is_err());
    }

    #[test]
    fn rejects_read_past_end() {
        let buf = [0; 3];

        assert!(read_u32_le(&buf, 0).is_err());
        assert!(read_u16_le(&buf, 2).is_err());
    }
}
