// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    env::temp_dir,
    ops::{Deref, DerefMut},
    path::PathBuf,
    rc::Rc,
};

use reflex::{
    cache::SubstitutionCache,
    core::{Expression, ExpressionFactory, HeapAllocator, LambdaTermType, Rewritable, Uuid},
};
use serde::{Deserialize, Serialize};
use walrus::{
    self, ir::Value, ActiveData, ActiveDataLocation, DataKind, ElementId, ElementKind, ExportItem,
    FunctionId, GlobalId, InitExpr, MemoryId, Module, TableId,
};

use crate::{
    allocator::{Arena, ArenaAllocator, VecAllocator},
    compiler::{
        error::TypedStackError,
        instruction::{self, CompiledInstruction},
        runtime::{builtin::RuntimeBuiltin, globals::RuntimeGlobal},
        wasm::{
            generate_indirect_function_wrapper, generate_stateful_function, RuntimeBuiltinMappings,
            RuntimeExportMappings, RuntimeGlobalMappings, RuntimeStdlibMappings,
            WasmCompiledFunctionMappings, WasmGeneratorError, WasmGeneratorOptions,
        },
        CapturingThunk, CompileWasm, CompiledBlock, CompiledFunctionId, CompiledLambda,
        CompiledThunk, CompilerOptions, CompilerStack, CompilerState, ConstValue, FunctionPointer,
        ParamsSignature, PureThunk, TypeSignature, ValueType,
    },
    factory::WasmTermFactory,
    stdlib::{self, Stdlib},
    term_type::{LambdaTerm, TermType, TypedTerm},
    ArenaPointer, ArenaRef, FunctionIndex, Term, WASM_PAGE_SIZE,
};

#[derive(Debug)]
pub enum WasmCompilerError {
    ModuleLoadError(anyhow::Error),
    TableNotFound,
    MultipleTables,
    DataSectionNotFound,
    MultipleDataSections,
    InvalidDataSection,
    MemoryNotFound,
    MultipleMemories,
    ParseError(PathBuf, String),
    InvalidFunctionTable,
    InvalidFunctionId(CompiledFunctionId),
    StackError(TypedStackError),
    CompilerError(anyhow::Error),
    OptimizationError(wasm_opt::OptimizationError),
    OptimizationFileSystemError(std::io::Error),
    RuntimeGlobalNotFound(RuntimeGlobal),
    RuntimeBuiltinNotFound(RuntimeBuiltin),
    StdlibBuiltinNotFound(Stdlib),
    GeneratorError(WasmGeneratorError),
}

impl std::error::Error for WasmCompilerError {}

impl std::fmt::Display for WasmCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleLoadError(err) => write!(f, "Failed to load WASM module: {err}"),
            Self::TableNotFound => write!(f, "Indirect function call table definition not found"),
            Self::MultipleTables => write!(f, "Multiple indirect function call table definitions"),
            Self::InvalidFunctionTable => {
                write!(f, "Invalid indirect function call table initializer")
            }
            Self::DataSectionNotFound => write!(f, "Data section definition not found"),
            Self::MultipleDataSections => write!(f, "Multiple data section definitions"),
            Self::MemoryNotFound => write!(f, "Memory definition not found"),
            Self::MultipleMemories => write!(f, "Multiple memory definitions"),
            Self::InvalidDataSection => write!(f, "Invalid data section definition"),
            Self::ParseError(input_path, err) => write!(
                f,
                "Failed to parse input file {}: {err}",
                input_path.display()
            ),
            Self::StackError(err) => write!(f, "Stack error: {err}"),
            Self::CompilerError(err) => write!(f, "Failed to compile WASM output: {err}"),
            Self::OptimizationError(err) => write!(f, "Failed to optimize WASM output: {err}"),
            Self::OptimizationFileSystemError(err) => {
                write!(f, "Failed to generate optimized WASM output: {err}")
            }
            Self::InvalidFunctionId(id) => write!(f, "Invalid function ID: {id}"),
            Self::GeneratorError(err) => write!(f, "Failed to generate WASM bytecode: {err}"),
            Self::RuntimeGlobalNotFound(target) => {
                write!(f, "Runtime global not found: {}", target.name())
            }
            Self::RuntimeBuiltinNotFound(target) => {
                write!(f, "Runtime function not found: {}", target.name())
            }
            Self::StdlibBuiltinNotFound(target) => {
                write!(f, "Standard library function not found: {}", target.name())
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum WasmCompilerMode {
    /// Standard WASM module
    Wasm,
    /// Cranelift-precompiled module
    Cranelift,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmProgram {
    pub(crate) compiler_mode: WasmCompilerMode,
    bytes: Vec<u8>,
}

impl WasmProgram {
    pub fn from_wasm(bytes: Vec<u8>) -> Self {
        Self {
            compiler_mode: WasmCompilerMode::Wasm,
            bytes,
        }
    }
    pub fn from_cwasm(bytes: Vec<u8>) -> Self {
        Self {
            compiler_mode: WasmCompilerMode::Cranelift,
            bytes,
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub fn compile_wasm_module<T: Expression + 'static>(
    expression: &T,
    export_name: &str,
    runtime: &[u8],
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
    compiler_mode: WasmCompilerMode,
    compiler_options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<WasmProgram, WasmCompilerError>
where
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T>,
    T::Builtin: Into<crate::stdlib::Stdlib>,
{
    // Abstract any free variables from any internal lambda functions within the expression
    let expression = expression
        .hoist_free_variables(factory, allocator)
        .unwrap_or_else(|| expression.clone());

    // Partially-evaluate any pure expressions within the expression
    let expression = expression
        .normalize(factory, allocator, &mut SubstitutionCache::new())
        .unwrap_or(expression);

    // Convert the expression into the WASM term representation
    let mut arena = VecAllocator::default();
    let allocator = &mut arena;
    let shared_arena = Rc::new(RefCell::new(allocator));
    let wasm_term = WasmTermFactory::from(Rc::clone(&shared_arena))
        .import(&expression, factory)
        .map_err(|term| anyhow::anyhow!("Failed to compile term: {}", term))
        .map_err(WasmCompilerError::CompilerError)?;

    let factory = {
        let factory_term = Term::new(
            TermType::Lambda(LambdaTerm {
                num_args: 0,
                body: wasm_term.pointer,
            }),
            &shared_arena,
        );
        let factory_pointer = shared_arena
            .deref()
            .borrow_mut()
            .deref_mut()
            .deref_mut()
            .allocate(factory_term);
        ArenaRef::<TypedTerm<LambdaTerm>, _>::new(Rc::clone(&shared_arena), factory_pointer)
    };

    // Compile the expression into a WASM module
    compile_module(
        [(String::from(export_name), factory)],
        runtime,
        compiler_mode,
        None,
        compiler_options,
        unoptimized,
    )
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WasmCompilerOptions {
    pub compiler: CompilerOptions,
    pub generator: WasmGeneratorOptions,
}

pub fn compile_module(
    entry_points: impl IntoIterator<
        Item = (String, ArenaRef<TypedTerm<LambdaTerm>, impl Arena + Clone>),
    >,
    runtime_wasm: &[u8],
    compiler_mode: WasmCompilerMode,
    heap_snapshot: Option<&[u8]>,
    options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<WasmProgram, WasmCompilerError> {
    let entry_points = entry_points.into_iter().collect::<Vec<_>>();

    // Create a new Wasm module based on the runtime bytes
    let mut ast = parse_wasm_ast(runtime_wasm)?;
    let export_mappings = parse_runtime_exports(&ast)?;

    // Locate the linear memory
    let memory_id = get_linear_memory_id(&ast)?;

    // Locate the dynamic function lookup table
    let main_function_table_id = get_main_function_table(&ast)?;
    let function_table_initializer_id = get_table_initializer(&ast, main_function_table_id)
        .ok_or(WasmCompilerError::InvalidFunctionTable)?;

    // Initialize the compiler state with the contents of the linear memory snapshot if one was provided,
    // otherwise load the linear memory snapshot from the inlined data sections in the WebAssembly module
    let mut compiler_state = if let Some(snapshot) = heap_snapshot {
        CompilerState::from_heap_snapshot::<Term>(snapshot)
    } else {
        let snapshot = collect_inline_data_snapshot(&ast, memory_id);
        CompilerState::from_heap_snapshot::<Term>(&snapshot)
    };

    // Compile the entry points, allocating any static expressions into the compiler state linear memory
    // (this will additionally compile all inner lambdas and thunks encountered along the way)
    let mut compiled_entry_points = entry_points.into_iter().fold(
        Ok(HashMap::<CompiledFunctionId, Vec<String>>::new()),
        |results, (export_name, lambda_term)| {
            let mut export_names = results?;
            let compiled_function_id = CompiledFunctionId::from(&lambda_term.as_inner());
            if !compiler_state
                .compiled_lambdas
                .contains_key(&compiled_function_id)
            {
                let params = (0..lambda_term.num_args())
                    .map(|_| ValueType::HeapPointer)
                    .collect::<ParamsSignature>();
                // Create a new compiler stack to be used within the function body,
                // with all the lambda arguments declared as scoped variables
                // and a block wrapper to catch short-circuiting signals
                let inner_stack = params
                    .iter()
                    .fold(CompilerStack::default(), |stack, value_type| {
                        stack.declare_variable(value_type)
                    })
                    .enter_block(&TypeSignature {
                        params: ParamsSignature::Void,
                        results: ParamsSignature::Single(ValueType::HeapPointer),
                    })
                    .map_err(WasmCompilerError::StackError)?;

                // Generate preliminary bytecode for the function body
                let body = lambda_term
                    .body()
                    .compile(inner_stack, &mut compiler_state, &options.compiler)
                    .map_err(|err| WasmCompilerError::CompilerError(anyhow::anyhow!("{}", err)))?;
                compiler_state
                    .compiled_lambdas
                    .insert(compiled_function_id, CompiledLambda { params, body });
            };
            match export_names.entry(compiled_function_id) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![export_name]);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push(export_name);
                }
            }
            Ok(export_names)
        },
    )?;

    // Dump the compiler state to get a comprehensive list of all the compiled lambdas and thunks,
    // as well as a heap snapshot containing any interned static terms
    let (mut heap_snapshot, mut compiled_lambdas, compiled_thunks) = compiler_state.into_parts();

    // Compile indirect call function wrappers for all thunks encountered during compilation
    let compiled_thunk_targets = compiled_thunks
        .into_iter()
        .map(|(term_hash, compiled_thunk)| {
            // Store the generated thunk alongside the compiled lambdas
            let compiled_function_id = CompiledFunctionId::from(term_hash);
            let entry = match compiled_lambdas.entry(compiled_function_id) {
                Entry::Occupied(_) => {
                    // If a mapping already exists for this ID, we have somehow generated a wrapper for a thunk that has
                    // the same hash as a compiled lambda function (which should be impossible)
                    Err(WasmCompilerError::InvalidFunctionId(compiled_function_id))
                }
                Entry::Vacant(entry) => Ok(entry),
            }?;
            let (thunk_function_body, target_uid_pointer, free_variables) = match compiled_thunk {
                CompiledThunk::Pure(PureThunk {
                    thunk_function_body,
                    target_uid_pointer,
                    ..
                }) => (thunk_function_body, target_uid_pointer, Vec::new()),
                CompiledThunk::Capturing(CapturingThunk {
                    thunk_function_body,
                    target_uid_pointer,
                    free_variables,
                    ..
                }) => (thunk_function_body, target_uid_pointer, free_variables),
            };
            entry.insert(CompiledLambda {
                params: ParamsSignature::from_iter(
                    free_variables.iter().map(|_| ValueType::HeapPointer),
                ),
                body: thunk_function_body,
            });
            Ok((term_hash, (compiled_function_id, target_uid_pointer)))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    // Sort all the compiled functions topologically to ensure they are linked in a valid order when generating WASM bytecode
    let compiled_functions = {
        // Register all internal lambdas as top-level evaluation roots, to ensure that all lambdas are reached
        for compiled_function_id in compiled_lambdas.keys().copied() {
            compiled_entry_points
                .entry(compiled_function_id)
                .or_insert(Vec::new());
        }
        // Sort the functions topologically (deepest first)
        sort_compiled_functions_by_call_graph_depth(
            compiled_lambdas,
            compiled_entry_points.keys().copied(),
        )
    }?;

    // Emit WASM bytecode for each of the compiled functions in order
    // (this relies on the functions having been topologically sorted so that later functions can reference prior ones)
    let function_ids = compiled_functions.fold(
        Ok(WasmCompiledFunctionMappings::default()),
        |results, (compiled_function_id, compiled_lambda)| {
            let mut function_ids = results?;
            let CompiledLambda { params, body } = compiled_lambda;
            let num_args = params.len();
            let function_id = generate_stateful_function(
                &mut ast,
                params.iter(),
                body,
                &export_mappings,
                memory_id,
                main_function_table_id,
                &mut function_ids,
                &options.generator,
            )
            .map_err(WasmCompilerError::GeneratorError)?;
            let function_index = {
                let wrapper_id = generate_indirect_function_wrapper(
                    &mut ast,
                    function_id,
                    num_args,
                    &export_mappings,
                );
                register_dynamic_function(&mut ast, function_table_initializer_id, wrapper_id)
            }?;
            function_ids.insert(compiled_function_id, function_id, function_index);
            Ok(function_ids)
        },
    )?;

    // Now that we know the final linked WASM function IDs for all the compiled thunk wrapper functions, we can patch
    // the real function addresses into the cached thunk heap term wrappers (overriding the placeholder value)
    for (compiled_function_id, target_uid_pointer) in compiled_thunk_targets.into_values() {
        let function_index = function_ids
            .get_indirect_call_function_index(compiled_function_id)
            .ok_or_else(|| WasmCompilerError::InvalidFunctionId(compiled_function_id))?;
        patch_heap_snapshot_builtin_target_uid(
            &mut heap_snapshot,
            target_uid_pointer,
            function_index,
        );
    }

    // Write the entry point functions into the module exports
    let exported_functions = compiled_entry_points
        .into_iter()
        .filter(|(_, export_names)| !export_names.is_empty());
    for (compiled_function_id, export_names) in exported_functions {
        let function_id = function_ids
            .get_function_id(compiled_function_id)
            .ok_or_else(|| WasmCompilerError::InvalidFunctionId(compiled_function_id))?;
        for export_name in export_names {
            // Add a WASM module export for the entry-point function
            ast.exports.add(&export_name, function_id);
        }
    }

    // Update the module's initial memory allocation
    let linear_memory_size = heap_snapshot.len();
    update_initial_heap_size(&mut ast, memory_id, linear_memory_size);

    // Replace the module's linear memory initialization instructions with the allocated contents
    let existing_data_section_ids = ast.data.iter().map(|data| data.id()).collect::<Vec<_>>();
    for data_id in existing_data_section_ids {
        ast.data.delete(data_id)
    }
    ast.data.add(
        DataKind::Active(ActiveData {
            location: ActiveDataLocation::Absolute(0),
            memory: memory_id,
        }),
        heap_snapshot,
    );

    // Emit the resulting WASM as bytes
    let wasm_bytes = ast.emit_wasm();

    let wasm_bytes = if unoptimized {
        Ok(wasm_bytes)
    } else {
        let unoptimized_filename = temp_dir().join(format!("{}.wasm", Uuid::new_v4()));
        let optimized_filename = temp_dir().join(format!("{}.wasm", Uuid::new_v4()));
        std::fs::write(&unoptimized_filename, wasm_bytes)
            .map_err(WasmCompilerError::OptimizationFileSystemError)
            .and_then(|_| {
                wasm_opt::OptimizationOptions::new_opt_level_4()
                    .debug_info(true)
                    .all_features()
                    .run(&unoptimized_filename, &optimized_filename)
                    .map_err(WasmCompilerError::OptimizationError)
                    .and_then(|_| {
                        std::fs::read(&optimized_filename)
                            .map_err(WasmCompilerError::OptimizationFileSystemError)
                    })
            })
    }?;

    match compiler_mode {
        WasmCompilerMode::Wasm => Ok(WasmProgram::from_wasm(wasm_bytes)),
        WasmCompilerMode::Cranelift => {
            let engine = wasmtime::Engine::default();
            engine
                .precompile_module(&wasm_bytes)
                .map_err(WasmCompilerError::CompilerError)
                .map(WasmProgram::from_cwasm)
        }
    }
}

pub fn parse_inline_memory_snapshot(wasm_bytes: &[u8]) -> Result<Vec<u8>, WasmCompilerError> {
    // Create a new WASM module based on the input bytes
    let ast = parse_wasm_ast(wasm_bytes)?;
    // Locate the named memory export
    let memory_id = get_linear_memory_id(&ast)?;
    // Parse any inline data sections that populate the given memory ID
    Ok(collect_inline_data_snapshot(&ast, memory_id))
}

fn patch_heap_snapshot_builtin_target_uid(
    heap_snapshot: &mut [u8],
    target_uid_pointer: ArenaPointer,
    function_index: FunctionIndex,
) {
    // Get the insertion offset
    let heap_offset = u32::from(target_uid_pointer) as usize;
    // Get the bytes to write at the given insertion offset
    // (WASM specifies that integers are encoded in little-endian format)
    let patch_bytes = u32::from(function_index).to_le_bytes();
    // Overwrite the bytes at the insertion offset
    heap_snapshot[heap_offset + 0] = patch_bytes[0];
    heap_snapshot[heap_offset + 1] = patch_bytes[1];
    heap_snapshot[heap_offset + 2] = patch_bytes[2];
    heap_snapshot[heap_offset + 3] = patch_bytes[3];
}

fn sort_compiled_functions_by_call_graph_depth(
    compiled_functions: impl IntoIterator<Item = (CompiledFunctionId, CompiledLambda)>,
    roots: impl IntoIterator<Item = CompiledFunctionId>,
) -> Result<impl Iterator<Item = (CompiledFunctionId, CompiledLambda)>, WasmCompilerError> {
    let mut compiled_functions = compiled_functions.into_iter().collect::<HashMap<_, _>>();
    let mut memoization_cache = HashMap::<CompiledFunctionId, usize>::default();
    for compiled_function_id in roots {
        let _ = get_compiled_function_call_graph_depth(
            compiled_function_id,
            &compiled_functions,
            &mut memoization_cache,
        )?;
    }
    let mut functions_with_depths = memoization_cache
        .into_iter()
        .map(|(compiled_function_id, depth)| {
            let compiled_lambda = compiled_functions
                .remove(&compiled_function_id)
                .ok_or_else(|| WasmCompilerError::InvalidFunctionId(compiled_function_id))?;
            Ok((compiled_function_id, compiled_lambda, depth))
        })
        .collect::<Result<Vec<_>, _>>()?;
    functions_with_depths.sort_by_key(|(compiled_function_id, _, depth)| {
        // Sorting shallowest nodes first ensures that child nodes occur before their respective parents
        let primary_sort = *depth;
        // Ensure deterministic sort order for nodes at the same depth by sorting by function hash
        let secondary_sort = *compiled_function_id;
        (primary_sort, secondary_sort)
    });
    Ok(functions_with_depths
        .into_iter()
        .map(|(compiled_function_id, compiled_lambda, _)| (compiled_function_id, compiled_lambda)))
}

fn get_compiled_function_call_graph_depth(
    compiled_function_id: CompiledFunctionId,
    compiled_functions: &HashMap<CompiledFunctionId, CompiledLambda>,
    memoization_cache: &mut HashMap<CompiledFunctionId, usize>,
) -> Result<usize, WasmCompilerError> {
    if let Some(existing_result) = memoization_cache.get(&compiled_function_id) {
        return Ok(*existing_result);
    }
    let compiled_lambda = compiled_functions
        .get(&compiled_function_id)
        .ok_or_else(|| WasmCompilerError::InvalidFunctionId(compiled_function_id))?;
    let depth = get_compiled_block_call_graph_depth(
        &compiled_lambda.body,
        compiled_functions,
        memoization_cache,
    )?;
    memoization_cache.insert(compiled_function_id, depth);
    Ok(depth)
}

fn get_compiled_block_call_graph_depth(
    block: &CompiledBlock,
    compiled_functions: &HashMap<CompiledFunctionId, CompiledLambda>,
    memoization_cache: &mut HashMap<CompiledFunctionId, usize>,
) -> Result<usize, WasmCompilerError> {
    block
        .iter()
        .map(|instruction| {
            get_compiled_instruction_call_graph_depth(
                instruction,
                compiled_functions,
                memoization_cache,
            )
        })
        .fold(Ok(0), |max_depth, instruction_depth| {
            let max_depth = max_depth?;
            let depth = instruction_depth?;
            Ok(depth.max(max_depth))
        })
}

fn get_compiled_instruction_call_graph_depth(
    instruction: &CompiledInstruction,
    compiled_functions: &HashMap<CompiledFunctionId, CompiledLambda>,
    memoization_cache: &mut HashMap<CompiledFunctionId, usize>,
) -> Result<usize, WasmCompilerError> {
    match instruction {
        CompiledInstruction::Const(instruction::core::Const {
            value: ConstValue::FunctionPointer(FunctionPointer::Lambda(target)),
        }) => {
            let target_depth = get_compiled_function_call_graph_depth(
                *target,
                compiled_functions,
                memoization_cache,
            )?;
            Ok(target_depth + 1)
        }
        CompiledInstruction::Block(instruction::core::Block { body, .. }) => {
            get_compiled_block_call_graph_depth(body, compiled_functions, memoization_cache)
        }
        CompiledInstruction::If(instruction::core::If {
            consequent,
            alternative,
            ..
        }) => {
            let consequent_depth = get_compiled_block_call_graph_depth(
                consequent,
                compiled_functions,
                memoization_cache,
            )?;
            let alternative_depth = get_compiled_block_call_graph_depth(
                alternative,
                compiled_functions,
                memoization_cache,
            )?;
            Ok(consequent_depth.max(alternative_depth))
        }
        CompiledInstruction::CallCompiledFunction(instruction::runtime::CallCompiledFunction {
            target,
            ..
        }) => {
            let target_depth = get_compiled_function_call_graph_depth(
                *target,
                compiled_functions,
                memoization_cache,
            )?;
            Ok(target_depth + 1)
        }
        _ => Ok(0),
    }
}

fn register_dynamic_function(
    ast: &mut Module,
    function_table_initializer_id: ElementId,
    function_id: FunctionId,
) -> Result<FunctionIndex, WasmCompilerError> {
    let entries = ast.elements.get_mut(function_table_initializer_id);
    let (table_id, start_offset) = match entries.kind {
        ElementKind::Active {
            table,
            offset: InitExpr::Value(Value::I32(offset)),
        } if offset >= 0 => Some((table, offset as usize)),
        _ => None,
    }
    .ok_or(WasmCompilerError::InvalidFunctionTable)?;
    let table = ast.tables.get_mut(table_id);
    let next_offset = (start_offset + entries.members.len()) as u32;
    let function_index = FunctionIndex::from(next_offset);
    entries.members.push(Some(function_id));
    table.initial += 1;
    if let Some(maximum) = table.maximum.as_mut() {
        *maximum = (*maximum).max(table.initial);
    }
    Ok(function_index)
}

fn parse_runtime_exports(module: &Module) -> Result<RuntimeExportMappings, WasmCompilerError> {
    let globals = parse_exported_globals(module)
        .map(|(name, global_id)| (String::from(name), global_id))
        .collect::<HashMap<_, _>>();
    let exported_functions = parse_exported_functions(module)
        .map(|(name, function_id)| (String::from(name), function_id))
        .collect::<HashMap<_, _>>();
    Ok(RuntimeExportMappings {
        globals: RuntimeGlobalMappings {
            null_pointer: get_builtin_global(&globals, RuntimeGlobal::NullPointer)?,
        },
        builtins: RuntimeBuiltinMappings {
            initialize: get_builtin_function(&exported_functions, RuntimeBuiltin::Initialize)?,
            evaluate: get_builtin_function(&exported_functions, RuntimeBuiltin::Evaluate)?,
            apply: get_builtin_function(&exported_functions, RuntimeBuiltin::Apply)?,
            combine_dependencies: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CombineDependencies,
            )?,
            combine_signals: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CombineSignals,
            )?,
            is_signal: get_builtin_function(&exported_functions, RuntimeBuiltin::IsSignal)?,
            is_truthy: get_builtin_function(&exported_functions, RuntimeBuiltin::IsTruthy)?,
            allocate_cell: get_builtin_function(&exported_functions, RuntimeBuiltin::AllocateCell)?,
            allocate_hashmap: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::AllocateHashmap,
            )?,
            allocate_list: get_builtin_function(&exported_functions, RuntimeBuiltin::AllocateList)?,
            allocate_string: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::AllocateString,
            )?,
            create_application: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateApplication,
            )?,
            create_boolean: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateBoolean,
            )?,
            create_builtin: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateBuiltin,
            )?,
            create_custom_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateCustomCondition,
            )?,
            create_pending_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreatePendingCondition,
            )?,
            create_error_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateErrorCondition,
            )?,
            create_type_error_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateTypeErrorCondition,
            )?,
            create_invalid_function_target_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateInvalidFunctionTargetCondition,
            )?,
            create_invalid_function_args_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateInvalidFunctionArgsCondition,
            )?,
            create_invalid_pointer_condition: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateInvalidPointerCondition,
            )?,
            create_constructor: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateConstructor,
            )?,
            create_date: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateDate)?,
            create_effect: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateEffect)?,
            create_float: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateFloat)?,
            create_hashset: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateHashset,
            )?,
            create_int: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateInt)?,
            create_lambda: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateLambda)?,
            create_nil: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateNil)?,
            create_partial: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreatePartial,
            )?,
            create_pointer: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreatePointer,
            )?,
            create_record: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateRecord)?,
            create_signal: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateSignal)?,
            create_tree: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateTree)?,
            create_empty_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateEmptyIterator,
            )?,
            create_evaluate_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateEvaluateIterator,
            )?,
            create_filter_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateFilterIterator,
            )?,
            create_flatten_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateFlattenIterator,
            )?,
            create_hashmap_keys_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateHashmapKeysIterator,
            )?,
            create_hashmap_values_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateHashmapValuesIterator,
            )?,
            create_indexed_accessor_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateIndexedAccessorIterator,
            )?,
            create_integers_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateIntegersIterator,
            )?,
            create_intersperse_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateIntersperseIterator,
            )?,
            create_map_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateMapIterator,
            )?,
            create_once_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateOnceIterator,
            )?,
            create_range_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateRangeIterator,
            )?,
            create_repeat_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateRepeatIterator,
            )?,
            create_skip_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateSkipIterator,
            )?,
            create_take_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateTakeIterator,
            )?,
            create_zip_iterator: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateZipIterator,
            )?,
            get_list_item: get_builtin_function(&exported_functions, RuntimeBuiltin::GetListItem)?,
            get_list_length: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::GetListLength,
            )?,
            get_state_value: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::GetStateValue,
            )?,
            get_string_char_offset: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::GetStringCharOffset,
            )?,
            init_hashmap: get_builtin_function(&exported_functions, RuntimeBuiltin::InitHashmap)?,
            init_list: get_builtin_function(&exported_functions, RuntimeBuiltin::InitList)?,
            init_string: get_builtin_function(&exported_functions, RuntimeBuiltin::InitString)?,
            insert_hashmap_entry: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::InsertHashmapEntry,
            )?,
            set_cell_field: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::SetCellField,
            )?,
            set_list_item: get_builtin_function(&exported_functions, RuntimeBuiltin::SetListItem)?,
        },
        stdlib: RuntimeStdlibMappings {
            abs: get_stdlib_function(&exported_functions, stdlib::Abs.into())?,
            accessor: get_stdlib_function(&exported_functions, stdlib::Accessor.into())?,
            add: get_stdlib_function(&exported_functions, stdlib::Add.into())?,
            and: get_stdlib_function(&exported_functions, stdlib::And.into())?,
            apply: get_stdlib_function(&exported_functions, stdlib::Apply.into())?,
            car: get_stdlib_function(&exported_functions, stdlib::Car.into())?,
            cdr: get_stdlib_function(&exported_functions, stdlib::Cdr.into())?,
            ceil: get_stdlib_function(&exported_functions, stdlib::Ceil.into())?,
            chain: get_stdlib_function(&exported_functions, stdlib::Chain.into())?,
            collect_hashmap: get_stdlib_function(
                &exported_functions,
                stdlib::CollectHashmap.into(),
            )?,
            collect_hashset: get_stdlib_function(
                &exported_functions,
                stdlib::CollectHashset.into(),
            )?,
            collect_list: get_stdlib_function(&exported_functions, stdlib::CollectList.into())?,
            collect_signal: get_stdlib_function(&exported_functions, stdlib::CollectSignal.into())?,
            collect_string: get_stdlib_function(&exported_functions, stdlib::CollectString.into())?,
            collect_tree: get_stdlib_function(&exported_functions, stdlib::CollectTree.into())?,
            cons: get_stdlib_function(&exported_functions, stdlib::Cons.into())?,
            construct: get_stdlib_function(&exported_functions, stdlib::Construct.into())?,
            construct_hashmap: get_stdlib_function(
                &exported_functions,
                stdlib::ConstructHashmap.into(),
            )?,
            construct_hashset: get_stdlib_function(
                &exported_functions,
                stdlib::ConstructHashset.into(),
            )?,
            construct_list: get_stdlib_function(&exported_functions, stdlib::ConstructList.into())?,
            construct_record: get_stdlib_function(
                &exported_functions,
                stdlib::ConstructRecord.into(),
            )?,
            debug: get_stdlib_function(&exported_functions, stdlib::Debug.into())?,
            decrement_variable: get_stdlib_function(
                &exported_functions,
                stdlib::DecrementVariable.into(),
            )?,
            divide: get_stdlib_function(&exported_functions, stdlib::Divide.into())?,
            effect: get_stdlib_function(&exported_functions, stdlib::Effect.into())?,
            ends_with: get_stdlib_function(&exported_functions, stdlib::EndsWith.into())?,
            eq: get_stdlib_function(&exported_functions, stdlib::Eq.into())?,
            equal: get_stdlib_function(&exported_functions, stdlib::Equal.into())?,
            filter: get_stdlib_function(&exported_functions, stdlib::Filter.into())?,
            flatten: get_stdlib_function(&exported_functions, stdlib::Flatten.into())?,
            floor: get_stdlib_function(&exported_functions, stdlib::Floor.into())?,
            fold: get_stdlib_function(&exported_functions, stdlib::Fold.into())?,
            format_error_message: get_stdlib_function(
                &exported_functions,
                stdlib::FormatErrorMessage.into(),
            )?,
            get: get_stdlib_function(&exported_functions, stdlib::Get.into())?,
            get_variable: get_stdlib_function(&exported_functions, stdlib::GetVariable.into())?,
            graph_ql_resolver: get_stdlib_function(
                &exported_functions,
                stdlib::GraphQlResolver.into(),
            )?,
            gt: get_stdlib_function(&exported_functions, stdlib::Gt.into())?,
            gte: get_stdlib_function(&exported_functions, stdlib::Gte.into())?,
            has: get_stdlib_function(&exported_functions, stdlib::Has.into())?,
            hash: get_stdlib_function(&exported_functions, stdlib::Hash.into())?,
            identity: get_stdlib_function(&exported_functions, stdlib::Identity.into())?,
            r#if: get_stdlib_function(&exported_functions, stdlib::If.into())?,
            if_error: get_stdlib_function(&exported_functions, stdlib::IfError.into())?,
            if_pending: get_stdlib_function(&exported_functions, stdlib::IfPending.into())?,
            increment_variable: get_stdlib_function(
                &exported_functions,
                stdlib::IncrementVariable.into(),
            )?,
            is_finite: get_stdlib_function(&exported_functions, stdlib::IsFinite.into())?,
            iterate: get_stdlib_function(&exported_functions, stdlib::Iterate.into())?,
            keys: get_stdlib_function(&exported_functions, stdlib::Keys.into())?,
            length: get_stdlib_function(&exported_functions, stdlib::Length.into())?,
            log: get_stdlib_function(&exported_functions, stdlib::Log.into())?,
            lt: get_stdlib_function(&exported_functions, stdlib::Lt.into())?,
            lte: get_stdlib_function(&exported_functions, stdlib::Lte.into())?,
            map: get_stdlib_function(&exported_functions, stdlib::Map.into())?,
            max: get_stdlib_function(&exported_functions, stdlib::Max.into())?,
            merge: get_stdlib_function(&exported_functions, stdlib::Merge.into())?,
            min: get_stdlib_function(&exported_functions, stdlib::Min.into())?,
            multiply: get_stdlib_function(&exported_functions, stdlib::Multiply.into())?,
            not: get_stdlib_function(&exported_functions, stdlib::Not.into())?,
            or: get_stdlib_function(&exported_functions, stdlib::Or.into())?,
            parse_date: get_stdlib_function(&exported_functions, stdlib::ParseDate.into())?,
            parse_float: get_stdlib_function(&exported_functions, stdlib::ParseFloat.into())?,
            parse_int: get_stdlib_function(&exported_functions, stdlib::ParseInt.into())?,
            parse_json: get_stdlib_function(&exported_functions, stdlib::ParseJson.into())?,
            pow: get_stdlib_function(&exported_functions, stdlib::Pow.into())?,
            push: get_stdlib_function(&exported_functions, stdlib::Push.into())?,
            push_front: get_stdlib_function(&exported_functions, stdlib::PushFront.into())?,
            raise: get_stdlib_function(&exported_functions, stdlib::Raise.into())?,
            remainder: get_stdlib_function(&exported_functions, stdlib::Remainder.into())?,
            replace: get_stdlib_function(&exported_functions, stdlib::Replace.into())?,
            resolve_args: get_stdlib_function(&exported_functions, stdlib::ResolveArgs.into())?,
            resolve_deep: get_stdlib_function(&exported_functions, stdlib::ResolveDeep.into())?,
            resolve_hashmap: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveHashmap.into(),
            )?,
            resolve_hashset: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveHashset.into(),
            )?,
            resolve_list: get_stdlib_function(&exported_functions, stdlib::ResolveList.into())?,
            resolve_query_branch: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveQueryBranch.into(),
            )?,
            resolve_query_leaf: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveQueryLeaf.into(),
            )?,
            resolve_record: get_stdlib_function(&exported_functions, stdlib::ResolveRecord.into())?,
            resolve_shallow: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveShallow.into(),
            )?,
            resolve_tree: get_stdlib_function(&exported_functions, stdlib::ResolveTree.into())?,
            round: get_stdlib_function(&exported_functions, stdlib::Round.into())?,
            scan: get_stdlib_function(&exported_functions, stdlib::Scan.into())?,
            sequence: get_stdlib_function(&exported_functions, stdlib::Sequence.into())?,
            set: get_stdlib_function(&exported_functions, stdlib::Set.into())?,
            set_variable: get_stdlib_function(&exported_functions, stdlib::SetVariable.into())?,
            skip: get_stdlib_function(&exported_functions, stdlib::Skip.into())?,
            slice: get_stdlib_function(&exported_functions, stdlib::Slice.into())?,
            split: get_stdlib_function(&exported_functions, stdlib::Split.into())?,
            starts_with: get_stdlib_function(&exported_functions, stdlib::StartsWith.into())?,
            stringify_json: get_stdlib_function(&exported_functions, stdlib::StringifyJson.into())?,
            subtract: get_stdlib_function(&exported_functions, stdlib::Subtract.into())?,
            take: get_stdlib_function(&exported_functions, stdlib::Take.into())?,
            throw: get_stdlib_function(&exported_functions, stdlib::Throw.into())?,
            to_request: get_stdlib_function(&exported_functions, stdlib::ToRequest.into())?,
            to_string: get_stdlib_function(&exported_functions, stdlib::ToString.into())?,
            urlencode: get_stdlib_function(&exported_functions, stdlib::Urlencode.into())?,
            unzip: get_stdlib_function(&exported_functions, stdlib::Unzip.into())?,
            values: get_stdlib_function(&exported_functions, stdlib::Values.into())?,
            zip: get_stdlib_function(&exported_functions, stdlib::Zip.into())?,
        },
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

fn get_stdlib_function(
    builtins: &HashMap<String, FunctionId>,
    target: Stdlib,
) -> Result<FunctionId, WasmCompilerError> {
    builtins
        .get(target.name())
        .copied()
        .ok_or_else(|| WasmCompilerError::StdlibBuiltinNotFound(target))
}

fn parse_wasm_ast(runtime_wasm: &[u8]) -> Result<Module, WasmCompilerError> {
    Module::from_buffer(runtime_wasm).map_err(WasmCompilerError::ModuleLoadError)
}

fn get_linear_memory_id(module: &walrus::Module) -> Result<MemoryId, WasmCompilerError> {
    let mut memories = module.memories.iter();
    match (memories.next(), memories.next()) {
        (Some(memory), None) => Ok(memory.id()),
        (Some(_), Some(_)) => Err(WasmCompilerError::MultipleMemories),
        (None, _) => Err(WasmCompilerError::MemoryNotFound),
    }
}

fn get_main_function_table(module: &walrus::Module) -> Result<TableId, WasmCompilerError> {
    let mut tables = module.tables.iter();
    match (tables.next(), tables.next()) {
        (Some(table), None) => Ok(table.id()),
        (Some(_), Some(_)) => Err(WasmCompilerError::MultipleTables),
        (None, _) => Err(WasmCompilerError::TableNotFound),
    }
}

fn get_table_initializer(ast: &Module, table_id: TableId) -> Option<ElementId> {
    ast.elements
        .iter()
        // Find the table initializers which correspond to this table
        .filter_map(|element| match element.kind {
            ElementKind::Active {
                table,
                offset: InitExpr::Value(Value::I32(offset)),
            } if table == table_id => Some((element.id(), offset)),
            _ => None,
        })
        // If there are multiple initializers, find the one with the highest offset
        .max_by(|(_, offset1), (_, offset2)| offset1.cmp(offset2))
        // Return the ID
        .map(|(element_id, _offset)| element_id)
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

fn update_initial_heap_size(
    ast: &mut walrus::Module,
    memory_id: MemoryId,
    linear_memory_size: usize,
) {
    // Determine how much linear memory is required to store the initial heap snapshot
    let required_pages = (1 + (linear_memory_size.saturating_sub(1) / WASM_PAGE_SIZE)) as u32;

    // If there is already enough memory allocated, nothing more to do
    let memory = ast.memories.get_mut(memory_id);
    if memory.initial >= required_pages {
        return;
    }

    // Otherwise increase the initial memory allocation to the next power of two
    memory.initial = required_pages.next_power_of_two();
}

fn collect_inline_data_snapshot(module: &walrus::Module, memory_id: MemoryId) -> Vec<u8> {
    let data_sections = module.data.iter().filter_map(|data| match &data.kind {
        DataKind::Active(ActiveData {
            location: ActiveDataLocation::Absolute(offset),
            memory,
        }) if memory == &memory_id => Some(((*offset as usize), data.value.clone())),
        _ => None,
    });
    data_sections.fold(Vec::new(), |mut linear_memory, (offset, data)| {
        let end_offset = offset + data.len();
        if linear_memory.len() < end_offset {
            linear_memory.resize(end_offset, 0);
        }
        linear_memory.splice(offset..end_offset, data).count();
        linear_memory
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
        term_type::{ApplicationTerm, BuiltinTerm, IntTerm, ListTerm, TermType, WasmExpression},
        ArenaPointer, ArenaRef, Term,
    };

    use super::*;

    fn create_mock_wasm_interpreter(
        wasm_module: &WasmProgram,
    ) -> Result<WasmInterpreter, InterpreterError> {
        let memory_name = "memory";
        let context = match wasm_module.compiler_mode {
            WasmCompilerMode::Wasm => {
                WasmContextBuilder::from_wasm(wasm_module.as_bytes(), memory_name)
            }
            WasmCompilerMode::Cranelift => {
                WasmContextBuilder::from_cwasm(wasm_module.as_bytes(), memory_name)
            }
        }?;
        let mut interpreter: WasmInterpreter = add_import_stubs(context)?.build()?.into();
        interpreter.initialize()?;
        Ok(interpreter)
    }

    #[test]
    fn primitive_expressions() {
        let mut arena = VecAllocator::default();
        let value = arena.allocate(Term::new(TermType::Int(IntTerm::from(5)), &arena));
        let main_function = arena.allocate(Term::new(
            TermType::Lambda(LambdaTerm {
                num_args: 0,
                body: value,
            }),
            &arena,
        ));

        let arena = Rc::new(RefCell::new(&mut arena));
        let entry_point = WasmExpression::new(arena.clone(), main_function)
            .as_lambda_term()
            .cloned()
            .unwrap();

        let wasm_bytes = compile_module(
            [("foo".into(), entry_point)],
            RUNTIME_BYTES,
            WasmCompilerMode::Wasm,
            None,
            &WasmCompilerOptions::default(),
            true,
        )
        .unwrap();

        let mut interpreter = create_mock_wasm_interpreter(&wasm_bytes).unwrap();

        let state = ArenaPointer::null();

        let interpreter = Rc::new(RefCell::new(&mut interpreter));
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
                .allocate(Term::new(TermType::Int(IntTerm::from(5)), &interpreter)),
        );

        assert_eq!(result.result(), expected_result);
        assert!(result.dependencies().is_none());
    }

    #[test]
    fn deeply_nested_applications() {
        let mut arena = VecAllocator::default();
        let value = {
            let mut current = arena.allocate(Term::new(TermType::Int(IntTerm::from(1)), &arena));

            let builtin = arena.allocate(Term::new(
                TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
                &arena,
            ));

            for i in 2..=64 {
                let new_int = arena.allocate(Term::new(TermType::Int(IntTerm::from(i)), &arena));

                let args = ListTerm::allocate([current, new_int], &mut arena);

                current = arena.allocate(Term::new(
                    TermType::Application(ApplicationTerm {
                        target: builtin,
                        args,
                        cache: Default::default(),
                    }),
                    &arena,
                ));
            }
            current
        };
        let main_function = arena.allocate(Term::new(
            TermType::Lambda(LambdaTerm {
                num_args: 0,
                body: value,
            }),
            &arena,
        ));

        let arena = Rc::new(RefCell::new(&mut arena));
        let entry_point = WasmExpression::new(arena.clone(), main_function)
            .as_lambda_term()
            .cloned()
            .unwrap();

        let wasm_bytes = compile_module(
            [("foo".into(), entry_point)],
            RUNTIME_BYTES,
            WasmCompilerMode::Wasm,
            None,
            &WasmCompilerOptions::default(),
            true,
        )
        .unwrap();

        let mut interpreter = create_mock_wasm_interpreter(&wasm_bytes).unwrap();

        let state = ArenaPointer::null();

        let interpreter = Rc::new(RefCell::new(&mut interpreter));

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
                .allocate(Term::new(TermType::Int(IntTerm::from(2080)), &interpreter)),
        );

        assert_eq!(result.result(), expected_result);
        assert!(result.dependencies().is_none());
    }
}
