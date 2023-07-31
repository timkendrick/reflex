// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{ArgType, DependencyList, GraphNode, SerializeJson, StackOffset};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, CompileWasm, CompiledBlockBuilder, CompilerOptions, CompilerResult,
        CompilerStack, CompilerState, ConstValue, Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct PointerTerm {
    pub target: ArenaPointer,
}
impl TermSize for PointerTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PointerTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        arena.read_value::<Term, _>(self.target, |term| term.hash(hasher, arena))
    }
}

impl<A: Arena + Clone> ArenaRef<PointerTerm, A> {
    pub fn target(&self) -> ArenaPointer {
        self.read_value(|term| term.target)
    }
    pub fn as_target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.target())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<PointerTerm, A> {
    fn size(&self) -> usize {
        self.as_target().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.as_target().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.as_target().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.as_target().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        self.as_target().dynamic_dependencies(deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.as_target().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        self.as_target().is_static()
    }
    fn is_atomic(&self) -> bool {
        self.as_target().is_atomic()
    }
    fn is_complex(&self) -> bool {
        self.as_target().is_complex()
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<PointerTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<PointerTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.as_target() == other.as_target()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<PointerTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<PointerTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<PointerTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pointer({:#x})",
            self.read_value(|term| u32::from(term.target))
        )
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<PointerTerm, A> {
    fn should_intern(&self, _eager: ArgType) -> bool {
        false
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<PointerTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let block = CompiledBlockBuilder::new(stack);
        // Push the target pointer onto the stack
        // => [Term]
        let block = block.push(instruction::core::Const {
            value: ConstValue::HeapPointer(target),
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn pointer() {
        assert_eq!(
            TermType::Pointer(PointerTerm {
                target: ArenaPointer(0x54321),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Pointer as u32, 0x54321],
        );
    }
}
