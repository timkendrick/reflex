// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use wasmtime::{Engine, ExternType, Instance, IntoFunc, Linker, Memory, Module, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use crate::{
    allocator::ArenaAllocator,
    pad_to_4_byte_offset,
    term_type::{TreeTerm, TypedTerm},
    ArenaRef, Term, TermPointer,
};

/// The 64kB page size constant mentioned in the WASM specification.
const WASM_PAGE_SIZE: u64 = 0x10000;

pub struct UnboundEvaluationResult {
    result_pointer: TermPointer,
    dependencies_pointer: Option<TermPointer>,
}

impl UnboundEvaluationResult {
    pub fn bind<'heap, A: ArenaAllocator>(
        self,
        arena: &'heap A,
    ) -> InterpreterEvaluationResult<'heap, A> {
        InterpreterEvaluationResult {
            arena: arena,
            result_pointer: self.result_pointer,
            dependencies_pointer: self.dependencies_pointer,
        }
    }
}

pub struct InterpreterEvaluationResult<'heap, A: ArenaAllocator> {
    arena: &'heap A,
    result_pointer: TermPointer,
    dependencies_pointer: Option<TermPointer>,
}

impl<'heap, A: ArenaAllocator> InterpreterEvaluationResult<'heap, A> {
    pub fn result(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get::<Term>(self.result_pointer))
    }

    pub fn dependencies(&self) -> Option<ArenaRef<'heap, TypedTerm<TreeTerm>, A>> {
        self.dependencies_pointer.map(|dependencies_pointer| {
            ArenaRef::new(
                self.arena,
                self.arena.get::<TypedTerm<TreeTerm>>(dependencies_pointer),
            )
        })
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    ImportNotFound((String, String, ExternType)),
    FunctionNotFound(anyhow::Error),
    WasiContexBuild(wasi_common::StringArrayError),
    Linking(anyhow::Error),
    ModuleCreation(anyhow::Error),
    LinearMemoryNotFound,
    FunctionEvaluation(anyhow::Error),
}

impl From<wasi_common::StringArrayError> for InterpreterError {
    fn from(value: wasi_common::StringArrayError) -> Self {
        InterpreterError::WasiContexBuild(value)
    }
}

pub struct WasmContextBuilder {
    store: Store<WasiCtx>,
    linker: Linker<WasiCtx>,
    module: Module,
}

impl WasmContextBuilder {
    pub fn new(program_bytes: &[u8]) -> Result<Self, InterpreterError> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();

        let engine = Engine::default();
        let store = Store::new(&engine, wasi);
        let mut linker = Linker::new(store.engine());

        wasmtime_wasi::add_to_linker(&mut linker, |s| s).map_err(InterpreterError::Linking)?;

        let module = unsafe {
            Module::deserialize(store.engine(), program_bytes)
                .map_err(InterpreterError::ModuleCreation)?
        };

        Ok(Self {
            store,
            linker,
            module,
        })
    }

    pub fn add_import<F, Params, Args>(
        mut self,
        module: &str,
        name: &str,
        func: F,
    ) -> Result<Self, InterpreterError>
    where
        F: IntoFunc<WasiCtx, Params, Args>,
    {
        self.linker
            .func_wrap(module, name, func)
            .map_err(InterpreterError::Linking)?;

        Ok(self)
    }

    pub fn build(mut self) -> Result<WasmContext, InterpreterError> {
        let instance = self
            .linker
            .instantiate(&mut self.store, &self.module)
            .map_err(InterpreterError::Linking)?;

        instance
            .get_typed_func::<(), ()>(&mut self.store, "_initialize")
            .map_err(InterpreterError::FunctionNotFound)?
            .call(&mut self.store, ())
            .map_err(InterpreterError::FunctionEvaluation)?;

        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or(InterpreterError::LinearMemoryNotFound)?;

        Ok(WasmContext {
            store: self.store,
            program: instance,
            memory,
        })
    }
}

pub struct WasmContext {
    store: Store<WasiCtx>,
    program: Instance,
    memory: Memory,
}

pub struct WasmInterpreter(WasmContext);

impl From<WasmContext> for WasmInterpreter {
    fn from(value: WasmContext) -> Self {
        Self(value)
    }
}

impl WasmInterpreter {
    pub fn interpret<'a>(
        &'a mut self,
        input: TermPointer,
        state: TermPointer,
    ) -> Result<UnboundEvaluationResult, InterpreterError> {
        let eval_func = self
            .0
            .program
            .get_typed_func::<(u32, u32), (u32, u32)>(&mut self.0.store, "evaluate")
            .map_err(InterpreterError::FunctionNotFound)?;

        let (result, dependencies) = eval_func
            .call(&mut self.0.store, (input.into(), state.into()))
            .map_err(InterpreterError::FunctionEvaluation)?;

        Ok(UnboundEvaluationResult {
            result_pointer: result.into(),
            dependencies_pointer: TermPointer::from(dependencies).as_non_null(),
        })
    }
}

impl ArenaAllocator for WasmInterpreter {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn allocate<T: crate::hash::TermSize>(&mut self, value: T) -> TermPointer {
        self.0.allocate(value)
    }

    fn get<T>(&self, offset: TermPointer) -> &T {
        self.0.get(offset)
    }

    fn get_mut<T>(&mut self, offset: TermPointer) -> &mut T {
        self.0.get_mut(offset)
    }

    fn slice<T: Sized>(&self, offset: TermPointer, count: usize) -> &[T] {
        self.0.slice(offset, count)
    }

    fn extend(&mut self, offset: TermPointer, size: usize) {
        self.0.extend(offset, size)
    }

    fn shrink(&mut self, offset: TermPointer, size: usize) {
        self.0.shrink(offset, size)
    }
}

impl ArenaAllocator for WasmContext {
    fn len(&self) -> usize {
        *self.get::<u32>(0.into()) as usize
    }

    fn allocate<T: crate::hash::TermSize>(&mut self, value: T) -> crate::TermPointer {
        let offset = TermPointer(self.len() as u32);
        let static_size = pad_to_4_byte_offset(std::mem::size_of::<T>());
        let actual_size = pad_to_4_byte_offset(value.size_of());
        self.extend(offset, static_size);
        let target = self.get_mut(offset);
        *target = value;
        if actual_size < static_size {
            self.shrink(offset.offset(static_size as u32), static_size - actual_size);
        }
        TermPointer::from(offset)
    }

    fn get<T>(&self, offset: crate::TermPointer) -> &T {
        let data = self.memory.data(&self.store);
        let offset = u32::from(offset) as usize;
        let item = &data[offset];
        unsafe { std::mem::transmute::<&u8, &T>(item) }
    }

    fn get_mut<T>(&mut self, offset: crate::TermPointer) -> &mut T {
        let data = self.memory.data_mut(&mut self.store);
        let offset = u32::from(offset) as usize;
        let item = &mut data[offset];
        unsafe { std::mem::transmute::<&mut u8, &mut T>(item) }
    }

    fn slice<T: Sized>(&self, offset: crate::TermPointer, count: usize) -> &[T] {
        let data = self.memory.data(&self.store);
        let offset = u32::from(offset) as usize;
        unsafe { std::slice::from_raw_parts((&data[offset]) as *const u8 as *const T, count) }
    }

    fn extend(&mut self, offset: crate::TermPointer, size: usize) {
        let offset = u32::from(offset);

        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            let curr_data_len = self.len();

            let pages_to_allocate = (curr_data_len + size) / WASM_PAGE_SIZE as usize
                - curr_data_len / WASM_PAGE_SIZE as usize;
            if pages_to_allocate > 0 {
                self.memory
                    .grow(&mut self.store, pages_to_allocate as u64)
                    .expect("Could not reallocate linear memory for Wasm context");
            }

            *self.get_mut::<u32>(0.into()) += size as u32;
        }
    }

    fn shrink(&mut self, offset: crate::TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            *self.get_mut::<u32>(0.into()) -= pad_to_4_byte_offset(size) as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Rem;

    use crate::{
        allocator::ArenaAllocator,
        interpreter::{WasmContext, WasmInterpreter},
        stdlib::{Add, Stdlib},
        term_type::{
            ApplicationTerm, BuiltinTerm, ConditionTerm, CustomCondition, EffectTerm, HashmapTerm,
            IntTerm, ListTerm, NilTerm, SignalTerm, SymbolTerm, TermType, TreeTerm, TypedTerm,
        },
        ArenaRef, Term, TermPointer,
    };

    use super::{InterpreterError, WasmContextBuilder};

    const RUNTIME_BYTES: &'static [u8] = include_bytes!("../build/runtime.cwasm");

    fn create_mock_wasm_context() -> Result<WasmContext, InterpreterError> {
        WasmContextBuilder::new(RUNTIME_BYTES)?
            .add_import("Math", "remainder", |a: f64, b: f64| a.rem(b))?
            .add_import("Math", "pow", |a: f64, b: f64| a.powf(b))?
            .add_import("Date", "parse", |_: u32, _: u32| 0u64)?
            .add_import("Date", "toISOString", |_: u64, _: u32| 0u32)?
            .add_import("Number", "toString", |_: f64, _: u32| 0u32)?
            .build()
    }

    #[test]
    fn atomic_expressions() {
        let mut interpreter: WasmInterpreter = create_mock_wasm_context().unwrap().into();

        let term = Term::new(TermType::Int(IntTerm::from(3)), &interpreter);

        let state = HashmapTerm::allocate(std::iter::empty(), &mut interpreter);

        let term_pointer = interpreter.allocate(term);

        let interpreter_result = interpreter
            .interpret(term_pointer.into(), state.into())
            .unwrap()
            .bind(&interpreter);

        assert!(matches!(
            interpreter_result.result().as_value().as_value(),
            TermType::Int(IntTerm { value: 3 })
        ));

        assert!(interpreter_result.dependencies().is_none());
    }

    #[test]
    fn evaluated_expressions() {
        let mut interpreter: WasmInterpreter = create_mock_wasm_context().unwrap().into();

        let int3 = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(3)), &interpreter));

        let int2 = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(2)), &interpreter));

        let add = interpreter.allocate(Term::new(
            TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
            &interpreter,
        ));

        let list = ListTerm::allocate([int3, int2], &mut interpreter);

        let application = interpreter.allocate(Term::new(
            TermType::Application(ApplicationTerm {
                target: add,
                args: list,
            }),
            &interpreter,
        ));

        let state = HashmapTerm::allocate(std::iter::empty(), &mut interpreter);

        let interpreter_result = interpreter
            .interpret(application.into(), state.into())
            .unwrap()
            .bind(&interpreter);

        let result = interpreter_result.result();

        assert!(matches!(
            result.as_value().as_value(),
            TermType::Int(IntTerm { value: 5 })
        ));

        assert!(interpreter_result.dependencies().is_none());
    }

    #[test]
    fn stateful_expressions() {
        let mut interpreter: WasmInterpreter = create_mock_wasm_context().unwrap().into();

        let condition = {
            let effect_type = interpreter.allocate(Term::new(
                TermType::Symbol(SymbolTerm { id: 123 }),
                &interpreter,
            ));

            let payload =
                interpreter.allocate(Term::new(TermType::Int(IntTerm { value: 3 }), &interpreter));

            let token = interpreter.allocate(Term::new(TermType::Nil(NilTerm), &interpreter));

            let condition = interpreter.allocate(Term::new(
                TermType::Condition(ConditionTerm::Custom(CustomCondition {
                    effect_type,
                    payload,
                    token,
                })),
                &interpreter,
            ));

            condition
        };

        let input = {
            let add_builtin = interpreter.allocate(Term::new(
                TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
                &interpreter,
            ));

            let stateful_arg = interpreter.allocate(Term::new(
                TermType::Effect(EffectTerm { condition }),
                &interpreter,
            ));

            let static_arg =
                interpreter.allocate(Term::new(TermType::Int(IntTerm::from(2)), &interpreter));

            let add_expression = {
                let term = Term::new(
                    TermType::Application(ApplicationTerm {
                        target: add_builtin,
                        args: ListTerm::allocate([stateful_arg, static_arg], &mut interpreter),
                    }),
                    &interpreter,
                );
                interpreter.allocate(term)
            };

            add_expression
        };

        let state = HashmapTerm::allocate(std::iter::empty(), &mut interpreter);

        let expected_dependencies = interpreter.allocate(Term::new(
            TermType::Tree(TreeTerm {
                left: condition,
                right: TermPointer::null(),
                length: 1,
            }),
            &interpreter,
        ));

        let expected_result = Term::new(
            TermType::Signal(SignalTerm {
                conditions: expected_dependencies,
            }),
            &interpreter,
        );

        let interpreter_result = interpreter
            .interpret(input.into(), state.into())
            .unwrap()
            .bind(&interpreter);
        assert_eq!(
            interpreter_result.result(),
            ArenaRef::new(&interpreter, &expected_result)
        );

        let refer = interpreter_result.dependencies();
        assert_eq!(
            refer,
            Some(ArenaRef::new(
                &interpreter,
                interpreter.get::<TypedTerm<TreeTerm>>(expected_dependencies)
            )),
        );

        let stateful_value =
            interpreter.allocate(Term::new(TermType::Int(IntTerm { value: 3 }), &interpreter));

        let updated_state = HashmapTerm::allocate([(condition, stateful_value)], &mut interpreter);

        let interpreter_result = interpreter
            .interpret(input.into(), updated_state.into())
            .unwrap()
            .bind(&interpreter);

        let expected_result = Term::new(TermType::Int(IntTerm { value: 2 + 3 }), &interpreter);

        assert_eq!(
            interpreter_result.result(),
            ArenaRef::new(&interpreter, &expected_result)
        );

        assert!(interpreter_result.dependencies().is_some());

        let refer = interpreter_result.dependencies();
        assert_eq!(
            refer,
            Some(ArenaRef::new(
                &interpreter,
                interpreter.get::<TypedTerm<TreeTerm>>(expected_dependencies)
            )),
        );
    }
}
