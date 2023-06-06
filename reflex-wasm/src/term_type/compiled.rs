// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::{
    core::{CompiledFunctionTermType, InstructionPointer, RefType, StackOffset},
    hash::HashId,
};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub struct CompiledFunctionIndex(u32);
impl TermSize for CompiledFunctionIndex {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CompiledFunctionIndex {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let Self(uid) = self;
        hasher.hash(uid, arena)
    }
}
impl From<u32> for CompiledFunctionIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CompiledTerm {
    pub target: CompiledFunctionIndex,
    pub num_args: u32,
}
impl TermSize for CompiledTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CompiledTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.num_args, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, CompiledTerm, A> {
    fn target(&self) -> CompiledFunctionIndex {
        self.as_deref().target
    }
    fn required_args(&self) -> u32 {
        self.as_deref().num_args
    }
}

impl<'heap, A: ArenaAllocator> CompiledFunctionTermType for ArenaRef<'heap, CompiledTerm, A> {
    fn address(&self) -> InstructionPointer {
        InstructionPointer::new(u32::from(self.target()) as usize)
    }
    fn hash(&self) -> HashId {}
    fn required_args(&self) -> StackOffset {
        self.required_args() as usize
    }
    fn optional_args(&self) -> StackOffset {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn compiled() {
        assert_eq!(
            TermType::Compiled(CompiledTerm {
                target: CompiledFunctionIndex::from(12345),
                num_args: 3,
            })
            .as_bytes(),
            [TermTypeDiscriminants::Compiled as u32, 12345, 3],
        );
    }
}
