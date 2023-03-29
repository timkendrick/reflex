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
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompilerOptions, CompilerResult, CompilerStack, CompilerState,
        ParamsSignature, TypeSignature, ValueType,
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
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let condition = self.condition();
        let block = CompiledBlockBuilder::new(stack);
        // Yield the condition onto the stack (this will be used as the state key)
        // => [ConditionTerm]
        let block =
            block.append_inner(|stack| condition.as_inner().compile(stack, state, options))?;
        // Duplicate the condition on the stack so that it can be assigned to a local variable
        // => [ConditionTerm, ConditionTerm]
        let block = block.push(instruction::core::Duplicate {
            value_type: ValueType::HeapPointer,
        });
        // Enter a new temporary scope with the key assigned to a variable
        // => [ConditionTerm]
        let block = block.push(instruction::core::ScopeStart {
            value_type: ValueType::HeapPointer,
        });
        // Attempt to load the corresponding value from global state
        // => [Option<Term>]
        let block = block.push(instruction::runtime::LoadStateValue);
        // Enter a new temporary scope with the result assigned to a variable
        // => []
        let block = block.push(instruction::core::ScopeStart {
            value_type: ValueType::HeapPointer,
        });
        // Load the result back from the temporary scope variable
        // => [Option<Term>]
        let block = block.push(instruction::core::GetScopeValue {
            value_type: ValueType::HeapPointer,
            scope_offset: 0,
        });
        // Push the null pointer onto the stack
        // => [Option<Term>, Null]
        let block = block.push(instruction::runtime::NullPointer);
        // Compare the pointers to determine whether a corresponding state value exists for the given key
        // => [bool]
        let block = block.push(instruction::core::Eq {
            value_type: ValueType::HeapPointer,
        });
        // Branch based on whether a corresponding state value exists,
        // pushing the value onto the stack if the key was present, or a signal term if not
        // => [Term]
        let block = block.append_inner(|stack| {
            let block_type = TypeSignature {
                params: ParamsSignature::Void,
                results: ParamsSignature::Single(ValueType::HeapPointer),
            };
            let inner_stack = stack.enter_block(&block_type)?;
            let (consequent_stack, alternative_stack) = (inner_stack.clone(), inner_stack);
            CompiledBlockBuilder::new(stack)
                .push(instruction::core::If {
                    block_type,
                    // If there was no corresponding value for the given key, construct a signal from the condition
                    consequent: {
                        let block = CompiledBlockBuilder::new(consequent_stack);
                        // Load the condition from the outer temporary scope and push onto the stack
                        // => [ConditionTerm]
                        let block = block.push(instruction::core::GetScopeValue {
                            value_type: ValueType::HeapPointer,
                            scope_offset: 1,
                        });
                        // Construct a new signal term from the condition and push it onto the stack
                        // => [SignalTerm]
                        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                            target: RuntimeBuiltin::CreateSignal,
                        });
                        block.finish::<CompilerError<_>>()
                    }?,
                    // Otherwise if there was a corresponding value for the given key, return the value
                    alternative: {
                        let block = CompiledBlockBuilder::new(alternative_stack);
                        // Load the value from the inner temporary scope and push onto the stack
                        // => [Term]
                        let block = block.push(instruction::core::GetScopeValue {
                            value_type: ValueType::HeapPointer,
                            scope_offset: 0,
                        });
                        // Evaluate the term if necessary
                        // => [Term]
                        let block = block.push(instruction::runtime::Evaluate);
                        block.finish::<CompilerError<_>>()
                    }?,
                })
                .finish::<CompilerError<_>>()
        })?;
        // Drop the inner temporary scope that was used to store the value
        // => [Term]
        let block = block.push(instruction::core::ScopeEnd {
            value_type: ValueType::HeapPointer,
        });
        // Drop the outer temporary scope that was used to store the key
        // => [Term]
        let block = block.push(instruction::core::ScopeEnd {
            value_type: ValueType::HeapPointer,
        });
        block.finish()
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
