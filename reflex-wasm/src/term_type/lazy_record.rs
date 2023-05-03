// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashSet;

use reflex::core::{
    ArgType, DependencyList, Eagerness, Expression, GraphNode, Internable, LazyRecordTermType,
    SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, MaybeLazyExpression,
        Strictness,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{list::compile_list, RecordTerm, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Array, PointerIter, Term,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LazyRecordTerm {
    pub fields: ArenaPointer,
    pub eagerness: Array<u32>,
}
impl TermSize for LazyRecordTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<u32>>() + self.eagerness.size_of()
    }
}
impl TermHash for LazyRecordTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let fields_hash = arena.read_value::<Term, _>(self.fields, |term| term.id());
        let eagerness_hash = TermHasher::default().hash(&self.eagerness, arena).finish();
        hasher
            .hash(&fields_hash, arena)
            .hash(&eagerness_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<LazyRecordTerm, A> {
    pub fn fields(&self) -> ArenaRef<TypedTerm<RecordTerm>, A> {
        ArenaRef::<TypedTerm<RecordTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.fields),
        )
    }
}

pub type LazyRecordTermPointerIter = std::array::IntoIter<ArenaPointer, 1>;

impl<A: Arena + Clone> PointerIter for ArenaRef<LazyRecordTerm, A> {
    type Iter<'a> = LazyRecordTermPointerIter
    where
        Self: 'a;
    fn iter<'a>(&self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        let fields_pointer = self.inner_pointer(|term| &term.fields);
        [fields_pointer].into_iter()
    }
}

#[derive(Clone)]
pub struct LazyRecordEagernessIterator<A: Arena> {
    inner: ArenaRef<Array<u32>, A>,
    offset: usize,
}

impl<A: Arena> LazyRecordEagernessIterator<A> {
    pub fn new(inner: ArenaRef<Array<u32>, A>) -> Self {
        Self { inner, offset: 0 }
    }
}

impl<A: Arena> Iterator for LazyRecordEagernessIterator<A> {
    type Item = ArgType;
    fn next(&mut self) -> Option<Self::Item> {
        match self
            .inner
            .read_value(|items| items.get(self.offset).copied())
        {
            Some(value) => {
                self.offset += 1;
                let eagerness = match value {
                    0 => ArgType::Lazy,
                    1 => ArgType::Eager,
                    2 | _ => ArgType::Strict,
                };
                Some(eagerness)
            }
            None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_items = self.inner.read_value(|items| items.length as usize);
        let length = num_items.saturating_sub(self.offset);
        (length, Some(length))
    }
}

impl<A: Arena> ExactSizeIterator for LazyRecordEagernessIterator<A> {}

impl<A: Arena + Clone> LazyRecordTermType<WasmExpression<A>> for ArenaRef<LazyRecordTerm, A> {
    type EagernessIterator<'a> = LazyRecordEagernessIterator<A>
    where
        Self: 'a;
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        self.fields().as_inner().keys().into()
    }
    fn values<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
        WasmExpression<A>: 'a,
    {
        self.fields().as_inner().values().into()
    }
    fn eagerness<'a>(&'a self) -> Self::EagernessIterator<'a>
    where
        Self: 'a,
    {
        LazyRecordEagernessIterator::new(self.inner_ref(|term| &term.eagerness))
    }
    fn get<'a>(
        &'a self,
        key: &WasmExpression<A>,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        self.fields().as_inner().get(key).map(|value| value.into())
    }
}

impl<A: Arena + Clone> LazyRecordTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<LazyRecordTerm>, A>
{
    type EagernessIterator<'a> = <ArenaRef<LazyRecordTerm, A> as LazyRecordTermType<
        WasmExpression<A>,
    >>::EagernessIterator<'a>
    where
        Self: 'a;
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LazyRecordTerm, A> as LazyRecordTermType<WasmExpression<A>>>::prototype(
            &self.as_inner(),
        )
    }
    fn values<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LazyRecordTerm, A> as LazyRecordTermType<WasmExpression<A>>>::values(
            &self.as_inner(),
        )
    }
    fn eagerness<'a>(&'a self) -> Self::EagernessIterator<'a>
    where
        Self: 'a,
    {
        <ArenaRef<LazyRecordTerm, A> as LazyRecordTermType<WasmExpression<A>>>::eagerness(
            &self.as_inner(),
        )
    }
    fn get<'a>(
        &'a self,
        key: &WasmExpression<A>,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LazyRecordTerm, A> as LazyRecordTermType<WasmExpression<A>>>::get(
            &self.as_inner(),
            key,
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<LazyRecordTerm, A> {
    fn size(&self) -> usize {
        1 + self.values().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.values().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.values().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.values().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.values().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.values().has_dynamic_dependencies(deep)
        } else {
            false
        }
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<LazyRecordTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        self.fields().as_inner().to_json()
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        self.fields().as_inner().patch(&target.fields().as_inner())
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<LazyRecordTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.fields() == other.fields() && self.fields() == other.fields()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<LazyRecordTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<LazyRecordTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<LazyRecordTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.fields().as_inner(), f)
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<LazyRecordTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        false
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LazyRecordTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let fields = self.fields();
        let fields = fields.as_inner();
        let keys = fields.keys();
        let values = fields.values();
        let values = values.as_inner();
        let eagerness = self.eagerness();
        let block = CompiledBlockBuilder::new(stack);
        // Push the keys list onto the stack
        // => [ListTerm]
        let block = block.append_inner(|stack| keys.as_inner().compile(stack, state, options))?;
        // Push the property values list onto the stack
        // => [ListTerm, ListTerm]
        let block = block.append_inner(|stack| {
            compile_list(
                values.iter().zip(eagerness).map(|(item, eagerness)| {
                    match eagerness {
                        ArgType::Lazy => (
                            MaybeLazyExpression::new(item, Eagerness::Lazy),
                            Strictness::NonStrict,
                        ),
                        ArgType::Eager => (
                            MaybeLazyExpression::new(item, Eagerness::Eager),
                            Strictness::NonStrict,
                        ),
                        ArgType::Strict => {
                            // Skip signal-testing for record field values that are already fully evaluated to a non-signal value
                            let strictness = if item.is_static() && item.as_signal_term().is_none()
                            {
                                Strictness::NonStrict
                            } else {
                                Strictness::Strict
                            };
                            (MaybeLazyExpression::new(item, Eagerness::Eager), strictness)
                        }
                    }
                }),
                stack,
                state,
                options,
            )
        })?;
        // Invoke the term constructor
        // => [RecordTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateRecord,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn record() {
        assert_eq!(
            TermType::LazyRecord(LazyRecordTerm {
                fields: ArenaPointer(0x54321),
                eagerness: Array {
                    capacity: 0,
                    length: 0,
                    items: []
                },
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::LazyRecord as u32,
                0x54321,
                0x00000000,
                0x00000000,
            ],
        );
    }
}
