// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ArgType, DependencyList, Expression, GraphNode, LazyResultTermType, SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{tree::TreeTerm, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct LazyResultTerm {
    pub value: ArenaPointer,
    pub dependencies: ArenaPointer,
}
impl TermSize for LazyResultTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LazyResultTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let value_hash = arena.read_value::<Term, _>(self.value, |term| term.id());
        let dependencies_hash = arena.read_value::<Term, _>(self.dependencies, |term| term.id());
        hasher
            .hash(&value_hash, arena)
            .hash(&dependencies_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<LazyResultTerm, A> {
    pub fn value(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.value))
    }
    pub fn dependencies(&self) -> ArenaRef<TypedTerm<TreeTerm>, A> {
        ArenaRef::<TypedTerm<TreeTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.dependencies),
        )
    }
}

impl<A: Arena + Clone> LazyResultTermType<WasmExpression<A>> for ArenaRef<LazyResultTerm, A> {
    fn value<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.value().into()
    }
    fn dependencies<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalListRef<'a>
    where
        <WasmExpression<A> as Expression>::SignalList: 'a,
        WasmExpression<A>: 'a,
    {
        self.dependencies().into()
    }
}

impl<A: Arena + Clone> LazyResultTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<LazyResultTerm>, A>
{
    fn value<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LazyResultTerm, A> as LazyResultTermType<WasmExpression<A>>>::value(
            &self.as_inner(),
        )
    }
    fn dependencies<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalListRef<'a>
    where
        <WasmExpression<A> as Expression>::SignalList: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LazyResultTerm, A> as LazyResultTermType<WasmExpression<A>>>::dependencies(
            &self.as_inner(),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<LazyResultTerm, A> {
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
        self.value().dynamic_dependencies(deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.value().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<LazyResultTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<LazyResultTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value() && self.dependencies() == other.dependencies()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<LazyResultTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<LazyResultTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<LazyResultTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lazy:{}:{}>", self.value(), self.dependencies())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<LazyResultTerm, A> {
    fn should_intern(&self, eager: ArgType) -> bool {
        self.value().should_intern(eager) && self.dependencies().as_inner().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LazyResultTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let dependencies = self.dependencies();
        let block = CompiledBlockBuilder::new(stack);
        // Yield the value onto the stack
        // => [Term]
        let block = block.append_inner(|stack| value.compile(stack, state, options))?;
        // Yield the dependencies onto the stack
        // => [Term, TreeTerm]
        let block =
            block.append_inner(|stack| dependencies.as_term().compile(stack, state, options))?;
        // Invoke the term constructor
        // => [LazyResultTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateLazyResult,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn effect() {
        assert_eq!(
            TermType::LazyResult(LazyResultTerm {
                value: ArenaPointer(0x54321),
                dependencies: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::LazyResult as u32, 0x54321, 0x98765],
        );
    }
}
