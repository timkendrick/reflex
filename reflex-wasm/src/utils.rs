// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
pub(crate) fn u32_get_byte(value: u32, index: u8) -> u8 {
    (0xFF & (value >> (index * 8))) as u8
}

pub(crate) fn u64_get_byte(value: u64, index: u8) -> u8 {
    (0xFF & (value >> (index * 8))) as u8
}

#[allow(dead_code)]
pub(crate) fn into_twos_complement(value: i32) -> u32 {
    if value >= 0 {
        value as u32
    } else {
        0xFFFFFFFFu32 - ((value + 1).abs() as u32)
    }
}

#[allow(dead_code)]
pub(crate) fn from_twos_complement(value: u32) -> i32 {
    if value <= 0x7FFFFFFF {
        value as i32
    } else {
        -0x80000000i32 + ((value - 0x80000000) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twos_complement() {
        assert_eq!(i32::MIN, -0x80000000);
        assert_eq!(i32::MAX, 0x7FFFFFFF);
        assert_eq!(into_twos_complement(0), 0);
        assert_eq!(into_twos_complement(1), 1);
        assert_eq!(into_twos_complement(0x10000000), 0x10000000);
        assert_eq!(into_twos_complement(0x7FFFFFFF), 0x7FFFFFFF);
        assert_eq!(into_twos_complement(-0x80000000), 0x80000000);
        assert_eq!(into_twos_complement(-1), 0xFFFFFFFF);

        assert_eq!(u32::MIN, 0);
        assert_eq!(u32::MAX, 0xFFFFFFFF);
        assert_eq!(from_twos_complement(0), 0);
        assert_eq!(from_twos_complement(1), 1);
        assert_eq!(from_twos_complement(0x10000000), 0x10000000);
        assert_eq!(from_twos_complement(0x7FFFFFFF), 0x7FFFFFFF);
        assert_eq!(from_twos_complement(0x80000000), -0x80000000);
        assert_eq!(from_twos_complement(0xFFFFFFFF), -1);
    }
}
