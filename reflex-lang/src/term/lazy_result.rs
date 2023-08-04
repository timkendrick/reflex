use std::{collections::HashSet, iter::once};

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::core::{
    CompoundNode, ConditionListType, ConditionType, DependencyList, DynamicState, Evaluate,
    EvaluationCache, EvaluationResult, Expression, ExpressionFactory, GraphNode, HeapAllocator,
    LazyResultTermType, Reducible, RefType, Rewritable, SerializeJson, StackOffset, Substitutions,
};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LazyResultTerm<T: Expression> {
    pub value: T,
    pub dependencies: T::SignalList,
}

impl<T: Expression> std::hash::Hash for LazyResultTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.id().hash(state);
        self.dependencies.id().hash(state);
    }
}

impl<T: Expression> LazyResultTerm<T> {
    pub fn new(value: T, dependencies: T::SignalList) -> Self {
        Self {
            value,
            dependencies,
        }
    }
}

impl<T: Expression> LazyResultTermType<T> for LazyResultTerm<T> {
    fn value<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        (&self.value).into()
    }
    fn dependencies<'a>(&'a self) -> T::SignalListRef<'a>
    where
        T: 'a,
    {
        (&self.dependencies).into()
    }
}

impl<T: Expression> GraphNode for LazyResultTerm<T> {
    fn size(&self) -> usize {
        1 + self.value.size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.value.capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.value.free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.value.count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        self.value.dynamic_dependencies(deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.value.has_dynamic_dependencies(deep)
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

impl<T: Expression> CompoundNode<T> for LazyResultTerm<T> {
    type Children<'a> = std::iter::Once<T::ExpressionRef<'a>>
        where
            T: 'a,
            Self: 'a;
    fn children<'a>(&'a self) -> Self::Children<'a>
    where
        T: 'a,
    {
        once((&self.value).into())
    }
}

impl<T: Expression + Rewritable<T> + Reducible<T>> Rewritable<T> for LazyResultTerm<T> {
    fn substitute_static(
        &self,
        substitutions: &Substitutions<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        let value = self
            .value
            .substitute_static(substitutions, factory, allocator, cache);
        if value.is_none() {
            return None;
        } else {
            Some(factory.create_lazy_result_term(
                value.unwrap_or_else(|| self.value.clone()),
                self.dependencies.clone(),
            ))
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
        let value = self
            .value
            .substitute_dynamic(deep, state, factory, allocator, cache);
        if value.is_none() {
            return None;
        } else {
            Some(factory.create_lazy_result_term(
                value.unwrap_or_else(|| self.value.clone()),
                self.dependencies.clone(),
            ))
        }
    }
    fn hoist_free_variables(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
    ) -> Option<T> {
        let value = self.value.hoist_free_variables(factory, allocator);
        if value.is_none() {
            return None;
        } else {
            Some(factory.create_lazy_result_term(
                value.unwrap_or_else(|| self.value.clone()),
                self.dependencies.clone(),
            ))
        }
    }
    fn normalize(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        let value = self.value.normalize(factory, allocator, cache);
        if value.is_none() {
            return None;
        } else {
            Some(factory.create_lazy_result_term(
                value.unwrap_or_else(|| self.value.clone()),
                self.dependencies.clone(),
            ))
        }
    }
}

impl<T: Expression> Evaluate<T> for LazyResultTerm<T> {
    fn evaluate(
        &self,
        _state: &impl DynamicState<T>,
        _factory: &impl ExpressionFactory<T>,
        _allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Option<EvaluationResult<T>> {
        Some(EvaluationResult::new(
            self.value.clone(),
            DependencyList::from_iter(
                self.dependencies
                    .iter()
                    .map(|condition| condition.as_deref().id()),
            ),
        ))
    }
}

impl<T: Expression> std::fmt::Display for LazyResultTerm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lazy:{}>", self.value)
    }
}

impl<T: Expression> SerializeJson for LazyResultTerm<T> {
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
