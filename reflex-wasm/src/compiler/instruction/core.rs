// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use walrus::{
    ir::{self, BinaryOp, LoadKind, MemArg, StoreKind, Value},
    Module,
};

use crate::{
    compiler::{
        error::TypedStackError,
        wasm::{
            generate::{
                GenerateWasm, WasmGeneratorBindings, WasmGeneratorError, WasmGeneratorOptions,
                WasmGeneratorOutput, WasmGeneratorResult,
            },
            types::parse_value_type,
        },
        CompiledBlock, CompiledBlockBuilder, CompilerStack, ConstValue, FunctionPointer,
        ParamsSignature, TypeSignature, TypedCompilerBlock, ValueType,
    },
    utils::{from_twos_complement_i32, from_twos_complement_i64},
};

/// Push a constant value onto the operand stack
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Const {
    /// Value to push onto the operand stack
    pub value: ConstValue,
}

impl TypedCompilerBlock for Const {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value } = self;
        Ok(stack
            // Push a value of the given type onto the operand stack
            .push_operand(value.get_type()))
    }
}

impl GenerateWasm for Const {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Const {
            value: match value {
                ConstValue::I32(value) => Value::I32(*value),
                ConstValue::U32(value) => Value::I32(from_twos_complement_i32(*value)),
                ConstValue::I64(value) => Value::I64(*value),
                ConstValue::U64(value) => Value::I64(from_twos_complement_i64(*value)),
                ConstValue::F32(value) => Value::F32(*value),
                ConstValue::F64(value) => Value::F64(*value),
                ConstValue::HeapPointer(value) => {
                    Value::I32(from_twos_complement_i32(u32::from(*value)))
                }
                ConstValue::FunctionPointer(value) => {
                    let function_index = match value {
                        FunctionPointer::Stdlib(target) => {
                            Ok(bindings.get_stdlib_indirect_call_function_index(*target))
                        }
                        FunctionPointer::Lambda(target_hash) => bindings
                            .get_compiled_function_indirect_call_function_index(*target_hash)
                            .ok_or_else(|| {
                                WasmGeneratorError::InvalidCompiledFunction(*target_hash)
                            }),
                    }?;
                    Value::I32(from_twos_complement_i32(u32::from(function_index)))
                }
            },
        });
        Ok(instructions)
    }
}

/// Duplicate the value at the top of the operand stack
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Duplicate {
    /// Type of the value being duplicated on the operand stack
    pub value_type: ValueType,
}

impl TypedCompilerBlock for Duplicate {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Ensure the value on top of the stack is of the expected type
            .assert_operand(*value_type)?
            // Push a copy of the value onto the stack
            .push_operand(*value_type))
    }
}

impl GenerateWasm for Duplicate {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalTee {
            local: bindings.temp_id(),
        });
        instructions.push(ir::LocalGet {
            local: bindings.temp_id(),
        });
        Ok(instructions)
    }
}

/// Pop the term at the top of the operand stack and discard it
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Drop {
    /// Type of the value being dropped from the operand stack
    pub value_type: ValueType,
}

impl TypedCompilerBlock for Drop {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Drop the top value from the operand stack
            .pop_operand(*value_type)?)
    }
}

impl GenerateWasm for Drop {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        _bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Drop {});
        Ok(instructions)
    }
}

/// Pop the top item of the operand stack and enter a new lexical scope whose variable is assigned to that value
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct ScopeStart {
    /// Type of the variable declared by the lexical scope
    pub value_type: ValueType,
}

impl TypedCompilerBlock for ScopeStart {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Pop the top value off the operand stack to use as the variable value
            .pop_operand(*value_type)?
            // Enter a new lexical scope with the correct variable type
            .enter_scope(*value_type))
    }
}

impl GenerateWasm for ScopeStart {
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

/// Pop the latest lexical scope
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct ScopeEnd {
    /// Type of the variable declared by the lexical scope
    /// (this must match the corresponding scope start instruction)
    pub value_type: ValueType,
}

impl TypedCompilerBlock for ScopeEnd {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Exit the current lexical scope, ensuring the lexical scope variable is of the expected type
            .leave_scope(*value_type)?)
    }
}

impl GenerateWasm for ScopeEnd {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        match bindings.leave_scope() {
            Some(_) => Ok(Default::default()),
            None => Err(WasmGeneratorError::StackError),
        }
    }
}

/// Push a variable defined in a containing lexical scope onto the operand stack
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct GetScopeValue {
    /// Type of the variable declared by the lexical scope
    /// (this must match the corresponding lexical scope's declaration)
    pub value_type: ValueType,
    /// Offset of the target lexical scope
    /// (where `0` is the current lexical scope, `1` is the immediate parent of the current lexical scope, etc)
    pub scope_offset: usize,
}

impl TypedCompilerBlock for GetScopeValue {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self {
            value_type,
            scope_offset,
        } = self;
        Ok(stack
            // Ensure the target lexical scope has the correct variable type
            .assert_lexical_scope(*scope_offset, *value_type)?
            // Push the variable onto the operand stack
            .push_operand(*value_type))
    }
}

impl GenerateWasm for GetScopeValue {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { scope_offset, .. } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::LocalGet {
            local: bindings.get_local(*scope_offset)?,
        });
        Ok(instructions)
    }
}

/// Enter a new control flow block
/// (this allows child instructions to break out of the block)
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Block {
    /// Description of operand stack items to inject into the block, along with the result type of the block
    pub block_type: TypeSignature,
    /// Instructions to execute within the block
    // TODO: Consider flattening nested blocks to use Block/End
    pub body: CompiledBlock,
}

impl TypedCompilerBlock for Block {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { block_type, body } = self;
        // Validate the block contents for type errors
        let _ = validate_block(block_type, body, stack)?;
        // Pop the block parameters from the stack and push the block results onto the stack
        let stack = stack
            .pop_operands(&block_type.params)?
            .push_operands(&block_type.results);
        Ok(stack)
    }
}

impl GenerateWasm for Block {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { block_type, body } = self;
        let body_instructions = body.emit_wasm(module, bindings, options)?;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.block(block_type, body_instructions, module, bindings, options);
        Ok(instructions)
    }
}

/// Unconditionally break out of the specified control flow block
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Break {
    /// Index of the control flow block to break out of
    /// (where `0` is the current control flow block, `1` is the immediate parent of the current control flow block, etc)
    pub target_block: usize,
    /// Description of operand stack items to return from the block
    /// (this must match the corresponding target block start instruction)
    pub result_type: ParamsSignature,
}

impl TypedCompilerBlock for Break {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self {
            target_block,
            result_type,
        } = self;
        // Dispose of any intermediate blocks between the current block and the target block
        (0..*target_block)
            .fold(Ok(stack.clone()), |results, _| {
                let stack = results?;
                // We don't care about the result types of the intermediate blocks, so no need to type-check them properly
                let dummy_result_type = ParamsSignature::Void;
                let intermediate_block_result_type =
                    stack.active_block().unwrap_or(&dummy_result_type);
                // Exit the intermediate block, passing whatever its type is to ensure stack type-checking will pass
                stack.leave_block(intermediate_block_result_type)
            })?
            // Ensure the correct values are on the operand stack to exit the target block
            .assert_operands(result_type)?
            // Exit the target block
            .leave_block(result_type)
    }
}

impl GenerateWasm for Break {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        _bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { target_block, .. } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.br(*target_block);
        Ok(instructions)
    }
}

/// Pop the top item from the operand stack, and if the value is not `0` then break out of the specified control flow block, otherwise continue with the current block
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct ConditionalBreak {
    /// Index of the control flow block to break out of
    /// (where `0` is the current control flow block, `1` is the immediate parent of the current control flow block, etc)
    pub target_block: usize,
    /// Description of operand stack items to return from the block
    /// (this must match the corresponding target block start instruction)
    pub result_type: ParamsSignature,
}

impl TypedCompilerBlock for ConditionalBreak {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self {
            target_block,
            result_type,
        } = self;
        // Pop the condition value from the stack
        let stack = stack.pop_operand(ValueType::U32)?;
        // Validate the 'break' branch for type errors
        let _ = Break {
            target_block: *target_block,
            result_type: result_type.clone(),
        }
        .get_type(&stack)?;
        // Continue with the current branch with the existing stack
        Ok(stack)
    }
}

impl GenerateWasm for ConditionalBreak {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        _bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { target_block, .. } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.br_if(*target_block);
        Ok(instructions)
    }
}

/// Pop the top item of the operand stack, and if the value is not `0` then enter the `consequent` block, otherwise enter the `alternative` block
/// (note that the `consequent` and `alternative` blocks count towards the target block offset when breaking out of parent control flow blocks)
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct If {
    /// Description of operand stack items to inject into the consequent/alternative blocks, along with the result type of the blocks
    /// (both blocks must have the same type)
    pub block_type: TypeSignature,
    /// Control flow block to execute if the condition is satisfied
    // TODO: Consider flattening nested blocks to use If/Else/EndIf
    pub consequent: CompiledBlock,
    /// Control flow block to execute if the condition is unsatisfied
    // TODO: Consider flattening conditional nested blocks to use If/Else/EndIf
    pub alternative: CompiledBlock,
}

impl TypedCompilerBlock for If {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self {
            block_type,
            consequent,
            alternative,
        } = self;
        // Validate both branches for type errors
        let _ = validate_block(block_type, consequent, stack)?;
        let _ = validate_block(block_type, alternative, stack)?;
        let stack = stack
            // Pop the condition from the stack
            .pop_operand(ValueType::U32)?
            // Pop the block parameters from the stack
            .pop_operands(&block_type.params)?
            // Push the block results onto the stack
            .push_operands(&block_type.results);
        Ok(stack)
    }
}

impl GenerateWasm for If {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self {
            block_type,
            consequent,
            alternative,
        } = self;
        let consequent_block = consequent.emit_wasm(module, bindings, options)?;
        let alternative_block = alternative.emit_wasm(module, bindings, options)?;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.if_else(
            block_type,
            consequent_block,
            alternative_block,
            module,
            bindings,
            options,
        );
        Ok(instructions)
    }
}

/// Pop the top three values from the operand stack, and if the top item is not `0`, push the bottom value back onto the stack, otherwise push the middle value onto the stack
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Select {
    /// Type of the value to select from the stack
    /// (this must match the corresponding value on the operand stack)
    pub value_type: ValueType,
}

impl TypedCompilerBlock for Select {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Pop the input values from the operand stack
            .pop_operands(&ParamsSignature::Triple(
                *value_type,
                *value_type,
                ValueType::U32,
            ))?
            // Push the result back onto the operand stack
            .push_operand(*value_type))
    }
}

impl GenerateWasm for Select {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        _bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value_type } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Select {
            ty: Some(parse_value_type(*value_type)),
        });
        Ok(instructions)
    }
}

/// Pop the top two values from the operand stack, and push a constant `1` if they are equal, or `0` if not
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Eq {
    /// Type of the values to compare for equality
    /// (this must match the corresponding values on the operand stack)
    pub value_type: ValueType,
}

impl TypedCompilerBlock for Eq {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Pop the input values from the operand stack
            .pop_operands(&ParamsSignature::Pair(*value_type, *value_type))?
            // Push the result onto the operand stack
            .push_operand(ValueType::U32))
    }
}

impl GenerateWasm for Eq {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        _bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value_type } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Binop {
            op: match value_type {
                ValueType::I32
                | ValueType::U32
                | ValueType::HeapPointer
                | ValueType::FunctionPointer => BinaryOp::I32Eq,
                ValueType::I64 | ValueType::U64 => BinaryOp::I64Eq,
                ValueType::F32 => BinaryOp::F32Eq,
                ValueType::F64 => BinaryOp::F64Eq,
            },
        });
        Ok(instructions)
    }
}

/// Pop the top two values from the operand stack, and push a constant `0` if they are equal, or `1` if not
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Ne {
    /// Type of the values to compare for equality
    /// (this must match the corresponding values on the operand stack)
    pub value_type: ValueType,
}

impl TypedCompilerBlock for Ne {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            // Pop the input values from the operand stack
            .pop_operands(&ParamsSignature::Pair(*value_type, *value_type))?
            // Push the result onto the operand stack
            .push_operand(ValueType::U32))
    }
}

impl GenerateWasm for Ne {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        _bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value_type } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Binop {
            op: match value_type {
                ValueType::I32
                | ValueType::U32
                | ValueType::HeapPointer
                | ValueType::FunctionPointer => BinaryOp::I32Ne,
                ValueType::I64 | ValueType::U64 => BinaryOp::I64Ne,
                ValueType::F32 => BinaryOp::F32Ne,
                ValueType::F64 => BinaryOp::F64Ne,
            },
        });
        Ok(instructions)
    }
}

/// Pop a target heap pointer address from the operand stack, read the value at that offset within the heap memory and push the value onto the operand stack
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct ReadHeapValue {
    /// Type of the value being read from the heap
    pub value_type: ValueType,
}

impl TypedCompilerBlock for ReadHeapValue {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack.push_operand(*value_type))
    }
}

impl GenerateWasm for ReadHeapValue {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value_type } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Load {
            memory: bindings.memory_id(),
            kind: match value_type {
                ValueType::I32
                | ValueType::U32
                | ValueType::HeapPointer
                | ValueType::FunctionPointer => LoadKind::I32 {
                    atomic: Default::default(),
                },
                ValueType::I64 | ValueType::U64 => LoadKind::I64 {
                    atomic: Default::default(),
                },
                ValueType::F32 => LoadKind::F32,
                ValueType::F64 => LoadKind::F64,
            },
            arg: MemArg {
                align: Default::default(),
                offset: Default::default(),
            },
        });
        Ok(instructions)
    }
}

/// Pop a value from the operand stack, then pop the target heap pointer address from the operand stack, then write the value to that offset in the heap memory
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct WriteHeapValue {
    /// Type of the value being written to the heap
    pub value_type: ValueType,
}

impl GenerateWasm for WriteHeapValue {
    fn emit_wasm(
        &self,
        _module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        _options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let Self { value_type } = self;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.push(ir::Store {
            memory: bindings.memory_id(),
            kind: match value_type {
                ValueType::I32
                | ValueType::U32
                | ValueType::HeapPointer
                | ValueType::FunctionPointer => StoreKind::I32 {
                    atomic: Default::default(),
                },
                ValueType::I64 | ValueType::U64 => StoreKind::I64 {
                    atomic: Default::default(),
                },
                ValueType::F32 => StoreKind::F32,
                ValueType::F64 => StoreKind::F64,
            },
            arg: MemArg {
                align: Default::default(),
                offset: Default::default(),
            },
        });
        Ok(instructions)
    }
}

impl TypedCompilerBlock for WriteHeapValue {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        let Self { value_type } = self;
        Ok(stack
            .pop_operand(*value_type)?
            .pop_operand(ValueType::HeapPointer)?
            .push_operand(*value_type))
    }
}

fn validate_block(
    block_type: &TypeSignature,
    instructions: &CompiledBlock,
    parent_stack: &CompilerStack,
) -> Result<CompilerStack, TypedStackError> {
    // Enter a new control flow block with the expected block type, capturing the block params from the existing stack
    let inner_stack = parent_stack.enter_block(block_type)?;
    let block = CompiledBlockBuilder::new(inner_stack);
    // Inject the instructions into the block
    let block = block.append_block(instructions.clone());
    // Finish the block and extract the resulting stack
    let (_, stack) = block.into_parts()?;
    let stack = stack
        // Ensure the correct values are on the operand stack to exit the block
        .assert_operands(&block_type.results)?
        // Exit the branch block
        .leave_block(&block_type.results)?;
    Ok(stack)
}
