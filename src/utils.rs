#![allow(dead_code)]

/// convert bytes data to u32
#[inline]
pub(crate) fn u32_from_le_bytes_ref(buf: &[u8]) -> u32 {
    let mut value: u32 = 0;
    for i in 0..4 {
        value |= (buf[i] as u32) << (i * 8)
    }
    value
}

/// convert bytes data to i32
#[inline]
pub(crate) fn i32_from_le_bytes_ref(buf: &[u8]) -> i32 {
    u32_from_le_bytes_ref(buf) as i32
}

/// convert bytes data to u32
#[inline]
pub(crate) fn u32_from_be_bytes_ref(buf: &[u8]) -> u32 {
    let mut value: u32 = 0;
    for i in 0..4 {
        value |= (buf[i] as u32) << ((3 - i) * 8)
    }
    value
}

/// convert bytes data to i32
#[inline]
pub(crate) fn i32_from_be_bytes_ref(buf: &[u8]) -> i32 {
    u32_from_be_bytes_ref(buf) as i32
}

/// convert bytes data to u16
#[inline]
pub(crate) fn u16_from_le_bytes_ref(buf: &[u8]) -> u16 {
    (buf[0] as u16) | ((buf[1] as u16) << 8)
}

/// convert bytes data to i16
#[inline]
pub(crate) fn i16_from_le_bytes_ref(buf: &[u8]) -> i16 {
    ((buf[0] as u16) | ((buf[1] as u16) << 8)) as i16
}

/// convert bytes data to u16
#[inline]
pub(crate) fn u16_from_be_bytes_ref(buf: &[u8]) -> u16 {
    (buf[1] as u16) | ((buf[0] as u16) << 8)
}

/// convert bytes data to i16
#[inline]
pub(crate) fn i16_from_be_bytes_ref(buf: &[u8]) -> i16 {
    ((buf[1] as u16) | ((buf[0] as u16) << 8)) as i16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_converters_le() {
        let bytes: [u8; 4] = [0x0C, 0x4E, 0x02, 0x00];
        let value1 = u32_from_le_bytes_ref(&bytes);
        let value2 = u32::from_le_bytes(bytes);
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_converters_be() {
        let bytes: [u8; 4] = [0x00, 0x12, 0x75, 0x27];
        let value1 = u32_from_be_bytes_ref(&bytes);
        let value2 = u32::from_be_bytes(bytes);
        assert_eq!(value1, value2);
    }
}
