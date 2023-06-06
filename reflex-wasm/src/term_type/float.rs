// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, FloatTermType, FloatValue, GraphNode, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FloatTerm {
    pub value: [u32; 2],
}
impl TermSize for FloatTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FloatTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<f64> for FloatTerm {
    fn from(value: f64) -> Self {
        Self {
            value: f64_to_chunks(value),
        }
    }
}
impl From<FloatTerm> for f64 {
    fn from(value: FloatTerm) -> Self {
        let FloatTerm { value, .. } = value;
        chunks_to_f64(value)
    }
}

fn f64_to_chunks(value: f64) -> [u32; 2] {
    let bytes = value.to_le_bytes();
    let low_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high_word = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    [low_word, high_word]
}

fn chunks_to_f64(value: [u32; 2]) -> f64 {
    let [low_word, high_word] = value;
    let low_bytes = low_word.to_le_bytes();
    let high_bytes = high_word.to_le_bytes();
    f64::from_le_bytes([
        low_bytes[0],
        low_bytes[1],
        low_bytes[2],
        low_bytes[3],
        high_bytes[0],
        high_bytes[1],
        high_bytes[2],
        high_bytes[3],
    ])
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, FloatTerm, A> {
    pub fn value(&self) -> f64 {
        chunks_to_f64(self.as_value().value)
    }
}

impl<'heap, A: ArenaAllocator> FloatTermType for ArenaRef<'heap, FloatTerm, A> {
    fn value(&self) -> FloatValue {
        self.value()
    }
}

impl<'heap, A: ArenaAllocator> FloatTermType for ArenaRef<'heap, TypedTerm<FloatTerm>, A> {
    fn value(&self) -> FloatValue {
        <ArenaRef<'heap, FloatTerm, A> as FloatTermType>::value(&self.as_inner())
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, FloatTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, FloatTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        match serde_json::Number::from_f64(self.value()) {
            Some(number) => Ok(JsonValue::Number(number)),
            None => Err(format!(
                "Unable to serialize float non-finite float as JSON value: {}",
                self
            )),
        }
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.value() == target.value() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, FloatTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, FloatTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, FloatTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, FloatTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn float() {
        let value = 3.142;
        assert_eq!(
            TermType::Float(FloatTerm::from(value)).as_bytes(),
            [
                TermTypeDiscriminants::Float as u32,
                f64_to_chunks(value)[0],
                f64_to_chunks(value)[1]
            ],
        );
    }
}
