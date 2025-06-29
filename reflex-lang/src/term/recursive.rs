// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{collections::HashSet, iter::once};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::core::{
    CompoundNode, DependencyList, DynamicState, EvaluationCache, Expression, ExpressionFactory,
    GraphNode, HeapAllocator, RecursiveTermType, Reducible, Rewritable, SerializeJson, StackOffset,
    Substitutions,
};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RecursiveTerm<T: Expression> {
    factory: T,
}

impl<T: Expression> std::hash::Hash for RecursiveTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.factory.id().hash(state);
    }
}

impl<T: Expression> RecursiveTerm<T> {
    pub fn new(factory: T) -> Self {
        Self { factory }
    }
}
impl<T: Expression> RecursiveTermType<T> for RecursiveTerm<T> {
    fn factory<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        (&self.factory).into()
    }
}
impl<T: Expression> GraphNode for RecursiveTerm<T> {
    fn size(&self) -> usize {
        1 + self.factory.size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.factory.capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.factory.free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.factory.count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        self.factory.dynamic_dependencies(deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.factory.has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        self.factory.is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}
impl<T: Expression> CompoundNode<T> for RecursiveTerm<T> {
    type Children<'a> = std::iter::Once<T::ExpressionRef<'a>>
        where
            T: 'a,
            Self: 'a;
    fn children<'a>(&'a self) -> Self::Children<'a>
    where
        T: 'a,
    {
        once((&self.factory).into())
    }
}
impl<T: Expression + Rewritable<T>> Rewritable<T> for RecursiveTerm<T> {
    fn substitute_static(
        &self,
        substitutions: &Substitutions<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        self.factory
            .substitute_static(substitutions, factory, allocator, cache)
            .map(|target| factory.create_recursive_term(target))
    }
    fn substitute_dynamic(
        &self,
        deep: bool,
        state: &impl DynamicState<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        self.factory
            .substitute_dynamic(deep, state, factory, allocator, cache)
            .map(|target| factory.create_recursive_term(target))
    }
    fn hoist_free_variables(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
    ) -> Option<T> {
        self.factory
            .hoist_free_variables(factory, allocator)
            .map(|target| factory.create_recursive_term(target))
    }
    fn normalize(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        self.factory
            .normalize(factory, allocator, cache)
            .map(|target| factory.create_recursive_term(target))
    }
}
impl<T: Expression + Reducible<T>> Reducible<T> for RecursiveTerm<T> {
    fn is_reducible(&self) -> bool {
        true
    }
    fn reduce(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        factory
            .create_application_term(
                self.factory.clone(),
                allocator.create_unit_list(factory.create_recursive_term(self.factory.clone())),
            )
            .reduce(factory, allocator, cache)
    }
}

impl<T: Expression> std::fmt::Display for RecursiveTerm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<recursive:{}>", format!("{}", self.factory))
    }
}

impl<T: Expression> SerializeJson for RecursiveTerm<T> {
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
