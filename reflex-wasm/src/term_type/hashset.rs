// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, HashmapTermType, HashsetTermType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer, TypedTerm,
};

use super::HashmapTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashsetTerm {
    pub entries: TermPointer,
}
impl TermSize for HashsetTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashsetTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.entries, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, HashsetTerm, A> {
    fn entries(&self) -> ArenaRef<'heap, TypedTerm<HashmapTerm>, A> {
        ArenaRef::new(self.arena, &self.as_deref().entries)
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> HashsetTermType<T> for ArenaRef<'heap, HashsetTerm, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    type ValuesIterator<'a> = <ArenaRef<'heap, HashmapTerm, A> as HashmapTermType<T>>::KeysIterator<'a>
    where
        T: 'a,
        Self: 'a;
    fn contains<'a>(&'a self, value: &T) -> bool {
        self.values().any({
            let value_id = value.id();
            move |value| {
                if value.as_deref().id() == value_id {
                    true
                } else {
                    false
                }
            }
        })
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        T: 'a,
    {
        self.entries().values()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn hashset() {
        assert_eq!(
            TermType::Hashset(HashsetTerm {
                entries: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Hashset as u32, 12345],
        );
    }
}
