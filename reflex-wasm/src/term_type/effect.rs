// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{EffectTermType, Expression};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, TermPointer, TypedTerm,
};

use super::ConditionTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EffectTerm {
    pub condition: TermPointer,
}
impl TermSize for EffectTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for EffectTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.condition, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, EffectTerm, A> {
    fn condition(&self) -> ArenaRef<'heap, TypedTerm<ConditionTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().condition))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> EffectTermType<T> for ArenaRef<'heap, EffectTerm, A>
where
    for<'a> T::Ref<'a, T::Signal<T>>: From<ArenaRef<'a, TypedTerm<ConditionTerm>, A>>,
{
    fn condition<'a>(&'a self) -> T::Ref<'a, T::Signal<T>>
    where
        T::Signal<T>: 'a,
        T: 'a,
    {
        self.condition().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn effect() {
        assert_eq!(
            TermType::Effect(EffectTerm {
                condition: TermPointer(12345)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Effect as u32, 12345],
        );
    }
}
