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
pub struct RecordTerm {
    pub keys: TermPointer,
    pub values: TermPointer,
}
impl TermSize for RecordTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RecordTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.keys, allocator)
            .hash(&self.values, allocator)
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
