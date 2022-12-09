// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::NilTermType;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct NilTerm;
impl TermSize for NilTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for NilTerm {
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}

impl<'heap, A: ArenaAllocator> NilTermType for ArenaPointer<'heap, NilTerm, A> {}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn nil() {
        assert_eq!(
            TermType::Nil(NilTerm).as_bytes(),
            [TermTypeDiscriminants::Nil as u32],
        );
    }
}
