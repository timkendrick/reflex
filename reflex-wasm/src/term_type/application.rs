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
pub struct ApplicationTerm {
    pub target: TermPointer,
    pub args: TermPointer,
}
impl TermSize for ApplicationTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ApplicationTerm {
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
    fn application() {
        assert_eq!(
            TermType::Application(ApplicationTerm {
                target: TermPointer(12345),
                args: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Application as u32, 12345, 67890],
        );
    }
}
