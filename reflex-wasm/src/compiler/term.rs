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

impl<A: Arena + Clone> CompileWasm for ArenaRef<ApplicationTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();
        // Push the application target onto the stack
        // Resulting stack state: [Term]
        instructions.extend(self.target().compile(eager, state, options)?);
        // Push the application arguments onto the stack
        // Resulting stack state: [Term, ListTerm]
        instructions.extend(self.args().as_term().compile(eager, state, options)?);
        // Invoke the term constructor
        // Resulting stack state: [ApplicationTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateApplication,
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
        // Push the value argument onto the stack
        // Resulting stack state: [value]
        instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.value() as i32),
        })));
        // Invoke the term constructor
        // Resulting stack state: [BooleanTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBoolean,
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
        // Push the function index argument onto the stack
        // Resulting stack state: [index]
        instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(u32::from(self.target()) as i32),
        })));
        // Invoke the term constructor
        // Resulting stack state: [BuiltinTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBuiltin,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<CellTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<CompiledTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ConditionTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ConstructorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<DateTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<EffectTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
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
        // Push the value argument onto the stack
        // Resulting stack state: [value]
        instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::F64(self.value()),
        })));
        // Invoke the term constructor
        // Resulting stack state: [FloatTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFloat,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<HashmapTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<HashsetTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<IntTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // Resulting stack state: [value]
        instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.value()),
        })));
        // Invoke the term constructor
        // Resulting stack state: [IntTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInt,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<LambdaTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<LetTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
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
        // Push the list capacity onto the stack
        // Resulting stack state: [capacity]
        instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.len() as i32),
        })));
        // Allocate the list term
        // Resulting stack state: [ListTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::AllocateList,
        ));
        // Allocate the list items
        for (idx, item) in self.iter().enumerate() {
            // Duplicate the list term pointer for use later
            // Resulting stack state: [ListTerm, ListTerm]
            instructions.push(CompiledInstruction::Duplicate);
            // Push the item index
            // Resulting stack state: [ListTerm, ListTerm, index]
            instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
                value: Value::I32(idx as i32),
            })));
            // Allocate the child item
            // Resulting stack state: [ListTerm, ListTerm, index, Term]
            instructions.extend(item.compile(eager, state, options)?);
            // Set the list term's value at the given index to the child item
            // Resulting stack state: [ListTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::SetListItem,
            ));
        }
        // Now that all the items have been added, push the list length onto the stack
        // Resulting stack state: [ListTerm, length]
        instructions.push(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.len() as i32),
        })));
        // Initialize the list term with the length that is on the stack
        // Resulting stack state: [ListTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::InitList,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<NilTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<PartialTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<PointerTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<RecordTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<SignalTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<StringTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<SymbolTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<TreeTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<VariableTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<EmptyIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<EvaluateIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<FilterIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<FlattenIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<HashmapValuesIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<IntegersIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<IntersperseIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<MapIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<OnceIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<RangeIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<RepeatIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<SkipIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<TakeIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ZipIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        todo!()
    }
}
