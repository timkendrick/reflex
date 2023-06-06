// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::core::{
    DependencyList, Eagerness, GraphNode, HashmapTermType, HashsetTermType, Internable,
    SerializeJson, StackOffset,
};
use reflex_utils::MapIntoIterator;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaPointer, ArenaRef, Array, IntoArenaRefIter,
};
use reflex_macros::PointerIter;

use super::{
    HashmapBucket, HashmapBucketKeysIterator, HashmapBucketsIterator, HashmapTerm, WasmExpression,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct HashsetTerm {
    pub entries: ArenaPointer,
}
impl TermSize for HashsetTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashsetTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.entries, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<HashsetTerm, A> {
    pub fn entries(&self) -> ArenaRef<TypedTerm<HashmapTerm>, A> {
        ArenaRef::<TypedTerm<HashmapTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|value| value.entries),
        )
    }
    pub fn num_values(&self) -> usize {
        self.entries().as_inner().num_entries()
    }
    pub fn values(
        &self,
    ) -> <ArenaRef<TypedTerm<HashmapTerm>, A> as HashmapTermType<WasmExpression<A>>>::KeysIterator<'_>
    {
        let entries = self.entries().as_inner();
        let buckets_pointer = entries.buckets_pointer();
        let buckets = HashmapBucketsIterator::new(
            entries.num_entries() as usize,
            Array::<HashmapBucket>::iter(buckets_pointer, &self.arena),
        );
        MapIntoIterator::new(IntoArenaRefIter::new(
            &self.arena,
            HashmapBucketKeysIterator::new(buckets),
        ))
    }
}

impl<A: Arena + Clone> HashsetTermType<WasmExpression<A>> for ArenaRef<HashsetTerm, A> {
    type ValuesIterator<'a> = <ArenaRef<HashmapTerm, A> as HashmapTermType<WasmExpression<A>>>::KeysIterator<'a>
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &WasmExpression<A>) -> bool {
        self.values().any({
            let value_id = value.read_value(|term| term.id());
            move |item| {
                if item.read_value(|term| term.id()) == value_id {
                    true
                } else {
                    false
                }
            }
        })
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.values()
    }
}

impl<A: Arena + Clone> HashsetTermType<WasmExpression<A>> for ArenaRef<TypedTerm<HashsetTerm>, A> {
    type ValuesIterator<'a> = <ArenaRef<HashsetTerm, A> as HashsetTermType<WasmExpression<A>>>::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &WasmExpression<A>) -> bool {
        <ArenaRef<HashsetTerm, A> as HashsetTermType<WasmExpression<A>>>::contains(
            &self.as_inner(),
            value,
        )
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        let entries = self.as_inner().entries().as_inner();
        let buckets_pointer = entries.buckets_pointer();
        let buckets = HashmapBucketsIterator::new(
            entries.num_entries() as usize,
            Array::<HashmapBucket>::iter(buckets_pointer, &self.arena),
        );
        MapIntoIterator::new(IntoArenaRefIter::new(
            &self.arena,
            HashmapBucketKeysIterator::new(buckets),
        ))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<HashsetTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<HashsetTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<HashsetTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.entries() == other.entries()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<HashsetTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<HashsetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<HashsetTerm, A> {
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

impl<A: Arena + Clone> Internable for ArenaRef<HashsetTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
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
                entries: ArenaPointer(0x54321),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Hashset as u32, 0x54321],
        );
    }
}
