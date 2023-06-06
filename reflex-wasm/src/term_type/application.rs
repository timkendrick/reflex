// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{ApplicationTermType, Expression, RefType, SerializeJson};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer, TypedTerm,
};

use super::ListTerm;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ApplicationTerm {
    pub target: TermPointer,
    pub args: TermPointer,
}
impl TermSize for ApplicationTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ApplicationTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, ApplicationTerm, A> {
    fn target(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().target))
    }
    fn args(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().args))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ApplicationTermType<T>
    for ArenaRef<'heap, ApplicationTerm, A>
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

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, ApplicationTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, ApplicationTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<apply:{}:{}>",
            self.target().as_deref(),
            self.args()
                .as_deref()
                .iter()
                .map(|item| item.as_deref())
                .map(|arg| format!("{}", arg))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, ApplicationTerm, A> {
    fn to_json(&self) -> Result<serde_json::Value, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn application() {
        assert_eq!(
            TermType::Application(ApplicationTerm {
                target: TermPointer(12345),
                args: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Application as u32, 12345, 67890],
        );
    }
}
