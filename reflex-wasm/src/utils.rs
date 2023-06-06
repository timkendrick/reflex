// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
pub(crate) fn u32_get_byte(value: u32, index: u8) -> u8 {
    (0xFF & (value >> (index * 8))) as u8
}

pub(crate) fn u64_get_byte(value: u64, index: u8) -> u8 {
    (0xFF & (value >> (index * 8))) as u8
}

pub(crate) fn i64_to_chunks(value: i64) -> [u32; 2] {
    let bytes = value.to_le_bytes();
    let low_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high_word = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    [low_word, high_word]
}

pub(crate) fn chunks_to_i64(value: [u32; 2]) -> i64 {
    let [low_word, high_word] = value;
    let low_bytes = low_word.to_le_bytes();
    let high_bytes = high_word.to_le_bytes();
    i64::from_le_bytes([
        low_bytes[0],
        low_bytes[1],
        low_bytes[2],
        low_bytes[3],
        high_bytes[0],
        high_bytes[1],
        high_bytes[2],
        high_bytes[3],
    ])
}

#[allow(dead_code)]
pub fn into_twos_complement_i32(value: i32) -> u32 {
    if value >= 0 {
        value as u32
    } else {
        0xFFFFFFFFu32 - ((value + 1).abs() as u32)
    }
}

pub fn from_twos_complement_i32(value: u32) -> i32 {
    if value <= 0x7FFFFFFF {
        value as i32
    } else {
        -0x80000000i32 + ((value - 0x80000000) as i32)
    }
}

pub fn into_twos_complement_i64(value: i64) -> u64 {
    if value >= 0 {
        value as u64
    } else {
        0xFFFFFFFFFFFFFFFFu64 - ((value + 1).abs() as u64)
    }
}

pub fn from_twos_complement_i64(value: u64) -> i64 {
    if value <= 0x7FFFFFFFFFFFFFFF {
        value as i64
    } else {
        -0x8000000000000000i64 + ((value - 0x8000000000000000) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twos_complement() {
        assert_eq!(i32::MIN, -0x80000000);
        assert_eq!(i32::MAX, 0x7FFFFFFF);
        assert_eq!(into_twos_complement_i32(0), 0);
        assert_eq!(into_twos_complement_i32(1), 1);
        assert_eq!(into_twos_complement_i32(0x10000000), 0x10000000);
        assert_eq!(into_twos_complement_i32(0x7FFFFFFF), 0x7FFFFFFF);
        assert_eq!(into_twos_complement_i32(-0x80000000), 0x80000000);
        assert_eq!(into_twos_complement_i32(-1), 0xFFFFFFFF);

        assert_eq!(u32::MIN, 0);
        assert_eq!(u32::MAX, 0xFFFFFFFF);
        assert_eq!(from_twos_complement_i32(0), 0);
        assert_eq!(from_twos_complement_i32(1), 1);
        assert_eq!(from_twos_complement_i32(0x10000000), 0x10000000);
        assert_eq!(from_twos_complement_i32(0x7FFFFFFF), 0x7FFFFFFF);
        assert_eq!(from_twos_complement_i32(0x80000000), -0x80000000);
        assert_eq!(from_twos_complement_i32(0xFFFFFFFF), -1);
    }
}
