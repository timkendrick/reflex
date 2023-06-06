// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct NilTerm;
impl TermSize for NilTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for NilTerm {
    fn hash(&self, hasher: TermHasher, _allocator: &impl TermAllocator) -> TermHasher {
        hasher
    }
}

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
