// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::path::Path;

use wasmtime::{
    Engine, ExternType, Instance, IntoFunc, Linker, Memory, Module, Store, WasmParams, WasmResults,
};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use crate::{
    allocator::ArenaAllocator,
    compiler::RuntimeBuiltin,
    hash::TermSize,
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
    pub fn bind<A: ArenaAllocator>(self, arena: A) -> InterpreterEvaluationResult<A> {
        InterpreterEvaluationResult {
            arena,
            result_pointer: self.result_pointer,
            dependencies_pointer: self.dependencies_pointer,
        }
    }
}

pub struct InterpreterEvaluationResult<A: ArenaAllocator> {
    arena: A,
    result_pointer: TermPointer,
    dependencies_pointer: Option<TermPointer>,
}

impl<A: ArenaAllocator + Clone> InterpreterEvaluationResult<A> {
    pub fn result(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.result_pointer)
    }

    pub fn dependencies(&self) -> Option<ArenaRef<TypedTerm<TreeTerm>, A>> {
        self.dependencies_pointer.map(|dependencies_pointer| {
            ArenaRef::<TypedTerm<TreeTerm>, _>::new(self.arena.clone(), dependencies_pointer)
        })
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    ImportNotFound {
        module: String,
        name: String,
        ty: ExternType,
    },
    FunctionNotFound(anyhow::Error),
    WasiContexBuild(wasi_common::StringArrayError),
    Linking(anyhow::Error),
    ModuleCreation(anyhow::Error),
    LinearMemoryNotFound(String),
    FunctionEvaluation(anyhow::Error),
}

impl std::error::Error for InterpreterError {}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::ImportNotFound { module, name, ty } => write!(
                f,
                "Import {name} from module {module} of type {ty:?} not found"
            ),
            InterpreterError::WasiContexBuild(err) => std::fmt::Display::fmt(err, f),

            InterpreterError::Linking(err)
            | InterpreterError::FunctionEvaluation(err)
            | InterpreterError::FunctionNotFound(err)
            | InterpreterError::ModuleCreation(err) => std::fmt::Display::fmt(err, f),
            InterpreterError::LinearMemoryNotFound(memory_name) => {
                write!(f, "Could not find memory by name {memory_name}")
            }
        }
    }
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
    memory_name: String,
}

impl WasmContextBuilder {
    pub fn from_cwasm(
        program_bytes: &[u8],
        memory_name: impl Into<String>,
    ) -> Result<Self, InterpreterError> {
        Self::from_module_factory(
            |engine| {
                unsafe { Module::deserialize(engine, program_bytes) }
                    .map_err(InterpreterError::ModuleCreation)
            },
            memory_name.into(),
        )
    }

    pub fn from_wasm(
        bytes: &[u8],
        memory_name: impl Into<String>,
    ) -> Result<Self, InterpreterError> {
        Self::from_module_factory(
            |e| Module::from_binary(e, bytes).map_err(InterpreterError::ModuleCreation),
            memory_name.into(),
        )
    }

    pub fn from_path(
        path: impl AsRef<Path>,
        memory_name: impl Into<String>,
    ) -> Result<Self, InterpreterError> {
        Self::from_module_factory(
            |engine| Module::from_file(engine, path).map_err(InterpreterError::ModuleCreation),
            memory_name.into(),
        )
    }

    fn from_module_factory(
        builder: impl FnOnce(&Engine) -> Result<Module, InterpreterError>,
        memory_name: String,
    ) -> Result<Self, InterpreterError> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();

        let engine = Engine::default();
        let store = Store::new(&engine, wasi);
        let mut linker = Linker::new(store.engine());
        let module = builder(store.engine())?;

        wasmtime_wasi::add_to_linker(&mut linker, |s| s).map_err(InterpreterError::Linking)?;

        Ok(Self {
            store,
            linker,
            module,
            memory_name,
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
            .get_memory(&mut self.store, &self.memory_name)
            .ok_or(InterpreterError::LinearMemoryNotFound(
                self.memory_name.clone(),
            ))?;

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

impl WasmContext {
    fn data(&self) -> &[u8] {
        self.memory.data(&self.store)
    }

    fn data_mut(&mut self) -> &mut [u8] {
        self.memory.data_mut(&mut self.store)
    }

    fn get_ref<T>(&self, offset: TermPointer) -> &T {
        let data = self.data();
        let offset = u32::from(offset) as usize;
        let item = &data[offset];
        unsafe { std::mem::transmute::<&u8, &T>(item) }
    }

    fn get_mut<T>(&mut self, offset: TermPointer) -> &mut T {
        let data = self.data_mut();
        let offset = u32::from(offset) as usize;
        let item = &mut data[offset];
        unsafe { std::mem::transmute::<&mut u8, &mut T>(item) }
    }
}

impl WasmInterpreter {
    pub fn interpret(
        &mut self,
        input: TermPointer,
        state: TermPointer,
    ) -> Result<UnboundEvaluationResult, InterpreterError> {
        let (result, dependencies) = self.call::<(u32, u32), (u32, u32)>(
            RuntimeBuiltin::Evaluate.name(),
            (input.into(), state.into()),
        )?;
        Ok(UnboundEvaluationResult {
            result_pointer: result.into(),
            dependencies_pointer: TermPointer::from(dependencies).as_non_null(),
        })
    }

    pub fn execute(
        &mut self,
        export_name: &str,
        state: TermPointer,
    ) -> Result<UnboundEvaluationResult, InterpreterError> {
        let (result, dependencies) = self.call::<u32, (u32, u32)>(export_name, state.into())?;
        Ok(UnboundEvaluationResult {
            result_pointer: result.into(),
            dependencies_pointer: TermPointer::from(dependencies).as_non_null(),
        })
    }

    pub fn call<I: WasmParams, O: WasmResults>(
        &mut self,
        export_name: &str,
        args: I,
    ) -> Result<O, InterpreterError> {
        let target = self
            .0
            .program
            .get_typed_func::<I, O>(&mut self.0.store, export_name)
            .map_err(InterpreterError::FunctionNotFound)?;

        let output = target
            .call(&mut self.0.store, args)
            .map_err(InterpreterError::FunctionEvaluation)?;

        Ok(output)
    }
}

pub struct WasmInterpreter(WasmContext);

impl From<WasmContext> for WasmInterpreter {
    fn from(value: WasmContext) -> Self {
        Self(value)
    }
}

impl ArenaAllocator for WasmContext {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;

    fn len(&self) -> usize {
        *self.get_ref::<u32>(0.into()) as usize
    }

    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        let offset = TermPointer(self.len() as u32);
        let static_size = pad_to_4_byte_offset(std::mem::size_of::<T>());
        let actual_size = pad_to_4_byte_offset(value.size_of());
        self.extend(offset, static_size);
        self.write(offset, value);
        if actual_size < static_size {
            self.shrink(offset.offset(static_size as u32), static_size - actual_size);
        }
        TermPointer::from(offset)
    }

    fn extend(&mut self, offset: TermPointer, size: usize) {
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

    fn shrink(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            *self.get_mut::<u32>(0.into()) -= pad_to_4_byte_offset(size) as u32;
        }
    }

    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        *self.get_mut(offset) = value
    }

    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V {
        selector(self.get_ref(offset))
    }

    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer {
        let target = self.get_ref(offset);
        let outer_pointer = target as *const T as usize;
        let inner_pointer = selector(target) as *const V as usize;
        offset.offset((inner_pointer - outer_pointer) as u32)
    }

    fn as_slice<'a>(&'a self, offset: TermPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        let data = self.data();
        let offset = u32::from(offset) as usize;
        &data[offset..(offset + length)]
    }
}

impl ArenaAllocator for WasmInterpreter {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;
    fn len(&self) -> usize {
        <WasmContext as ArenaAllocator>::len(&self.0)
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        <WasmContext as ArenaAllocator>::allocate(&mut self.0, value)
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        <WasmContext as ArenaAllocator>::extend(&mut self.0, offset, size)
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        <WasmContext as ArenaAllocator>::shrink(&mut self.0, offset, size)
    }
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        <WasmContext as ArenaAllocator>::write(&mut self.0, offset, value)
    }
    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V {
        <WasmContext as ArenaAllocator>::read_value::<T, V>(&self.0, offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer {
        <WasmContext as ArenaAllocator>::inner_pointer::<T, V>(&self.0, offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: TermPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        <WasmContext as ArenaAllocator>::as_slice(&self.0, offset, length)
    }
}

pub mod mocks {

    use super::{InterpreterError, WasmContextBuilder};

    pub fn add_import_stubs(
        builder: WasmContextBuilder,
    ) -> Result<WasmContextBuilder, InterpreterError> {
        builder
            .add_import("Date", "parse", |_: u32, _: u32| 0u64)?
            .add_import("Date", "toISOString", |_: u64, _: u32| 0u32)?
            .add_import("Number", "toString", |_: f64, _: u32| 0u32)?
            .add_import("Math", "remainder", |_: f64, _: f64| 0f64)?
            .add_import("Math", "acos", |_: f64| 0f64)?
            .add_import("Math", "acosh", |_: f64| 0f64)?
            .add_import("Math", "asin", |_: f64| 0f64)?
            .add_import("Math", "asinh", |_: f64| 0f64)?
            .add_import("Math", "atan", |_: f64| 0f64)?
            .add_import("Math", "atan2", |_: f64, _: f64| 0f64)?
            .add_import("Math", "atanh", |_: f64| 0f64)?
            .add_import("Math", "cbrt", |_: f64| 0f64)?
            .add_import("Math", "cos", |_: f64| 0f64)?
            .add_import("Math", "cosh", |_: f64| 0f64)?
            .add_import("Math", "exp", |_: f64| 0f64)?
            .add_import("Math", "expm1", |_: f64| 0f64)?
            .add_import("Math", "hypot", |_: f64, _: f64| 0f64)?
            .add_import("Math", "log", |_: f64| 0f64)?
            .add_import("Math", "log2", |_: f64| 0f64)?
            .add_import("Math", "log10", |_: f64| 0f64)?
            .add_import("Math", "log1p", |_: f64| 0f64)?
            .add_import("Math", "pow", |_: f64, _: f64| 0f64)?
            .add_import("Math", "sin", |_: f64| 0f64)?
            .add_import("Math", "sinh", |_: f64| 0f64)?
            .add_import("Math", "sqrt", |_: f64| 0f64)?
            .add_import("Math", "tan", |_: f64, _: f64| 0f64)?
            .add_import("Math", "tanh", |_: f64, _: f64| 0f64)
    }
}

#[cfg(test)]
mod tests {
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
    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    use super::{mocks::add_import_stubs, InterpreterError, WasmContextBuilder};

    const RUNTIME_BYTES: &'static [u8] = include_bytes!("../build/runtime.wasm");

    fn create_mock_wasm_context() -> Result<WasmContext, InterpreterError> {
        add_import_stubs(WasmContextBuilder::from_wasm(RUNTIME_BYTES, "memory")?)?.build()
    }

    #[test]
    fn atomic_expressions() {
        let mut interpreter: WasmInterpreter = create_mock_wasm_context().unwrap().into();

        let input = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(3)), &interpreter));

        let state = HashmapTerm::allocate(std::iter::empty(), &mut interpreter);

        let interpreter = Rc::new(RefCell::new(interpreter));

        let interpreter_result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .interpret(input.into(), state.into())
            .unwrap()
            .bind(Rc::clone(&interpreter));

        let expected_result = ArenaRef::<Term, _>::new(
            Rc::clone(&interpreter),
            interpreter
                .deref()
                .borrow_mut()
                .deref_mut()
                .allocate(Term::new(TermType::Int(IntTerm { value: 3 }), &interpreter)),
        );

        assert_eq!(interpreter_result.result(), expected_result);
        assert!(interpreter_result.dependencies().is_none());
    }

    #[test]
    fn evaluated_expressions() {
        let mut interpreter: WasmInterpreter = create_mock_wasm_context().unwrap().into();

        let input = {
            let int3 =
                interpreter.allocate(Term::new(TermType::Int(IntTerm::from(3)), &interpreter));

            let int2 =
                interpreter.allocate(Term::new(TermType::Int(IntTerm::from(2)), &interpreter));

            let add = interpreter.allocate(Term::new(
                TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
                &interpreter,
            ));

            let arg_list = ListTerm::allocate([int3, int2], &mut interpreter);

            interpreter.allocate(Term::new(
                TermType::Application(ApplicationTerm {
                    target: add,
                    args: arg_list,
                }),
                &interpreter,
            ))
        };

        let state = HashmapTerm::allocate(std::iter::empty(), &mut interpreter);

        let interpreter = Rc::new(RefCell::new(interpreter));

        let interpreter_result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .interpret(input.into(), state.into())
            .unwrap()
            .bind(Rc::clone(&interpreter));

        let expected_result = ArenaRef::<Term, _>::new(
            Rc::clone(&interpreter),
            interpreter
                .deref()
                .borrow_mut()
                .deref_mut()
                .allocate(Term::new(TermType::Int(IntTerm { value: 5 }), &interpreter)),
        );

        assert_eq!(interpreter_result.result(), expected_result);
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
        let expected_result = interpreter.allocate(expected_result);

        let interpreter = Rc::new(RefCell::new(interpreter));

        let interpreter_result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .interpret(input.into(), state.into())
            .unwrap()
            .bind(Rc::clone(&interpreter));

        assert_eq!(
            interpreter_result.result(),
            ArenaRef::<Term, _>::new(Rc::clone(&interpreter), expected_result),
        );

        let result_dependencies = interpreter_result.dependencies();
        assert_eq!(
            result_dependencies,
            Some(ArenaRef::<TypedTerm<TreeTerm>, _>::new(
                Rc::clone(&interpreter),
                expected_dependencies
            )),
        );

        let stateful_value = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .allocate(Term::new(TermType::Int(IntTerm { value: 3 }), &interpreter));

        let updated_state = HashmapTerm::allocate(
            [(condition, stateful_value)],
            interpreter.deref().borrow_mut().deref_mut(),
        );

        let expected_result = Term::new(TermType::Int(IntTerm { value: 2 + 3 }), &interpreter);
        let expected_result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .allocate(expected_result);

        let interpreter_result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .interpret(input.into(), updated_state.into())
            .unwrap()
            .bind(Rc::clone(&interpreter));

        assert_eq!(
            interpreter_result.result(),
            ArenaRef::<Term, _>::new(Rc::clone(&interpreter), expected_result)
        );

        assert!(interpreter_result.dependencies().is_some());

        let result_dependencies = interpreter_result.dependencies();
        assert_eq!(
            result_dependencies,
            Some(ArenaRef::<TypedTerm<TreeTerm>, _>::new(
                Rc::clone(&interpreter),
                expected_dependencies
            )),
        );
    }
}
