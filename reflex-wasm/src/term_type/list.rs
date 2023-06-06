// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    collections::HashSet,
    iter::{once, Copied},
};

use reflex::{
    core::{
        DependencyList, Expression, ExpressionListType, GraphNode, ListTermType, SerializeJson,
        StackOffset, StructPrototypeType,
    },
    hash::HashId,
};
use reflex_utils::{json::is_empty_json_object, MapIntoIterator};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermType,
    term_type::TypedTerm,
    ArenaRef, Array, ArrayIter, IntoArenaRefIterator, Term, TermPointer,
};

use super::WasmExpression;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ListTerm {
    pub items: Array<TermPointer>,
}
impl TermSize for ListTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<TermPointer>>()
            + self.items.size_of()
    }
}
impl TermHash for ListTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.items, arena)
    }
}
impl ListTerm {
    pub fn allocate(
        values: impl IntoIterator<
            Item = TermPointer,
            IntoIter = impl ExactSizeIterator<Item = TermPointer>,
        >,
        arena: &mut impl ArenaAllocator,
    ) -> TermPointer {
        let values = values.into_iter();
        let term = Term::new(
            TermType::List(ListTerm {
                items: Default::default(),
            }),
            arena,
        );
        let term_size = term.size_of();
        let instance = arena.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<TermPointer>>()) as u32);
        Array::<TermPointer>::extend(list, values, arena);
        let hash = {
            arena
                .get::<Term>(instance)
                .hash(Default::default(), arena)
                .finish()
        };
        arena.get_mut::<Term>(instance).set_hash(hash);
        instance
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, ListTerm, A> {
    pub fn items(&self) -> ArenaRef<'heap, Array<TermPointer>, A> {
        ArenaRef::new(self.arena, &self.as_value().items)
    }
    pub fn iter<'a>(
        &'a self,
    ) -> IntoArenaRefIterator<'heap, Term, A, Copied<ArrayIter<'heap, TermPointer>>> {
        IntoArenaRefIterator::new(self.arena, self.items().iter().copied())
    }
    pub fn len(&self) -> usize {
        self.items().len()
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, ListTerm, A> {
    fn size(&self) -> usize {
        1 + self.iter().map(|term| term.size()).sum::<usize>()
    }
    fn capture_depth(&self) -> StackOffset {
        self.iter()
            .map(|term| term.capture_depth())
            .max()
            .unwrap_or(0)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.iter().flat_map(|term| term.free_variables()).collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.iter()
            .map(|term| term.count_variable_usages(offset))
            .sum()
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.iter()
                .flat_map(|term| term.dynamic_dependencies(deep))
                .collect()
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.iter().any(|term| term.has_dynamic_dependencies(deep))
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.iter().all(|term| term.is_atomic())
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<'heap, A: ArenaAllocator> ListTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<ListTerm>, A>
{
    fn items<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<'heap, A> as Expression>::ExpressionList: 'a,
        WasmExpression<'heap, A>: 'a,
    {
        (*self).into()
    }
}

impl<'heap, A: ArenaAllocator> StructPrototypeType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<ListTerm>, A>
{
    fn keys<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<'heap, A> as Expression>::ExpressionList: 'a,
        WasmExpression<'heap, A>: 'a,
    {
        (*self).into()
    }
}

impl<'heap, A: ArenaAllocator> ExpressionListType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<ListTerm>, A>
{
    type Iterator<'a> = MapIntoIterator<
        IntoArenaRefIterator<'heap, Term, A, Copied<ArrayIter<'heap, TermPointer>>>,
        ArenaRef<'heap, Term, A>,
        <WasmExpression<'heap, A> as Expression>::ExpressionRef<'a>,
    >
    where
        WasmExpression<'heap, A>: 'a,
        Self: 'a;
    fn id(&self) -> HashId {
        self.as_value().id()
    }
    fn len(&self) -> usize {
        self.as_inner().items().len()
    }
    fn get<'a>(
        &'a self,
        index: usize,
    ) -> Option<<WasmExpression<'heap, A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<'heap, A>: 'a,
    {
        self.as_inner()
            .items()
            .get(index)
            .copied()
            .map(|pointer| ArenaRef::new(self.arena, self.arena.get::<Term>(pointer)).into())
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        MapIntoIterator::new(self.as_inner().iter())
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, ListTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        self.iter()
            .map(|key| key.to_json())
            .collect::<Result<Vec<_>, String>>()
            .map(|values| JsonValue::Array(values))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        let updates = target
            .iter()
            .zip(self.iter())
            .map(|(current, previous)| previous.patch(&current))
            .chain(
                target
                    .iter()
                    .skip(self.len())
                    .map(|item| item.to_json().map(Some)),
            )
            .collect::<Result<Vec<_>, _>>()?;
        let updates = reflex_utils::json::json_object(
            updates
                .into_iter()
                .enumerate()
                .filter_map(|(index, item)| item.map(|value| (index.to_string(), value)))
                .chain(if target.len() != self.len() {
                    Some((String::from("length"), JsonValue::from(target.len())))
                } else {
                    None
                }),
        );
        if is_empty_json_object(&updates) {
            Ok(None)
        } else {
            Ok(Some(updates))
        }
    }
}

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, ListTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Clarify PartialEq implementations for container terms
        // This assumes that lists with the same length and hash are almost certainly identical
        self.len() == other.len()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, ListTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, ListTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, ListTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_displayed_items = 100;
        let items = self.iter();
        let num_items = items.len();
        write!(
            f,
            "[{}]",
            if num_items <= max_displayed_items {
                items
                    .map(|item| format!("{}", item))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                items
                    .take(max_displayed_items - 1)
                    .map(|item| format!("{}", item))
                    .chain(once(format!(
                        "...{} more items",
                        num_items - (max_displayed_items - 1)
                    )))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        term_type::{TermType, TermTypeDiscriminants},
    };

    use super::*;

    #[test]
    fn list() {
        assert_eq!(
            TermType::List(ListTerm {
                items: Default::default()
            })
            .as_bytes(),
            [TermTypeDiscriminants::List as u32, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let entries = [TermPointer(12345), TermPointer(67890)];
            let instance = ListTerm::allocate(entries, &mut allocator);
            let result = allocator.get::<Term>(instance).as_bytes();
            // TODO: Test term hashing
            let _hash = result[0];
            let discriminant = result[1];
            let data_length = result[2];
            let data_capacity = result[3];
            let data = &result[4..];
            assert_eq!(discriminant, TermTypeDiscriminants::List as u32);
            assert_eq!(data_length, entries.len() as u32);
            assert_eq!(data_capacity, entries.len() as u32);
            assert_eq!(data, [12345, 67890]);
        }
    }
}
