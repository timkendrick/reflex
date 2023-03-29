// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::iter::repeat;

use walrus::{ir, Module};

use crate::{
    compiler::{
        error::TypedStackError,
        instruction,
        runtime::builtin::RuntimeBuiltin,
        wasm::{
            parse_function_type_signature, parse_value_type, GenerateWasm, WasmGeneratorBindings,
            WasmGeneratorError, WasmGeneratorOptions, WasmGeneratorOutput, WasmGeneratorResult,
        },
        CompiledBlock, CompiledFunctionId, CompilerStack, ParamsSignature, TypeSignature,
        TypedCompilerBlock, ValueType,
    },
    stdlib::Stdlib,
};

/// Push a null pointer onto the operand stack
#[derive(Clone, PartialEq, Hash, Debug)]
pub struct NullPointer;

impl TypedCompilerBlock for NullPointer {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        Ok(stack
            // Push a term pointer onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for NullPointer {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::GlobalGet {
            global: bindings.null_pointer(),
        });
        Ok(instructions)
    }
}

/// Pop the top item of the operand stack and enter a new lexical scope whose variable is assigned to that value
/// (note that this lexical scope will be marked as a 'variable' scope, allowing its value to be retrieved within any
/// child scopes by looking up the variable offset on the stack)
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct DeclareVariable {
    /// Type of the variable declared by the lexical scope
    pub value_type: ValueType,
}

impl TypedCompilerBlock for DeclareVariable {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Pop the top value off the operand stack to use as the variable value
            .pop_operand(*value_type)?
            // Enter a new lexical scope with the correct variable type
            .declare_variable(*value_type))
    }
}

impl GenerateWasm for DeclareVariable {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value_type } = self;
        let scope_local_id = module.locals.add(parse_value_type(*value_type));
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalSet {
            local: scope_local_id,
        });
        bindings.enter_scope(scope_local_id);
        Ok(instructions)
    }
}

/// Pop a term pointer from the operand stack, look that key up in the global state object,
/// and push either the corresponding value term reference or a null pointer depending on whether the key exists
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LoadStateValue;

impl TypedCompilerBlock for LoadStateValue {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        Ok(stack
            // Pop the input term pointer onto the operand stack
            .pop_operand(ValueType::HeapPointer)?
            // Push the result term pointer onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for LoadStateValue {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalGet {
            local: bindings.state_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().get_state_value,
        });
        instructions.push(ir::LocalGet {
            local: bindings.dependencies_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().combine_dependencies,
        });
        instructions.push(ir::LocalSet {
            local: bindings.dependencies_id(),
        });
        Ok(instructions)
    }
}

/// Invoke an interpreter builtin function, popping the required number of arguments from the operand stack
/// (arguments are passed to the function in the same order they were added to the operand stack, i.e. not reversed)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CallRuntimeBuiltin {
    /// Runtime function to invoke
    pub target: RuntimeBuiltin,
}

impl TypedCompilerBlock for CallRuntimeBuiltin {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { target } = self;
        let TypeSignature { params, results } = target.signature();
        Ok(stack
            // Pop the function arguments from the operand stack
            .pop_operands(&params)?
            // Push the function results onto the operand stack
            .push_operands(&results))
    }
}

impl GenerateWasm for CallRuntimeBuiltin {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { target } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Call {
            func: bindings.builtins().get(*target),
        });
        Ok(instructions)
    }
}

/// Invoke a standard library function known at compile-time, popping the required number of arguments from the operand stack
/// (arguments are passed to the function in the same order they were added to the operand stack, i.e. not reversed)
/// If the function has variadic arguments, the final argument is assumed to be a heap pointer to a list term containing the variadic arguments
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CallStdlib {
    /// Standard library method to invoke
    pub target: Stdlib,
}

impl TypedCompilerBlock for CallStdlib {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { target } = self;
        let arity = target.arity();
        let num_positional_args = arity.required().len() + arity.optional().len();
        // Variadic arguments are passed as a heap pointer to an argument list
        let num_variadic_args = arity.variadic().map(|_| 1).unwrap_or(0);
        let num_args = num_positional_args + num_variadic_args;
        let arg_types = (0..num_args).map(|_| ValueType::HeapPointer);
        Ok(stack
            // Pop the function arguments from the operand stack
            .pop_operands(&ParamsSignature::from_iter(arg_types))?
            // Push the function result onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for CallStdlib {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { target } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalGet {
            local: bindings.state_id(),
        });
        instructions.push(ir::Call {
            func: bindings.get_stdlib_function_id(*target),
        });
        instructions.push(ir::LocalGet {
            local: bindings.dependencies_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().combine_dependencies,
        });
        instructions.push(ir::LocalSet {
            local: bindings.dependencies_id(),
        });
        Ok(instructions)
    }
}

/// Invoke a user-defined function known at compile-time, popping the required number of arguments from the operand stack
/// (arguments are passed to the function in the same order they were added to the operand stack, i.e. not reversed)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CallCompiledFunction {
    /// Type signature of the target function
    pub signature: TypeSignature,
    /// ID of the target function
    pub target: CompiledFunctionId,
}

impl TypedCompilerBlock for CallCompiledFunction {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { signature, .. } = self;
        let TypeSignature { params, results } = signature;
        Ok(stack
            // Pop the function arguments from the operand stack
            .pop_operands(params)?
            // Push the function results onto the operand stack
            .push_operands(results))
    }
}

impl GenerateWasm for CallCompiledFunction {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { target, .. } = self;
        match bindings.get_compiled_function_id(*target) {
            None => Err(WasmGeneratorError::InvalidCompiledFunction(*target)),
            Some(function_id) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.state_id(),
                });
                instructions.push(ir::Call { func: function_id });
                instructions.push(ir::LocalGet {
                    local: bindings.dependencies_id(),
                });
                instructions.push(ir::Call {
                    func: bindings.builtins().combine_dependencies,
                });
                instructions.push(ir::LocalSet {
                    local: bindings.dependencies_id(),
                });
                Ok(instructions)
            }
        }
    }
}

/// Pop the argument list term pointer from the operand stack,
/// then pop the target function index from the operand stack,
/// then invoke the corresponding function, pushing the result onto the operand stack
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CallDynamic {
    /// Type signature of the target function
    pub signature: TypeSignature,
}

impl TypedCompilerBlock for CallDynamic {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        Ok(stack
            // Pop the target function index and argument list term pointer from the operand stack
            .pop_operands(&ParamsSignature::Pair(
                ValueType::U32,
                ValueType::HeapPointer,
            ))?
            // Push the function result onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for CallDynamic {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { signature } = self;
        let (params, results) = parse_function_type_signature(signature);
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalGet {
            local: bindings.state_id(),
        });
        instructions.push(ir::CallIndirect {
            ty: module.types.add(&params, &results),
            table: bindings.main_function_table_id(),
        });
        instructions.push(ir::LocalGet {
            local: bindings.dependencies_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().combine_dependencies,
        });
        instructions.push(ir::LocalSet {
            local: bindings.dependencies_id(),
        });
        Ok(instructions)
    }
}

/// Pop the top item of the operand stack, evaluate it, and push the result onto the operand stack.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Evaluate;

impl TypedCompilerBlock for Evaluate {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        Ok(stack
            // Pop the target term pointer from the operand stack
            .pop_operand(ValueType::HeapPointer)?
            // Push the evaluation result term pointer onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for Evaluate {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalGet {
            local: bindings.state_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().evaluate,
        });
        instructions.push(ir::LocalGet {
            local: bindings.dependencies_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().combine_dependencies,
        });
        instructions.push(ir::LocalSet {
            local: bindings.dependencies_id(),
        });
        Ok(instructions)
    }
}

/// Pop the argument list term pointer from the operand stack,
/// then pop the application target term pointer from the operand stack,
/// then apply the corresponding target term to the arguments list, pushing the result onto the operand stack
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Apply;

impl TypedCompilerBlock for Apply {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        Ok(stack
            // Pop the application target term pointer and argument list term pointer from the operand stack
            .pop_operands(&ParamsSignature::Pair(
                ValueType::HeapPointer,
                ValueType::HeapPointer,
            ))?
            // Push the application result onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for Apply {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalGet {
            local: bindings.state_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().apply,
        });
        instructions.push(ir::LocalGet {
            local: bindings.dependencies_id(),
        });
        instructions.push(ir::Call {
            func: bindings.builtins().combine_dependencies,
        });
        instructions.push(ir::LocalSet {
            local: bindings.dependencies_id(),
        });
        Ok(instructions)
    }
}

// TODO: remove unused runtime::CollectSignals bytecode instruction
/// Pop the specified number of term pointers from the top of the operand stack,
/// and push with a single combined signal term pointer onto top of the operand stack (in the case where one or moreof the operands is a signal)
/// or a null pointer (in the case where none of the terms is a signal)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CollectSignals {
    /// Number of operand stack values to test for signals
    pub count: usize,
    /// Whether to preserve the original values on the operand stack ('peek' mode) or remove them ('pop' mode)
    pub retain_existing_items: bool,
}

impl TypedCompilerBlock for CollectSignals {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self {
            count,
            retain_existing_items,
        } = self;
        // Ensure the required number of term pointers exists on the operand stack
        let expected_operands =
            ParamsSignature::from_iter(repeat(ValueType::HeapPointer).take(*count));
        let stack = if *retain_existing_items {
            stack.assert_operands(&expected_operands)
        } else {
            stack.assert_operands(&expected_operands)
        }?;
        Ok(stack
            // Push the combined signal result onto the operand stack
            .push_operand(ValueType::HeapPointer))
    }
}

impl GenerateWasm for CollectSignals {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self {
            count,
            retain_existing_items,
        } = self;
        match *count {
            // If there are no signals to test, nothing more to do
            0 => Ok(Default::default()),
            1 => {
                // The single-signal use case is slightly simpler as there is no need to accumulate the combined signal
                let instructions = {
                    let mut instructions = CompiledBlock::default();
                    // Pop the term pointer from the top of the stack and assign it as a temporary lexical scope variable
                    // => []
                    instructions.push(instruction::core::ScopeStart {
                        value_type: ValueType::HeapPointer,
                    });
                    // If the original stack items are to be preserved on the stack, push the term pointer back onto the stack
                    // (this ensures the original stack item remains on the stack beneath the optional signal result)
                    // => [Term]
                    if *retain_existing_items {
                        instructions.push(instruction::core::GetScopeValue {
                            scope_offset: 0,
                            value_type: ValueType::HeapPointer,
                        });
                    }
                    // Push another copy of the term pointer back onto the stack
                    // (this will be used as the 'true' branch of the select instruction)
                    // => [Term, Term]
                    instructions.push(instruction::core::GetScopeValue {
                        scope_offset: 0,
                        value_type: ValueType::HeapPointer,
                    });
                    // Push a null pointer onto the stack
                    // (this will be used as the 'false' branch of the select instruction)
                    // => [Term, Term, NULL]
                    instructions.push(instruction::runtime::NullPointer);
                    // Push another copy of the term pointer onto the stack
                    // (this will be used for testing whether the term is a signal)
                    // => [Term, Term, NULL, Term]
                    instructions.push(instruction::core::GetScopeValue {
                        scope_offset: 0,
                        value_type: ValueType::HeapPointer,
                    });
                    // Determine whether the term is a signal
                    // (this will be used as the 'condition' input to the select instruction)
                    // => [Term, Term, NULL, bool]
                    instructions.push(instruction::runtime::CallRuntimeBuiltin {
                        target: RuntimeBuiltin::IsSignal,
                    });
                    // Select either the term pointer (in the case where it is a signal term)
                    // or the null pointer (in the case where it is not a signal term)
                    // => [Term, Option<SignalTerm>]
                    instructions.push(instruction::core::Select {
                        value_type: ValueType::HeapPointer,
                    });
                    // Dispose of the temporary lexical scope
                    // => [Term, Option<SignalTerm>]
                    instructions.push(instruction::core::ScopeEnd {
                        value_type: ValueType::HeapPointer,
                    });
                    instructions
                };
                instructions.emit_wasm(module, bindings, options)
            }
            count => {
                // Accumulate multiple signals into a single combined signal term
                let instructions = {
                    let mut instructions = CompiledBlock::default();
                    // Pop each of the term pointers from the top of the operand stack and assign to temporary lexical scope variables
                    // => []
                    for _ in 0..count {
                        instructions.push(instruction::core::ScopeStart {
                            value_type: ValueType::HeapPointer,
                        });
                    }
                    // Now that the term pointers have been captured in lexical scopes,
                    // if the original items are to be preserved on the stack,
                    // push each term pointer back onto the stack in the original order
                    // (this ensures the values remain on the operand stack beneath the combined signal)
                    // => [Term...]
                    if *retain_existing_items {
                        for scope_offset in (0..count).rev() {
                            instructions.push(instruction::core::GetScopeValue {
                                scope_offset: scope_offset,
                                value_type: ValueType::HeapPointer,
                            });
                        }
                    }
                    // Push a null pointer onto the operand stack to be used as the combined signal accumulator
                    // => [Term..., NULL]
                    instructions.push(instruction::runtime::NullPointer);
                    // For each of the captured terms in turn, push the term onto the stack,
                    // test whether it is a signal, and combine any signal terms with the accumulated signal,
                    // leaving the accumulator on top of the stack throughout the iteration
                    // => [Term..., Option<SignalTerm>]
                    for scope_offset in (0..count).rev() {
                        // Push the current term onto the top of the operand stack
                        // (this will be used as the 'true' branch of the select instruction)
                        // [Term..., Option<SignalTerm>, Term]
                        instructions.push(instruction::core::GetScopeValue {
                            scope_offset,
                            value_type: ValueType::HeapPointer,
                        });
                        // Push a null pointer onto the stack
                        // (this will be used as the 'false' branch of the select instruction)
                        // => [Term..., Option<SignalTerm>, Term, NULL]
                        instructions.push(instruction::runtime::NullPointer);
                        // Push another copy of the term pointer onto the stack
                        // (this will be used for testing whether the term is a signal)
                        // => [Term..., Option<SignalTerm>, Term, NULL, Term]
                        instructions.push(instruction::core::GetScopeValue {
                            scope_offset,
                            value_type: ValueType::HeapPointer,
                        });
                        // Determine whether the term is a signal
                        // (this will be used as the 'condition' input to the select instruction)
                        // => [Term..., Option<SignalTerm>, Term, NULL, bool]
                        instructions.push(instruction::runtime::CallRuntimeBuiltin {
                            target: RuntimeBuiltin::IsSignal,
                        });
                        // Select either the term pointer (in the case where it is a signal term)
                        // or the null pointer (in the case where it is not a signal term)
                        // => [Term..., Option<SignalTerm>, Option<SignalTerm>]
                        instructions.push(instruction::core::Select {
                            value_type: ValueType::HeapPointer,
                        });
                        // Combine the result with the accumulated signal result
                        // => [Term..., Option<SignalTerm>]
                        instructions.push(instruction::runtime::CallRuntimeBuiltin {
                            target: RuntimeBuiltin::CombineSignals,
                        });
                    }
                    // Dispose of the temporary term pointer lexical scopes
                    // => [Term..., Option<SignalTerm>]
                    for _ in 0..count {
                        instructions.push(instruction::core::ScopeEnd {
                            value_type: ValueType::HeapPointer,
                        });
                    }
                    instructions
                };
                instructions.emit_wasm(module, bindings, options)
            }
        }
    }
}

/// Peek at the term pointer on top of the operand stack, and if it is a signal,
/// break out of the specified control flow block
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BreakOnSignal {
    /// Index of the control flow block to break out of
    /// (where `0` is the current control flow block, `1` is the immediate parent of the current control flow block, etc)
    pub target_block: usize,
}

impl TypedCompilerBlock for BreakOnSignal {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { target_block } = self;
        // Ensure there is a term pointer on top of the operand stack
        let stack = stack.assert_operand(ValueType::HeapPointer)?;
        // Validate the 'break' branch for type errors
        let _ = instruction::core::Break {
            target_block: *target_block,
            result_type: ParamsSignature::Single(ValueType::HeapPointer),
        }
        .get_type(&stack)?;
        // Continue with the current branch with the existing stack
        Ok(stack.clone())
    }
}

impl GenerateWasm for BreakOnSignal {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { target_block } = self;
        let instructions = {
            let mut instructions = CompiledBlock::default();
            // Duplicate the term pointer on top of the stack
            // => [Term, Term]
            instructions.push(instruction::core::Duplicate {
                value_type: ValueType::HeapPointer,
            });
            // Invoke the builtin function to determine whether the value is a signal
            // => [Term, bool]
            instructions.push(instruction::runtime::CallRuntimeBuiltin {
                target: RuntimeBuiltin::IsSignal,
            });
            // Short circuit if a signal term was encountered
            // => [Term]
            instructions.push(instruction::core::ConditionalBreak {
                target_block: *target_block,
                result_type: ParamsSignature::Single(ValueType::HeapPointer),
            });
            instructions
        };
        instructions.emit_wasm(module, bindings, options)
    }
}
