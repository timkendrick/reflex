// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub struct FunctionIndex(u32);
impl TermSize for FunctionIndex {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FunctionIndex {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        let Self(uid) = self;
        hasher.hash(uid, allocator)
    }
}
impl From<Stdlib> for FunctionIndex {
    fn from(value: Stdlib) -> Self {
        Self(value as u32)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BuiltinTerm {
    pub uid: FunctionIndex,
}
impl TermSize for BuiltinTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BuiltinTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.uid, allocator)
    }
}
impl From<Stdlib> for BuiltinTerm {
    fn from(value: Stdlib) -> Self {
        Self { uid: value.into() }
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn builtin() {
        assert_eq!(
            TermType::Builtin(BuiltinTerm::from(Stdlib::Add)).as_bytes(),
            [TermTypeDiscriminants::Builtin as u32, Stdlib::Add as u32],
        );
    }
}
