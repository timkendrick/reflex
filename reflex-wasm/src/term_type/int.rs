// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{IntTermType, IntValue};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct IntTerm {
    value: i32,
}
impl TermSize for IntTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for IntTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<i32> for IntTerm {
    fn from(value: i32) -> Self {
        Self { value }
    }
}
impl Into<i32> for IntTerm {
    fn into(self) -> i32 {
        let Self { value } = self;
        value
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, IntTerm, A> {
    fn value(&self) -> i32 {
        self.as_deref().value
    }
}

impl<'heap, A: ArenaAllocator> IntTermType for ArenaRef<'heap, IntTerm, A> {
    fn value(&self) -> IntValue {
        self.value()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn int() {
        assert_eq!(
            TermType::Int(IntTerm::from(12345)).as_bytes(),
            [TermTypeDiscriminants::Int as u32, 12345],
        );
    }
}
