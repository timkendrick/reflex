// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, LetTermType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer,
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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(&self.initializer, arena)
            .hash(&self.body, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, LetTerm, A> {
    fn initializer(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().initializer))
    }
    fn body(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().body))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> LetTermType<T> for ArenaRef<'heap, LetTerm, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    fn initializer<'a>(&'a self) -> T::Ref<'a, T>
    where
        T: 'a,
    {
        self.initializer().into()
    }
    fn body<'a>(&'a self) -> T::Ref<'a, T>
    where
        T: 'a,
    {
        self.body().into()
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
