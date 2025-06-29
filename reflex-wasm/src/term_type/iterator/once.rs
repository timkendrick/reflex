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
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct OnceIteratorTerm {
    pub value: ArenaPointer,
}
impl TermSize for OnceIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for OnceIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let value_hash = arena.read_value::<Term, _>(self.value, |term| term.id());
        hasher.hash(&value_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<OnceIteratorTerm, A> {
    pub fn value(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.value))
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<OnceIteratorTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<OnceIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<OnceIteratorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<OnceIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<OnceIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnceIterator")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<OnceIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.value().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.value().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.value().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.value().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.value().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.value().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.value().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<OnceIteratorTerm, A> {
    fn should_intern(&self, eager: ArgType) -> bool {
        self.value().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<OnceIteratorTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let block = CompiledBlockBuilder::new(stack);
        // Push the value argument onto the stack
        // => [Term]
        let block = block.append_inner(|stack| value.compile(stack, state, options))?;
        // Invoke the term constructor
        // => [OnceIteratorTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateOnceIterator,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn once_iterator() {
        assert_eq!(
            TermType::OnceIterator(OnceIteratorTerm {
                value: ArenaPointer(0x54321),
            })
            .as_bytes(),
            [TermTypeDiscriminants::OnceIterator as u32, 0x54321],
        );
    }
}
