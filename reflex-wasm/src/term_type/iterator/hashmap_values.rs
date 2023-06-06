// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashmapValuesIteratorTerm {
    pub source: TermPointer,
}
impl TermSize for HashmapValuesIteratorTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashmapValuesIteratorTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.source, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn hashmap_values_iterator() {
        assert_eq!(
            TermType::HashmapValuesIterator(HashmapValuesIteratorTerm {
                source: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::HashmapValuesIterator as u32, 12345],
        );
    }
}
