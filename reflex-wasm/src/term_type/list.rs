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
pub struct ListTerm {
    pub items: Array<TermPointer>,
}
impl TermSize for ListTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<TermPointer>>() + self.items.size()
    }
}
impl TermHash for ListTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.items, allocator)
    }
}
impl ListTerm {
    pub fn allocate(
        values: impl IntoIterator<
            Item = TermPointer,
            IntoIter = impl ExactSizeIterator<Item = TermPointer>,
        >,
        allocator: &mut impl TermAllocator,
    ) -> TermPointer {
        let values = values.into_iter();
        let term = Term::new(
            TermType::List(ListTerm {
                items: Default::default(),
            }),
            allocator,
        );
        let term_size = term.size();
        let instance = allocator.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<TermPointer>>()) as u32);
        Array::<TermPointer>::extend(list, values, allocator);
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
