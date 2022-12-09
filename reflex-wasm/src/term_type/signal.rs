// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, SignalTermType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, TermPointer, TypedTerm,
};

use super::TreeTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SignalTerm {
    pub conditions: TermPointer,
}
impl TermSize for SignalTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for SignalTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.conditions, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, SignalTerm, A> {
    fn conditions(&self) -> ArenaRef<'heap, TypedTerm<TreeTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().args))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> SignalTermType<T> for ArenaRef<'heap, SignalTerm, A>
where
    for<'a> T::Ref<'a, T::SignalList<T>>: From<ArenaRef<'a, TypedTerm<TreeTerm>, A>>,
{
    fn signals<'a>(&'a self) -> T::Ref<'a, T::SignalList<T>>
    where
        T::SignalList<T>: 'a,
        T: 'a,
    {
        self.conditions().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn signal() {
        assert_eq!(
            TermType::Signal(SignalTerm {
                conditions: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Signal as u32, 12345],
        );
    }
}
