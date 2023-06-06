// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, DependencyList, Eagerness, Expression, GraphNode, Internable, LambdaTermType,
    SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaPointer, ArenaRef, Term,
};

use reflex_macros::PointerIter;

use super::WasmExpression;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct LambdaTerm {
    pub num_args: u32,
    pub body: ArenaPointer,
}
impl TermSize for LambdaTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LambdaTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.num_args, arena).hash(&self.body, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<LambdaTerm, A> {
    pub fn num_args(&self) -> u32 {
        self.read_value(|term| term.num_args)
    }
    pub fn body(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.body))
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.num_args() as usize, 0, false)
    }
}

impl<A: Arena + Clone> LambdaTermType<WasmExpression<A>> for ArenaRef<LambdaTerm, A> {
    fn num_args<'a>(&'a self) -> StackOffset {
        self.num_args() as StackOffset
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.body().into()
    }
}

impl<A: Arena + Clone> LambdaTermType<WasmExpression<A>> for ArenaRef<TypedTerm<LambdaTerm>, A> {
    fn num_args<'a>(&'a self) -> StackOffset {
        <ArenaRef<LambdaTerm, A> as LambdaTermType<WasmExpression<A>>>::num_args(&self.as_inner())
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LambdaTerm, A> as LambdaTermType<WasmExpression<A>>>::body(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<LambdaTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<LambdaTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<LambdaTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.num_args() == other.num_args() && self.body() == other.body()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<LambdaTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<LambdaTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<LambdaTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function:{}>", self.num_args())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<LambdaTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        // FIXME: Lambda terms should always be compiled
        self.capture_depth() == 0
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
                num_args: 0x54321,
                body: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Lambda as u32, 0x54321, 0x98765],
        );
    }
}
