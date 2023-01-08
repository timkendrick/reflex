// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashMap;

use walrus::{
    self, ActiveData, ActiveDataLocation, DataId, DataKind, ExportId, ExportItem, FunctionId,
    GlobalId, GlobalKind, InitExpr, MemoryId,
};

use crate::{
    allocator::ArenaAllocator,
    interpreter::{mocks::add_import_stubs, InterpreterError, WasmContextBuilder, WasmInterpreter},
};

// WASM specificies that memory is allocated in 64KiB pages
const WASM_PAGE_SIZE: u32 = 64 * 1024;

#[derive(Debug)]
pub enum WasmSnapshotError {
    ModuleLoadError(anyhow::Error),
    InterpreterError(InterpreterError),
    MemoryNotFound(String),
    FunctionNotFound(String),
    DataSectionNotFound,
    MultipleDataSections,
    InvalidDataSection,
    InvalidAstTransformation,
}

impl std::error::Error for WasmSnapshotError {}

impl std::fmt::Display for WasmSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleLoadError(err) => write!(f, "Failed to load WASM module: {err}"),
            Self::InterpreterError(err) => write!(f, "Failed to initialize interpreter: {err}"),
            Self::MemoryNotFound(name) => write!(f, "Memory definition not found: {name}"),
            Self::FunctionNotFound(name) => write!(f, "Function definition not found: {name}"),
            Self::DataSectionNotFound => write!(f, "Data section definition not found"),
            Self::MultipleDataSections => write!(f, "Multiple data section definitions"),
            Self::InvalidDataSection => write!(f, "Invalid data section definition"),
            Self::InvalidAstTransformation => write!(f, "Invalid AST transformation"),
        }
    }
}

pub struct MemorySnapshot {
    linear_memory: Vec<u8>,
    updated_globals: HashMap<String, wasmtime::Val>,
}

pub fn inline_heap_snapshot(
    wasm_bytes: &[u8],
    memory_name: &str,
) -> Result<Vec<u8>, WasmSnapshotError> {
    // Instantiate the runtime WASM module in an interpreter
    let mut interpreter =
        load_wasm_module(wasm_bytes, memory_name).map_err(WasmSnapshotError::InterpreterError)?;

    // Capture a memory snapshot of the ininitalized WASM module
    let MemorySnapshot {
        linear_memory,
        updated_globals,
    } = capture_initial_memory_snapshot(&mut interpreter)?;

    // Create a new WASM module based on the input bytes
    let mut ast = parse_wasm_ast(wasm_bytes)?;

    // Inline the updated global values from the snapshot
    let global_id_mappings = parse_exported_globals(&ast)
        .map(|(name, global_id, export_id)| (String::from(name), (global_id, export_id)))
        .collect::<HashMap<_, _>>();
    let global_values = updated_globals.into_iter().filter_map(|(key, value)| {
        let (global_id, export_id) = global_id_mappings.get(&key).copied()?;
        let value = match value {
            wasmtime::Val::I32(value) => Some(walrus::ir::Value::I32(value)),
            wasmtime::Val::I64(value) => Some(walrus::ir::Value::I64(value)),
            wasmtime::Val::F32(value) => Some(walrus::ir::Value::F32(f32::from_bits(value))),
            wasmtime::Val::F64(value) => Some(walrus::ir::Value::F64(f64::from_bits(value))),
            wasmtime::Val::V128(value) => Some(walrus::ir::Value::V128(value)),
            _ => None,
        }?;
        Some((global_id, export_id, value))
    });
    for (global_id, export_id, value) in global_values {
        let global = ast.globals.get_mut(global_id);
        global.kind = GlobalKind::Local(InitExpr::Value(value));
        global.mutable = false;
        ast.exports.delete(export_id);
    }

    // Update the module's initial memory allocation
    let linear_memory_size = linear_memory.len();
    let memory_id = get_named_memory_id(&ast, memory_name)
        .ok_or_else(|| WasmSnapshotError::MemoryNotFound(String::from(memory_name)))?;
    update_initial_heap_size(&mut ast, memory_id, linear_memory_size);

    // Update the module's linear memory initialization instruction with the allocated contents
    let heap_snapshot_id = get_data_section_instruction_id(&ast)?;
    update_data_segment(&mut ast, heap_snapshot_id, linear_memory);

    // Clear the _initialize method body
    let init_function_id = get_named_function_id(&ast, "_initialize")
        .ok_or_else(|| WasmSnapshotError::FunctionNotFound(String::from("_initialize")))?;
    clear_function_body(&mut ast, init_function_id)?;

    // Emit the resulting WASM as bytes
    Ok(ast.emit_wasm())
}

fn update_initial_heap_size(
    ast: &mut walrus::Module,
    memory_id: MemoryId,
    linear_memory_size: usize,
) {
    // Determine how much linear memory is required to store the initial heap snapshot
    let required_pages = 1 + ((linear_memory_size as u32).saturating_sub(1) / WASM_PAGE_SIZE);

    // If there is already enough memory allocated, nothing more to do
    let memory = ast.memories.get_mut(memory_id);
    if memory.initial >= required_pages {
        return;
    }

    // Otherwise increase the initial memory allocation to the next power of two
    memory.initial = required_pages.next_power_of_two();
}

fn update_data_segment(
    ast: &mut walrus::Module,
    data_segment_id: DataId,
    linear_memory: impl Into<Vec<u8>>,
) {
    ast.data.get_mut(data_segment_id).value = linear_memory.into();
}

fn load_wasm_module(
    runtime_wasm: &[u8],
    memory_name: &str,
) -> Result<WasmInterpreter, InterpreterError> {
    let builder = WasmContextBuilder::from_wasm(runtime_wasm, memory_name)?;
    let interpreter: WasmInterpreter = add_import_stubs(builder)
        .and_then(|builder| builder.build())?
        .into();
    Ok(interpreter)
}

fn capture_initial_memory_snapshot(
    interpreter: &mut WasmInterpreter,
) -> Result<MemorySnapshot, WasmSnapshotError> {
    // Snapshot the initial values of the interpreter globals
    let initial_global_values = capture_interpreter_globals(interpreter);

    // Invoke the _initialize function to pre-fill the linear memory
    interpreter
        .initialize()
        .map_err(WasmSnapshotError::InterpreterError)?;

    // Snapshot the updated values of the interpreter globals
    let updated_global_values = capture_interpreter_globals(interpreter);

    // Capture an updated heap snapshot
    let heap_snapshot = Vec::<u8>::from(&interpreter.data()[0..interpreter.len()]);

    // Determine the set of globals whose values have been mutated
    let modified_global_values = updated_global_values
        .into_iter()
        .filter_map(|(key, value)| {
            let initial_value = initial_global_values.get(&key)?;
            if !runtime_values_are_equal(&value, initial_value) {
                Some((key, value))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    Ok(MemorySnapshot {
        linear_memory: heap_snapshot,
        updated_globals: modified_global_values,
    })
}

fn capture_interpreter_globals(
    interpreter: &mut WasmInterpreter,
) -> HashMap<String, wasmtime::Val> {
    interpreter
        .get_globals()
        .map(|(export_name, value)| (String::from(export_name), value))
        .collect::<HashMap<_, _>>()
}

fn parse_wasm_ast(runtime_wasm: &[u8]) -> Result<walrus::Module, WasmSnapshotError> {
    walrus::Module::from_buffer(runtime_wasm).map_err(WasmSnapshotError::ModuleLoadError)
}

fn get_named_memory_id(ast: &walrus::Module, export_name: &str) -> Option<MemoryId> {
    parse_exported_memories(ast).find_map(|(name, memory_id)| {
        if name == export_name {
            Some(memory_id)
        } else {
            None
        }
    })
}

fn get_named_function_id(module: &walrus::Module, export_name: &str) -> Option<FunctionId> {
    parse_exported_functions(module).find_map(|(name, function_id)| {
        if name == export_name {
            Some(function_id)
        } else {
            None
        }
    })
}

fn clear_function_body(
    module: &mut walrus::Module,
    function_id: FunctionId,
) -> Result<(), WasmSnapshotError> {
    let init_function = match &mut module.funcs.get_mut(function_id).kind {
        walrus::FunctionKind::Local(func) => Some(func),
        _ => None,
    }
    .ok_or_else(|| WasmSnapshotError::InvalidAstTransformation)?;
    init_function.builder_mut().func_body().instrs_mut().clear();
    Ok(())
}

fn get_data_section_instruction_id(module: &walrus::Module) -> Result<DataId, WasmSnapshotError> {
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
                Err(WasmSnapshotError::InvalidDataSection)
            }
        }
        (Some(_), Some(_)) => Err(WasmSnapshotError::MultipleDataSections),
        (None, _) => Err(WasmSnapshotError::DataSectionNotFound),
    }
}

fn parse_exported_memories(module: &walrus::Module) -> impl Iterator<Item = (&str, MemoryId)> + '_ {
    module
        .exports
        .iter()
        .filter_map(|export| match export.item {
            ExportItem::Memory(id) => Some((export.name.as_str(), id)),
            _ => None,
        })
}

fn parse_exported_globals(
    module: &walrus::Module,
) -> impl Iterator<Item = (&str, GlobalId, ExportId)> + '_ {
    module
        .exports
        .iter()
        .filter_map(|export| match export.item {
            ExportItem::Global(id) => Some((export.name.as_str(), id, export.id())),
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

fn runtime_values_are_equal(left: &wasmtime::Val, right: &wasmtime::Val) -> bool {
    match (left, right) {
        (wasmtime::Val::I32(left), wasmtime::Val::I32(right)) => left == right,
        (wasmtime::Val::I64(left), wasmtime::Val::I64(right)) => left == right,
        (wasmtime::Val::F32(left), wasmtime::Val::F32(right)) => left == right,
        (wasmtime::Val::F64(left), wasmtime::Val::F64(right)) => left == right,
        (wasmtime::Val::V128(left), wasmtime::Val::V128(right)) => left == right,
        (wasmtime::Val::ExternRef(left), wasmtime::Val::ExternRef(right)) => match (left, right) {
            (Some(left), Some(right)) => left.ptr_eq(right),
            (None, None) => true,
            _ => false,
        },
        (wasmtime::Val::FuncRef(left), wasmtime::Val::FuncRef(right)) => match (left, right) {
            (Some(_), Some(_)) => true,
            (None, None) => true,
            _ => false,
        },
        _ => false,
    }
}
