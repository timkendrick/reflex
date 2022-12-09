// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionListType, ListTermType, RecordTermType, RefType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer, TypedTerm,
};

use super::ListTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RecordTerm {
    pub keys: TermPointer,
    pub values: TermPointer,
}
impl TermSize for RecordTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RecordTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.keys, arena).hash(&self.values, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, RecordTerm, A> {
    fn keys(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().keys))
    }
    fn values(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().values))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> RecordTermType<T> for ArenaRef<'heap, RecordTerm, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
    for<'a> T::Ref<'a, T::StructPrototype<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
    for<'a> T::Ref<'a, T::ExpressionList<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn prototype<'a>(&'a self) -> T::Ref<'a, T::StructPrototype<T>>
    where
        T::StructPrototype<T>: 'a,
        T: 'a,
    {
        self.keys().into()
    }
    fn values<'a>(&'a self) -> T::Ref<'a, T::ExpressionList<T>>
    where
        T::ExpressionList<T>: 'a,
        T: 'a,
    {
        self.values().into()
    }
    fn get<'a>(&'a self, key: &T) -> Option<T::Ref<'a, T>>
    where
        T: 'a,
    {
        self.keys()
            .as_deref()
            .items()
            .as_deref()
            .iter()
            .map(|item| item.as_deref())
            .position(|existing_key| existing_key.id() == key.id())
            .and_then(|index| {
                self.values()
                    .as_deref()
                    .items()
                    .as_deref()
                    .get(index)
                    .map(|value| value.into())
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn record() {
        assert_eq!(
            TermType::Record(RecordTerm {
                keys: TermPointer(12345),
                values: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Record as u32, 12345, 67890],
        );
    }
}
