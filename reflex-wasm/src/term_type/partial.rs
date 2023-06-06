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
pub struct PartialTerm {
    pub target: TermPointer,
    pub args: TermPointer,
}
impl TermSize for PartialTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PartialTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.target, allocator)
            .hash(&self.args, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn partial() {
        assert_eq!(
            TermType::Partial(PartialTerm {
                target: TermPointer(12345),
                args: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Partial as u32, 12345, 67890],
        );
    }
}
