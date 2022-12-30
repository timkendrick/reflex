// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Expression, GraphNode, NodeId, RecordTermType, RefType, SerializeJson,
    StackOffset,
};
use reflex_utils::json::is_empty_json_object;
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, Term, TermPointer,
};

use super::{ListTerm, WasmExpression};

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

impl<A: ArenaAllocator + Clone> ArenaRef<RecordTerm, A> {
    fn keys(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.keys),
        )
    }
    fn values(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.values),
        )
    }
    fn get<T: Expression>(&self, key: &T) -> Option<ArenaRef<Term, A>> {
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
                    .map(|pointer| ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
            })
    }
}

impl<A: ArenaAllocator + Clone> RecordTermType<WasmExpression<A>> for ArenaRef<RecordTerm, A> {
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        self.keys().into()
    }
    fn values<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
        WasmExpression<A>: 'a,
    {
        self.values().into()
    }
    fn get<'a>(
        &'a self,
        key: &WasmExpression<A>,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        self.get(key).map(|value| value.into())
    }
}

impl<A: ArenaAllocator + Clone> RecordTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<RecordTerm>, A>
{
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<RecordTerm, A> as RecordTermType<WasmExpression<A>>>::prototype(&self.as_inner())
    }
    fn values<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<RecordTerm, A> as RecordTermType<WasmExpression<A>>>::values(&self.as_inner())
    }
    fn get<'a>(
        &'a self,
        key: &WasmExpression<A>,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<RecordTerm, A> as RecordTermType<WasmExpression<A>>>::get(&self.as_inner(), key)
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<RecordTerm, A> {
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<RecordTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        let keys = self.keys().as_inner();
        let values = self.values().as_inner();
        let keys = keys.iter();
        let values = values.iter();
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
        let keys = self.keys().as_inner();
        let target_keys = target.keys().as_inner();
        if keys.len() != target_keys.len() {
            return Err(format!(
                "Prototype has changed from {} to {}",
                self.keys(),
                target.keys()
            ));
        }
        let target_values = target.values().as_inner();
        let target_entries = target_keys.iter().zip(target_values.iter());
        let updates = JsonValue::Object(
            target_entries
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<RecordTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.keys() == other.keys() && self.values() == other.values()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<RecordTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<RecordTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<RecordTerm, A> {
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
