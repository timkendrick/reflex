// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, DependencyList, Expression, GraphNode, LambdaTermType, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, Term, TermPointer,
};

use super::WasmExpression;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LambdaTerm {
    pub num_args: u32,
    pub body: TermPointer,
}
impl TermSize for LambdaTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LambdaTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.num_args, arena).hash(&self.body, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, LambdaTerm, A> {
    pub fn num_args(&self) -> u32 {
        self.as_value().num_args
    }
    pub fn body(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_value().body))
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.num_args() as usize, 0, false)
    }
}

impl<'heap, A: ArenaAllocator> LambdaTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, LambdaTerm, A>
{
    fn num_args<'a>(&'a self) -> StackOffset {
        self.num_args() as StackOffset
    }
    fn body<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        self.body().into()
    }
}

impl<'heap, A: ArenaAllocator> LambdaTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<LambdaTerm>, A>
{
    fn num_args<'a>(&'a self) -> StackOffset {
        <ArenaRef<'heap, LambdaTerm, A> as LambdaTermType<WasmExpression<'heap, A>>>::num_args(
            &self.as_inner(),
        )
    }
    fn body<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        <ArenaRef<'heap, LambdaTerm, A> as LambdaTermType<WasmExpression<'heap, A>>>::body(
            &self.as_inner(),
        )
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, LambdaTerm, A> {
    fn size(&self) -> usize {
        1 + self.body().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.body()
            .capture_depth()
            .saturating_sub(self.num_args() as StackOffset)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let num_args = self.num_args() as StackOffset;
        self.body()
            .free_variables()
            .into_iter()
            .filter_map(|offset| {
                if offset < num_args {
                    None
                } else {
                    Some(offset - num_args)
                }
            })
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.body()
            .count_variable_usages(offset + (self.num_args() as StackOffset))
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.body().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.body().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, LambdaTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, LambdaTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.num_args() == other.num_args() && self.body() == other.body()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, LambdaTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, LambdaTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, LambdaTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function:{}>", self.num_args())
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
