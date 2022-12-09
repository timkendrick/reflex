// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{StackOffset, VariableTermType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VariableTerm {
    pub stack_offset: u32,
}
impl TermSize for VariableTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for VariableTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.stack_offset, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, VariableTerm, A> {
    fn stack_offset(&self) -> u32 {
        self.as_deref().stack_offset
    }
}

impl<'heap, A: ArenaAllocator> VariableTermType for ArenaRef<'heap, VariableTerm, A> {
    fn offset(&self) -> StackOffset {
        self.stack_offset() as StackOffset
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn variable() {
        assert_eq!(
            TermType::Variable(VariableTerm {
                stack_offset: 12345,
            })
            .as_bytes(),
            [TermTypeDiscriminants::Variable as u32, 12345],
        );
    }
}
