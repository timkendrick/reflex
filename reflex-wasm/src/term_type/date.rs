// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DateTerm {
    pub timestamp: [u32; 2],
}
impl TermSize for DateTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for DateTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.timestamp, allocator)
    }
}
impl From<i64> for DateTerm {
    fn from(value: i64) -> Self {
        Self {
            timestamp: i64_to_chunks(value),
        }
    }
}
impl Into<i64> for DateTerm {
    fn into(self) -> i64 {
        let Self { timestamp, .. } = self;
        chunks_to_i64(timestamp)
    }
}

fn i64_to_chunks(value: i64) -> [u32; 2] {
    let bytes = value.to_le_bytes();
    let low_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high_word = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    [low_word, high_word]
}

fn chunks_to_i64(value: [u32; 2]) -> i64 {
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

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn date() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert_eq!(
            TermType::Date(DateTerm::from(timestamp)).as_bytes(),
            [
                TermTypeDiscriminants::Date as u32,
                i64_to_chunks(timestamp)[0],
                i64_to_chunks(timestamp)[1]
            ],
        );
    }
}
