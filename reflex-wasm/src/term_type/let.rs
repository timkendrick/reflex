// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Eagerness, Expression, GraphNode, Internable, LetTermType, SerializeJson,
    StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerStackValue, CompilerState, CompilerVariableBindings,
        MaybeLazyExpression, ParamsSignature, TypeSignature, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct LetTerm {
    pub initializer: ArenaPointer,
    pub body: ArenaPointer,
}
impl TermSize for LetTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LetTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let initializer_hash = arena.read_value::<Term, _>(self.initializer, |term| term.id());
        let body_hash = arena.read_value::<Term, _>(self.body, |term| term.id());
        hasher
            .hash(&initializer_hash, arena)
            .hash(&body_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<LetTerm, A> {
    pub fn initializer(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.initializer))
    }
    pub fn body(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.body))
    }
}

impl<A: Arena + Clone> LetTermType<WasmExpression<A>> for ArenaRef<LetTerm, A> {
    fn initializer<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.initializer().into()
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.body().into()
    }
}

impl<A: Arena + Clone> LetTermType<WasmExpression<A>> for ArenaRef<TypedTerm<LetTerm>, A> {
    fn initializer<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LetTerm, A> as LetTermType<WasmExpression<A>>>::initializer(&self.as_inner())
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LetTerm, A> as LetTermType<WasmExpression<A>>>::body(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<LetTerm, A> {
    fn size(&self) -> usize {
        1 + self.initializer().size() + self.body().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.initializer()
            .capture_depth()
            .max(self.body().capture_depth().saturating_sub(1))
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.initializer()
            .free_variables()
            .into_iter()
            .chain(
                self.body()
                    .free_variables()
                    .into_iter()
                    .filter_map(|offset| if offset == 0 { None } else { Some(offset - 1) }),
            )
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.initializer().count_variable_usages(offset)
            + self.body().count_variable_usages(offset + 1)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        // TODO: Verify shallow dynamic dependencies for Let term
        self.initializer()
            .dynamic_dependencies(deep)
            .union(self.body().dynamic_dependencies(deep))
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        // TODO: Verify shallow dynamic dependencies for Let term
        self.initializer().has_dynamic_dependencies(deep)
            || self.body().has_dynamic_dependencies(deep)
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<LetTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<LetTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.initializer() == other.initializer() && self.body() == other.body()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<LetTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<LetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<LetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<let:{}:{}>", self.initializer(), self.body())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<LetTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        false
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LetTerm, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let initializer = self.initializer();
        let body = self.body();
        let mut instructions = CompiledBlock::default();
        // If the variable initializer is an alias to a variable declared in a parent scope, handle that as a special case
        if let Some(alias) = initializer.as_variable_term() {
            // Determine the scope offset of the aliased target variable
            let target_offset = alias.as_inner().stack_offset();
            // Create a new compiler scope that includes the local variable
            let inner_bindings = bindings.push_alias(target_offset);
            // Yield the expression body onto the stack (this will be evaluated within the new scope)
            // => [Term]
            instructions.append_block(body.compile(state, &inner_bindings, options, stack)?);
        } else {
            let eagerness = if options.lazy_variable_initializers
                || (initializer.is_static() && initializer.as_signal_term().is_none())
            {
                Eagerness::Lazy
            } else {
                Eagerness::Eager
            };
            let is_strict = matches!(eagerness, Eagerness::Eager);
            // Yield the initializer term onto the stack
            // => [Term]
            instructions.append_block(
                MaybeLazyExpression::new(initializer, eagerness)
                    .compile(state, bindings, options, stack)?,
            );
            // Pop the initializer term and assign it to a temporary lexical scope
            // => []
            instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
            // Create a new compiler scope that includes the local variable
            let inner_bindings = bindings.push_local();
            // If the initializer is evaluated in strict mode, test for signals
            if is_strict {
                // Push a copy of the initializer result onto the stack
                // => [Term]
                instructions.push(CompiledInstruction::GetScopeValue {
                    value_type: ValueType::HeapPointer,
                    scope_offset: 0,
                });
                // Invoke the builtin function to determine whether the value is a signal
                // => [bool]
                instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                    RuntimeBuiltin::IsSignal,
                ));
                // Short circuit if a signal term was encountered
                // TODO: Consolidate signal-testing code across multiple use cases
                // => []
                instructions.push(CompiledInstruction::ConditionalBreak {
                    // Retain the evaluated condition term pointer on the stack, preceded by any existing captured stack values
                    block_type: TypeSignature {
                        params: ParamsSignature::from_iter(stack.value_types()),
                        results: ParamsSignature::Single(ValueType::HeapPointer),
                    },
                    // Return the signal term
                    handler: {
                        let mut instructions = CompiledBlock::default();
                        // If there were any captured values saved onto the operand stack we need to discard them
                        if stack.depth() > 0 {
                            // Discard any preceding stack arguments that had been captured for use in the continuation block closure
                            let num_signal_scopes = stack.rev().fold(
                                Ok(0usize),
                                |num_signal_scopes, stack_value| {
                                    let num_signal_scopes = num_signal_scopes?;
                                    match stack_value {
                                        CompilerStackValue::Lazy(value_type) => {
                                            // If the captured stack value does not need to be checked for signals,
                                            // pop it from the operand stack and move on
                                            instructions
                                                .push(CompiledInstruction::Drop(value_type));
                                            Ok(num_signal_scopes)
                                        }
                                        CompilerStackValue::Strict => {
                                            // Pop the captured value from the operand stack and store it in a temporary scope for signal-testing
                                            instructions.push(CompiledInstruction::ScopeStart(
                                                ValueType::HeapPointer,
                                            ));
                                            // Reinstate a copy of the captured value on the operand stack (true branch)
                                            instructions.push(CompiledInstruction::GetScopeValue {
                                                value_type: ValueType::HeapPointer,
                                                scope_offset: 0,
                                            });
                                            // Push a null pointer onto the operand stack (false branch)
                                            instructions.push(CompiledInstruction::NullPointer);
                                            // Push another copy of the captured value onto the operand stack for signal comparison
                                            instructions.push(CompiledInstruction::GetScopeValue {
                                                value_type: ValueType::HeapPointer,
                                                scope_offset: 0,
                                            });
                                            // Dispose the temporary signal-testing scope
                                            instructions.push(CompiledInstruction::ScopeEnd(
                                                ValueType::HeapPointer,
                                            ));
                                            // Determine whether the captured value is a signal (condition)
                                            instructions.push(
                                                CompiledInstruction::CallRuntimeBuiltin(
                                                    RuntimeBuiltin::IsSignal,
                                                ),
                                            );
                                            // Select either the captured value or the null pointer depending on whether the captured value is a signal
                                            instructions.push(CompiledInstruction::Select(
                                                ValueType::HeapPointer,
                                            ));
                                            // Push the existing accumulated signal onto the operand stack
                                            instructions.push(CompiledInstruction::GetScopeValue {
                                                value_type: ValueType::HeapPointer,
                                                scope_offset: 0,
                                            });
                                            // Combine with the existing accumulated signal
                                            instructions.push(
                                                CompiledInstruction::CallRuntimeBuiltin(
                                                    RuntimeBuiltin::CombineSignals,
                                                ),
                                            );
                                            // Create a new lexical scope containing the accumulated signal result
                                            instructions.push(CompiledInstruction::ScopeStart(
                                                ValueType::HeapPointer,
                                            ));
                                            Ok(num_signal_scopes + 1)
                                        }
                                    }
                                },
                            )?;
                            // Drop the temporary signal-testing scopes
                            for _ in 0..num_signal_scopes {
                                instructions
                                    .push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                            }
                        }
                        // Push the accumulated signal term pointer onto the top of the stack
                        instructions.push(CompiledInstruction::GetScopeValue {
                            value_type: ValueType::HeapPointer,
                            scope_offset: 0,
                        });
                        instructions
                    },
                });
            }
            // Yield the expression body onto the stack (this will be evaluated within the new scope)
            // => [Term]
            instructions.append_block(body.compile(state, &inner_bindings, options, stack)?);
            // Drop the temporary lexical scope, leaving the result on the stack
            // => [Term]
            instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
        }
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn r#let() {
        assert_eq!(
            TermType::Let(LetTerm {
                initializer: ArenaPointer(0x54321),
                body: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Let as u32, 0x54321, 0x98765],
        );
    }
}
