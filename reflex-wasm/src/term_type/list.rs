// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::iter::Copied;

use reflex::{
    core::{
        Expression, ExpressionListType, FromRefTypeIterator, ListTermType, RefType,
        StructPrototypeType,
    },
    hash::HashId,
};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermType,
    ArenaRef, Array, ArrayIter, IntoArenaRefIterator, Term, TermPointer, TypedTerm,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ListTerm {
    pub items: Array<TermPointer>,
}
impl TermSize for ListTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<TermPointer>>() + self.items.size()
    }
}
impl TermHash for ListTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.items, arena)
    }
}
impl ListTerm {
    pub fn allocate(
        values: impl IntoIterator<
            Item = TermPointer,
            IntoIter = impl ExactSizeIterator<Item = TermPointer>,
        >,
        arena: &mut impl ArenaAllocator,
    ) -> TermPointer {
        let values = values.into_iter();
        let term = Term::new(
            TermType::List(ListTerm {
                items: Default::default(),
            }),
            arena,
        );
        let term_size = term.size();
        let instance = arena.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<TermPointer>>()) as u32);
        Array::<TermPointer>::extend(list, values, arena);
        let hash = {
            arena
                .get::<Term>(instance)
                .hash(Default::default(), arena)
                .finish()
        };
        arena.get_mut::<Term>(instance).set_hash(hash);
        instance
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
    fn items(&self) -> ArenaRef<'heap, Array<TermPointer>, A> {
        ArenaRef::new(self.arena, &self.as_deref().items)
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ListTermType<T>
    for ArenaRef<'heap, TypedTerm<ListTerm>, A>
where
    for<'a> T::Ref<'a, T::ExpressionList<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn items<'a>(&'a self) -> T::Ref<'a, T::ExpressionList<T>>
    where
        T::ExpressionList<T>: 'a,
        T: 'a,
    {
        self.into()
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> StructPrototypeType<T>
    for ArenaRef<'heap, TypedTerm<ListTerm>, A>
where
    for<'a> T::Ref<'a, T::ExpressionList<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn keys<'a>(&'a self) -> T::Ref<'a, T::ExpressionList<T>>
    where
        T::ExpressionList<T>: 'a,
        T: 'a,
    {
        self.into()
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ExpressionListType<T>
    for ArenaRef<'heap, TypedTerm<ListTerm>, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    type Iterator<'a> = FromRefTypeIterator<'a, T, T::Ref<'a, T>, IntoArenaRefIterator<'a, Term, A, Copied<ArrayIter<'a, TermPointer>>>>
    where
        T: 'a,
        Self: 'a;
    fn id(&self) -> HashId {
        self.as_deref().id()
    }
    fn len(&self) -> usize {
        self.items().len()
    }
    fn get<'a>(&'a self, index: usize) -> Option<T::Ref<'a, T>>
    where
        T: 'a,
    {
        self.items()
            .get(index)
            .copied()
            .map(|pointer| ArenaRef::new(self.arena, self.arena.get::<Term>(pointer)).into())
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a> {
        FromRefTypeIterator::new(IntoArenaRefIterator::new(
            self.arena,
            self.items().iter().copied(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        term_type::{TermType, TermTypeDiscriminants},
    };

    use super::*;

    #[test]
    fn list() {
        assert_eq!(
            TermType::List(ListTerm {
                items: Default::default()
            })
            .as_bytes(),
            [TermTypeDiscriminants::List as u32, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let entries = [TermPointer(12345), TermPointer(67890)];
            let instance = ListTerm::allocate(entries, &mut allocator);
            let result = allocator.get::<Term>(instance).as_bytes();
            // TODO: Test term hashing
            let _hash = result[0];
            let discriminant = result[1];
            let data_length = result[2];
            let data_capacity = result[3];
            let data = &result[4..];
            assert_eq!(discriminant, TermTypeDiscriminants::List as u32);
            assert_eq!(data_length, entries.len() as u32);
            assert_eq!(data_capacity, entries.len() as u32);
            assert_eq!(data, [12345, 67890]);
        }
    }
}
