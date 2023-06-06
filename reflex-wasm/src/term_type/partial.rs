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

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, PartialTerm, A> {
    pub fn target(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_value().target))
    }
    pub fn args(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_value().args))
    }
    pub fn arity(&self) -> Option<Arity> {
        self.target()
            .arity()
            .map(|arity| arity.partial(self.args().as_inner().len()))
    }
}

impl<'heap, A: ArenaAllocator> PartialApplicationTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, PartialTerm, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        self.target().into()
    }
    fn args<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<'heap, A>: 'a,
        <WasmExpression<'heap, A> as Expression>::ExpressionList: 'a,
    {
        self.args().into()
    }
}

impl<'heap, A: ArenaAllocator> PartialApplicationTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<PartialTerm>, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<'heap, A>: 'a,
    {
        <ArenaRef<'heap, PartialTerm, A> as PartialApplicationTermType<WasmExpression<'heap, A>>>::target(&self.as_inner())
    }
    fn args<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<'heap, A>: 'a,
        <WasmExpression<'heap, A> as Expression>::ExpressionList: 'a,
    {
        <ArenaRef<'heap, PartialTerm, A> as PartialApplicationTermType<WasmExpression<'heap, A>>>::args(
            &self.as_inner(),
        )
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, PartialTerm, A> {
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
            let eager_args = self
                .target()
                .arity()
                .map(|arity| get_eager_args(self.args().as_inner().iter(), &arity));
            match eager_args {
                None => target_dependencies,
                Some(args) => args.fold(target_dependencies, |combined_dependencies, arg| {
                    combined_dependencies.union(arg.dynamic_dependencies(deep))
                }),
            }
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.target().has_dynamic_dependencies(deep)
            || (if deep {
                self.args().has_dynamic_dependencies(deep)
            } else {
                let eager_args = self
                    .target()
                    .arity()
                    .map(|arity| get_eager_args(self.args().as_inner().iter(), &arity));
                match eager_args {
                    None => false,
                    Some(mut args) => args.any(|arg| arg.has_dynamic_dependencies(deep)),
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, PartialTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, PartialTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, PartialTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, PartialTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, PartialTerm, A> {
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
