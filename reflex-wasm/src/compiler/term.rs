// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::Eagerness;
use walrus::ir::{Const, Instr, Value};

use crate::{
    allocator::Arena,
    compiler::{
        CompileWasm, CompiledExpression, CompiledInstruction, CompilerOptions, CompilerResult,
        CompilerState, RuntimeBuiltin,
    },
    term_type::*,
    ArenaRef,
};

impl<A: Arena + Clone> CompileWasm for ArenaRef<IntTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.value()),
        })));

        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInt,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<FloatTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::F64(self.value()),
        })));

        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFloat,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<BooleanTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.value() as i32),
        })));

        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBoolean,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ListTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        // Get the list capacity -> Capacity
        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.len() as i32),
        })));

        // Allocate the list -> List
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::AllocateList,
        ));

        for (idx, item) in self.iter().enumerate() {
            // Duplicate the list pointer -> List List
            instructions.push_back(CompiledInstruction::Duplicate);
            // Push the item index -> List List Idx
            instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
                value: Value::I32(idx as i32),
            })));

            // Compile the child item -> List List Idx Item
            instructions.extend(item.compile(eager, state, options)?);
            // Set the item to the index -> List
            instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::SetListItem,
            ));
        }

        // Instantiate List with correct length -> List Length
        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.len() as i32),
        })));

        // Initialize the list term -> List
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::InitList,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<BuiltinTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        // Put the builtin term's number onto the stack
        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(u32::from(self.target()) as i32),
        })));

        // Create builtin term
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBuiltin,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ApplicationTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        // Compile the target
        instructions.extend(self.target().compile(eager, state, options)?);

        // Compile the args list
        instructions.extend(self.args().as_term().compile(eager, state, options)?);

        // Create application term
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateApplication,
        ));

        Ok(instructions)
    }
}
