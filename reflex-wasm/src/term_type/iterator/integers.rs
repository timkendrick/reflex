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
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct IntegersIteratorTerm;
impl TermSize for IntegersIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for IntegersIteratorTerm {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<IntegersIteratorTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<IntegersIteratorTerm, A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<IntegersIteratorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<IntegersIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<IntegersIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntegersIterator")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<IntegersIteratorTerm, A> {
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

impl<A: Arena + Clone> Internable for ArenaRef<IntegersIteratorTerm, A> {
    fn should_intern(&self, _eager: ArgType) -> bool {
        true
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<IntegersIteratorTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let block = CompiledBlockBuilder::new(stack);
        // Invoke the term constructor
        // => [IntegersIteratorTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateIntegersIterator,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn integers_iterator() {
        assert_eq!(
            TermType::IntegersIterator(IntegersIteratorTerm).as_bytes(),
            [TermTypeDiscriminants::IntegersIterator as u32],
        );
    }
}
