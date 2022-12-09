// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::slice;

use reflex::{
    core::{Expression, StringTermType, StringValue},
    hash::HashId,
};

use crate::{
    allocator::{ArenaAllocator, VecAllocator},
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermType,
    ArenaPointer, ArenaRef, Array, Term, TermPointer, TypedTerm,
};

#[derive(Eq, Clone, Copy)]
#[repr(C)]
pub struct StringTerm {
    pub length: u32,
    pub data: Array<u32>,
}
impl PartialEq for StringTerm {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }
        let (self_whole_words, self_trailing_bytes) = get_chunked_bytes(self, self.length);
        let (other_whole_words, other_trailing_bytes) = get_chunked_bytes(other, other.length);
        self_whole_words
            .zip(other_whole_words)
            .all(|(left, right)| *left == *right)
            && self_trailing_bytes
                .zip(other_trailing_bytes)
                .all(|(left, right)| *left == *right)
    }
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
        arena.get_mut::<Term>(instance).set_hash(hash);
        instance
    }
}
impl std::fmt::Display for StringTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}
impl std::fmt::Debug for StringTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

fn get_chunked_bytes(
    words: &[u32],
    num_bytes: usize,
) -> (impl Iterator<Item = u32>, impl Iterator<Item = u8>) {
    let num_whole_words = num_bytes / 4;
    let num_trailing_bytes = num_bytes % 4;
    debug_assert_eq!(
        words.len(),
        num_whole_words + if num_trailing_bytes == 0 { 0 } else { 1 }
    );
    let whole_words = words.iter().take(num_whole_words).copied();
    let trailing_bytes = &words[num_whole_words..]
        .iter()
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

impl<'heap, T: Expression, A: ArenaAllocator> StringTermType<T>
    for ArenaPointer<'heap, TypedTerm<StringTerm>, A>
where
    for<'a> T::Ref<'a, T::String>: From<ArenaRef<'a, TypedTerm<StringTerm>, A>>,
{
    fn value<'a>(&'a self) -> T::Ref<'a, T::String>
    where
        T::String: 'a,
        T: 'a,
    {
        self.into()
    }
}

impl<'heap> StringValue for StringTerm {
    fn id(&self) -> HashId {
        self.hash(TermHasher::default(), &VecAllocator::default())
    }
    fn as_str(&self) -> &str {
        let start_pointer = self.data.items.as_ptr() as *const u8;
        let num_bytes = self.data.size_of();
        // First, we build a &[u8]...
        let slice = unsafe { slice::from_raw_parts(start_pointer, num_bytes) };
        // ... and then convert that slice into a string slice
        std::str::from_utf8(slice).unwrap()
    }
    fn from_static(_self: Option<Self>, value: &'static str) -> Option<Self> {
        None
    }
}

impl<'heap, A: ArenaAllocator> StringValue for ArenaPointer<'heap, TypedTerm<StringTerm>, A> {
    fn id(&self) -> &str {
        self.get_inner().id()
    }
    fn as_str(&self) -> &str {
        self.get_inner().as_str()
    }
    fn from_static(_self: Option<Self>, value: &'static str) -> Option<Self> {
        Self::from_static(_self, value)
    }
}

impl<'heap, A: ArenaAllocator> PartialEq for ArenaPointer<'heap, TypedTerm<StringTerm>, A> {
    fn eq(&self, other: &Self) -> bool {
        self.get_inner().eq(other)
    }
}

impl<'heap, A: ArenaAllocator> Eq for ArenaPointer<'heap, TypedTerm<StringTerm>, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaPointer<'heap, TypedTerm<StringTerm>, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.get_inner(), f)
    }
}
impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaPointer<'heap, TypedTerm<StringTerm>, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.get_inner(), f)
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
