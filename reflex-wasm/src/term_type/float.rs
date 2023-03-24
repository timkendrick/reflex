// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Eagerness, FloatTermType, FloatValue, GraphNode, Internable, SerializeJson,
    StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, CompilerVariableBindings,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct FloatTerm {
    pub value: [u32; 2],
}
impl TermSize for FloatTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FloatTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<f64> for FloatTerm {
    fn from(value: f64) -> Self {
        Self {
            value: f64_to_chunks(value),
        }
    }
}
impl From<FloatTerm> for f64 {
    fn from(value: FloatTerm) -> Self {
        let FloatTerm { value, .. } = value;
        chunks_to_f64(value)
    }
}

fn f64_to_chunks(value: f64) -> [u32; 2] {
    let bytes = value.to_le_bytes();
    let low_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high_word = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    [low_word, high_word]
}

fn chunks_to_f64(value: [u32; 2]) -> f64 {
    let [low_word, high_word] = value;
    let low_bytes = low_word.to_le_bytes();
    let high_bytes = high_word.to_le_bytes();
    f64::from_le_bytes([
        low_bytes[0],
        low_bytes[1],
        low_bytes[2],
        low_bytes[3],
        high_bytes[0],
        high_bytes[1],
        high_bytes[2],
        high_bytes[3],
    ])
}

impl<A: Arena + Clone> ArenaRef<FloatTerm, A> {
    pub fn value(&self) -> f64 {
        chunks_to_f64(self.read_value(|term| term.value))
    }
}

impl<A: Arena + Clone> FloatTermType for ArenaRef<FloatTerm, A> {
    fn value(&self) -> FloatValue {
        self.value()
    }
}

impl<A: Arena + Clone> FloatTermType for ArenaRef<TypedTerm<FloatTerm>, A> {
    fn value(&self) -> FloatValue {
        <ArenaRef<FloatTerm, A> as FloatTermType>::value(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<FloatTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<FloatTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        match serde_json::Number::from_f64(self.value()) {
            Some(number) => Ok(JsonValue::Number(number)),
            None => Err(format!(
                "Unable to serialize float non-finite float as JSON value: {}",
                self
            )),
        }
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.value() == target.value() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<FloatTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<FloatTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<FloatTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<FloatTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<FloatTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        true
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<FloatTerm, A> {
    fn compile(
        &self,
        _state: &mut CompilerState,
        _bindings: &CompilerVariableBindings,
        _options: &CompilerOptions,
        _stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let value = self.value();
        let mut instructions = CompiledBlock::default();
        // Push the value argument onto the stack
        // => [value]
        instructions.push(CompiledInstruction::f64_const(value));
        // Invoke the term constructor
        // => [FloatTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFloat,
        ));
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn float() {
        let value = 3.142;
        assert_eq!(
            TermType::Float(FloatTerm::from(value)).as_bytes(),
            [
                TermTypeDiscriminants::Float as u32,
                f64_to_chunks(value)[0],
                f64_to_chunks(value)[1]
            ],
        );
    }
}
