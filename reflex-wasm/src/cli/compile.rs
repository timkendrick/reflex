// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashMap;

use reflex::core::Eagerness;
use walrus::{
    self,
    ir::{Call, GlobalGet, Instr, LocalGet, LocalTee},
    ActiveData, ActiveDataLocation, DataId, DataKind, ExportItem, FunctionId, GlobalId,
};

use crate::{
    allocator::Arena,
    compiler::{
        CompileWasm, CompiledExpression, CompiledInstruction, CompilerOptions, CompilerScope,
        CompilerState, RuntimeBuiltin, RuntimeGlobal,
    },
    term_type::WasmExpression,
    ArenaRef, Term,
};

#[derive(Debug)]
pub enum WasmCompilerError {
    ModuleLoadError(anyhow::Error),
    DataSectionNotFound,
    MultipleDataSections,
    InvalidDataSection,
    CompilerError(anyhow::Error),
    RuntimeGlobalNotFound(RuntimeGlobal),
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
            Self::RuntimeGlobalNotFound(target) => {
                write!(f, "Runtime global not found: {}", target.name())
            }
            Self::RuntimeBuiltinNotFound(target) => {
                write!(f, "Runtime function not found: {}", target.name())
            }
        }
    }
}

pub fn compile_module(
    entry_points: impl IntoIterator<Item = (String, WasmExpression<impl Arena + Clone>)>,
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
    null_pointer: GlobalId,
    initialize: FunctionId,
    evaluate: FunctionId,
    allocate_cell: FunctionId,
    allocate_hashmap: FunctionId,
    allocate_list: FunctionId,
    allocate_string: FunctionId,
    create_application: FunctionId,
    create_boolean: FunctionId,
    create_builtin: FunctionId,
    create_compiled: FunctionId,
    create_custom_condition: FunctionId,
    create_pending_condition: FunctionId,
    create_error_condition: FunctionId,
    create_type_error_condition: FunctionId,
    create_invalid_function_target_condition: FunctionId,
    create_invalid_function_args_condition: FunctionId,
    create_invalid_pointer_condition: FunctionId,
    create_constructor: FunctionId,
    create_date: FunctionId,
    create_effect: FunctionId,
    create_float: FunctionId,
    create_hashset: FunctionId,
    create_int: FunctionId,
    create_lambda: FunctionId,
    create_let: FunctionId,
    create_nil: FunctionId,
    create_partial: FunctionId,
    create_pointer: FunctionId,
    create_record: FunctionId,
    create_signal: FunctionId,
    create_tree: FunctionId,
    create_variable: FunctionId,
    create_empty_iterator: FunctionId,
    create_evaluate_iterator: FunctionId,
    create_filter_iterator: FunctionId,
    create_flatten_iterator: FunctionId,
    create_hashmap_keys_iterator: FunctionId,
    create_hashmap_values_iterator: FunctionId,
    create_integers_iterator: FunctionId,
    create_intersperse_iterator: FunctionId,
    create_map_iterator: FunctionId,
    create_once_iterator: FunctionId,
    create_range_iterator: FunctionId,
    create_repeat_iterator: FunctionId,
    create_skip_iterator: FunctionId,
    create_take_iterator: FunctionId,
    create_zip_iterator: FunctionId,
    get_string_char_offset: FunctionId,
    init_hashmap: FunctionId,
    init_list: FunctionId,
    init_string: FunctionId,
    insert_hashmap_entry: FunctionId,
    set_cell_field: FunctionId,
    set_list_item: FunctionId,
    write: FunctionId,
}
impl RuntimeBuiltinMappings {
    fn get(&self, builtin: RuntimeBuiltin) -> FunctionId {
        match builtin {
            RuntimeBuiltin::Initialize => self.initialize,
            RuntimeBuiltin::Evaluate => self.evaluate,
            RuntimeBuiltin::AllocateCell => self.allocate_cell,
            RuntimeBuiltin::AllocateHashmap => self.allocate_hashmap,
            RuntimeBuiltin::AllocateList => self.allocate_list,
            RuntimeBuiltin::AllocateString => self.allocate_string,
            RuntimeBuiltin::CreateApplication => self.create_application,
            RuntimeBuiltin::CreateBoolean => self.create_boolean,
            RuntimeBuiltin::CreateBuiltin => self.create_builtin,
            RuntimeBuiltin::CreateCompiled => self.create_compiled,
            RuntimeBuiltin::CreateCustomCondition => self.create_custom_condition,
            RuntimeBuiltin::CreatePendingCondition => self.create_pending_condition,
            RuntimeBuiltin::CreateErrorCondition => self.create_error_condition,
            RuntimeBuiltin::CreateTypeErrorCondition => self.create_type_error_condition,
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition => {
                self.create_invalid_function_target_condition
            }
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition => {
                self.create_invalid_function_args_condition
            }
            RuntimeBuiltin::CreateInvalidPointerCondition => self.create_invalid_pointer_condition,
            RuntimeBuiltin::CreateConstructor => self.create_constructor,
            RuntimeBuiltin::CreateDate => self.create_date,
            RuntimeBuiltin::CreateEffect => self.create_effect,
            RuntimeBuiltin::CreateFloat => self.create_float,
            RuntimeBuiltin::CreateHashset => self.create_hashset,
            RuntimeBuiltin::CreateInt => self.create_int,
            RuntimeBuiltin::CreateLambda => self.create_lambda,
            RuntimeBuiltin::CreateLet => self.create_let,
            RuntimeBuiltin::CreateNil => self.create_nil,
            RuntimeBuiltin::CreatePartial => self.create_partial,
            RuntimeBuiltin::CreatePointer => self.create_pointer,
            RuntimeBuiltin::CreateRecord => self.create_record,
            RuntimeBuiltin::CreateSignal => self.create_signal,
            RuntimeBuiltin::CreateTree => self.create_tree,
            RuntimeBuiltin::CreateVariable => self.create_variable,
            RuntimeBuiltin::CreateEmptyIterator => self.create_empty_iterator,
            RuntimeBuiltin::CreateEvaluateIterator => self.create_evaluate_iterator,
            RuntimeBuiltin::CreateFilterIterator => self.create_filter_iterator,
            RuntimeBuiltin::CreateFlattenIterator => self.create_flatten_iterator,
            RuntimeBuiltin::CreateHashmapKeysIterator => self.create_hashmap_keys_iterator,
            RuntimeBuiltin::CreateHashmapValuesIterator => self.create_hashmap_values_iterator,
            RuntimeBuiltin::CreateIntegersIterator => self.create_integers_iterator,
            RuntimeBuiltin::CreateIntersperseIterator => self.create_intersperse_iterator,
            RuntimeBuiltin::CreateMapIterator => self.create_map_iterator,
            RuntimeBuiltin::CreateOnceIterator => self.create_once_iterator,
            RuntimeBuiltin::CreateRangeIterator => self.create_range_iterator,
            RuntimeBuiltin::CreateRepeatIterator => self.create_repeat_iterator,
            RuntimeBuiltin::CreateSkipIterator => self.create_skip_iterator,
            RuntimeBuiltin::CreateTakeIterator => self.create_take_iterator,
            RuntimeBuiltin::CreateZipIterator => self.create_zip_iterator,
            RuntimeBuiltin::GetStringCharOffset => self.get_string_char_offset,
            RuntimeBuiltin::InitHashmap => self.init_hashmap,
            RuntimeBuiltin::InitList => self.init_list,
            RuntimeBuiltin::InitString => self.init_string,
            RuntimeBuiltin::InsertHashmapEntry => self.insert_hashmap_entry,
            RuntimeBuiltin::SetCellField => self.set_cell_field,
            RuntimeBuiltin::SetListItem => self.set_list_item,
            RuntimeBuiltin::Write => self.write,
        }
    }
}

fn parse_runtime_builtins(
    module: &walrus::Module,
) -> Result<RuntimeBuiltinMappings, WasmCompilerError> {
    let globals = parse_exported_globals(module)
        .map(|(name, global_id)| (String::from(name), global_id))
        .collect::<HashMap<_, _>>();
    let builtins = parse_exported_functions(module)
        .map(|(name, function_id)| (String::from(name), function_id))
        .collect::<HashMap<_, _>>();
    Ok(RuntimeBuiltinMappings {
        null_pointer: get_builtin_global(&globals, RuntimeGlobal::NullPointer)?,
        initialize: get_builtin_function(&builtins, RuntimeBuiltin::Initialize)?,
        evaluate: get_builtin_function(&builtins, RuntimeBuiltin::Evaluate)?,
        allocate_cell: get_builtin_function(&builtins, RuntimeBuiltin::AllocateCell)?,
        allocate_hashmap: get_builtin_function(&builtins, RuntimeBuiltin::AllocateHashmap)?,
        allocate_list: get_builtin_function(&builtins, RuntimeBuiltin::AllocateList)?,
        allocate_string: get_builtin_function(&builtins, RuntimeBuiltin::AllocateString)?,
        create_application: get_builtin_function(&builtins, RuntimeBuiltin::CreateApplication)?,
        create_boolean: get_builtin_function(&builtins, RuntimeBuiltin::CreateBoolean)?,
        create_builtin: get_builtin_function(&builtins, RuntimeBuiltin::CreateBuiltin)?,
        create_compiled: get_builtin_function(&builtins, RuntimeBuiltin::CreateCompiled)?,
        create_custom_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateCustomCondition,
        )?,
        create_pending_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreatePendingCondition,
        )?,
        create_error_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateErrorCondition,
        )?,
        create_type_error_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateTypeErrorCondition,
        )?,
        create_invalid_function_target_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition,
        )?,
        create_invalid_function_args_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition,
        )?,
        create_invalid_pointer_condition: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateInvalidPointerCondition,
        )?,
        create_constructor: get_builtin_function(&builtins, RuntimeBuiltin::CreateConstructor)?,
        create_date: get_builtin_function(&builtins, RuntimeBuiltin::CreateDate)?,
        create_effect: get_builtin_function(&builtins, RuntimeBuiltin::CreateEffect)?,
        create_float: get_builtin_function(&builtins, RuntimeBuiltin::CreateFloat)?,
        create_hashset: get_builtin_function(&builtins, RuntimeBuiltin::CreateHashset)?,
        create_int: get_builtin_function(&builtins, RuntimeBuiltin::CreateInt)?,
        create_lambda: get_builtin_function(&builtins, RuntimeBuiltin::CreateLambda)?,
        create_let: get_builtin_function(&builtins, RuntimeBuiltin::CreateLet)?,
        create_nil: get_builtin_function(&builtins, RuntimeBuiltin::CreateNil)?,
        create_partial: get_builtin_function(&builtins, RuntimeBuiltin::CreatePartial)?,
        create_pointer: get_builtin_function(&builtins, RuntimeBuiltin::CreatePointer)?,
        create_record: get_builtin_function(&builtins, RuntimeBuiltin::CreateRecord)?,
        create_signal: get_builtin_function(&builtins, RuntimeBuiltin::CreateSignal)?,
        create_tree: get_builtin_function(&builtins, RuntimeBuiltin::CreateTree)?,
        create_variable: get_builtin_function(&builtins, RuntimeBuiltin::CreateVariable)?,
        create_empty_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateEmptyIterator,
        )?,
        create_evaluate_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateEvaluateIterator,
        )?,
        create_filter_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateFilterIterator,
        )?,
        create_flatten_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateFlattenIterator,
        )?,
        create_hashmap_keys_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateHashmapKeysIterator,
        )?,
        create_hashmap_values_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateHashmapValuesIterator,
        )?,
        create_integers_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateIntegersIterator,
        )?,
        create_intersperse_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateIntersperseIterator,
        )?,
        create_map_iterator: get_builtin_function(&builtins, RuntimeBuiltin::CreateMapIterator)?,
        create_once_iterator: get_builtin_function(&builtins, RuntimeBuiltin::CreateOnceIterator)?,
        create_range_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateRangeIterator,
        )?,
        create_repeat_iterator: get_builtin_function(
            &builtins,
            RuntimeBuiltin::CreateRepeatIterator,
        )?,
        create_skip_iterator: get_builtin_function(&builtins, RuntimeBuiltin::CreateSkipIterator)?,
        create_take_iterator: get_builtin_function(&builtins, RuntimeBuiltin::CreateTakeIterator)?,
        create_zip_iterator: get_builtin_function(&builtins, RuntimeBuiltin::CreateZipIterator)?,
        get_string_char_offset: get_builtin_function(
            &builtins,
            RuntimeBuiltin::GetStringCharOffset,
        )?,
        init_hashmap: get_builtin_function(&builtins, RuntimeBuiltin::InitHashmap)?,
        init_list: get_builtin_function(&builtins, RuntimeBuiltin::InitList)?,
        init_string: get_builtin_function(&builtins, RuntimeBuiltin::InitString)?,
        insert_hashmap_entry: get_builtin_function(&builtins, RuntimeBuiltin::InsertHashmapEntry)?,
        set_cell_field: get_builtin_function(&builtins, RuntimeBuiltin::SetCellField)?,
        set_list_item: get_builtin_function(&builtins, RuntimeBuiltin::SetListItem)?,
        write: get_builtin_function(&builtins, RuntimeBuiltin::Write)?,
    })
}

fn get_builtin_global(
    globals: &HashMap<String, GlobalId>,
    target: RuntimeGlobal,
) -> Result<GlobalId, WasmCompilerError> {
    globals
        .get(target.name())
        .copied()
        .ok_or_else(|| WasmCompilerError::RuntimeGlobalNotFound(target))
}

fn get_builtin_function(
    builtins: &HashMap<String, FunctionId>,
    target: RuntimeBuiltin,
) -> Result<FunctionId, WasmCompilerError> {
    builtins
        .get(target.name())
        .copied()
        .ok_or_else(|| WasmCompilerError::RuntimeBuiltinNotFound(target))
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

fn compile_entry_points<A: Arena + Clone>(
    entry_points: impl IntoIterator<Item = (String, ArenaRef<Term, A>)>,
    heap_snapshot: &[u8],
) -> Result<(Vec<(String, CompiledExpression)>, Vec<u8>), WasmCompilerError> {
    // Initialize the compiler state with the contents of the initialized linear memory
    let mut compiler_state = CompilerState::from_heap_snapshot::<Term>(heap_snapshot);
    let compiled_entry_points = entry_points
        .into_iter()
        .map(|(export_name, expression)| {
            let function_body = expression
                .compile(
                    Eagerness::Eager,
                    &CompilerScope::default(),
                    &mut compiler_state,
                    &CompilerOptions::default(),
                )
                .map_err(|err| WasmCompilerError::CompilerError(anyhow::anyhow!("{}", err)))?;
            Ok((export_name, function_body))
        })
        .collect::<Result<Vec<_>, _>>()?;
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
            CompiledInstruction::Null => vec![Instr::GlobalGet(GlobalGet {
                global: builtin_mappings.null_pointer,
            })],
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

fn parse_exported_globals(module: &walrus::Module) -> impl Iterator<Item = (&str, GlobalId)> + '_ {
    module
        .exports
        .iter()
        .filter_map(|export| match export.item {
            ExportItem::Global(id) => Some((export.name.as_str(), id)),
            _ => None,
        })
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
    const RUNTIME_BYTES: &[u8] = include_bytes!("../../build/runtime.wasm");

    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    use crate::{
        allocator::{ArenaAllocator, VecAllocator},
        interpreter::{
            mocks::add_import_stubs, InterpreterError, WasmContextBuilder, WasmInterpreter,
        },
        stdlib::{Add, Stdlib},
        term_type::{ApplicationTerm, BuiltinTerm, IntTerm, ListTerm, TermType},
        ArenaPointer, ArenaRef, Term,
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

        let state = ArenaPointer::null();

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

        let state = ArenaPointer::null();

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
