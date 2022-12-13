// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::core::{
    DependencyList, Expression, GraphNode, HashmapTermType, HashsetTermType, RefType,
    SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, IntoArenaRefIterator, Term, TermPointer,
};

use super::{HashmapBucketKeysIterator, HashmapTerm};

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
    fn entries(&self) -> ArenaRef<'heap, TypedTerm<HashmapTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().entries))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> HashsetTermType<T> for ArenaRef<'heap, HashsetTerm, A>
where
    for<'a> T::ExpressionRef<'a>: From<ArenaRef<'a, Term, A>>,
{
    type ValuesIterator<'a> = <ArenaRef<'heap, HashmapTerm, A> as HashmapTermType<T>>::ValuesIterator<'a>
    where
        T: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &T) -> bool {
        self.values().any({
            let value_id = value.id();
            move |value| {
                if value.as_deref().id() == value_id {
                    true
                } else {
                    false
                }
            }
        })
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        T: 'a,
    {
        self.entries().values()
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> HashsetTermType<T>
    for ArenaRef<'heap, TypedTerm<HashsetTerm>, A>
where
    for<'a> T::ExpressionRef<'a>: From<ArenaRef<'a, Term, A>>,
{
    type ValuesIterator<'a> = <ArenaRef<'heap, HashsetTerm, A> as HashsetTermType<T>>::ValuesIterator<'a>
    where
        T: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &T) -> bool {
        <ArenaRef<'heap, HashsetTerm, A> as HashsetTermType<T>>::contains(&self.as_inner(), value)
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        T: 'a,
    {
        <ArenaRef<'heap, HashsetTerm, A> as HashsetTermType<T>>::values(&self.as_inner())
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
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, HashsetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_displayed_values = 10;
        let entries = self.entries().as_inner();
        let values = IntoArenaRefIterator::<'heap, Term, A, _>::new(
            self.arena,
            HashmapBucketKeysIterator::new(
                entries.num_entries() as usize,
                entries.buckets().as_deref().iter(),
            ),
        );
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
