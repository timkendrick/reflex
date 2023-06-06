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
pub struct LetTerm {
    pub initializer: TermPointer,
    pub body: TermPointer,
}
impl TermSize for LetTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LetTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.initializer, allocator)
            .hash(&self.body, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn r#let() {
        assert_eq!(
            TermType::Let(LetTerm {
                initializer: TermPointer(12345),
                body: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Let as u32, 12345, 67890],
        );
    }
}
