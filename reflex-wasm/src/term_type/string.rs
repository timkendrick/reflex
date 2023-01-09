// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, ops::Deref, slice, str::from_utf8_unchecked};

use reflex::{
    core::{
        DependencyList, Eagerness, Expression, GraphNode, Internable, SerializeJson, StackOffset,
        StringTermType, StringValue,
    },
    hash::HashId,
};
use serde_json::Value as JsonValue;

use super::WasmExpression;
use crate::{
    allocator::{Arena, ArenaAllocator},
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermType, TypedTerm},
    ArenaPointer, ArenaRef, Array, Term,
};
use reflex_macros::PointerIter;

#[derive(Clone, Copy, PointerIter)]
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
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.length, arena).hash(&self.data, arena)
    }
}
impl StringTerm {
    pub fn allocate(value: &str, arena: &mut impl ArenaAllocator) -> ArenaPointer {
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
        let hash = arena.read_value::<Term, _>(instance, |term| {
            TermHasher::default().hash(term, arena).finish()
        });
        arena.write::<u32>(Term::get_hash_pointer(instance), u32::from(hash));
        instance
    }
    pub fn as_str(&self) -> &str {
        let start_pointer = self.data.items.as_ptr() as *const u8;
        let num_bytes = self.length as usize;
        // First, we build a &[u8]...
        let slice = unsafe { slice::from_raw_parts(start_pointer, num_bytes) };
        // ... and then convert that slice into a string slice
        // FIXME: Prevent panic on invalid UTF-8 bytes
        std::str::from_utf8(slice).expect("Invalid UTF-8 bytes")
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

impl<A: Arena + Clone> ArenaRef<StringTerm, A> {
    pub fn len(&self) -> usize {
        self.read_value(|term| term.length as usize)
    }
    pub fn data(&self) -> ArenaRef<Array<u32>, A> {
        self.inner_ref(|term| &term.data)
    }
    pub fn string_hash(&self) -> HashId {
        self.read_value(|term| {
            // FIXME: Convert to 64-bit hashes
            u32::from(term.hash(TermHasher::default(), &self.arena).finish()) as HashId
        })
    }
    fn offset(&self) -> ArenaPointer {
        self.inner_pointer(|term| &term.data.items[0])
    }
    fn as_utf8<'a>(&'a self) -> Utf8Bytes<A::Slice<'a>> {
        Utf8Bytes(self.arena.as_slice(self.offset(), self.len()))
    }
}

pub struct Utf8Bytes<T: Deref<Target = [u8]>>(T);
impl<T: Deref<Target = [u8]>> Deref for Utf8Bytes<T> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { from_utf8_unchecked(self.0.deref()) }
    }
}
impl<T: Deref<Target = [u8]>> From<Utf8Bytes<T>> for String {
    fn from(value: Utf8Bytes<T>) -> Self {
        String::from(value.deref())
    }
}

impl<A: Arena + Clone> StringValue for ArenaRef<StringTerm, A> {
    type StringRef<'a> = Utf8Bytes<A::Slice<'a>>
        where
            Self: 'a;
    fn id(&self) -> HashId {
        self.string_hash()
    }
    fn as_str<'a>(&'a self) -> Self::StringRef<'a> {
        self.as_utf8()
    }
}

impl<A: Arena + Clone> StringValue for ArenaRef<TypedTerm<StringTerm>, A> {
    type StringRef<'a> = Utf8Bytes<A::Slice<'a>>
        where
            Self: 'a;
    fn id(&self) -> HashId {
        <ArenaRef<StringTerm, A> as StringValue>::id(&self.as_inner())
    }
    fn as_str<'a>(&'a self) -> Self::StringRef<'a> {
        let inner = self.as_inner();
        Utf8Bytes(self.arena.as_slice(inner.offset(), inner.len()))
    }
}

impl<A: Arena + Clone> StringTermType<WasmExpression<A>> for ArenaRef<TypedTerm<StringTerm>, A> {
    fn value<'a>(&'a self) -> <WasmExpression<A> as Expression>::StringRef<'a>
    where
        <WasmExpression<A> as Expression>::String: 'a,
        WasmExpression<A>: 'a,
    {
        self.clone()
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<StringTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<StringTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::String(String::from(self.as_utf8())))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.as_str().deref() == target.as_str().deref() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<StringTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Clarify PartialEq implementations for container terms
        // This assumes that strings with the same length and hash are almost certainly identical
        self.len() == other.len()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<StringTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<StringTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<StringTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
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

impl<A: Arena + Clone> Internable for ArenaRef<StringTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
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
            let result = allocator.get_ref::<Term>(instance).as_bytes();
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
