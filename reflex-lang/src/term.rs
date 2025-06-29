// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{collections::HashSet, hash::Hash};

use reflex::{core::NodeId, hash::hash_object};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{ExpressionList, Signal, SignalList, StructPrototype};

mod application;
mod boolean;
mod builtin;
mod compiled;
mod constructor;
mod effect;
mod float;
mod hashmap;
mod hashset;
mod int;
mod lambda;
mod lazy_result;
mod r#let;
mod list;
mod nil;
mod partial;
mod record;
mod recursive;
mod signal;
mod string;
mod symbol;
mod timestamp;
mod variable;

pub use application::*;
pub use boolean::*;
pub use builtin::*;
pub use compiled::*;
pub use constructor::*;
pub use effect::*;
pub use float::*;
pub use hashmap::*;
pub use hashset::*;
pub use int::*;
pub use lambda::*;
pub use lazy_result::*;
pub use list::*;
pub use nil::*;
pub use partial::*;
pub use r#let::*;
pub use record::*;
pub use recursive::*;
pub use signal::*;
pub use string::*;
pub use symbol::*;
pub use timestamp::*;
pub use variable::*;

use reflex::{
    core::{
        Applicable, Arity, CompoundNode, DependencyList, DynamicState, Evaluate, EvaluationCache,
        EvaluationResult, Expression, ExpressionFactory, GraphNode, HeapAllocator, Reducible,
        Rewritable, SerializeJson, StackOffset, Substitutions,
    },
    hash::HashId,
};

#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Term<T: Expression>
where
    T::String: Hash,
{
    #[serde(bound(
        serialize = "<T as Expression>::String: Serialize, <T as Expression>::ExpressionList: Serialize, <T as Expression>::SignalList: Serialize, <T as Expression>::Signal: Serialize, <T as Expression>::StructPrototype: Serialize",
        deserialize = "<T as Expression>::String: Deserialize<'de>, <T as Expression>::ExpressionList: Deserialize<'de>, <T as Expression>::SignalList: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>, <T as Expression>::StructPrototype: Deserialize<'de>"
    ))]
    Nil(NilTerm),
    Boolean(BooleanTerm),
    Int(IntTerm),
    Float(FloatTerm),
    String(StringTerm<T>),
    Symbol(SymbolTerm),
    Timestamp(TimestampTerm),
    Variable(VariableTerm),
    Effect(EffectTerm<T>),
    Let(LetTerm<T>),
    Lambda(LambdaTerm<T>),
    LazyResult(LazyResultTerm<T>),
    Application(ApplicationTerm<T>),
    PartialApplication(PartialApplicationTerm<T>),
    Recursive(RecursiveTerm<T>),
    Builtin(BuiltinTerm<T>),
    CompiledFunction(CompiledFunctionTerm),
    Record(RecordTerm<T>),
    Constructor(ConstructorTerm<T>),
    List(ListTerm<T>),
    HashMap(HashMapTerm<T>),
    HashSet(HashSetTerm<T>),
    Signal(SignalTerm<T>),
}

impl<T: Expression + Applicable<T>> Expression for Term<T>
where
    T: Hash,
    T::String: Hash,
{
    type String = T::String;
    type Builtin = T::Builtin;
    type Signal = Signal<Self>;
    type SignalList = SignalList<Self>;
    type StructPrototype = StructPrototype<Self>;
    type ExpressionList = ExpressionList<Self>;
    type NilTerm = NilTerm;
    type BooleanTerm = BooleanTerm;
    type IntTerm = IntTerm;
    type FloatTerm = FloatTerm;
    type StringTerm = StringTerm<Self>;
    type SymbolTerm = SymbolTerm;
    type TimestampTerm = TimestampTerm;
    type VariableTerm = VariableTerm;
    type EffectTerm = EffectTerm<Self>;
    type LetTerm = LetTerm<Self>;
    type LambdaTerm = LambdaTerm<Self>;
    type LazyResultTerm = LazyResultTerm<Self>;
    type ApplicationTerm = ApplicationTerm<Self>;
    type PartialApplicationTerm = PartialApplicationTerm<Self>;
    type RecursiveTerm = RecursiveTerm<Self>;
    type BuiltinTerm = BuiltinTerm<Self>;
    type CompiledFunctionTerm = CompiledFunctionTerm;
    type RecordTerm = RecordTerm<Self>;
    type ConstructorTerm = ConstructorTerm<Self>;
    type ListTerm = ListTerm<Self>;
    type HashmapTerm = HashMapTerm<Self>;
    type HashsetTerm = HashSetTerm<Self>;
    type SignalTerm = SignalTerm<Self>;

    type StringRef<'a> = &'a <Self as Expression>::String
    where
        <Self as Expression>::String: 'a,
        Self: 'a;

    type SignalRef<'a> = &'a <Self as Expression>::Signal
    where
        <Self as Expression>::Signal: 'a,
        Self: 'a;

    type StructPrototypeRef<'a> = &'a <Self as Expression>::StructPrototype
    where
        <Self as Expression>::StructPrototype: 'a,
        Self: 'a;

    type SignalListRef<'a> = &'a <Self as Expression>::SignalList
    where
        <Self as Expression>::SignalList: 'a,
        Self: 'a;

    type ExpressionListRef<'a> = &'a <Self as Expression>::ExpressionList
    where
        <Self as Expression>::ExpressionList: 'a,
        Self: 'a;

    type ExpressionRef<'a> = &'a Self
    where
        Self: 'a;
}
impl<T: Expression + Applicable<T>> NodeId for Term<T>
where
    T: Hash,
    T::String: Hash,
{
    fn id(&self) -> HashId {
        hash_object(self)
    }
}
impl<T: Expression + Applicable<T>> GraphNode for Term<T>
where
    T::String: Hash,
{
    fn size(&self) -> usize {
        match self {
            Self::Nil(term) => term.size(),
            Self::Boolean(term) => term.size(),
            Self::Int(term) => term.size(),
            Self::Float(term) => term.size(),
            Self::String(term) => term.size(),
            Self::Symbol(term) => term.size(),
            Self::Timestamp(term) => term.size(),
            Self::Variable(term) => term.size(),
            Self::Effect(term) => term.size(),
            Self::Let(term) => term.size(),
            Self::Lambda(term) => term.size(),
            Self::LazyResult(term) => term.size(),
            Self::Application(term) => term.size(),
            Self::PartialApplication(term) => term.size(),
            Self::Recursive(term) => term.size(),
            Self::Builtin(term) => term.size(),
            Self::CompiledFunction(term) => term.size(),
            Self::Record(term) => term.size(),
            Self::Constructor(term) => term.size(),
            Self::List(term) => term.size(),
            Self::HashMap(term) => term.size(),
            Self::HashSet(term) => term.size(),
            Self::Signal(term) => term.size(),
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match self {
            Self::Nil(term) => term.capture_depth(),
            Self::Boolean(term) => term.capture_depth(),
            Self::Int(term) => term.capture_depth(),
            Self::Float(term) => term.capture_depth(),
            Self::String(term) => term.capture_depth(),
            Self::Symbol(term) => term.capture_depth(),
            Self::Timestamp(term) => term.capture_depth(),
            Self::Variable(term) => term.capture_depth(),
            Self::Effect(term) => term.capture_depth(),
            Self::Let(term) => term.capture_depth(),
            Self::Lambda(term) => term.capture_depth(),
            Self::LazyResult(term) => term.capture_depth(),
            Self::Application(term) => term.capture_depth(),
            Self::PartialApplication(term) => term.capture_depth(),
            Self::Recursive(term) => term.capture_depth(),
            Self::Builtin(term) => term.capture_depth(),
            Self::CompiledFunction(term) => term.capture_depth(),
            Self::Record(term) => term.capture_depth(),
            Self::Constructor(term) => term.capture_depth(),
            Self::List(term) => term.capture_depth(),
            Self::HashMap(term) => term.capture_depth(),
            Self::HashSet(term) => term.capture_depth(),
            Self::Signal(term) => term.capture_depth(),
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match self {
            Self::Nil(term) => term.free_variables(),
            Self::Boolean(term) => term.free_variables(),
            Self::Int(term) => term.free_variables(),
            Self::Float(term) => term.free_variables(),
            Self::String(term) => term.free_variables(),
            Self::Symbol(term) => term.free_variables(),
            Self::Timestamp(term) => term.free_variables(),
            Self::Variable(term) => term.free_variables(),
            Self::Effect(term) => term.free_variables(),
            Self::Let(term) => term.free_variables(),
            Self::Lambda(term) => term.free_variables(),
            Self::LazyResult(term) => term.free_variables(),
            Self::Application(term) => term.free_variables(),
            Self::PartialApplication(term) => term.free_variables(),
            Self::Recursive(term) => term.free_variables(),
            Self::Builtin(term) => term.free_variables(),
            Self::CompiledFunction(term) => term.free_variables(),
            Self::Record(term) => term.free_variables(),
            Self::Constructor(term) => term.free_variables(),
            Self::List(term) => term.free_variables(),
            Self::HashMap(term) => term.free_variables(),
            Self::HashSet(term) => term.free_variables(),
            Self::Signal(term) => term.free_variables(),
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match self {
            Self::Nil(term) => term.count_variable_usages(offset),
            Self::Boolean(term) => term.count_variable_usages(offset),
            Self::Int(term) => term.count_variable_usages(offset),
            Self::Float(term) => term.count_variable_usages(offset),
            Self::String(term) => term.count_variable_usages(offset),
            Self::Symbol(term) => term.count_variable_usages(offset),
            Self::Timestamp(term) => term.count_variable_usages(offset),
            Self::Variable(term) => term.count_variable_usages(offset),
            Self::Effect(term) => term.count_variable_usages(offset),
            Self::Let(term) => term.count_variable_usages(offset),
            Self::Lambda(term) => term.count_variable_usages(offset),
            Self::LazyResult(term) => term.count_variable_usages(offset),
            Self::Application(term) => term.count_variable_usages(offset),
            Self::PartialApplication(term) => term.count_variable_usages(offset),
            Self::Recursive(term) => term.count_variable_usages(offset),
            Self::Builtin(term) => term.count_variable_usages(offset),
            Self::CompiledFunction(term) => term.count_variable_usages(offset),
            Self::Record(term) => term.count_variable_usages(offset),
            Self::Constructor(term) => term.count_variable_usages(offset),
            Self::List(term) => term.count_variable_usages(offset),
            Self::HashMap(term) => term.count_variable_usages(offset),
            Self::HashSet(term) => term.count_variable_usages(offset),
            Self::Signal(term) => term.count_variable_usages(offset),
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match self {
            Self::Nil(term) => term.dynamic_dependencies(deep),
            Self::Boolean(term) => term.dynamic_dependencies(deep),
            Self::Int(term) => term.dynamic_dependencies(deep),
            Self::Float(term) => term.dynamic_dependencies(deep),
            Self::String(term) => term.dynamic_dependencies(deep),
            Self::Symbol(term) => term.dynamic_dependencies(deep),
            Self::Timestamp(term) => term.dynamic_dependencies(deep),
            Self::Variable(term) => term.dynamic_dependencies(deep),
            Self::Effect(term) => term.dynamic_dependencies(deep),
            Self::Let(term) => term.dynamic_dependencies(deep),
            Self::Lambda(term) => term.dynamic_dependencies(deep),
            Self::LazyResult(term) => term.dynamic_dependencies(deep),
            Self::Application(term) => term.dynamic_dependencies(deep),
            Self::PartialApplication(term) => term.dynamic_dependencies(deep),
            Self::Recursive(term) => term.dynamic_dependencies(deep),
            Self::Builtin(term) => term.dynamic_dependencies(deep),
            Self::CompiledFunction(term) => term.dynamic_dependencies(deep),
            Self::Record(term) => term.dynamic_dependencies(deep),
            Self::Constructor(term) => term.dynamic_dependencies(deep),
            Self::List(term) => term.dynamic_dependencies(deep),
            Self::HashMap(term) => term.dynamic_dependencies(deep),
            Self::HashSet(term) => term.dynamic_dependencies(deep),
            Self::Signal(term) => term.dynamic_dependencies(deep),
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match self {
            Self::Nil(term) => term.has_dynamic_dependencies(deep),
            Self::Boolean(term) => term.has_dynamic_dependencies(deep),
            Self::Int(term) => term.has_dynamic_dependencies(deep),
            Self::Float(term) => term.has_dynamic_dependencies(deep),
            Self::String(term) => term.has_dynamic_dependencies(deep),
            Self::Symbol(term) => term.has_dynamic_dependencies(deep),
            Self::Timestamp(term) => term.has_dynamic_dependencies(deep),
            Self::Variable(term) => term.has_dynamic_dependencies(deep),
            Self::Effect(term) => term.has_dynamic_dependencies(deep),
            Self::Let(term) => term.has_dynamic_dependencies(deep),
            Self::Lambda(term) => term.has_dynamic_dependencies(deep),
            Self::LazyResult(term) => term.has_dynamic_dependencies(deep),
            Self::Application(term) => term.has_dynamic_dependencies(deep),
            Self::PartialApplication(term) => term.has_dynamic_dependencies(deep),
            Self::Recursive(term) => term.has_dynamic_dependencies(deep),
            Self::Builtin(term) => term.has_dynamic_dependencies(deep),
            Self::CompiledFunction(term) => term.has_dynamic_dependencies(deep),
            Self::Record(term) => term.has_dynamic_dependencies(deep),
            Self::Constructor(term) => term.has_dynamic_dependencies(deep),
            Self::List(term) => term.has_dynamic_dependencies(deep),
            Self::HashMap(term) => term.has_dynamic_dependencies(deep),
            Self::HashSet(term) => term.has_dynamic_dependencies(deep),
            Self::Signal(term) => term.has_dynamic_dependencies(deep),
        }
    }
    fn is_static(&self) -> bool {
        match self {
            Self::Nil(term) => term.is_static(),
            Self::Boolean(term) => term.is_static(),
            Self::Int(term) => term.is_static(),
            Self::Float(term) => term.is_static(),
            Self::String(term) => term.is_static(),
            Self::Symbol(term) => term.is_static(),
            Self::Timestamp(term) => term.is_static(),
            Self::Variable(term) => term.is_static(),
            Self::Effect(term) => term.is_static(),
            Self::Let(term) => term.is_static(),
            Self::Lambda(term) => term.is_static(),
            Self::LazyResult(term) => term.is_static(),
            Self::Application(term) => term.is_static(),
            Self::PartialApplication(term) => term.is_static(),
            Self::Recursive(term) => term.is_static(),
            Self::Builtin(term) => term.is_static(),
            Self::CompiledFunction(term) => term.is_static(),
            Self::Record(term) => term.is_static(),
            Self::Constructor(term) => term.is_static(),
            Self::List(term) => term.is_static(),
            Self::HashMap(term) => term.is_static(),
            Self::HashSet(term) => term.is_static(),
            Self::Signal(term) => term.is_static(),
        }
    }
    fn is_atomic(&self) -> bool {
        match self {
            Self::Nil(term) => term.is_atomic(),
            Self::Boolean(term) => term.is_atomic(),
            Self::Int(term) => term.is_atomic(),
            Self::Float(term) => term.is_atomic(),
            Self::String(term) => term.is_atomic(),
            Self::Symbol(term) => term.is_atomic(),
            Self::Timestamp(term) => term.is_atomic(),
            Self::Variable(term) => term.is_atomic(),
            Self::Effect(term) => term.is_atomic(),
            Self::Let(term) => term.is_atomic(),
            Self::Lambda(term) => term.is_atomic(),
            Self::LazyResult(term) => term.is_atomic(),
            Self::Application(term) => term.is_atomic(),
            Self::PartialApplication(term) => term.is_atomic(),
            Self::Recursive(term) => term.is_atomic(),
            Self::Builtin(term) => term.is_atomic(),
            Self::CompiledFunction(term) => term.is_atomic(),
            Self::Record(term) => term.is_atomic(),
            Self::Constructor(term) => term.is_atomic(),
            Self::List(term) => term.is_atomic(),
            Self::HashMap(term) => term.is_atomic(),
            Self::HashSet(term) => term.is_atomic(),
            Self::Signal(term) => term.is_atomic(),
        }
    }
    fn is_complex(&self) -> bool {
        match self {
            Self::Nil(term) => term.is_complex(),
            Self::Boolean(term) => term.is_complex(),
            Self::Int(term) => term.is_complex(),
            Self::Float(term) => term.is_complex(),
            Self::String(term) => term.is_complex(),
            Self::Symbol(term) => term.is_complex(),
            Self::Timestamp(term) => term.is_complex(),
            Self::Variable(term) => term.is_complex(),
            Self::Effect(term) => term.is_complex(),
            Self::Let(term) => term.is_complex(),
            Self::Lambda(term) => term.is_complex(),
            Self::LazyResult(term) => term.is_complex(),
            Self::Application(term) => term.is_complex(),
            Self::PartialApplication(term) => term.is_complex(),
            Self::Recursive(term) => term.is_complex(),
            Self::Builtin(term) => term.is_complex(),
            Self::CompiledFunction(term) => term.is_complex(),
            Self::Record(term) => term.is_complex(),
            Self::Constructor(term) => term.is_complex(),
            Self::List(term) => term.is_complex(),
            Self::HashMap(term) => term.is_complex(),
            Self::HashSet(term) => term.is_complex(),
            Self::Signal(term) => term.is_complex(),
        }
    }
}
pub enum TermChildren<'a, T: Expression + 'a> {
    Let(<LetTerm<T> as CompoundNode<T>>::Children<'a>),
    Lambda(<LambdaTerm<T> as CompoundNode<T>>::Children<'a>),
    LazyResult(<LazyResultTerm<T> as CompoundNode<T>>::Children<'a>),
    Application(<ApplicationTerm<T> as CompoundNode<T>>::Children<'a>),
    PartialApplication(<PartialApplicationTerm<T> as CompoundNode<T>>::Children<'a>),
    Recursive(<RecursiveTerm<T> as CompoundNode<T>>::Children<'a>),
    Record(<RecordTerm<T> as CompoundNode<T>>::Children<'a>),
    List(<ListTerm<T> as CompoundNode<T>>::Children<'a>),
    HashMap(<HashMapTerm<T> as CompoundNode<T>>::Children<'a>),
    HashSet(<HashSetTerm<T> as CompoundNode<T>>::Children<'a>),
    Empty,
}
impl<'a, T: Expression + 'a> Iterator for TermChildren<'a, T> {
    type Item = T::ExpressionRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Let(iter) => iter.next(),
            Self::Lambda(iter) => iter.next(),
            Self::LazyResult(iter) => iter.next(),
            Self::Application(iter) => iter.next(),
            Self::PartialApplication(iter) => iter.next(),
            Self::Recursive(iter) => iter.next(),
            Self::Record(iter) => iter.next(),
            Self::List(iter) => iter.next(),
            Self::HashMap(iter) => iter.next(),
            Self::HashSet(iter) => iter.next(),
            Self::Empty => None,
        }
    }
}
impl<T: Expression> CompoundNode<T> for Term<T>
where
    T::String: Hash,
    for<'a> T::ExpressionRef<'a>: From<&'a T>,
{
    type Children<'a> = TermChildren<'a, T>
        where
            T: 'a,
            Self: 'a;
    fn children<'a>(&'a self) -> Self::Children<'a>
    where
        T: 'a,
    {
        match self {
            Self::Let(term) => TermChildren::Let(term.children()),
            Self::Lambda(term) => TermChildren::Lambda(term.children()),
            Self::LazyResult(term) => TermChildren::LazyResult(term.children()),
            Self::Application(term) => TermChildren::Application(term.children()),
            Self::PartialApplication(term) => TermChildren::PartialApplication(term.children()),
            Self::Recursive(term) => TermChildren::Recursive(term.children()),
            Self::Record(term) => TermChildren::Record(term.children()),
            Self::List(term) => TermChildren::List(term.children()),
            Self::HashMap(term) => TermChildren::HashMap(term.children()),
            Self::HashSet(term) => TermChildren::HashSet(term.children()),
            _ => TermChildren::Empty,
        }
    }
}
impl<T: Expression + Rewritable<T> + Reducible<T> + Applicable<T> + Evaluate<T>> Rewritable<T>
    for Term<T>
where
    T::String: Hash,
{
    fn substitute_static(
        &self,
        substitutions: &Substitutions<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        match self {
            Self::Variable(term) => {
                term.substitute_static(substitutions, factory, allocator, cache)
            }
            Self::Effect(term) => term.substitute_static(substitutions, factory, allocator, cache),
            Self::Let(term) => term.substitute_static(substitutions, factory, allocator, cache),
            Self::Lambda(term) => term.substitute_static(substitutions, factory, allocator, cache),
            Self::LazyResult(term) => {
                term.substitute_static(substitutions, factory, allocator, cache)
            }
            Self::Application(term) => {
                term.substitute_static(substitutions, factory, allocator, cache)
            }
            Self::PartialApplication(term) => {
                term.substitute_static(substitutions, factory, allocator, cache)
            }
            Self::Recursive(term) => {
                term.substitute_static(substitutions, factory, allocator, cache)
            }
            Self::Record(term) => term.substitute_static(substitutions, factory, allocator, cache),
            Self::List(term) => term.substitute_static(substitutions, factory, allocator, cache),
            Self::HashMap(term) => term.substitute_static(substitutions, factory, allocator, cache),
            Self::HashSet(term) => term.substitute_static(substitutions, factory, allocator, cache),
            _ => None,
        }
    }
    fn substitute_dynamic(
        &self,
        deep: bool,
        state: &impl DynamicState<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        match self {
            Self::Variable(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::Effect(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::Let(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::Lambda(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::LazyResult(term) => {
                term.substitute_dynamic(deep, state, factory, allocator, cache)
            }
            Self::Application(term) => {
                term.substitute_dynamic(deep, state, factory, allocator, cache)
            }
            Self::PartialApplication(term) => {
                term.substitute_dynamic(deep, state, factory, allocator, cache)
            }
            Self::Recursive(term) => {
                term.substitute_dynamic(deep, state, factory, allocator, cache)
            }
            Self::Record(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::List(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::HashMap(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            Self::HashSet(term) => term.substitute_dynamic(deep, state, factory, allocator, cache),
            _ => None,
        }
    }
    fn normalize(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        match self {
            Self::Variable(term) => term.normalize(factory, allocator, cache),
            Self::Effect(term) => term.normalize(factory, allocator, cache),
            Self::Let(term) => term.normalize(factory, allocator, cache),
            Self::Lambda(term) => term.normalize(factory, allocator, cache),
            Self::LazyResult(term) => term.normalize(factory, allocator, cache),
            Self::Application(term) => term.normalize(factory, allocator, cache),
            Self::PartialApplication(term) => term.normalize(factory, allocator, cache),
            Self::Recursive(term) => term.normalize(factory, allocator, cache),
            Self::Record(term) => term.normalize(factory, allocator, cache),
            Self::List(term) => term.normalize(factory, allocator, cache),
            Self::HashMap(term) => term.normalize(factory, allocator, cache),
            Self::HashSet(term) => term.normalize(factory, allocator, cache),
            _ => None,
        }
    }
    fn hoist_free_variables(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
    ) -> Option<T> {
        match self {
            Self::Variable(term) => term.hoist_free_variables(factory, allocator),
            Self::Effect(term) => term.hoist_free_variables(factory, allocator),
            Self::Let(term) => term.hoist_free_variables(factory, allocator),
            Self::Lambda(term) => term.hoist_free_variables(factory, allocator),
            Self::LazyResult(term) => term.hoist_free_variables(factory, allocator),
            Self::Application(term) => term.hoist_free_variables(factory, allocator),
            Self::PartialApplication(term) => term.hoist_free_variables(factory, allocator),
            Self::Recursive(term) => term.hoist_free_variables(factory, allocator),
            Self::Record(term) => term.hoist_free_variables(factory, allocator),
            Self::List(term) => term.hoist_free_variables(factory, allocator),
            Self::HashMap(term) => term.hoist_free_variables(factory, allocator),
            Self::HashSet(term) => term.hoist_free_variables(factory, allocator),
            _ => None,
        }
    }
}

impl<T: Expression + Rewritable<T> + Reducible<T> + Applicable<T> + Evaluate<T>> Reducible<T>
    for Term<T>
where
    T::String: Hash,
{
    fn is_reducible(&self) -> bool {
        match self {
            Self::Let(term) => term.is_reducible(),
            Self::Application(term) => term.is_reducible(),
            Self::Recursive(term) => term.is_reducible(),
            _ => false,
        }
    }
    fn reduce(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        match self {
            Self::Let(term) => term.reduce(factory, allocator, cache),
            Self::Application(term) => term.reduce(factory, allocator, cache),
            Self::Recursive(term) => term.reduce(factory, allocator, cache),
            _ => None,
        }
    }
}
impl<T: Expression + Rewritable<T> + Applicable<T>> Applicable<T> for Term<T>
where
    T::String: Hash,
{
    fn arity(&self) -> Option<Arity> {
        match self {
            Self::Lambda(term) => Applicable::<T>::arity(term),
            Self::PartialApplication(term) => Applicable::<T>::arity(term),
            Self::Builtin(term) => Applicable::<T>::arity(term),
            Self::CompiledFunction(term) => Applicable::<T>::arity(term),
            Self::Constructor(term) => Applicable::<T>::arity(term),
            _ => None,
        }
    }
    fn apply(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        match self {
            Self::Lambda(term) => term.apply(args, factory, allocator, cache),
            Self::PartialApplication(term) => term.apply(args, factory, allocator, cache),
            Self::Builtin(term) => Applicable::apply(term, args, factory, allocator, cache),
            Self::CompiledFunction(term) => term.apply(args, factory, allocator, cache),
            Self::Constructor(term) => term.apply(args, factory, allocator, cache),
            _ => Err(format!("Invalid function application target: {}", self)),
        }
    }
    fn should_parallelize(&self, args: &[T]) -> bool {
        match self {
            Self::Lambda(term) => term.should_parallelize(args),
            Self::PartialApplication(term) => term.should_parallelize(args),
            Self::Builtin(term) => term.should_parallelize(args),
            Self::CompiledFunction(term) => term.should_parallelize(args),
            Self::Constructor(term) => term.should_parallelize(args),
            _ => false,
        }
    }
}
impl<T: Expression + Rewritable<T> + Reducible<T> + Applicable<T> + Evaluate<T>> Evaluate<T>
    for Term<T>
where
    T::String: Hash,
{
    fn evaluate(
        &self,
        state: &impl DynamicState<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<EvaluationResult<T>> {
        match self {
            Self::Effect(term) => term.evaluate(state, factory, allocator, cache),
            Self::LazyResult(term) => term.evaluate(state, factory, allocator, cache),
            Self::Application(term) => term.evaluate(state, factory, allocator, cache),
            _ => {
                if self.is_reducible() {
                    self.reduce(factory, allocator, cache).map(|result| {
                        result
                            .evaluate(state, factory, allocator, cache)
                            .unwrap_or_else(|| {
                                EvaluationResult::new(result, DependencyList::empty())
                            })
                    })
                } else {
                    None
                }
            }
        }
    }
}

impl<T: Expression> std::fmt::Display for Term<T>
where
    T::String: Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil(term) => std::fmt::Display::fmt(term, f),
            Self::Boolean(term) => std::fmt::Display::fmt(term, f),
            Self::Int(term) => std::fmt::Display::fmt(term, f),
            Self::Float(term) => std::fmt::Display::fmt(term, f),
            Self::String(term) => std::fmt::Display::fmt(term, f),
            Self::Symbol(term) => std::fmt::Display::fmt(term, f),
            Self::Timestamp(term) => std::fmt::Display::fmt(term, f),
            Self::Variable(term) => std::fmt::Display::fmt(term, f),
            Self::Effect(term) => std::fmt::Display::fmt(term, f),
            Self::Let(term) => std::fmt::Display::fmt(term, f),
            Self::Lambda(term) => std::fmt::Display::fmt(term, f),
            Self::LazyResult(term) => std::fmt::Display::fmt(term, f),
            Self::Application(term) => std::fmt::Display::fmt(term, f),
            Self::PartialApplication(term) => std::fmt::Display::fmt(term, f),
            Self::Recursive(term) => std::fmt::Display::fmt(term, f),
            Self::CompiledFunction(term) => std::fmt::Display::fmt(term, f),
            Self::Builtin(term) => std::fmt::Display::fmt(term, f),
            Self::Record(term) => std::fmt::Display::fmt(term, f),
            Self::Constructor(term) => std::fmt::Display::fmt(term, f),
            Self::List(term) => std::fmt::Display::fmt(term, f),
            Self::HashMap(term) => std::fmt::Display::fmt(term, f),
            Self::HashSet(term) => std::fmt::Display::fmt(term, f),
            Self::Signal(term) => std::fmt::Display::fmt(term, f),
        }
    }
}

impl<T: Expression + Applicable<T> + Hash> SerializeJson for Term<T>
where
    T::String: Hash,
{
    fn to_json(&self) -> Result<JsonValue, String> {
        match self {
            Self::Nil(term) => term.to_json(),
            Self::Boolean(term) => term.to_json(),
            Self::Int(term) => term.to_json(),
            Self::Float(term) => term.to_json(),
            Self::String(term) => term.to_json(),
            Self::Symbol(term) => term.to_json(),
            Self::Timestamp(term) => term.to_json(),
            Self::Variable(term) => term.to_json(),
            Self::Effect(term) => term.to_json(),
            Self::Let(term) => term.to_json(),
            Self::Lambda(term) => term.to_json(),
            Self::LazyResult(term) => term.to_json(),
            Self::Application(term) => term.to_json(),
            Self::PartialApplication(term) => term.to_json(),
            Self::Recursive(term) => term.to_json(),
            Self::CompiledFunction(term) => term.to_json(),
            Self::Builtin(term) => term.to_json(),
            Self::Record(term) => term.to_json(),
            Self::Constructor(term) => term.to_json(),
            Self::List(term) => term.to_json(),
            Self::HashMap(term) => term.to_json(),
            Self::HashSet(term) => term.to_json(),
            Self::Signal(term) => term.to_json(),
        }
    }

    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        match (self, target) {
            (Self::Nil(term), Self::Nil(other)) => term.patch(other),
            (Self::Boolean(term), Self::Boolean(other)) => term.patch(other),
            (Self::Int(term), Self::Int(other)) => term.patch(other),
            (Self::Float(term), Self::Float(other)) => term.patch(other),
            (Self::String(term), Self::String(other)) => term.patch(other),
            (Self::Symbol(term), Self::Symbol(other)) => term.patch(other),
            (Self::Timestamp(term), Self::Timestamp(other)) => term.patch(other),
            (Self::Variable(term), Self::Variable(other)) => term.patch(other),
            (Self::Effect(term), Self::Effect(other)) => term.patch(other),
            (Self::Let(term), Self::Let(other)) => term.patch(other),
            (Self::Lambda(term), Self::Lambda(other)) => term.patch(other),
            (Self::LazyResult(term), Self::LazyResult(other)) => term.patch(other),
            (Self::Application(term), Self::Application(other)) => term.patch(other),
            (Self::PartialApplication(term), Self::PartialApplication(other)) => term.patch(other),
            (Self::Recursive(term), Self::Recursive(other)) => term.patch(other),
            (Self::CompiledFunction(term), Self::CompiledFunction(other)) => term.patch(other),
            (Self::Builtin(term), Self::Builtin(other)) => term.patch(other),
            (Self::Record(term), Self::Record(other)) => term.patch(other),
            (Self::Constructor(term), Self::Constructor(other)) => term.patch(other),
            (Self::List(term), Self::List(other)) => term.patch(other),
            (Self::HashMap(term), Self::HashMap(other)) => term.patch(other),
            (Self::HashSet(term), Self::HashSet(other)) => term.patch(other),
            (Self::Signal(term), Self::Signal(other)) => term.patch(other),
            _ => target.to_json().map(Some),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
    use reflex::core::SignalType;
    use reflex_stdlib::{Multiply, Stdlib};

    #[test]
    fn serialization() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::default();

        let input = factory.create_int_term(5);
        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: CachedSharedTerm<Stdlib> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(input, deserialized);

        let input =
            factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
                SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_symbol_term(123),
                },
            )]));
        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: CachedSharedTerm<Stdlib> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(input, deserialized);

        let input = {
            // ((lambda (foo)
            //     (* (+ 2 3) foo))
            //     2)
            factory.create_application_term(
                factory.create_lambda_term(
                    1,
                    factory.create_application_term(
                        factory.create_builtin_term(Multiply),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Multiply),
                                allocator.create_pair(
                                    factory.create_int_term(2),
                                    factory.create_int_term(3),
                                ),
                            ),
                            factory.create_variable_term(0),
                        ),
                    ),
                ),
                allocator.create_unit_list(factory.create_int_term(2)),
            )
        };
        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: CachedSharedTerm<Stdlib> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(input, deserialized);
    }
}
