// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{cell::RefCell, ops::Deref, path::Path, rc::Rc};

use wasmtime::{
    Engine, ExternType, Instance, IntoFunc, Linker, Memory, Module, Store, Val, WasmParams,
    WasmResults,
};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use crate::{
    allocator::{Arena, ArenaAllocator, ArenaIterator},
    compiler::RuntimeBuiltin,
    hash::TermSize,
    pad_to_4_byte_offset,
    term_type::{TreeTerm, TypedTerm},
    ArenaPointer, ArenaRef, PointerIter, Term,
};

// Memory is allocated in 64KiB pages according to WASM spec
const WASM_PAGE_SIZE: usize = 64 * 1024;

pub struct UnboundEvaluationResult {
    result_pointer: ArenaPointer,
    dependencies_pointer: Option<ArenaPointer>,
}

impl UnboundEvaluationResult {
    pub fn bind<A: Arena>(self, arena: A) -> InterpreterEvaluationResult<A> {
        InterpreterEvaluationResult {
            arena,
            result_pointer: self.result_pointer,
            dependencies_pointer: self.dependencies_pointer,
        }
    }
}

impl PointerIter for Rc<RefCell<WasmInterpreter>> {
    type Iter<'a> = ArenaIterator<'a, Term, Self>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        let (start_offset, end_offset) = {
            let interpreter = self.borrow();
            let interpreter = interpreter.deref();
            let start_offset = interpreter.start_offset();
            let end_offset = interpreter.end_offset();
            (start_offset, end_offset)
        };
        ArenaIterator::new(&self, start_offset, end_offset)
    }
}

pub struct InterpreterEvaluationResult<A: Arena> {
    arena: A,
    result_pointer: ArenaPointer,
    dependencies_pointer: Option<ArenaPointer>,
}

impl<A: Arena + Clone> InterpreterEvaluationResult<A> {
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
    ModuleLoadError(anyhow::Error),
    GlobalNotFound(String),
    MemoryNotFound(String),
    InvalidFunctionDefinition(String, anyhow::Error),
    InvalidFunctionEvaluation(String, anyhow::Error),
    WasiContextError(wasi_common::StringArrayError),
    WasiLinkError(anyhow::Error),
}

impl std::error::Error for InterpreterError {}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::ModuleLoadError(err) => {
                write!(f, "Unable to load WASM module: {err}")
            }
            InterpreterError::GlobalNotFound(global_name) => {
                write!(f, "Unable to find exported global: \"{global_name}\"")
            }
            InterpreterError::MemoryNotFound(memory_name) => {
                write!(f, "Unable to find exported memory: \"{memory_name}\"")
            }
            InterpreterError::InvalidFunctionDefinition(name, err) => {
                write!(f, "Invalid exported function \"{name}\": {err}")
            }
            InterpreterError::InvalidFunctionEvaluation(name, err) => {
                write!(f, "Failed to evaluate function \"{name}\": {err}")
            }
            InterpreterError::WasiContextError(err) => std::fmt::Display::fmt(err, f),
            InterpreterError::WasiLinkError(err) => std::fmt::Display::fmt(err, f),
        }
    }
}

impl From<wasi_common::StringArrayError> for InterpreterError {
    fn from(value: wasi_common::StringArrayError) -> Self {
        InterpreterError::WasiContextError(value)
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
                    .map_err(InterpreterError::ModuleLoadError)
            },
            memory_name.into(),
        )
    }

    pub fn from_wasm(
        bytes: &[u8],
        memory_name: impl Into<String>,
    ) -> Result<Self, InterpreterError> {
        Self::from_module_factory(
            |e| Module::from_binary(e, bytes).map_err(InterpreterError::ModuleLoadError),
            memory_name.into(),
        )
    }

    pub fn from_path(
        path: impl AsRef<Path>,
        memory_name: impl Into<String>,
    ) -> Result<Self, InterpreterError> {
        Self::from_module_factory(
            |engine| Module::from_file(engine, path).map_err(InterpreterError::ModuleLoadError),
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

        wasmtime_wasi::add_to_linker(&mut linker, |s| s)
            .map_err(InterpreterError::WasiLinkError)?;

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
            .map_err(InterpreterError::WasiLinkError)?;

        Ok(self)
    }

    pub fn build(mut self) -> Result<WasmContext, InterpreterError> {
        let instance = self
            .linker
            .instantiate(&mut self.store, &self.module)
            .map_err(InterpreterError::WasiLinkError)?;

        let memory = instance
            .get_memory(&mut self.store, &self.memory_name)
            .ok_or(InterpreterError::MemoryNotFound(self.memory_name.clone()))?;

        Ok(WasmContext::new(instance, self.store, memory))
    }
}

pub struct WasmContext {
    instance: Instance,
    store: Store<WasiCtx>,
    memory: Memory,
    exports: Vec<(String, ExternType)>,
}

impl WasmContext {
    pub fn new(instance: Instance, store: Store<WasiCtx>, memory: Memory) -> Self {
        let mut store = store;
        let exports = {
            let exports = instance
                .exports(&mut store)
                .map(|export| (String::from(export.name()), export.into_extern()))
                .collect::<Vec<_>>();
            exports
                .into_iter()
                .map(|(key, value)| (key, value.ty(&store)))
                .collect()
        };
        Self {
            instance,
            store,
            memory,
            exports,
        }
    }

    fn data(&self) -> &[u8] {
        self.memory.data(&self.store)
    }

    fn data_mut(&mut self) -> &mut [u8] {
        self.memory.data_mut(&mut self.store)
    }

    fn get_ref<T>(&self, offset: ArenaPointer) -> &T {
        let data = self.data();
        let offset = u32::from(offset) as usize;
        let item = &data[offset];
        unsafe { std::mem::transmute::<&u8, &T>(item) }
    }

    fn get_mut<T>(&mut self, offset: ArenaPointer) -> &mut T {
        let data = self.data_mut();
        let offset = u32::from(offset) as usize;
        let item = &mut data[offset];
        unsafe { std::mem::transmute::<&mut u8, &mut T>(item) }
    }

    fn start_offset(&self) -> ArenaPointer {
        ArenaPointer::from(std::mem::size_of::<u32>() as u32)
    }

    fn end_offset(&self) -> ArenaPointer {
        ArenaPointer::from(*self.get_ref::<u32>(0.into()))
    }

    pub fn get_global(&mut self, export_name: &str) -> Option<Val> {
        self.instance
            .get_global(&mut self.store, export_name)
            .map(|global| global.get(&mut self.store))
    }

    pub fn get_globals(&mut self) -> impl Iterator<Item = (&str, Val)> + '_ {
        self.exports
            .iter()
            .filter_map(|(export_name, export_type)| match export_type {
                ExternType::Global(_) => Some(export_name),
                _ => None,
            })
            .filter_map(|export_name| {
                self.instance
                    .get_global(&mut self.store, export_name)
                    .map(|global| global.get(&mut self.store))
                    .map(|value| (export_name.as_str(), value))
            })
    }
}

impl WasmInterpreter {
    pub fn data(&self) -> &[u8] {
        self.0.data()
    }

    pub fn start_offset(&self) -> ArenaPointer {
        self.0.start_offset()
    }

    pub fn end_offset(&self) -> ArenaPointer {
        self.0.end_offset()
    }

    #[must_use]
    pub fn interpret(
        &mut self,
        input: ArenaPointer,
        state: ArenaPointer,
    ) -> Result<UnboundEvaluationResult, InterpreterError> {
        let (result, dependencies) = self.call::<(u32, u32), (u32, u32)>(
            RuntimeBuiltin::Evaluate.name(),
            (input.into(), state.into()),
        )?;
        Ok(UnboundEvaluationResult {
            result_pointer: result.into(),
            dependencies_pointer: ArenaPointer::from(dependencies).as_non_null(),
        })
    }

    #[must_use]
    pub fn execute(
        &mut self,
        export_name: &str,
        state: ArenaPointer,
    ) -> Result<UnboundEvaluationResult, InterpreterError> {
        let (result, dependencies) = self.call::<u32, (u32, u32)>(export_name, state.into())?;
        Ok(UnboundEvaluationResult {
            result_pointer: result.into(),
            dependencies_pointer: ArenaPointer::from(dependencies).as_non_null(),
        })
    }

    #[must_use]
    pub fn call<I: WasmParams, O: WasmResults>(
        &mut self,
        export_name: &str,
        args: I,
    ) -> Result<O, InterpreterError> {
        let target = self
            .0
            .instance
            .get_typed_func::<I, O>(&mut self.0.store, export_name)
            .map_err(|err| {
                InterpreterError::InvalidFunctionDefinition(String::from(export_name), err)
            })?;

        let output = target.call(&mut self.0.store, args).map_err(|err| {
            InterpreterError::InvalidFunctionEvaluation(String::from(export_name), err)
        })?;

        Ok(output)
    }

    #[must_use]
    pub fn initialize(&mut self) -> Result<(), InterpreterError> {
        self.call::<(), ()>("_initialize", ())
    }
}

pub struct WasmInterpreter(WasmContext);

impl WasmInterpreter {
    pub fn get_global(&mut self, export_name: &str) -> Option<Val> {
        self.0.get_global(export_name)
    }
    pub fn get_globals(&mut self) -> impl Iterator<Item = (&str, Val)> {
        self.0.get_globals()
    }
}

impl From<WasmContext> for WasmInterpreter {
    fn from(value: WasmContext) -> Self {
        Self(value)
    }
}

impl Arena for WasmContext {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;

    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        selector(self.get_ref(offset))
    }

    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        let target = self.get_ref(offset);
        let outer_pointer = target as *const T as usize;
        let inner_pointer = selector(target) as *const V as usize;
        offset.offset((inner_pointer - outer_pointer) as u32)
    }

    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        let data = self.data();
        let offset = u32::from(offset) as usize;
        &data[offset..(offset + length)]
    }
}

impl ArenaAllocator for WasmContext {
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        let offset = self.end_offset();
        let static_size = pad_to_4_byte_offset(std::mem::size_of::<T>());
        let actual_size = pad_to_4_byte_offset(value.size_of());
        self.extend(offset, static_size);
        self.write(offset, value);
        if actual_size < static_size {
            self.shrink(offset.offset(static_size as u32), static_size - actual_size);
        }
        ArenaPointer::from(offset)
    }

    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        let next_offset = self.end_offset();
        if offset != next_offset {
            panic!("Invalid allocator offset");
        } else {
            let existing_length = u32::from(next_offset) as usize;
            let target_length = existing_length + size;

            let num_existing_pages = self.memory.size(&self.store) as usize;
            let num_target_pages = 1 + (target_length.saturating_sub(1) / WASM_PAGE_SIZE);
            if num_target_pages > num_existing_pages {
                let pages_to_allocate = num_target_pages.next_power_of_two() - num_existing_pages;
                self.memory
                    .grow(&mut self.store, pages_to_allocate as u64)
                    .expect("Could not reallocate linear memory for Wasm context");
            }

            *self.get_mut::<u32>(0.into()) = target_length as u32;
        }
    }

    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        if offset != self.end_offset() {
            panic!("Invalid allocator offset");
        } else {
            *self.get_mut::<u32>(0.into()) -= pad_to_4_byte_offset(size) as u32;
        }
    }

    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        *self.get_mut(offset) = value
    }
}

impl Arena for WasmInterpreter {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        <WasmContext as Arena>::read_value::<T, V>(&self.0, offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        <WasmContext as Arena>::inner_pointer::<T, V>(&self.0, offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        <WasmContext as Arena>::as_slice(&self.0, offset, length)
    }
}

impl ArenaAllocator for WasmInterpreter {
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        <WasmContext as ArenaAllocator>::allocate(&mut self.0, value)
    }
    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        <WasmContext as ArenaAllocator>::extend(&mut self.0, offset, size)
    }
    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        <WasmContext as ArenaAllocator>::shrink(&mut self.0, offset, size)
    }
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        <WasmContext as ArenaAllocator>::write(&mut self.0, offset, value)
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
        interpreter::WasmInterpreter,
        stdlib::{Add, Stdlib},
        term_type::{
            ApplicationTerm, BuiltinTerm, ConditionTerm, CustomCondition, EffectTerm, HashmapTerm,
            IntTerm, ListTerm, NilTerm, SignalTerm, SymbolTerm, TermType, TreeTerm, TypedTerm,
        },
        ArenaPointer, ArenaRef, Term,
    };
    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    use super::{mocks::add_import_stubs, InterpreterError, WasmContextBuilder};

    const RUNTIME_BYTES: &'static [u8] = include_bytes!("../build/runtime.wasm");

    fn create_mock_wasm_interpreter() -> Result<WasmInterpreter, InterpreterError> {
        let mut interpreter: WasmInterpreter =
            add_import_stubs(WasmContextBuilder::from_wasm(RUNTIME_BYTES, "memory")?)?
                .build()?
                .into();
        interpreter.initialize()?;
        Ok(interpreter)
    }

    #[test]
    fn atomic_expressions() {
        let mut interpreter: WasmInterpreter = create_mock_wasm_interpreter().unwrap().into();

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
        let mut interpreter: WasmInterpreter = create_mock_wasm_interpreter().unwrap().into();

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
        let mut interpreter: WasmInterpreter = create_mock_wasm_interpreter().unwrap().into();

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
                right: ArenaPointer::null(),
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
