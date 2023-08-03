// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ArgType, Arity, DependencyList, Expression, GraphNode, LambdaTermType, SerializeJson,
    StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompiledFunctionId, CompiledLambda, CompilerOptions, CompilerResult,
        CompilerStack, CompilerState, ConstValue, FunctionPointer, Internable, ParamsSignature,
        TypeSignature, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct LambdaTerm {
    pub num_args: u32,
    pub body: ArenaPointer,
}
impl TermSize for LambdaTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LambdaTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let body_hash = arena.read_value::<Term, _>(self.body, |term| term.id());
        hasher.hash(&self.num_args, arena).hash(&body_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<LambdaTerm, A> {
    pub fn num_args(&self) -> u32 {
        self.read_value(|term| term.num_args)
    }
    pub fn body(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.body))
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.num_args() as usize, 0, false)
    }
}

impl<A: Arena + Clone> LambdaTermType<WasmExpression<A>> for ArenaRef<LambdaTerm, A> {
    fn num_args<'a>(&'a self) -> StackOffset {
        self.num_args() as StackOffset
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.body().into()
    }
}

impl<A: Arena + Clone> LambdaTermType<WasmExpression<A>> for ArenaRef<TypedTerm<LambdaTerm>, A> {
    fn num_args<'a>(&'a self) -> StackOffset {
        <ArenaRef<LambdaTerm, A> as LambdaTermType<WasmExpression<A>>>::num_args(&self.as_inner())
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LambdaTerm, A> as LambdaTermType<WasmExpression<A>>>::body(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<LambdaTerm, A> {
    fn size(&self) -> usize {
        1 + self.body().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.body()
            .capture_depth()
            .saturating_sub(self.num_args() as StackOffset)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let num_args = self.num_args() as StackOffset;
        self.body()
            .free_variables()
            .into_iter()
            .filter_map(|offset| {
                if offset < num_args {
                    None
                } else {
                    Some(offset - num_args)
                }
            })
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.body()
            .count_variable_usages(offset + (self.num_args() as StackOffset))
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.body().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.body().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<LambdaTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<LambdaTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.num_args() == other.num_args() && self.body() == other.body()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<LambdaTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<LambdaTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<LambdaTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function:{}>", self.num_args())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<LambdaTerm, A> {
    fn should_intern(&self, _eager: ArgType) -> bool {
        false
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LambdaTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        // Ensure there are no free variable references within the lambda body
        // (any free variables should have been extracted out by an earlier compiler pass)
        // TODO: Consider relaxing restriction on compiled lambdas having been lambda-lifted (this would entail implementing 'sparse' binding mappings to combine separate chunks of variables bindings separated by offsets)
        if let Some(scope_offset) = self.free_variables().into_iter().next() {
            return Err(CompilerError::UnboundVariable(scope_offset));
        }
        // Note that all lambda functions will be linked into the main module in a later compiler phase,
        // which means they can be invoked similarly to the standard library builtins,
        // using the function lookup table to perform an indirect call to the compiled function wrapper.
        let compiled_function_id = CompiledFunctionId::from(self);
        // Compile the lambda body if it has not yet already been compiled
        if !state.compiled_lambdas.contains_key(&compiled_function_id) {
            let num_args = self.num_args() as StackOffset;
            let body = self.body();
            let params = ParamsSignature::from_iter((0..num_args).map(|_| ValueType::HeapPointer));
            let eagerness = options.lazy_lambda_args;
            let block = match eagerness {
                ArgType::Lazy => {
                    // Create a new compiler stack to be used for the function body,
                    // with all the lambda arguments declared as scoped variables
                    // and a block wrapper to catch short-circuiting signals
                    let inner_stack = params
                        .iter()
                        .fold(CompilerStack::default(), |stack, value_type| {
                            stack.declare_variable(value_type)
                        })
                        .enter_block(&TypeSignature {
                            params: ParamsSignature::Void,
                            results: ParamsSignature::Single(ValueType::HeapPointer),
                        })
                        .map_err(CompilerError::StackError)?;
                    let block = CompiledBlockBuilder::new(inner_stack);
                    // Yield the lambda body onto the operand stack, evaluating it within the correct stack scope
                    // => [Term]
                    let block = block.append_inner(|stack| body.compile(stack, state, options))?;
                    // Dispose the lexical scopes corresponding to the variable declarations
                    // => [Term]
                    let block = params.iter().fold(block, |block, value_type| {
                        block.push(instruction::core::ScopeEnd { value_type })
                    });
                    block
                }
                ArgType::Eager | ArgType::Strict => {
                    // Create a new compiler stack to be used for the function body,
                    // with all the lambda arguments declared as local lexical scopes
                    // and a block wrapper to catch short-circuiting signals
                    let inner_stack = params
                        .iter()
                        .fold(CompilerStack::default(), |stack, value_type| {
                            stack.enter_scope(value_type)
                        })
                        .enter_block(&TypeSignature {
                            params: ParamsSignature::Void,
                            results: ParamsSignature::Single(ValueType::HeapPointer),
                        })
                        .map_err(CompilerError::StackError)?;
                    let block = CompiledBlockBuilder::new(inner_stack);
                    // Iterate over each of the arguments, evaluating each one and storing the result in a new local lexical scope
                    // => []
                    let (block, num_arg_scopes) = (0..num_args).fold(
                        (block, 0usize),
                        |(block, num_arg_scopes), arg_index| {
                            // Push the argument value onto the operand stack
                            // => [Term]
                            let block = block.push(instruction::core::GetScopeValue {
                                value_type: ValueType::HeapPointer,
                                scope_offset: (num_args - arg_index - 1) + num_arg_scopes,
                            });
                            // Evaluate the argument value
                            // => [Term]
                            let block = block.push(instruction::runtime::Evaluate);
                            // Pop the evaluated argument value from the top of the stack and assign to a temporary lexical scope variable
                            // => []
                            let block = block.push(instruction::core::ScopeStart {
                                value_type: ValueType::HeapPointer,
                            });
                            (block, num_arg_scopes + 1)
                        },
                    );
                    // Create an iterator that iterates over the lexical scope offsets of the values corresponding to
                    // each argument, starting with the first argument and ending with the last argument
                    let arg_scope_offsets = (0..num_arg_scopes).map(|index| {
                        let arg_scope_offset = num_arg_scopes - 1 - index;
                        arg_scope_offset
                    });
                    let has_strict_args =
                        num_arg_scopes > 0 && matches!(eagerness, ArgType::Strict);
                    let block = if has_strict_args {
                        // Iterate over each of the evaluated arguments to determine whether the argument evaluated to a
                        // signal term, combining all signal results into an accumuated signal result
                        // => [Option<SignalTerm>]
                        let block = arg_scope_offsets.clone().enumerate().fold(
                            block,
                            |block, (index, arg_scope_offset)| {
                                // Push a copy of the argument value onto the operand stack (true case)
                                // => [{Option<SignalTerm>}, Term]
                                let block = block.push(instruction::core::GetScopeValue {
                                    value_type: ValueType::HeapPointer,
                                    scope_offset: arg_scope_offset,
                                });
                                // Push a null pointer onto the operand stack (false case)
                                // => [{Option<SignalTerm>}, Term, NULL]
                                let block = block.push(instruction::runtime::NullPointer);
                                // Push another copy of the argument value onto the operand stack (for signal testing)
                                // => [{Option<SignalTerm>}, Term, NULL, Term]
                                let block = block.push(instruction::core::GetScopeValue {
                                    value_type: ValueType::HeapPointer,
                                    scope_offset: arg_scope_offset,
                                });
                                // Determine whether the argument value is a signal (condition case)
                                // => [{Option<SignalTerm>}, Term, NULL, bool]
                                let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                                    target: RuntimeBuiltin::IsSignal,
                                });
                                // Select either the argument value or the null pointer depending on whether the argument value is a signal
                                // => [{Option<SignalTerm>}, Option<SignalTerm>]
                                let block = block.push(instruction::core::Select {
                                    value_type: ValueType::HeapPointer,
                                });
                                // If this is not the first argument to be tested for signals, combine it with the existing result from the previous iteration
                                // => [Option<SignalTerm>]
                                let block = if index > 0 {
                                    block.push(instruction::runtime::CallRuntimeBuiltin {
                                        target: RuntimeBuiltin::CombineSignals,
                                    })
                                } else {
                                    block
                                };
                                block
                            },
                        );
                        // Pop the combined signal result off the top of the stack and into a new temporary lexical scope
                        // => []
                        let block = block.push(instruction::core::ScopeStart {
                            value_type: ValueType::HeapPointer,
                        });
                        // The combined signal result is the most recently-declared lexical scope
                        let combined_signal_scope_offset = 0;
                        // Push the combined signal result back onto the stack
                        // => [Option<SignalTerm>]
                        let block = block.push(instruction::core::GetScopeValue {
                            scope_offset: combined_signal_scope_offset,
                            value_type: ValueType::HeapPointer,
                        });
                        // Push another copy of the combined signal result onto the stack for comparing against the null pointer
                        // => [Option<SignalTerm>, Option<SignalTerm>]
                        let block = block.push(instruction::core::GetScopeValue {
                            scope_offset: combined_signal_scope_offset,
                            value_type: ValueType::HeapPointer,
                        });
                        // Push a null pointer onto the stack to use for comparing against the combined signal term result
                        // => [Option<SignalTerm>, Option<SignalTerm>, NULL]
                        let block = block.push(instruction::runtime::NullPointer);
                        // Determine whether the combined signal result is not equal to the null pointer
                        // => [Option<SignalTerm>, bool]
                        let block = block.push(instruction::core::Ne {
                            value_type: ValueType::HeapPointer,
                        });
                        // Dispose the temporary combined signal result lexical scope
                        // => [Option<SignalTerm>, bool]
                        let block = block.push(instruction::core::ScopeEnd {
                            value_type: ValueType::HeapPointer,
                        });
                        // Break out of the current control flow block if a signal term was encountered
                        // => [Option<SignalTerm>]
                        let block = block.push(instruction::core::ConditionalBreak {
                            target_block: 0,
                            result_type: ParamsSignature::Single(ValueType::HeapPointer),
                        });
                        // Otherwise drop the null signal result
                        // => []
                        let block = block.push(instruction::core::Drop {
                            value_type: ValueType::HeapPointer,
                        });
                        block
                    } else {
                        block
                    };
                    // Iterate over each of the evaluated arguments, declaring a new variable scope for each one
                    // => []
                    let (block, num_variable_scopes) = arg_scope_offsets.fold(
                        (block, 0usize),
                        |(block, num_preceding_variable_scopes), arg_scope_offset| {
                            // Push the evaluated argument value onto the operand stack
                            // (making sure to skip over any lexical scopes created by this iterator)
                            // => [Term]
                            let block = block.push(instruction::core::GetScopeValue {
                                value_type: ValueType::HeapPointer,
                                scope_offset: arg_scope_offset + num_preceding_variable_scopes,
                            });
                            // Declare a new variable scope whose value is set to the evaluated argument value
                            // => []
                            let block = block.push(instruction::runtime::DeclareVariable {
                                value_type: ValueType::HeapPointer,
                            });
                            (block, num_preceding_variable_scopes + 1)
                        },
                    );
                    // Yield the lambda body onto the operand stack, evaluating it within the current stack scope
                    // => [Term]
                    let block = block.append_inner(|stack| body.compile(stack, state, options))?;
                    // Dispose the lexical scopes corresponding to the variable declarations
                    // => [Term]
                    let block = (0..num_variable_scopes).fold(block, |block, _| {
                        block.push(instruction::core::ScopeEnd {
                            value_type: ValueType::HeapPointer,
                        })
                    });
                    // Dispose the lexical scopes corresponding to the evaluated argument values
                    // => [Term]
                    let block = (0..num_arg_scopes).fold(block, |block, _| {
                        block.push(instruction::core::ScopeEnd {
                            value_type: ValueType::HeapPointer,
                        })
                    });
                    block
                }
            };
            // Add the compiled lambda to the compiler cache
            let compiled_body = block.finish::<CompilerError<_>>()?;
            state.compiled_lambdas.insert(
                compiled_function_id,
                CompiledLambda {
                    params,
                    body: compiled_body,
                },
            );
        }
        // Create a builtin term that references the compiled lambda
        let block = CompiledBlockBuilder::new(stack);
        // Push a pointer to the compiled function onto the stack
        // => [FunctionPointer]
        let block = block.push(instruction::core::Const {
            value: ConstValue::FunctionPointer(FunctionPointer::Lambda(compiled_function_id)),
        });
        // Invoke the term constructor
        // => [BuiltinTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateBuiltin,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn lambda() {
        assert_eq!(
            TermType::Lambda(LambdaTerm {
                num_args: 0x54321,
                body: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Lambda as u32, 0x54321, 0x98765],
        );
    }
}
