// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashMap;

use reflex::core::Eagerness;
use walrus::{
    self,
    ir::{Call, Instr, LocalGet, LocalTee},
    ActiveData, ActiveDataLocation, DataId, DataKind, ExportItem, FunctionId,
};

use crate::{
    allocator::ArenaAllocator,
    compiler::{
        CompileWasm, CompiledExpression, CompiledInstruction, CompilerError, CompilerState,
        RuntimeBuiltin,
    },
    term_type::WasmExpression,
    ArenaRef, PointerIter, Term,
};

#[derive(Debug)]
pub enum WasmCompilerError {
    ModuleLoadError(anyhow::Error),
    DataSectionNotFound,
    MultipleDataSections,
    InvalidDataSection,
    CompilerError(CompilerError),
    RuntimeBuiltinNotFound(RuntimeBuiltin),
}

impl std::error::Error for WasmCompilerError {}

impl std::fmt::Display for WasmCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleLoadError(err) => write!(f, "Failed to load WASM module: {err}"),
            Self::DataSectionNotFound => write!(f, "Data section definition not found"),
            Self::MultipleDataSections => write!(f, "Multiple data section definitions"),
            Self::InvalidDataSection => write!(f, "Invalid data section definition"),
            Self::CompilerError(err) => write!(f, "Failed to compile WASM output: {err}"),
            Self::RuntimeBuiltinNotFound(err) => {
                write!(f, "Runtime builtin not found: {}", err.name())
            }
        }
    }
}

pub fn compile_module<A: ArenaAllocator + PointerIter + Clone>(
    entry_points: impl IntoIterator<Item = (String, WasmExpression<A>)>,
    runtime_wasm: &[u8],
) -> Result<Vec<u8>, WasmCompilerError> {
    // Create a new Wasm module based on the runtime bytes
    let mut ast = parse_wasm_ast(runtime_wasm)?;
    let builtin_mappings = parse_runtime_builtins(&ast)?;

    // Locate the data section
    let heap_snapshot_id = get_data_section_instruction_id(&ast)?;
    let initial_heap_snapshot = &ast.data.get(heap_snapshot_id).value;

    // Compile the entry points, allocating any static expressions into the compiler state linear memory
    let (entry_points, updated_heap_snapshot) =
        compile_entry_points(entry_points, initial_heap_snapshot)?;

    // Write the entry point functions into the module
    for (export_name, function_body) in entry_points.into_iter() {
        let compiled_function_body =
            link_compiled_chunk(function_body, &mut ast, &builtin_mappings);
        let term_factory = create_term_factory_function(&mut ast, compiled_function_body);
        let function_id =
            create_entry_point_function(&mut ast, term_factory, builtin_mappings.evaluate);
        ast.exports.add(&export_name, function_id);
    }

    // Update the module's linear memory initialization instruction with the allocated contents
    ast.data.get_mut(heap_snapshot_id).value = updated_heap_snapshot;

    // Emit the resulting WASM as bytes
    Ok(ast.emit_wasm())
}

#[derive(Clone, Copy, Debug)]
struct RuntimeBuiltinMappings {
    initialize: FunctionId,
    evaluate: FunctionId,
    create_application: FunctionId,
    create_boolean: FunctionId,
    create_builtin: FunctionId,
    create_float: FunctionId,
    create_int: FunctionId,
    allocate_list: FunctionId,
    init_list: FunctionId,
    set_list_item: FunctionId,
}
impl RuntimeBuiltinMappings {
    fn get(&self, builtin: RuntimeBuiltin) -> FunctionId {
        match builtin {
            RuntimeBuiltin::Initialize => self.initialize,
            RuntimeBuiltin::Evaluate => self.evaluate,
            RuntimeBuiltin::CreateApplication => self.create_application,
            RuntimeBuiltin::CreateBoolean => self.create_boolean,
            RuntimeBuiltin::CreateBuiltin => self.create_builtin,
            RuntimeBuiltin::CreateFloat => self.create_float,
            RuntimeBuiltin::CreateInt => self.create_int,
            RuntimeBuiltin::AllocateList => self.allocate_list,
            RuntimeBuiltin::InitList => self.init_list,
            RuntimeBuiltin::SetListItem => self.set_list_item,
        }
    }
}

fn parse_runtime_builtins(
    module: &walrus::Module,
) -> Result<RuntimeBuiltinMappings, WasmCompilerError> {
    let builtins = parse_exported_functions(module)
        .map(|(name, function_id)| (String::from(name), function_id))
        .collect::<HashMap<_, _>>();
    Ok(RuntimeBuiltinMappings {
        initialize: get_builtin_function(&builtins, RuntimeBuiltin::Initialize)?,
        evaluate: get_builtin_function(&builtins, RuntimeBuiltin::Evaluate)?,
        create_application: get_builtin_function(&builtins, RuntimeBuiltin::CreateApplication)?,
        create_boolean: get_builtin_function(&builtins, RuntimeBuiltin::CreateBoolean)?,
        create_builtin: get_builtin_function(&builtins, RuntimeBuiltin::CreateBuiltin)?,
        create_float: get_builtin_function(&builtins, RuntimeBuiltin::CreateFloat)?,
        create_int: get_builtin_function(&builtins, RuntimeBuiltin::CreateInt)?,
        allocate_list: get_builtin_function(&builtins, RuntimeBuiltin::AllocateList)?,
        init_list: get_builtin_function(&builtins, RuntimeBuiltin::InitList)?,
        set_list_item: get_builtin_function(&builtins, RuntimeBuiltin::SetListItem)?,
    })
}

fn get_builtin_function(
    builtins: &HashMap<String, FunctionId>,
    target: RuntimeBuiltin,
) -> Result<FunctionId, WasmCompilerError> {
    builtins
        .get(target.name())
        .copied()
        .ok_or_else(|| WasmCompilerError::RuntimeBuiltinNotFound(RuntimeBuiltin::Initialize))
}

fn parse_wasm_ast(runtime_wasm: &[u8]) -> Result<walrus::Module, WasmCompilerError> {
    walrus::Module::from_buffer(runtime_wasm).map_err(WasmCompilerError::ModuleLoadError)
}

fn get_data_section_instruction_id(module: &walrus::Module) -> Result<DataId, WasmCompilerError> {
    let mut data_instructions = module.data.iter();
    match (data_instructions.next(), data_instructions.next()) {
        (Some(data), None) => {
            if matches!(
                &data.kind,
                DataKind::Active(ActiveData {
                    location: ActiveDataLocation::Absolute(0),
                    ..
                })
            ) {
                Ok(data.id())
            } else {
                Err(WasmCompilerError::InvalidDataSection)
            }
        }
        (Some(_), Some(_)) => Err(WasmCompilerError::MultipleDataSections),
        (None, _) => Err(WasmCompilerError::DataSectionNotFound),
    }
}

fn compile_entry_points(
    entry_points: impl IntoIterator<Item = (String, ArenaRef<Term, impl ArenaAllocator + Clone>)>,
    heap_snapshot: &[u8],
) -> Result<(Vec<(String, CompiledExpression)>, Vec<u8>), WasmCompilerError> {
    // Initialize the compiler state with the contents of the initialized linear memory
    let mut compiler_state = CompilerState::from_heap_snapshot(heap_snapshot);
    let compiled_entry_points = entry_points
        .into_iter()
        .map(|(export_name, expression)| {
            let function_body =
                expression.compile(Eagerness::Eager, &mut compiler_state, &Default::default())?;
            Ok((export_name, function_body))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(WasmCompilerError::CompilerError)?;
    let updated_heap_snapshot = compiler_state.into_linear_memory();
    Ok((compiled_entry_points, updated_heap_snapshot))
}

fn link_compiled_chunk(
    compiled_expression: CompiledExpression,
    module: &mut walrus::Module,
    builtin_mappings: &RuntimeBuiltinMappings,
) -> Vec<Instr> {
    let temp_local = module.locals.add(walrus::ValType::I32);
    compiled_expression
        .into_iter()
        .flat_map(|instruction| match instruction {
            CompiledInstruction::Wasm(instruction) => vec![instruction],
            CompiledInstruction::CallRuntimeBuiltin(builtin) => vec![Instr::Call(Call {
                func: builtin_mappings.get(builtin),
            })],
            CompiledInstruction::CallStdlib(_) => todo!(),
            CompiledInstruction::Duplicate => vec![
                Instr::LocalTee(LocalTee { local: temp_local }),
                Instr::LocalGet(LocalGet { local: temp_local }),
            ],
        })
        .collect()
}

fn create_term_factory_function(
    module: &mut walrus::Module,
    instructions: impl IntoIterator<Item = walrus::ir::Instr>,
) -> FunctionId {
    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[walrus::ValType::I32]);

    instructions
        .into_iter()
        .fold(&mut builder.func_body(), |acc, next| acc.instr(next));

    builder.finish(vec![], &mut module.funcs)
}

fn create_entry_point_function(
    module: &mut walrus::Module,
    node_factory: FunctionId,
    evaluate_function: FunctionId,
) -> FunctionId {
    let state = module.locals.add(walrus::ValType::I32);
    let mut builder = walrus::FunctionBuilder::new(
        &mut module.types,
        &[walrus::ValType::I32],
        &[walrus::ValType::I32, walrus::ValType::I32],
    );

    builder
        .func_body()
        .call(node_factory)
        .local_get(state)
        .call(evaluate_function);

    builder.finish(vec![state], &mut module.funcs)
}

fn parse_exported_functions(
    module: &walrus::Module,
) -> impl Iterator<Item = (&str, FunctionId)> + '_ {
    module
        .exports
        .iter()
        .filter_map(|export| match export.item {
            ExportItem::Function(id) => Some((export.name.as_str(), id)),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    const RUNTIME_BYTES: &[u8] = include_bytes!("../build/runtime.wasm");

    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    use crate::{
        allocator::VecAllocator,
        interpreter::{
            mocks::add_import_stubs, InterpreterError, WasmContextBuilder, WasmInterpreter,
        },
        stdlib::{Add, Stdlib},
        term_type::{ApplicationTerm, BuiltinTerm, IntTerm, ListTerm, TermType},
        ArenaRef, Term, TermPointer,
    };

    use super::*;

    fn create_mock_wasm_interpreter(
        wasm_bytes: &[u8],
    ) -> Result<WasmInterpreter, InterpreterError> {
        let mut interpreter: WasmInterpreter =
            add_import_stubs(WasmContextBuilder::from_wasm(wasm_bytes, "memory")?)?
                .build()?
                .into();
        interpreter.initialize()?;
        Ok(interpreter)
    }

    #[test]
    fn primitive_expressions() {
        let mut arena = VecAllocator::default();
        let term_pointer = arena.allocate(Term::new(TermType::Int(IntTerm::from(5)), &arena));

        let arena = Rc::new(RefCell::new(arena));
        let expression = WasmExpression::new(arena.clone(), term_pointer);

        let wasm_bytes = compile_module([("foo".into(), expression)], RUNTIME_BYTES).unwrap();

        let interpreter = create_mock_wasm_interpreter(&wasm_bytes).unwrap();

        let state = TermPointer::null();

        let interpreter = Rc::new(RefCell::new(interpreter));
        let result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .execute("foo", state)
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

        assert_eq!(result.result(), expected_result);
        assert!(result.dependencies().is_none());
    }

    #[test]
    fn deeply_nested_applications() {
        let mut arena = VecAllocator::default();

        let mut current = arena.allocate(Term::new(TermType::Int(IntTerm::from(1)), &arena));

        let add = arena.allocate(Term::new(
            TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
            &arena,
        ));

        for i in 2..=100 {
            let new_int = arena.allocate(Term::new(TermType::Int(IntTerm::from(i)), &arena));

            let list = ListTerm::allocate([current, new_int], &mut arena);

            current = arena.allocate(Term::new(
                TermType::Application(ApplicationTerm {
                    target: add,
                    args: list,
                }),
                &arena,
            ));
        }

        let arena = Rc::new(RefCell::new(arena));
        let expression = WasmExpression::new(arena.clone(), current);

        let wasm_bytes = compile_module([("foo".into(), expression)], RUNTIME_BYTES).unwrap();

        let interpreter = create_mock_wasm_interpreter(&wasm_bytes).unwrap();

        let state = TermPointer::null();

        let interpreter = Rc::new(RefCell::new(interpreter));

        let result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .execute("foo", state)
            .unwrap()
            .bind(Rc::clone(&interpreter));

        let expected_result = ArenaRef::<Term, _>::new(
            Rc::clone(&interpreter),
            interpreter
                .deref()
                .borrow_mut()
                .deref_mut()
                .allocate(Term::new(
                    TermType::Int(IntTerm { value: 5050 }),
                    &interpreter,
                )),
        );

        assert_eq!(result.result(), expected_result);
        assert!(result.dependencies().is_none());
    }
}
