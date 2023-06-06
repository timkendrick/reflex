// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{BooleanTermType, RefType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BooleanTerm {
    value: u32,
}
impl TermSize for BooleanTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BooleanTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<bool> for BooleanTerm {
    fn from(value: bool) -> Self {
        Self {
            value: value as u32,
        }
    }
}
impl Into<bool> for BooleanTerm {
    fn into(self) -> bool {
        let Self { value, .. } = self;
        value != 0
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, BooleanTerm, A> {
    fn value(&self) -> bool {
        self.as_deref().value as bool
    }
}

impl<'heap, A: ArenaAllocator> BooleanTermType for ArenaRef<'heap, BooleanTerm, A> {
    fn value(&self) -> bool {
        self.value()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn boolean() {
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(false)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 0],
        );
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(true)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 1],
        );
    }
}
