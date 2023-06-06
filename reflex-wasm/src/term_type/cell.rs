// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHashState, TermHasher, TermSize},
    term_type::TermType,
    Array, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CellTerm {
    pub fields: Array<u32>,
}
impl TermSize for CellTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<u32>>() + self.fields.size()
    }
}
impl TermHash for CellTerm {
    fn hash(&self, hasher: TermHasher, _allocator: &impl TermAllocator) -> TermHasher {
        hasher
    }
}
impl CellTerm {
    pub fn allocate(
        values: impl IntoIterator<Item = u32, IntoIter = impl ExactSizeIterator<Item = u32>>,
        allocator: &mut impl TermAllocator,
    ) -> TermPointer {
        let values = values.into_iter();
        let term = Term::new(
            TermType::Cell(Self {
                fields: Default::default(),
            }),
            allocator,
        );
        let term_size = term.size();
        let instance = allocator.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<u32>>()) as u32);
        Array::<u32>::extend(list, values, allocator);
        let hash = TermHashState::from(u32::from(instance));
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
    fn cell() {
        assert_eq!(
            TermType::Cell(CellTerm {
                fields: Default::default()
            })
            .as_bytes(),
            [TermTypeDiscriminants::Cell as u32, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let entries = [12345, 67890];
            let instance = CellTerm::allocate(entries, &mut allocator);
            let result = allocator.get::<Term>(instance).as_bytes();
            let hash = result[0];
            let discriminant = result[1];
            let data_length = result[2];
            let data_capacity = result[3];
            let data = &result[4..];
            assert_eq!(hash, u32::from(instance));
            assert_eq!(discriminant, TermTypeDiscriminants::Cell as u32);
            assert_eq!(data_length, entries.len() as u32);
            assert_eq!(data_capacity, entries.len() as u32);
            assert_eq!(data, [12345, 67890]);
        }
    }
}
