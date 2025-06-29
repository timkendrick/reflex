// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::core::{
    Applicable, Arity, ConstructorTermType, DependencyList, EvaluationCache, Expression,
    ExpressionFactory, ExpressionListType, GraphNode, HeapAllocator, RefType, SerializeJson,
    StackOffset, StructPrototypeType,
};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ConstructorTerm<T: Expression> {
    prototype: T::StructPrototype,
}

impl<T: Expression> std::hash::Hash for ConstructorTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.prototype.keys().as_deref().id().hash(state);
    }
}

impl<T: Expression> ConstructorTerm<T> {
    pub fn new(prototype: T::StructPrototype) -> Self {
        Self { prototype }
    }
}
impl<T: Expression> ConstructorTermType<T> for ConstructorTerm<T> {
    fn prototype<'a>(&'a self) -> T::StructPrototypeRef<'a>
    where
        T::StructPrototype: 'a,
        T: 'a,
    {
        (&self.prototype).into()
    }
}
impl<T: Expression> GraphNode for ConstructorTerm<T> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::new()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}
impl<T: Expression> Applicable<T> for ConstructorTerm<T> {
    fn arity(&self) -> Option<Arity> {
        Some(Arity::lazy(
            0,
            self.prototype().as_deref().keys().as_deref().len(),
            false,
        ))
    }
    fn apply(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        Ok(factory.create_record_term(
            allocator.clone_struct_prototype((&self.prototype).into()),
            allocator.create_list(args),
        ))
    }
    fn should_parallelize(&self, _args: &[T]) -> bool {
        false
    }
}

impl<T: Expression> std::fmt::Display for ConstructorTerm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<constructor:{{{}}}>",
            self.prototype
                .keys()
                .as_deref()
                .iter()
                .map(|key| format!("{}", key.as_deref()))
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}
impl<T: Expression> SerializeJson for ConstructorTerm<T> {
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
