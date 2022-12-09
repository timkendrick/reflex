// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use crate::{
    allocator::ArenaAllocator,
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
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FilterIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(&self.source, arena)
            .hash(&self.predicate, arena)
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
