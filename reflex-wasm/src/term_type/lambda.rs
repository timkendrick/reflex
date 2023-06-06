// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, LambdaTermType, StackOffset};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LambdaTerm {
    pub num_args: u32,
    pub body: TermPointer,
}
impl TermSize for LambdaTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LambdaTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.num_args, arena).hash(&self.body, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, LambdaTerm, A> {
    fn num_args(&self) -> u32 {
        self.as_deref().num_args
    }
    fn body(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().body))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> LambdaTermType<T> for ArenaRef<'heap, LambdaTerm, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    fn num_args<'a>(&'a self) -> StackOffset {
        self.num_args() as StackOffset
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
    fn lambda() {
        assert_eq!(
            TermType::Lambda(LambdaTerm {
                num_args: 12345,
                body: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Lambda as u32, 12345, 67890],
        );
    }
}
