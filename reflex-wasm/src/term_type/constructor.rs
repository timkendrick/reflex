// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, ConstructorTermType, DependencyList, Eagerness, Expression, GraphNode, Internable,
    RefType, SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, CompilerVariableBindings,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{ListTerm, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct ConstructorTerm {
    pub keys: ArenaPointer,
}
impl TermSize for ConstructorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ConstructorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let keys_hash = arena.read_value::<Term, _>(self.keys, |term| term.id());
        hasher.hash(&keys_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<ConstructorTerm, A> {
    pub fn keys(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.keys),
        )
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.keys().as_inner().len(), 0, false)
    }
}

impl<A: Arena + Clone> ConstructorTermType<WasmExpression<A>> for ArenaRef<ConstructorTerm, A> {
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        self.keys().into()
    }
}

impl<A: Arena + Clone> ConstructorTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<ConstructorTerm>, A>
{
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<ConstructorTerm, A> as ConstructorTermType<WasmExpression<A>>>::prototype(
            &self.as_inner(),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<ConstructorTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<ConstructorTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<ConstructorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.keys() == other.keys()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<ConstructorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<ConstructorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<ConstructorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<constructor:{{{}}}>", self.keys())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<ConstructorTerm, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        self.keys().as_inner().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ConstructorTerm, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let keys = self.keys();
        let mut instructions = CompiledBlock::default();
        // Push the prototype key list onto the stack
        // => [ListTerm]
        instructions.append_block(
            keys.as_deref()
                .as_inner()
                .compile(state, bindings, options, stack)?,
        );
        // Invoke the term constructor
        // => [ConstructorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateConstructor,
        ));
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn constructor() {
        assert_eq!(
            TermType::Constructor(ConstructorTerm {
                keys: ArenaPointer(0x54321)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Constructor as u32, 0x54321],
        );
    }
}
