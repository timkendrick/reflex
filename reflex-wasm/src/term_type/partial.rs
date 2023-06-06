// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, PartialApplicationTermType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer, TypedTerm,
};

use super::ListTerm;

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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, PartialTerm, A> {
    fn target(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().target))
    }
    fn args(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().args))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> PartialApplicationTermType<T>
    for ArenaRef<'heap, PartialTerm, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
    for<'a> T::Ref<'a, T::ExpressionList<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn target<'a>(&'a self) -> T::Ref<'a, T>
    where
        T: 'a,
    {
        self.target().into()
    }
    fn args<'a>(&'a self) -> T::Ref<'a, T::ExpressionList<T>>
    where
        T: 'a,
        T::ExpressionList<T>: 'a,
    {
        self.args().into()
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
