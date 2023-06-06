// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RangeIteratorTerm {
    pub offset: i32,
    pub length: u32,
}
impl TermSize for RangeIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RangeIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.offset, arena).write_u32(self.length)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    fn twos_complement(value: i32) -> u32 {
        if value >= 0 {
            value as u32
        } else {
            0xFFFFFFFF - ((value.abs() - 1) as u32)
        }
    }

    #[test]
    fn range_iterator() {
        assert_eq!(
            TermType::RangeIterator(RangeIteratorTerm {
                offset: 12345,
                length: 67890,
            })
            .as_bytes(),
            [TermTypeDiscriminants::RangeIterator as u32, 12345, 67890],
        );
        assert_eq!(
            TermType::RangeIterator(RangeIteratorTerm {
                offset: -12345,
                length: 67890,
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::RangeIterator as u32,
                twos_complement(-12345),
                67890
            ],
        );
    }
}
