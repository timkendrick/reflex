// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ArgType, Arity, DependencyList, Expression, GraphNode, PartialApplicationTermType,
    SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, Term, TermPointer,
};

use super::{ListTerm, WasmExpression};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PartialTerm {
    pub target: TermPointer,
    pub args: TermPointer,
}
impl TermSize for PartialTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PartialTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<PartialTerm, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.target))
    }
    pub fn args(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.args),
        )
    }
    pub fn arity(&self) -> Option<Arity> {
        self.target()
            .arity()
            .map(|arity| arity.partial(self.args().as_inner().len()))
    }
}

impl<A: ArenaAllocator + Clone> PartialApplicationTermType<WasmExpression<A>>
    for ArenaRef<PartialTerm, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.target().into()
    }
    fn args<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<A>: 'a,
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
    {
        self.args().into()
    }
}

impl<A: ArenaAllocator + Clone> PartialApplicationTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<PartialTerm>, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<PartialTerm, A> as PartialApplicationTermType<WasmExpression<A>>>::target(
            &self.as_inner(),
        )
    }
    fn args<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<A>: 'a,
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
    {
        <ArenaRef<PartialTerm, A> as PartialApplicationTermType<WasmExpression<A>>>::args(
            &self.as_inner(),
        )
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<PartialTerm, A> {
    fn size(&self) -> usize {
        1 + self.target().size() + self.args().size()
    }
    fn capture_depth(&self) -> StackOffset {
        let target_depth = self.target().capture_depth();
        let arg_depth = self.args().capture_depth();
        target_depth.max(arg_depth)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let target_free_variables = self.target().free_variables();
        let args_free_variables = self.args().free_variables();
        if target_free_variables.is_empty() {
            args_free_variables
        } else if args_free_variables.is_empty() {
            target_free_variables
        } else {
            let mut combined = target_free_variables;
            combined.extend(args_free_variables);
            combined
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.target().count_variable_usages(offset) + self.args().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        let target_dependencies = self.target().dynamic_dependencies(deep);
        if deep {
            target_dependencies.union(self.args().dynamic_dependencies(deep))
        } else {
            match self.target().arity() {
                None => target_dependencies,
                Some(arity) => get_eager_args(self.args().as_inner().iter(), &arity).fold(
                    target_dependencies,
                    |combined_dependencies, arg| {
                        combined_dependencies.union(arg.dynamic_dependencies(deep))
                    },
                ),
            }
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.target().has_dynamic_dependencies(deep)
            || (if deep {
                self.args().has_dynamic_dependencies(deep)
            } else {
                match self.target().arity() {
                    None => false,
                    Some(arity) => get_eager_args(self.args().as_inner().iter(), &arity)
                        .any(|arg| arg.has_dynamic_dependencies(deep)),
                }
            })
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        self.target().is_atomic() && self.args().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<PartialTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<PartialTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<PartialTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<PartialTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<PartialTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<partial:{}:{}>",
            self.args().as_inner().len(),
            self.target()
        )
    }
}

fn get_eager_args<T>(args: impl IntoIterator<Item = T>, arity: &Arity) -> impl Iterator<Item = T> {
    arity
        .iter()
        .zip(args)
        .filter_map(|(arg_type, arg)| match arg_type {
            ArgType::Strict | ArgType::Eager => Some(arg),
            ArgType::Lazy => None,
        })
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn partial() {
        assert_eq!(
            TermType::Partial(PartialTerm {
                target: TermPointer(12345),
                args: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Partial as u32, 12345, 67890],
        );
    }
}
