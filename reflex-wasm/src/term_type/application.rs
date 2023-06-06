// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ApplicationTermType, ArgType, Arity, DependencyList, Eagerness, Expression, GraphNode,
    Internable, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHashState, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaPointer, ArenaRef, PointerIter, Term,
};

use reflex_macros::PointerIter;

use super::{ListTerm, TreeTerm, WasmExpression};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ApplicationTerm {
    pub target: ArenaPointer,
    pub args: ArenaPointer,
    pub cache: ApplicationCache,
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct ApplicationCache {
    pub value: ArenaPointer,
    pub dependencies: ArenaPointer,
    pub overall_state_hash: u32,
    pub minimal_state_hash: u32,
}

pub type ApplicationTermPointerIter =
    std::iter::Chain<std::array::IntoIter<ArenaPointer, 2>, ApplicationCachePointerIter>;

impl<A: Arena + Clone> PointerIter for ArenaRef<ApplicationTerm, A> {
    type Iter<'a> = ApplicationTermPointerIter
    where
        Self: 'a;
    fn iter<'a>(&self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        let pointers = [
            self.inner_pointer(|term| &term.target),
            self.inner_pointer(|term| &term.args),
        ];
        let cache = self.inner_ref::<ApplicationCache>(|term| &term.cache);
        let cache_pointers: ApplicationCachePointerIter = PointerIter::iter(&cache);
        pointers.into_iter().chain(cache_pointers)
    }
}

impl TermSize for ApplicationTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ApplicationTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

impl TermSize for ApplicationCache {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Default for ApplicationCache {
    fn default() -> Self {
        Self {
            value: ArenaPointer::null(),
            dependencies: ArenaPointer::null(),
            overall_state_hash: u32::from(ArenaPointer::null()),
            minimal_state_hash: u32::from(ArenaPointer::null()),
        }
    }
}

impl<A: Arena + Clone> ArenaRef<ApplicationTerm, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|value| value.target))
    }
    pub fn args(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|value| value.args),
        )
    }
    pub fn cache(&self) -> ArenaRef<ApplicationCache, A> {
        self.inner_ref(|value| &value.cache)
    }
}

impl<A: Arena + Clone> ArenaRef<ApplicationCache, A> {
    pub fn value(&self) -> Option<ArenaRef<Term, A>> {
        let pointer = self.read_value(|value| value.value).as_non_null()?;
        Some(ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
    }
    pub fn dependencies(&self) -> Option<ArenaRef<TypedTerm<TreeTerm>, A>> {
        let pointer = self.read_value(|value| value.dependencies).as_non_null()?;
        Some(ArenaRef::<TypedTerm<TreeTerm>, _>::new(
            self.arena.clone(),
            pointer,
        ))
    }
    pub fn overall_state_hash(&self) -> Option<TermHashState> {
        let value = self.read_value(|value| value.overall_state_hash);
        if value == u32::from(ArenaPointer::null()) {
            None
        } else {
            Some(TermHashState::from(value))
        }
    }
    pub fn minimal_state_hash(&self) -> Option<TermHashState> {
        let value = self.read_value(|value| value.minimal_state_hash);
        if value == u32::from(ArenaPointer::null()) {
            None
        } else {
            Some(TermHashState::from(value))
        }
    }
}

impl<A: Arena + Clone> ApplicationTermType<WasmExpression<A>> for ArenaRef<ApplicationTerm, A> {
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

impl<A: Arena + Clone> ApplicationTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<ApplicationTerm>, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<ApplicationTerm, A> as ApplicationTermType<WasmExpression<A>>>::target(
            &self.as_inner(),
        )
    }
    fn args<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<A>: 'a,
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
    {
        <ArenaRef<ApplicationTerm, A> as ApplicationTermType<WasmExpression<A>>>::args(
            &self.as_inner(),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<ApplicationTerm, A> {
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
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<ApplicationTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!("Unable to patch terms: {}, {}", self, target))
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<ApplicationTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<ApplicationTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<ApplicationTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<ApplicationTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<apply:{}:({})>",
            self.target(),
            self.args()
                .as_inner()
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<_>>()
                .join(", ")
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

impl<A: Arena + Clone> Internable for ArenaRef<ApplicationTerm, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        eager == Eagerness::Lazy
            && self.target().capture_depth() == 0
            && self.args().capture_depth() == 0
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
                target: ArenaPointer(0x54321),
                args: ArenaPointer(0x98765),
                cache: ApplicationCache {
                    value: ArenaPointer::null(),
                    dependencies: ArenaPointer::null(),
                    overall_state_hash: u32::from(ArenaPointer::null()),
                    minimal_state_hash: u32::from(ArenaPointer::null()),
                },
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::Application as u32,
                0x54321,
                0x98765,
                0xFFFFFFFF,
                0xFFFFFFFF,
                0xFFFFFFFF,
                0xFFFFFFFF,
            ],
        );
    }
}
