// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::core::{
    DependencyList, GraphNode, HashmapTermType, HashsetTermType, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};

use super::{HashmapTerm, WasmExpression};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashsetTerm {
    pub entries: TermPointer,
}
impl TermSize for HashsetTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashsetTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.entries, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, HashsetTerm, A> {
    pub fn num_values(&self) -> u32 {
        self.entries().as_inner().num_entries()
    }
    pub fn values<'a>(&'a self) -> <ArenaRef<'heap, TypedTerm<HashmapTerm>, A> as HashmapTermType<WasmExpression<'heap, A>>>::KeysIterator<'a>{
        self.entries().keys()
    }
    fn entries(&self) -> ArenaRef<'heap, TypedTerm<HashmapTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_value().entries))
    }
}

impl<'heap, A: ArenaAllocator> HashsetTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, HashsetTerm, A>
{
    type ValuesIterator<'a> = <ArenaRef<'heap, HashmapTerm, A> as HashmapTermType<WasmExpression<'heap, A>>>::KeysIterator<'a>
    where
        WasmExpression<'heap, A>: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &WasmExpression<'heap, A>) -> bool {
        self.values().any({
            let value_id = value.as_value().id();
            move |value| {
                if value.as_value().id() == value_id {
                    true
                } else {
                    false
                }
            }
        })
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        self.values()
    }
}

impl<'heap, A: ArenaAllocator> HashsetTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<HashsetTerm>, A>
{
    type ValuesIterator<'a> = <ArenaRef<'heap, HashsetTerm, A> as HashsetTermType<WasmExpression<'heap, A>>>::ValuesIterator<'a>
    where
        WasmExpression<'heap, A>: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &WasmExpression<'heap, A>) -> bool {
        <ArenaRef<'heap, HashsetTerm, A> as HashsetTermType<WasmExpression<'heap, A>>>::contains(
            &self.as_inner(),
            value,
        )
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        <ArenaRef<'heap, HashsetTerm, A> as HashsetTermType<WasmExpression<'heap, A>>>::values(
            &self.as_inner(),
        )
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, HashsetTerm, A> {
    fn size(&self) -> usize {
        1 + self.entries().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.entries().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.entries().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.entries().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.entries().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.entries().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.entries().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, HashsetTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, HashsetTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.entries() == other.entries()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, HashsetTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, HashsetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, HashsetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_displayed_values = 10;
        let values = self.values();
        let num_values = values.len();
        write!(
            f,
            "HashSet({})",
            if num_values <= max_displayed_values {
                values
                    .map(|value| format!("{}", value))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                values
                    .take(max_displayed_values - 1)
                    .map(|value| format!("{}", value))
                    .chain(once(format!(
                        "...{} more values",
                        num_values - (max_displayed_values - 1)
                    )))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn hashset() {
        assert_eq!(
            TermType::Hashset(HashsetTerm {
                entries: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Hashset as u32, 12345],
        );
    }
}
