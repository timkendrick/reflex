// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::core::{
    DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset, VariableTermType,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        error::CompilerError, instruction, CompileWasm, CompiledBlockBuilder, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
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
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.stack_offset, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<VariableTerm, A> {
    pub fn stack_offset(&self) -> StackOffset {
        self.read_value(|term| term.stack_offset as StackOffset)
    }
}

impl<A: Arena + Clone> VariableTermType for ArenaRef<VariableTerm, A> {
    fn offset(&self) -> StackOffset {
        self.stack_offset()
    }
}

impl<A: Arena + Clone> VariableTermType for ArenaRef<TypedTerm<VariableTerm>, A> {
    fn offset(&self) -> StackOffset {
        <ArenaRef<VariableTerm, A> as VariableTermType>::offset(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<VariableTerm, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        (self.stack_offset()) + 1
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::from_iter(once(self.stack_offset()))
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        if offset == (self.stack_offset()) {
            1
        } else {
            0
        }
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<VariableTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<VariableTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.stack_offset() == other.stack_offset()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<VariableTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<VariableTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<VariableTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<variable:{}>", self.stack_offset())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<VariableTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        false
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<VariableTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let stack_offset = self.stack_offset();
        if let Some(scope_offset) = stack.lookup_variable(stack_offset) {
            let block = CompiledBlockBuilder::new(stack);
            // Copy the lexically-scoped variable onto the stack
            // => [Term]
            let block = block.push(instruction::core::GetScopeValue {
                value_type: ValueType::HeapPointer,
                scope_offset,
            });
            let block = if options.lazy_variable_initializers {
                // If the variable was initialized lazily, evaluate the result now that we're using it
                // => [Term]
                block.push(instruction::runtime::Evaluate)
            } else {
                // Otherwise the variable will already have been evaluated at initialization time, so no need to evaluate again
                block
            };
            block.finish()
        } else {
            Err(CompilerError::UnboundVariable(stack_offset))
        }
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
                stack_offset: 0x54321,
            })
            .as_bytes(),
            [TermTypeDiscriminants::Variable as u32, 0x54321],
        );
    }
}
