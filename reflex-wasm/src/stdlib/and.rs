// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledFunctionCall,
        CompiledInstruction, CompilerError, CompilerOptions, CompilerResult, CompilerStack,
        CompilerStackValue, CompilerState, CompilerVariableBindings, ParamsSignature,
        TypeSignature, ValueType,
    },
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct And;
impl And {
    pub const UUID: Uuid = uuid!("223539c0-3858-4257-a53d-55fa93e2e7ba");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Lazy],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for And {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}

impl<'a, A: Arena + Clone> CompileWasm<A> for CompiledFunctionCall<'a, A, And> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let Self {
            builtin,
            target,
            args,
            ..
        } = self;
        let mut instructions = CompiledBlock::default();
        let (condition, consequent) = {
            let mut arg_list = args.iter();
            let condition = arg_list.next();
            let consequent = arg_list.next();
            match (condition, consequent) {
                (Some(condition), Some(consequent)) => Ok((condition, consequent)),
                _ => Err(CompilerError::InvalidFunctionArgs {
                    target: target.clone(),
                    arity: builtin.arity(),
                    args: args.iter().collect(),
                }),
            }
        }?;
        // Yield the condition onto the stack
        // => [Term]
        instructions.append_block(condition.compile(state, bindings, options, stack)?);
        let stack = stack.push_strict();
        // Duplicate the condition onto the stack to test whether it is a signal
        // => [Term, Term]
        instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
        // Invoke the builtin function to determine whether the value is a signal
        // => [Term, bool]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::IsSignal,
        ));
        // Short circuit if a signal term was encountered
        // TODO: Consolidate signal-testing code across multiple use cases
        // => [Term]
        instructions.push(CompiledInstruction::ConditionalBreak {
            // Retain the evaluated condition term pointer on the stack, preceded by any existing captured stack values
            block_type: TypeSignature {
                params: ParamsSignature::from_iter(stack.value_types()),
                results: ParamsSignature::Single(ValueType::HeapPointer),
            },
            // Return the signal term
            handler: {
                let mut instructions = CompiledBlock::default();
                // If there were any captured values saved onto the operand stack we need to discard them and then
                // push the signal term pointer back on top of the stack
                if stack.depth() > 1 {
                    // Pop the signal term pointer from the top of the stack and store in a new temporary lexical scope
                    instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                    let stack = stack.pop();
                    // Discard any preceding stack arguments that had been captured for use in the continuation block closure
                    let num_signal_scopes =
                        stack
                            .rev()
                            .fold(Ok(0usize), |num_signal_scopes, stack_value| {
                                let num_signal_scopes = num_signal_scopes?;
                                match stack_value {
                                    CompilerStackValue::Lazy(value_type) => {
                                        // If the captured stack value does not need to be checked for signals,
                                        // pop it from the operand stack and move on
                                        instructions.push(CompiledInstruction::Drop(value_type));
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
                                        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                                            RuntimeBuiltin::IsSignal,
                                        ));
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
                                        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                                            RuntimeBuiltin::CombineSignals,
                                        ));
                                        // Create a new lexical scope containing the accumulated signal result
                                        instructions.push(CompiledInstruction::ScopeStart(
                                            ValueType::HeapPointer,
                                        ));
                                        Ok(num_signal_scopes + 1)
                                    }
                                }
                            })?;
                    // Push the accumulated signal term pointer onto the top of the stack
                    instructions.push(CompiledInstruction::GetScopeValue {
                        value_type: ValueType::HeapPointer,
                        scope_offset: 0,
                    });
                    // Drop the temporary signal-testing scopes
                    for _ in 0..num_signal_scopes {
                        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                    }
                    // Drop the temporary lexical scope
                    instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                }
                instructions
            },
        });
        // Duplicate the evaluated result
        // => [Term, Term]
        instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
        // Create a new lexical scope containing a copy of the evaluated result
        // => [Term]
        instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
        // Update the inner compiler scope to take into account the intermediate lexical scope
        let inner_bindings = bindings.offset(1);
        // Invoke the runtime builtin to determine whether the condition is truthy
        // => [bool]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::IsTruthy,
        ));
        // Switch on the result, returning either the consequent or the evaluated result depending on the condition
        // => [Term]
        let stack = stack.pop();
        instructions.push(CompiledInstruction::If {
            block_type: TypeSignature {
                params: ParamsSignature::from_iter(stack.value_types()),
                results: ParamsSignature::from_iter(
                    stack.value_types().chain([ValueType::HeapPointer]),
                ),
            },
            consequent: {
                // Yield the consequent onto the stack
                // => [Term]
                consequent.compile(state, &inner_bindings, options, &stack)?
            },
            alternative: {
                let mut instructions = CompiledBlock::default();
                // Push the stored result onto the stack
                // => [Term]
                instructions.push(CompiledInstruction::GetScopeValue {
                    value_type: ValueType::HeapPointer,
                    scope_offset: 0,
                });
                instructions
            },
        });
        // Drop the temporary lexical scope
        // => [Term]
        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
        Ok(instructions)
    }
}
