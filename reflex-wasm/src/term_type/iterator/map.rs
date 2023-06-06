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
pub struct MapIteratorTerm {
    pub source: TermPointer,
    pub iteratee: TermPointer,
}
impl TermSize for MapIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for MapIteratorTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.source, allocator)
            .hash(&self.iteratee, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn map_iterator() {
        assert_eq!(
            TermType::MapIterator(MapIteratorTerm {
                source: TermPointer(12345),
                iteratee: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::MapIterator as u32, 12345, 67890],
        );
    }
}
