// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TakeIteratorTerm {
    pub source: TermPointer,
    pub count: u32,
}
impl TermSize for TakeIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TakeIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.source, arena).write_u32(self.count)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn take_iterator() {
        assert_eq!(
            TermType::TakeIterator(TakeIteratorTerm {
                source: TermPointer(12345),
                count: 67890,
            })
            .as_bytes(),
            [TermTypeDiscriminants::TakeIterator as u32, 12345, 67890],
        );
    }
}
