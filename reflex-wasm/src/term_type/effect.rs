// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Eagerness, EffectTermType, Expression, GraphNode, Internable, SerializeJson,
    StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, CompilerVariableBindings, ParamsSignature,
        TypeSignature, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{ConditionTerm, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct EffectTerm {
    pub condition: ArenaPointer,
}
impl TermSize for EffectTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for EffectTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let condition_hash = arena.read_value::<Term, _>(self.condition, |term| term.id());
        hasher.hash(&condition_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<EffectTerm, A> {
    pub fn condition(&self) -> ArenaRef<TypedTerm<ConditionTerm>, A> {
        ArenaRef::<TypedTerm<ConditionTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.condition),
        )
    }
}

impl<A: Arena + Clone> EffectTermType<WasmExpression<A>> for ArenaRef<EffectTerm, A> {
    fn condition<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalRef<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        self.condition().into()
    }
}

impl<A: Arena + Clone> EffectTermType<WasmExpression<A>> for ArenaRef<TypedTerm<EffectTerm>, A> {
    fn condition<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalRef<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<EffectTerm, A> as EffectTermType<WasmExpression<A>>>::condition(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<EffectTerm, A> {
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
        DependencyList::of(self.condition().read_value(|condition| condition.id()))
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        true
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<EffectTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<EffectTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.condition() == other.condition()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<EffectTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<effect:{}>", self.condition())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<EffectTerm, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        eager == Eagerness::Lazy && self.condition().as_inner().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<EffectTerm, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let condition = self.condition();
        let mut instructions = CompiledBlock::default();
        // Yield the condition onto the stack (this will be used as the state key)
        // => [ConditionTerm]
        instructions.append_block(
            condition
                .as_inner()
                .compile(state, bindings, options, stack)?,
        );
        // Duplicate the condition on the stack so that it can be assigned to a local variable
        // => [ConditionTerm, ConditionTerm]
        instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
        // Enter a new temporary scope with the key assigned to a variable
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
        // Attempt to load the corresponding value from global state
        // => [Option<Term>]
        instructions.push(CompiledInstruction::LoadStateValue);
        // Enter a new temporary scope with the result assigned to a variable
        // => []
        instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
        // Load the result back from the temporary scope variable
        // => [Option<Term>]
        instructions.push(CompiledInstruction::GetScopeValue {
            value_type: ValueType::HeapPointer,
            scope_offset: 0,
        });
        // Push the null pointer onto the stack
        // => [Option<Term>, Null]
        instructions.push(CompiledInstruction::NullPointer);
        // Compare the pointers to determine whether a corresponding state value exists for the given key
        // => [bool]
        instructions.push(CompiledInstruction::Eq(ValueType::HeapPointer));
        // Branch based on whether a corresponding state value exists,
        // pushing the value onto the stack if the key was present, or a signal term if not
        // => [Term]
        instructions.push(CompiledInstruction::If {
            block_type: TypeSignature {
                params: ParamsSignature::Void,
                results: ParamsSignature::Single(ValueType::HeapPointer),
            },
            // If there was no corresponding value for the given key, construct a signal from the condition
            consequent: {
                let mut instructions = CompiledBlock::default();
                // Load the condition from the outer temporary scope and push onto the stack
                // => [ConditionTerm]
                instructions.push(CompiledInstruction::GetScopeValue {
                    value_type: ValueType::HeapPointer,
                    scope_offset: 1,
                });
                // Construct a new signal term from the condition and push it onto the stack
                // => [SignalTerm]
                instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                    RuntimeBuiltin::CreateSignal,
                ));
                instructions
            },
            // Otherwise if there was a corresponding value for the given key, return the value
            alternative: {
                let mut instructions = CompiledBlock::default();
                // Load the value from the inner temporary scope and push onto the stack
                // => [Term]
                instructions.push(CompiledInstruction::GetScopeValue {
                    value_type: ValueType::HeapPointer,
                    scope_offset: 0,
                });
                // Evaluate the term if necessary
                // => [Term]
                instructions.push(CompiledInstruction::Evaluate);
                instructions
            },
        });
        // Drop the inner temporary scope that was used to store the value
        // => [Term]
        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
        // Drop the outer temporary scope that was used to store the key
        // => [Term]
        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn effect() {
        assert_eq!(
            TermType::Effect(EffectTerm {
                condition: ArenaPointer(0x54321)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Effect as u32, 0x54321],
        );
    }
}
