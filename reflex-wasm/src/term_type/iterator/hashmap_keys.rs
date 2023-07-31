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
pub struct HashmapKeysIteratorTerm {
    pub source: ArenaPointer,
}

impl TermSize for HashmapKeysIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashmapKeysIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let source_hash = arena.read_value::<Term, _>(self.source, |term| term.id());
        hasher.hash(&source_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<HashmapKeysIteratorTerm, A> {
    pub fn source(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.source))
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<HashmapKeysIteratorTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<HashmapKeysIteratorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashmapKeysIterator")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<HashmapKeysIteratorTerm, A> {
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

impl<A: Arena + Clone> Internable for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn should_intern(&self, _eager: ArgType) -> bool {
        self.capture_depth() == 0
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let block = CompiledBlockBuilder::new(stack);
        // Push the source argument onto the stack
        // => [Term]
        let block = block.append_inner(|stack| source.compile(stack, state, options))?;
        // Invoke the term constructor
        // => [HashmapKeysIteratorTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateHashmapKeysIterator,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn hashmap_keys_iterator() {
        assert_eq!(
            TermType::HashmapKeysIterator(HashmapKeysIteratorTerm {
                source: ArenaPointer(0x54321),
            })
            .as_bytes(),
            [TermTypeDiscriminants::HashmapKeysIterator as u32, 0x54321],
        );
    }
}
