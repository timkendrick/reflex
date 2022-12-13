// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Expression, GraphNode, LetTermType, RefType, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LetTerm {
    pub initializer: TermPointer,
    pub body: TermPointer,
}
impl TermSize for LetTerm {
    fn size_of(&self) -> usize {
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
    for<'a> T::ExpressionRef<'a>: From<ArenaRef<'a, Term, A>>,
{
    fn initializer<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        self.initializer().into()
    }
    fn body<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        self.body().into()
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> LetTermType<T>
    for ArenaRef<'heap, TypedTerm<LetTerm>, A>
where
    for<'a> T::ExpressionRef<'a>: From<ArenaRef<'a, Term, A>>,
{
    fn initializer<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        <ArenaRef<'heap, LetTerm, A> as LetTermType<T>>::initializer(&self.as_inner())
    }
    fn body<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        <ArenaRef<'heap, LetTerm, A> as LetTermType<T>>::body(&self.as_inner())
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, LetTerm, A> {
    fn size(&self) -> usize {
        1 + self.initializer().size() + self.body().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.initializer()
            .capture_depth()
            .max(self.body().capture_depth().saturating_sub(1))
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.initializer()
            .free_variables()
            .into_iter()
            .chain(
                self.body()
                    .free_variables()
                    .into_iter()
                    .filter_map(|offset| if offset == 0 { None } else { Some(offset - 1) }),
            )
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.initializer().count_variable_usages(offset)
            + self.body().count_variable_usages(offset + 1)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        // TODO: Verify shallow dynamic dependencies for Let term
        self.initializer()
            .dynamic_dependencies(deep)
            .union(self.body().dynamic_dependencies(deep))
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        // TODO: Verify shallow dynamic dependencies for Let term
        self.initializer().has_dynamic_dependencies(deep)
            || self.body().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, LetTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, LetTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.initializer() == other.initializer() && self.body() == other.body()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, LetTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, LetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, LetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<let:{}:{}>", self.initializer(), self.body())
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
