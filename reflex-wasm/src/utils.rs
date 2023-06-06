// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
pub(crate) fn u32_get_byte(value: u32, index: u8) -> u8 {
    (0xFF & (value >> (index * 8))) as u8
}

pub(crate) fn u64_get_byte(value: u64, index: u8) -> u8 {
    (0xFF & (value >> (index * 8))) as u8
}
