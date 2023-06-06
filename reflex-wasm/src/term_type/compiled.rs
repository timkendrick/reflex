// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub struct CompiledFunctionIndex(u32);
impl TermSize for CompiledFunctionIndex {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CompiledFunctionIndex {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        let Self(uid) = self;
        hasher.hash(uid, allocator)
    }
}
impl From<u32> for CompiledFunctionIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CompiledTerm {
    pub target: CompiledFunctionIndex,
    pub num_args: u32,
}
impl TermSize for CompiledTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CompiledTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.target, allocator)
            .hash(&self.num_args, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn compiled() {
        assert_eq!(
            TermType::Compiled(CompiledTerm {
                target: CompiledFunctionIndex::from(12345),
                num_args: 3,
            })
            .as_bytes(),
            [TermTypeDiscriminants::Compiled as u32, 12345, 3],
        );
    }
}
