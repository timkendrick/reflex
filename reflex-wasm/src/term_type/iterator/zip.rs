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
pub struct ZipIteratorTerm {
    pub left: TermPointer,
    pub right: TermPointer,
}
impl TermSize for ZipIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ZipIteratorTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.left, allocator)
            .hash(&self.right, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn zip_iterator() {
        assert_eq!(
            TermType::ZipIterator(ZipIteratorTerm {
                left: TermPointer(12345),
                right: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::ZipIterator as u32, 12345, 67890],
        );
    }
}
