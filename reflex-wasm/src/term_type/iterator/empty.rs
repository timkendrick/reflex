// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EmptyIteratorTerm;
impl TermSize for EmptyIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for EmptyIteratorTerm {
    fn hash(&self, hasher: TermHasher, _allocator: &impl TermAllocator) -> TermHasher {
        hasher
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn empty_iterator() {
        assert_eq!(
            TermType::EmptyIterator(EmptyIteratorTerm).as_bytes(),
            [TermTypeDiscriminants::EmptyIterator as u32],
        );
    }
}
