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
pub struct IntersperseIteratorTerm {
    pub source: TermPointer,
    pub separator: TermPointer,
}
impl TermSize for IntersperseIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for IntersperseIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(&self.source, arena)
            .hash(&self.separator, arena)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn intersperse_iterator() {
        assert_eq!(
            TermType::IntersperseIterator(IntersperseIteratorTerm {
                source: TermPointer(12345),
                separator: TermPointer(67890),
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::IntersperseIterator as u32,
                12345,
                67890
            ],
        );
    }
}
