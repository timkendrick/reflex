// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::Hasher,
};

use reflex::{
    core::{
        Arity, CompiledFunctionTermType, DependencyList, Eagerness, GraphNode, InstructionPointer,
        Internable, SerializeJson, StackOffset,
    },
    hash::HashId,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};
use reflex_macros::PointerIter;

#[derive(PartialEq, Eq, Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct CompiledFunctionIndex(u32);
impl TermSize for CompiledFunctionIndex {
    fn size_of(&self) -> usize {
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
impl From<CompiledFunctionIndex> for u32 {
    fn from(value: CompiledFunctionIndex) -> Self {
        let CompiledFunctionIndex(value) = value;
        value
    }
}
impl std::fmt::Display for CompiledFunctionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct CompiledTerm {
    pub target: CompiledFunctionIndex,
    pub num_args: u32,
}
impl TermSize for CompiledTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CompiledTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.num_args, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<CompiledTerm, A> {
    pub fn target(&self) -> CompiledFunctionIndex {
        self.read_value(|term| term.target)
    }
    pub fn num_args(&self) -> u32 {
        self.read_value(|term| term.num_args)
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.num_args() as usize, 0, false)
    }
}

impl<A: ArenaAllocator + Clone> CompiledFunctionTermType for ArenaRef<CompiledTerm, A> {
    fn address(&self) -> InstructionPointer {
        InstructionPointer::new(u32::from(self.target()) as usize)
    }
    fn hash(&self) -> HashId {
        let mut hasher = DefaultHasher::default();
        hasher.write_u32(u32::from(self.target()));
        hasher.finish()
    }
    fn required_args(&self) -> StackOffset {
        self.num_args() as usize
    }
    fn optional_args(&self) -> StackOffset {
        0
    }
}

impl<A: ArenaAllocator + Clone> CompiledFunctionTermType for ArenaRef<TypedTerm<CompiledTerm>, A> {
    fn address(&self) -> InstructionPointer {
        <ArenaRef<CompiledTerm, A> as CompiledFunctionTermType>::address(&self.as_inner())
    }
    fn hash(&self) -> HashId {
        <ArenaRef<CompiledTerm, A> as CompiledFunctionTermType>::hash(&self.as_inner())
    }
    fn required_args(&self) -> StackOffset {
        <ArenaRef<CompiledTerm, A> as CompiledFunctionTermType>::required_args(&self.as_inner())
    }
    fn optional_args(&self) -> StackOffset {
        <ArenaRef<CompiledTerm, A> as CompiledFunctionTermType>::optional_args(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<CompiledTerm, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::new()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<CompiledTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<CompiledTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address() && self.num_args() == other.num_args()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<CompiledTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<CompiledTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<CompiledTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<compiled:{}>", self.target())
    }
}

impl<A: ArenaAllocator + Clone> Internable for ArenaRef<CompiledTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
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
