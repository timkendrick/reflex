// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashSet;

use reflex::core::{DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, CompilerVariableBindings,
    },
    hash::{TermHash, TermHasher, TermSize},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct IndexedAccessorIteratorTerm {
    pub source: ArenaPointer,
    pub index: u32,
}

impl TermSize for IndexedAccessorIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for IndexedAccessorIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let source_hash = arena.read_value::<Term, _>(self.source, |term| term.id());
        hasher.hash(&source_hash, arena).hash(&self.index, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<IndexedAccessorIteratorTerm, A> {
    pub fn source(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.source))
    }
    pub fn index(&self) -> usize {
        self.read_value(|term| term.index) as usize
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<IndexedAccessorIteratorTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<IndexedAccessorIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source() && self.index() == other.index()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<IndexedAccessorIteratorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<IndexedAccessorIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<IndexedAccessorIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexedAccessorIterator")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<IndexedAccessorIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.source().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.source().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.source().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.source().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.source().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.source().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.source().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<IndexedAccessorIteratorTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<IndexedAccessorIteratorTerm, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let source = self.source();
        let index = self.index();
        let mut instructions = CompiledBlock::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(state, bindings, options, stack)?);
        // Push the index argument onto the stack
        // => [Term, index]
        instructions.push(CompiledInstruction::u32_const(index as u32));
        // Invoke the term constructor
        // => [IndexedAccessorIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateIndexedAccessorIterator,
        ));
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn hashmap_keys_iterator() {
        assert_eq!(
            TermType::IndexedAccessorIterator(IndexedAccessorIteratorTerm {
                source: ArenaPointer(0x54321),
                index: 3,
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::IndexedAccessorIterator as u32,
                0x54321,
                3
            ],
        );
    }
}
