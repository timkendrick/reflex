// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SkipIteratorTerm {
    pub source: TermPointer,
    pub count: u32,
}
impl TermSize for SkipIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for SkipIteratorTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.source, allocator).write_u32(self.count)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn skip_iterator() {
        assert_eq!(
            TermType::SkipIterator(SkipIteratorTerm {
                source: TermPointer(12345),
                count: 67890,
            })
            .as_bytes(),
            [TermTypeDiscriminants::SkipIterator as u32, 12345, 67890],
        );
    }
}
