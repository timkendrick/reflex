// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{ConstructorTermType, Expression, RefType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, TermPointer, TypedTerm,
};

use super::ListTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ConstructorTerm {
    pub keys: TermPointer,
}
impl TermSize for ConstructorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ConstructorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.keys, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, ConstructorTerm, A> {
    fn keys(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().keys))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ConstructorTermType<T>
    for ArenaRef<'heap, ConstructorTerm, A>
where
    for<'a> T::Ref<'a, T::StructPrototype<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn prototype<'a>(&'a self) -> T::Ref<'a, T::StructPrototype<T>>
    where
        T::StructPrototype<T>: 'a,
        T: 'a,
    {
        self.keys().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn constructor() {
        assert_eq!(
            TermType::Constructor(ConstructorTerm {
                keys: TermPointer(12345)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Constructor as u32, 12345],
        );
    }
}
