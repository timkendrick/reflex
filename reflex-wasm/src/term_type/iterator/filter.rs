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
pub struct FilterIteratorTerm {
    pub source: TermPointer,
    pub predicate: TermPointer,
}
impl TermSize for FilterIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FilterIteratorTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.source, allocator)
            .hash(&self.predicate, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn filter_iterator() {
        assert_eq!(
            TermType::FilterIterator(FilterIteratorTerm {
                source: TermPointer(12345),
                predicate: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::FilterIterator as u32, 12345, 67890],
        );
    }
}
