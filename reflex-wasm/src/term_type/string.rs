// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermType,
    Array, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct StringTerm {
    pub length: u32,
    pub data: Array<u32>,
}
impl TermSize for StringTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<u32>() + self.data.size()
    }
}
impl TermHash for StringTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.length, allocator)
            .hash(&self.data, allocator)
    }
}
impl StringTerm {
    pub fn allocate(value: &str, allocator: &mut impl TermAllocator) -> TermPointer {
        let term = Term::new(
            TermType::String(StringTerm {
                length: value.len() as u32,
                data: Default::default(),
            }),
            allocator,
        );
        let term_size = term.size();
        let instance = allocator.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<u32>>()) as u32);
        Array::<u32>::extend(list, get_string_chunks(value), allocator);
        // TODO: Test term hashing
        let _hash = {
            allocator
                .get::<Term>(instance)
                .hash(Default::default(), allocator)
                .finish()
        };
        let hash = {
            allocator
                .get::<Term>(instance)
                .hash(Default::default(), allocator)
                .finish()
        };
        allocator.get_mut::<Term>(instance).set_hash(hash);
        instance
    }
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
