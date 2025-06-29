// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::{
    core::{
        Applicable, Arity, CompiledFunctionTermType, DependencyList, EvaluationCache, Expression,
        ExpressionFactory, GraphNode, HeapAllocator, InstructionPointer, SerializeJson,
        StackOffset,
    },
    hash::HashId,
};

#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct CompiledFunctionTerm {
    address: InstructionPointer,
    hash: HashId,
    required_args: StackOffset,
    optional_args: StackOffset,
    variadic_args: bool,
}
impl CompiledFunctionTerm {
    pub fn new(
        address: InstructionPointer,
        hash: HashId,
        required_args: StackOffset,
        optional_args: StackOffset,
        variadic_args: bool,
    ) -> Self {
        Self {
            address,
            hash,
            required_args,
            optional_args,
            variadic_args,
        }
    }
}
impl CompiledFunctionTermType for CompiledFunctionTerm {
    fn address(&self) -> InstructionPointer {
        self.address
    }
    fn hash(&self) -> HashId {
        self.hash
    }
    fn required_args(&self) -> StackOffset {
        self.required_args
    }
    fn optional_args(&self) -> StackOffset {
        self.optional_args
    }
    fn variadic_args(&self) -> bool {
        self.variadic_args
    }
}
impl GraphNode for CompiledFunctionTerm {
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
impl<T: Expression> Applicable<T> for CompiledFunctionTerm {
    fn arity(&self) -> Option<Arity> {
        Some(Arity::lazy(self.required_args, self.optional_args, false))
    }
    fn apply(
        &self,
        _args: impl ExactSizeIterator<Item = T>,
        _factory: &impl ExpressionFactory<T>,
        _allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        Err(format!(
            "Compiled functions cannot be invoked directly: {}",
            self,
        ))
    }
    fn should_parallelize(&self, _args: &[T]) -> bool {
        false
    }
}

impl std::fmt::Display for CompiledFunctionTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<compiled:{:x}>", self.address)
    }
}

impl SerializeJson for CompiledFunctionTerm {
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
