// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{BuiltinTermType, Expression};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    ArenaRef,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub struct FunctionIndex(u32);
impl TermSize for FunctionIndex {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FunctionIndex {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let Self(uid) = self;
        hasher.hash(uid, arena)
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
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BuiltinTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.uid, arena)
    }
}
impl From<Stdlib> for BuiltinTerm {
    fn from(value: Stdlib) -> Self {
        Self { uid: value.into() }
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, BuiltinTerm, A> {
    fn target(&self) -> FunctionIndex {
        self.as_deref().uid
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> BuiltinTermType<T> for ArenaRef<'heap, BuiltinTerm, A>
where
    T::Builtin: From<FunctionIndex>,
{
    fn target<'a>(&'a self) -> T::Builtin
    where
        T: 'a,
        T::Builtin: 'a,
    {
        self.target().into()
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
