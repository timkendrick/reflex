// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Expression, GraphNode, RecordTermType, RefType, SerializeJson, StackOffset,
};
use reflex_utils::json::is_empty_json_object;
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, Term, TermPointer,
};

use super::ListTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RecordTerm {
    pub keys: TermPointer,
    pub values: TermPointer,
}
impl TermSize for RecordTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RecordTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.keys, arena).hash(&self.values, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, RecordTerm, A> {
    fn keys(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().keys))
    }
    fn values(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().values))
    }
    fn get<T: Expression>(&self, key: &T) -> Option<ArenaRef<'heap, Term, A>> {
        self.keys()
            .as_inner()
            .iter()
            .map(|item| item.as_deref())
            .position(|existing_key| existing_key.id() == key.id())
            .and_then(|index| {
                self.values()
                    .as_inner()
                    .items()
                    .get(index)
                    .copied()
                    .map(|pointer| ArenaRef::new(self.arena, self.arena.get::<Term>(pointer)))
            })
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> RecordTermType<T> for ArenaRef<'heap, RecordTerm, A>
where
    for<'a> T::ExpressionRef<'a>: From<ArenaRef<'a, Term, A>>,
    for<'a> T::StructPrototypeRef<'a, T>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
    for<'a> T::ExpressionListRef<'a, T>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn prototype<'a>(&'a self) -> T::StructPrototypeRef<'a, T>
    where
        T::StructPrototype<T>: 'a,
        T: 'a,
    {
        self.keys().into()
    }
    fn values<'a>(&'a self) -> T::ExpressionListRef<'a, T>
    where
        T::ExpressionList<T>: 'a,
        T: 'a,
    {
        self.values().into()
    }
    fn get<'a>(&'a self, key: &T) -> Option<T::ExpressionRef<'a>>
    where
        T: 'a,
    {
        self.get(key).map(|value| value.into())
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> RecordTermType<T>
    for ArenaRef<'heap, TypedTerm<RecordTerm>, A>
where
    for<'a> T::ExpressionRef<'a>: From<ArenaRef<'a, Term, A>>,
    for<'a> T::StructPrototypeRef<'a, T>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
    for<'a> T::ExpressionListRef<'a, T>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn prototype<'a>(&'a self) -> T::StructPrototypeRef<'a, T>
    where
        <T as Expression>::StructPrototype<T>: 'a,
        T: 'a,
    {
        <ArenaRef<'heap, RecordTerm, A> as RecordTermType<T>>::prototype(&self.as_inner())
    }
    fn values<'a>(&'a self) -> <T as Expression>::ExpressionListRef<'a, T>
    where
        <T as Expression>::ExpressionList<T>: 'a,
        T: 'a,
    {
        <ArenaRef<'heap, RecordTerm, A> as RecordTermType<T>>::values(&self.as_inner())
    }
    fn get<'a>(&'a self, key: &T) -> Option<<T as Expression>::ExpressionRef<'a>>
    where
        T: 'a,
    {
        <ArenaRef<'heap, RecordTerm, A> as RecordTermType<T>>::get(&self.as_inner(), key)
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, RecordTerm, A> {
    fn size(&self) -> usize {
        1 + self.keys().size() + self.values().size()
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
        true
    }
    fn is_atomic(&self) -> bool {
        self.values().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, RecordTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        let keys = self.keys().as_inner().iter();
        let values = self.values().as_inner().iter();
        let entries = keys.zip(values);
        let fields = entries
            .map(|(key, value)| {
                let key = key.to_json()?;
                let value = value.to_json()?;
                match key {
                    JsonValue::String(key) => Ok((key, value)),
                    _ => Err(format!("Invalid JSON object key: {}", key.to_string())),
                }
            })
            .collect::<Result<JsonMap<_, _>, String>>()?;
        Ok(JsonValue::Object(fields))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.keys().as_inner().len() != target.keys().as_inner().len() {
            return Err(format!(
                "Prototype has changed from {} to {}",
                self.keys(),
                target.keys()
            ));
        }
        let keys = self.keys().as_inner().iter();
        let values = self.values().as_inner().iter();
        let entries = keys.zip(values);
        let updates = JsonValue::Object(
            entries
                .map(|(key, new_value)| {
                    let previous_value = self.get(&key).ok_or_else(|| {
                        format!(
                            "Prototype has changed, key {} not present in {}",
                            key.to_string(),
                            self.keys()
                        )
                    })?;
                    Ok(previous_value
                        .patch(&new_value)?
                        .map(|value_patch| (key, value_patch)))
                })
                .filter_map(|entry| entry.transpose()) // Filter out unchanged fields
                .map(|entry| {
                    entry.and_then(|(key, value)| match key.to_json()? {
                        JsonValue::String(key) => Ok((key, value)),
                        _ => Err(format!("Invalid JSON object key: {}", key.to_string())),
                    })
                })
                .collect::<Result<JsonMap<_, _>, _>>()?,
        );
        if is_empty_json_object(&updates) {
            Ok(None)
        } else {
            Ok(Some(updates))
        }
    }
}

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, RecordTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.keys() == other.keys() && self.values() == other.values()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, RecordTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, RecordTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, RecordTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.keys().as_inner().len() {
            0 => write!(f, "{{}}"),
            _ => write!(
                f,
                "{{ {} }}",
                self.keys()
                    .as_inner()
                    .iter()
                    .zip(self.values().as_inner().iter())
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn record() {
        assert_eq!(
            TermType::Record(RecordTerm {
                keys: TermPointer(12345),
                values: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Record as u32, 12345, 67890],
        );
    }
}
