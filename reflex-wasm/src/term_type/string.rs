// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, slice};

use reflex::{
    core::{
        DependencyList, Expression, GraphNode, SerializeJson, StackOffset, StringTermType,
        StringValue,
    },
    hash::HashId,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermType, TypedTerm},
    ArenaRef, Array, Term, TermPointer,
};

use super::WasmExpression;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct StringTerm {
    pub length: u32,
    pub data: Array<u32>,
}
impl TermSize for StringTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<u32>() + self.data.size_of()
    }
}
impl TermHash for StringTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.length, arena).hash(&self.data, arena)
    }
}
impl StringTerm {
    pub fn allocate(value: &str, arena: &mut impl ArenaAllocator) -> TermPointer {
        let term = Term::new(
            TermType::String(StringTerm {
                length: value.len() as u32,
                data: Default::default(),
            }),
            arena,
        );
        let term_size = term.size_of();
        let instance = arena.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<u32>>()) as u32);
        Array::<u32>::extend(list, get_string_chunks(value), arena);
        // TODO: Test term hashing
        let _hash = {
            arena
                .get::<Term>(instance)
                .hash(Default::default(), arena)
                .finish()
        };
        let hash = {
            arena
                .get::<Term>(instance)
                .hash(Default::default(), arena)
                .finish()
        };
        arena.write::<u32>(Term::get_hash_pointer(instance), u32::from(hash));
        instance
    }
    pub fn as_str(&self) -> &str {
        let start_pointer = self.data.items.as_ptr() as *const u8;
        let num_bytes = self.data.size_of();
        // First, we build a &[u8]...
        let slice = unsafe { slice::from_raw_parts(start_pointer, num_bytes) };
        // ... and then convert that slice into a string slice
        std::str::from_utf8(slice).unwrap()
    }
}

impl PartialEq for StringTerm {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }
        let (self_whole_words, self_trailing_bytes) =
            get_chunked_bytes(&self.data.items, self.length as usize);
        let (other_whole_words, other_trailing_bytes) =
            get_chunked_bytes(&other.data.items, other.length as usize);
        self_whole_words
            .zip(other_whole_words)
            .all(|(left, right)| left == right)
            && self_trailing_bytes
                .zip(other_trailing_bytes)
                .all(|(left, right)| left == right)
    }
}
impl Eq for StringTerm {}

impl std::fmt::Debug for StringTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for StringTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<StringTerm, A> {
    pub fn string_hash(&self) -> HashId {
        // FIXME: Convert to 64-bit hashes
        u32::from(
            self.as_value()
                .hash(TermHasher::default(), &self.arena)
                .finish(),
        ) as HashId
    }
    pub fn as_str(&self) -> &str {
        self.as_value().as_str()
    }
}

impl<A: ArenaAllocator + Clone> StringValue for ArenaRef<StringTerm, A> {
    fn id(&self) -> HashId {
        self.string_hash()
    }
    fn as_str(&self) -> &str {
        self.as_str()
    }
    fn from_static(_self: Option<Self>, _value: &'static str) -> Option<Self> {
        // FIXME: Implement StringValue::from_static() for WASM StringTerm type
        None
    }
}

impl<A: ArenaAllocator + Clone> StringValue for ArenaRef<TypedTerm<StringTerm>, A> {
    fn id(&self) -> HashId {
        <ArenaRef<StringTerm, A> as StringValue>::id(&self.as_inner())
    }
    fn as_str(&self) -> &str {
        self.as_inner_value().as_str()
    }
    fn from_static(_self: Option<Self>, _value: &'static str) -> Option<Self> {
        // FIXME: Implement StringValue::from_static() for WASM StringTerm type
        None
    }
}

// impl<A: ArenaAllocator + Clone> StringTermType<ArenaRef<Term, A>>
//     for ArenaRef<TypedTerm<StringTerm>, A>
// {
//     fn value<'a>(&'a self) -> <ArenaRef<Term, A> as Expression>::StringRef<'a>
//     where
//         <ArenaRef<Term, A> as Expression>::String: 'a,
//         ArenaRef<Term, A>: 'a,
//     {
//         (*self).into()
//     }
// }

impl<A: ArenaAllocator + Clone> StringTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<StringTerm>, A>
{
    fn value<'a>(&'a self) -> <WasmExpression<A> as Expression>::StringRef<'a>
    where
        <WasmExpression<A> as Expression>::String: 'a,
        WasmExpression<A>: 'a,
    {
        self.clone()
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<StringTerm, A> {
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<StringTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::String(String::from(self.as_str())))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.as_str() == target.as_str() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<StringTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.as_value() == other.as_value()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<StringTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<StringTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<StringTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

fn get_chunked_bytes(
    words: &[u32],
    num_bytes: usize,
) -> (
    impl Iterator<Item = u32> + '_,
    impl Iterator<Item = u8> + '_,
) {
    let num_whole_words = num_bytes / 4;
    let num_trailing_bytes = num_bytes % 4;
    debug_assert_eq!(
        words.len(),
        num_whole_words + if num_trailing_bytes == 0 { 0 } else { 1 }
    );
    let whole_words = words.iter().take(num_whole_words).copied();
    let trailing_bytes = (&words[num_whole_words..])
        .iter()
        .copied()
        .flat_map(|word| word.to_le_bytes())
        .take(num_trailing_bytes);
    (whole_words, trailing_bytes)
}

fn get_string_chunks(value: &str) -> Vec<u32> {
    value
        .as_bytes()
        .chunks(4)
        .map(|chunk| {
            (chunk.get(0).copied().unwrap_or(0) as u32)
                | (chunk.get(1).copied().unwrap_or(0) as u32) << 8
                | (chunk.get(2).copied().unwrap_or(0) as u32) << 16
                | (chunk.get(3).copied().unwrap_or(0) as u32) << 24
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        pad_to_4_byte_offset,
        term_type::{TermType, TermTypeDiscriminants},
    };

    use super::*;

    #[test]
    fn string() {
        assert_eq!(
            TermType::String(StringTerm {
                length: 12345,
                data: Default::default(),
            })
            .as_bytes(),
            [TermTypeDiscriminants::String as u32, 12345, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let value = "foobarbaz";
            let instance = StringTerm::allocate(value, &mut allocator);
            let result = allocator.get::<Term>(instance).as_bytes();
            // TODO: Test term hashing
            let _hash = result[0];
            let discriminant = result[1];
            let length = result[2];
            let data_length = result[3];
            let data_capacity = result[4];
            let data = &result[5..];
            assert_eq!(discriminant, TermTypeDiscriminants::String as u32);
            assert_eq!(length, value.len() as u32);
            assert_eq!(data_length, (pad_to_4_byte_offset(value.len()) / 4) as u32);
            assert_eq!(
                data_capacity,
                (pad_to_4_byte_offset(value.len()) / 4) as u32
            );
            assert_eq!(data, &get_string_chunks(value));
        }
    }
}
