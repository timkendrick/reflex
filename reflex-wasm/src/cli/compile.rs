// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    env::temp_dir,
    ffi::OsStr,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use anyhow::Context;
use derivative::Derivative;
use reflex::{
    cache::SubstitutionCache,
    core::{
        Arity, Expression, ExpressionFactory, HeapAllocator, LambdaTermType, ModuleLoader,
        Reducible, Rewritable, Uuid,
    },
};
use reflex_parser::{create_parser, ParserBuiltin, Syntax, SyntaxParser};
use strum::IntoEnumIterator;
use walrus::{
    self,
    ir::{InstrSeqId, Value},
    ActiveData, ActiveDataLocation, DataKind, ElementId, ElementKind, ExportItem, FunctionId,
    GlobalId, InitExpr, InstrSeqBuilder, LocalId, MemoryId, Module, TableId, ValType,
};

use crate::{
    allocator::{Arena, ArenaAllocator, ArenaIterator, VecAllocator},
    compiler::{
        error::TypedStackError,
        instruction::{self, CompiledInstruction},
        runtime::{builtin::RuntimeBuiltin, globals::RuntimeGlobal},
        wasm::generate::{
            generate_cached_function_wrapper, generate_indirect_function_wrapper,
            generate_stateful_function, RuntimeBuiltinMappings, RuntimeExportMappings,
            RuntimeGlobalMappings, RuntimeStdlibMappings, WasmCompiledFunctionMappings,
            WasmGeneratorError, WasmGeneratorOptions,
        },
        CapturingThunk, CompileWasm, CompiledBlock, CompiledFunctionId, CompiledLambda,
        CompiledThunk, CompilerOptions, CompilerStack, CompilerState, ConstValue, FunctionPointer,
        ParamsSignature, PureThunk, TypeSignature, ValueType,
    },
    factory::WasmTermFactory,
    hash::TermHasher,
    stdlib,
    term_type::{BuiltinTerm, LambdaTerm, TermType, TypedTerm},
    ArenaPointer, ArenaRef, FunctionIndex, IntoArenaRefIterator, PointerIter, Term, WASM_PAGE_SIZE,
};

const CACHED_FUNCTION_TEMPLATE: &'static [u8] =
    include_bytes!("../../templates/cached_function.wasm");

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum RuntimeEntryPointSyntax {
    Wasm,
    PrecompiledWasm,
    Source(Syntax),
}

impl RuntimeEntryPointSyntax {
    pub fn infer(file_extension: &OsStr) -> Option<Self> {
        match file_extension.to_str()? {
            "wasm" => Some(Self::Wasm),
            "cwasm" => Some(Self::PrecompiledWasm),
            _ => Syntax::infer(file_extension).map(Self::Source),
        }
    }
}

impl FromStr for RuntimeEntryPointSyntax {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "wasm" => Ok(Self::Wasm),
            "cwasm" => Ok(Self::PrecompiledWasm),
            input => Syntax::from_str(input).map(Self::Source),
        }
    }
}

pub trait CompilerEntryPoint<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
>
{
    fn export_name(&self) -> &ModuleEntryPoint;
    fn root(&self) -> &CompilerRootConfig;
    fn transform(&self, expression: &T, factory: &TFactory, allocator: &TAllocator) -> Option<T>;
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct GraphRootEntryPoint<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    export_name: ModuleEntryPoint,
    root: CompilerRootConfig,
    _expression: PhantomData<T>,
    _factory: PhantomData<TFactory>,
    _allocator: PhantomData<TAllocator>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    GraphRootEntryPoint<T, TFactory, TAllocator>
{
    pub fn new(export_name: ModuleEntryPoint, root: CompilerRootConfig) -> Self {
        Self {
            export_name,
            root,
            _expression: PhantomData,
            _factory: PhantomData,
            _allocator: PhantomData,
        }
    }
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    CompilerEntryPoint<T, TFactory, TAllocator> for GraphRootEntryPoint<T, TFactory, TAllocator>
{
    fn export_name(&self) -> &ModuleEntryPoint {
        &self.export_name
    }
    fn root(&self) -> &CompilerRootConfig {
        &self.root
    }
    fn transform(
        &self,
        _expression: &T,
        _factory: &TFactory,
        _allocator: &TAllocator,
    ) -> Option<T> {
        None
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct ExpressionFactoryEntryPoint<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    export_name: ModuleEntryPoint,
    root: CompilerRootConfig,
    _expression: PhantomData<T>,
    _factory: PhantomData<TFactory>,
    _allocator: PhantomData<TAllocator>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    ExpressionFactoryEntryPoint<T, TFactory, TAllocator>
{
    pub fn new(export_name: ModuleEntryPoint, root: CompilerRootConfig) -> Self {
        Self {
            export_name,
            root,
            _expression: PhantomData,
            _factory: PhantomData,
            _allocator: PhantomData,
        }
    }
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    CompilerEntryPoint<T, TFactory, TAllocator>
    for ExpressionFactoryEntryPoint<T, TFactory, TAllocator>
{
    fn export_name(&self) -> &ModuleEntryPoint {
        &self.export_name
    }
    fn root(&self) -> &CompilerRootConfig {
        &self.root
    }
    fn transform(&self, expression: &T, factory: &TFactory, _allocator: &TAllocator) -> Option<T> {
        Some(factory.create_lambda_term(0, expression.clone()))
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct ModuleEntryPoint(String);

impl ModuleEntryPoint {
    fn as_str(&self) -> &str {
        let Self(value) = self;
        value.as_str()
    }
}

impl Default for ModuleEntryPoint {
    fn default() -> Self {
        Self::from("main")
    }
}

impl<'a> From<&'a str> for ModuleEntryPoint {
    fn from(value: &'a str) -> Self {
        Self(String::from(value))
    }
}

impl From<String> for ModuleEntryPoint {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ModuleEntryPoint> for String {
    fn from(value: ModuleEntryPoint) -> Self {
        let ModuleEntryPoint(value) = value;
        value
    }
}

#[derive(Debug, Clone)]
pub enum CompilerRootConfig {
    Lisp(LispCompilerRootConfig),
    Json(JsonCompilerRootConfig),
    JavaScript(JavaScriptCompilerRootConfig),
}

impl std::str::FromStr for CompilerRootConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match split_at_separator(':', s) {
            Some((entry_point_format, root)) if entry_point_format != "" => {
                match entry_point_format.to_lowercase().as_str() {
                    "sexpr" | "lisp" => LispCompilerRootConfig::from_str(root).map(Self::Lisp),
                    "javascript" | "js" => {
                        JavaScriptCompilerRootConfig::from_str(root).map(Self::JavaScript)
                    }
                    "json" => JsonCompilerRootConfig::from_str(root).map(Self::Json),
                    _ => Err(format!(
                        "Unsupported entry point format: {}",
                        entry_point_format
                    )),
                }
            }
            _ => Err(format!("Missing entry point format")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LispCompilerRootConfig {
    pub path: PathBuf,
}

impl From<PathBuf> for LispCompilerRootConfig {
    fn from(value: PathBuf) -> Self {
        Self { path: value }
    }
}

impl std::str::FromStr for LispCompilerRootConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
        })
    }
}

#[derive(Debug, Clone)]
pub struct JsonCompilerRootConfig {
    pub path: PathBuf,
}

impl From<PathBuf> for JsonCompilerRootConfig {
    fn from(value: PathBuf) -> Self {
        Self { path: value }
    }
}

impl std::str::FromStr for JsonCompilerRootConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
        })
    }
}

#[derive(Debug, Clone)]
pub struct JavaScriptCompilerRootConfig {
    pub path: PathBuf,
}

impl From<PathBuf> for JavaScriptCompilerRootConfig {
    fn from(value: PathBuf) -> Self {
        Self { path: value }
    }
}

impl std::str::FromStr for JavaScriptCompilerRootConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
        })
    }
}

#[derive(Debug)]
pub enum WasmCompilerError {
    ReadError(PathBuf, std::io::Error),
    ParseError(PathBuf, String),
    ModuleLoadError(anyhow::Error),
    TableNotFound,
    MultipleTables,
    DataSectionNotFound,
    MultipleDataSections,
    InvalidDataSection,
    MemoryNotFound,
    MultipleMemories,
    InvalidFunctionTable,
    IndirectFunctionCallArityLookupNotFound,
    InvalidIndirectFunctionCallArityLookup,
    InvalidFunctionId(CompiledFunctionId),
    StackError(TypedStackError),
    CompilerError(anyhow::Error),
    TemplateError(anyhow::Error),
    OptimizationError(wasm_opt::OptimizationError),
    OptimizationFileSystemError(std::io::Error),
    RuntimeGlobalNotFound(RuntimeGlobal),
    RuntimeBuiltinNotFound(RuntimeBuiltin),
    StdlibBuiltinNotFound(stdlib::Stdlib),
    GeneratorError(WasmGeneratorError),
}

impl std::error::Error for WasmCompilerError {}

impl std::fmt::Display for WasmCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(input_path, err) => write!(
                f,
                "Failed to read input file {}: {err}",
                input_path.display()
            ),
            Self::ParseError(input_path, err) => write!(
                f,
                "Failed to parse input file {}: {err}",
                input_path.display()
            ),
            Self::ModuleLoadError(err) => write!(f, "Failed to load WASM module: {err:?}"),
            Self::TableNotFound => write!(f, "Indirect function call table definition not found"),
            Self::MultipleTables => write!(f, "Multiple indirect function call table definitions"),
            Self::InvalidFunctionTable => {
                write!(f, "Invalid indirect function call table initializer")
            }
            Self::IndirectFunctionCallArityLookupNotFound => {
                write!(f, "Indirect function call arity lookup function not found")
            }
            Self::InvalidIndirectFunctionCallArityLookup => {
                write!(f, "Invalid indirect function call arity lookup function")
            }
            Self::DataSectionNotFound => write!(f, "Data section definition not found"),
            Self::MultipleDataSections => write!(f, "Multiple data section definitions"),
            Self::MemoryNotFound => write!(f, "Memory definition not found"),
            Self::MultipleMemories => write!(f, "Multiple memory definitions"),
            Self::InvalidDataSection => write!(f, "Invalid data section definition"),
            Self::StackError(err) => write!(f, "Stack error: {err}"),
            Self::CompilerError(err) => write!(f, "Failed to compile WASM output: {err:?}"),
            Self::TemplateError(err) => write!(f, "Failed to generate WASM template: {err:?}"),
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

pub fn parse_and_compile_module<
    'a,
    T: Expression + 'static,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    entry_points: impl IntoIterator<Item = &'a (impl CompilerEntryPoint<T, TFactory, TAllocator> + 'a)>,
    module_loader: (impl ModuleLoader<Output = T> + Clone + 'static),
    env_vars: impl IntoIterator<Item = (String, String)>,
    runtime: &[u8],
    factory: &TFactory,
    allocator: &TAllocator,
    compiler_options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<Vec<u8>, WasmCompilerError>
where
    T::Builtin: ParserBuiltin + Into<crate::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    let env = env_vars.into_iter().collect::<HashMap<_, _>>();
    let entry_points = entry_points
        .into_iter()
        .map({
            |entry_point| {
                compile_module_entry_point(
                    entry_point,
                    &env,
                    module_loader.clone(),
                    factory,
                    allocator,
                )
                .map(|expression| (entry_point.export_name(), expression))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Compile the expression into a WASM module
    compile_wasm_module(
        entry_points,
        runtime,
        factory,
        allocator,
        compiler_options,
        unoptimized,
    )
}

fn compile_module_entry_point<
    T: Expression + 'static,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    entry_point: &impl CompilerEntryPoint<T, TFactory, TAllocator>,
    env_vars: &HashMap<String, String>,
    module_loader: impl ModuleLoader<Output = T> + 'static,
    factory: &TFactory,
    allocator: &TAllocator,
) -> Result<T, WasmCompilerError>
where
    T::Builtin: ParserBuiltin + Into<crate::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    let env_vars = env_vars
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()));
    match entry_point.root() {
        CompilerRootConfig::Lisp(LispCompilerRootConfig { path }) => {
            compile_generic_module_entry_point(
                path,
                Syntax::Lisp,
                env_vars,
                module_loader,
                factory,
                allocator,
            )
        }
        CompilerRootConfig::Json(JsonCompilerRootConfig { path }) => {
            compile_generic_module_entry_point(
                path,
                Syntax::Json,
                env_vars,
                module_loader,
                factory,
                allocator,
            )
        }
        CompilerRootConfig::JavaScript(JavaScriptCompilerRootConfig { path }) => {
            compile_generic_module_entry_point(
                path,
                Syntax::JavaScript,
                env_vars,
                module_loader,
                factory,
                allocator,
            )
        }
    }
    .map(|expression| {
        entry_point
            .transform(&expression, factory, allocator)
            .unwrap_or(expression)
    })
}

fn compile_generic_module_entry_point<T: Expression + 'static>(
    input_path: &Path,
    syntax: Syntax,
    env_vars: impl IntoIterator<Item = (String, String)>,
    module_loader: impl ModuleLoader<Output = T> + 'static,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> Result<T, WasmCompilerError>
where
    T::Builtin: ParserBuiltin + Into<crate::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    // Parse the input file into an expression
    let source = std::fs::read_to_string(input_path)
        .map_err(|err| WasmCompilerError::ReadError(input_path.into(), err))?;
    let parser = create_parser(
        syntax,
        Some(input_path),
        module_loader,
        env_vars,
        factory,
        allocator,
    );
    let expression = parser
        .parse(&source)
        .map_err(|err| WasmCompilerError::ParseError(input_path.into(), err))?;
    Ok(expression)
}

fn compile_wasm_module<'a, T: Expression + 'static>(
    entry_points: impl IntoIterator<Item = (&'a ModuleEntryPoint, T)>,
    runtime: &[u8],
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
    compiler_options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<Vec<u8>, WasmCompilerError>
where
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T>,
    T::Builtin: Into<stdlib::Stdlib>,
{
    let mut arena = VecAllocator::default();
    let shared_arena = Rc::new(RefCell::new(&mut arena));

    let entry_point_functions = entry_points
        .into_iter()
        .map(|(export_name, expression)| {
            // Abstract any free variables from any internal lambda functions within the expression
            let expression = expression
                .hoist_free_variables(factory, allocator)
                .unwrap_or_else(|| expression.clone());

            // Partially-evaluate any pure expressions within the expression
            let expression = expression
                .normalize(factory, allocator, &mut SubstitutionCache::new())
                .unwrap_or(expression);

            // Convert the expression into the WASM term representation
            let wasm_term = WasmTermFactory::from(Rc::clone(&shared_arena))
                .import(&expression, factory)
                .map_err(|term| anyhow::anyhow!("Failed to compile term: {}", term))
                .map_err(WasmCompilerError::CompilerError)?;

            // Create a zero-argument factory function that returns the evaluated expression
            let entry_point_function = {
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

            Ok((export_name, entry_point_function))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Compile the expression into a WASM module
    compile_module(
        entry_point_functions,
        runtime,
        None,
        compiler_options,
        unoptimized,
    )
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WasmCompilerOptions {
    pub compiler: CompilerOptions,
    pub generator: WasmGeneratorOptions,
    pub runtime: WasmCompilerRuntimeOptions,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WasmCompilerRuntimeOptions {
    pub memoize_lambdas: bool,
}

pub fn compile_module<'a>(
    entry_points: impl IntoIterator<
        Item = (
            &'a ModuleEntryPoint,
            ArenaRef<TypedTerm<LambdaTerm>, impl Arena + Clone>,
        ),
    >,
    runtime_wasm: &[u8],
    heap_snapshot: Option<&[u8]>,
    options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<Vec<u8>, WasmCompilerError> {
    // wasm-opt doesn't currently support block params
    let overridden_options = if !unoptimized && !options.generator.disable_block_params {
        Some(WasmCompilerOptions {
            generator: WasmGeneratorOptions {
                disable_block_params: true,
                ..options.generator
            },
            compiler: options.compiler,
            runtime: options.runtime,
        })
    } else {
        None
    };
    let options = overridden_options.as_ref().unwrap_or(options);

    let entry_points = entry_points.into_iter().collect::<Vec<_>>();

    let mut cached_function_template_module = parse_wasm_ast(CACHED_FUNCTION_TEMPLATE)?;
    let cached_function_template_id = cached_function_template_module
        .exports
        .iter()
        .find_map(|export| match &export.item {
            ExportItem::Function(function_id) if export.name.as_str() == "main" => {
                Some(*function_id)
            }
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("Missing template main function"))
        .map_err(WasmCompilerError::TemplateError)?;

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
        Ok(HashMap::<CompiledFunctionId, Vec<&ModuleEntryPoint>>::new()),
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
            let function_identifier = CompiledFunctionId::from(term_hash);
            let entry = match compiled_lambdas.entry(function_identifier) {
                Entry::Occupied(_) => {
                    // If a mapping already exists for this ID, we have somehow generated a wrapper for a thunk that has
                    // the same hash as a compiled lambda function (which should be impossible)
                    Err(WasmCompilerError::InvalidFunctionId(function_identifier))
                }
                Entry::Vacant(entry) => Ok(entry),
            }?;
            let (thunk_function_body, compiled_function_term, free_variables) = match compiled_thunk
            {
                CompiledThunk::Pure(PureThunk {
                    thunk_function_body,
                    compiled_function_term,
                    ..
                }) => (thunk_function_body, compiled_function_term, Vec::new()),
                CompiledThunk::Capturing(CapturingThunk {
                    thunk_function_body,
                    compiled_function_term,
                    free_variables,
                    ..
                }) => (thunk_function_body, compiled_function_term, free_variables),
            };
            entry.insert(CompiledLambda {
                params: ParamsSignature::from_iter(
                    free_variables.iter().map(|_| ValueType::HeapPointer),
                ),
                body: thunk_function_body,
            });
            Ok((term_hash, (function_identifier, compiled_function_term)))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    // Sort all the compiled functions topologically to ensure they are linked in a valid order when generating WASM bytecode
    let compiled_functions = {
        // Register all internal lambdas as top-level evaluation roots, to ensure that all lambdas are reached
        for function_identifier in compiled_lambdas.keys().copied() {
            compiled_entry_points
                .entry(function_identifier)
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
        |results, (function_identifier, compiled_lambda)| {
            let mut function_ids = results?;
            let CompiledLambda { params, body } = compiled_lambda;
            let num_args = params.len();
            // Compile the function into a WASM function
            let compiled_function_id = generate_stateful_function(
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
            // Generate a cache wrapper for the compiled function that memoizes previous results for specific combinations of arguments
            let cached_function_id = if options.runtime.memoize_lambdas {
                generate_cached_function_wrapper(
                    &mut ast,
                    FunctionPointer::Lambda(function_identifier),
                    params,
                    compiled_function_id,
                    (
                        &mut cached_function_template_module,
                        cached_function_template_id,
                    ),
                )
                .with_context(|| "Failed to generate cached function wrapper template")
                .map_err(WasmCompilerError::TemplateError)?
            } else {
                compiled_function_id
            };
            // Generate an 'indirect call' wrapper that can be used to dynamically call the function at runtime via its
            // index within the WASM call_indirect table.
            // The indirect call wrapper takes an argument list term (pointer which allows dynamic arguments to be
            // passed to the function) and a pointer to the current state, and invokes the cached function wrapper
            let function_index = {
                let wrapper_id = generate_indirect_function_wrapper(
                    &mut ast,
                    cached_function_id,
                    num_args,
                    &export_mappings,
                );
                register_dynamic_function(&mut ast, function_table_initializer_id, wrapper_id)
            }?;
            // Store the cached function wrapper (to be used by later functions) and the indirect call wrapper
            function_ids.insert(
                function_identifier,
                cached_function_id,
                function_index,
                // TODO: Differentiate between eager/strict/lazy lambda arguments
                Arity::eager(num_args, 0, false),
            );
            Ok(function_ids)
        },
    )?;

    if !compiled_thunk_targets.is_empty() {
        // Now that we know the final linked WASM function IDs for all the compiled thunk wrapper functions, we can
        // patch the real function addresses into the cached thunk heap term wrappers (overriding the placeholder value)
        for (compiled_function_id, compiled_function_term) in compiled_thunk_targets.values() {
            let function_index = function_ids
                .get_indirect_call_function_index(*compiled_function_id)
                .ok_or_else(|| WasmCompilerError::InvalidFunctionId(*compiled_function_id))?;
            patch_heap_snapshot_builtin_target_uid(
                &mut heap_snapshot,
                *compiled_function_term,
                function_index,
            );
        }
        // Patching the thunk wrappers will have caused hashes to become invalid, so we need to recompute the hashes of
        // any affected terms
        recompute_invalidated_term_hashes(
            &mut heap_snapshot,
            compiled_thunk_targets
                .into_values()
                .map(|(_, compiled_function_term)| compiled_function_term),
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
            ast.exports.add(export_name.as_str(), function_id);
        }
    }

    // Rewrite the implementation of the function that exposes the arity of the stdlib indirect call wrappers
    // to additionally expose the arity of the compiled function wrappers
    let stdlib_indirect_call_arities = stdlib::Stdlib::iter()
        .map(|builtin| (FunctionIndex::from(u32::from(builtin)), builtin.arity()));
    let compiled_function_indirect_call_arities = function_ids
        .iter()
        .map(|(_, (_, function_index, arity))| (*function_index, *arity));
    update_indirect_call_arity_lookup_function(
        &mut ast,
        "__indirect_function_arity",
        main_function_table_id,
        stdlib_indirect_call_arities.chain(compiled_function_indirect_call_arities),
    )?;

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
    Ok(wasm_bytes)
}

fn recompute_invalidated_term_hashes(
    heap_snapshot: &mut [u8],
    invalidated_terms: impl IntoIterator<Item = ArenaPointer>,
) {
    let invalidated_terms = invalidated_terms.into_iter().collect::<HashSet<_>>();
    // Construct a topologically-sorted iterator of invalidated term pointers
    let updated_terms = {
        let mut updated_hashes = HashMap::<ArenaPointer, Option<usize>>::default();
        let arena = &*heap_snapshot;
        let start_offset = ArenaPointer::from(std::mem::size_of::<u32>() as u32);
        let end_offset = ArenaPointer::from(arena.len() as u32);
        let mut queue = VecDeque::from_iter(
            ArenaIterator::<Term, _>::new(&arena, start_offset, end_offset)
                .as_arena_refs::<Term>(&arena),
        );
        while let Some(term) = queue.pop_front() {
            let term_pointer = term.as_pointer();
            if updated_hashes.contains_key(&term_pointer) {
                continue;
            }
            let children = PointerIter::iter(&term)
                .map(|pointer| arena.read_value::<ArenaPointer, _>(pointer, |target| *target))
                .as_arena_refs::<Term>(&arena);
            let unprocessed_children = children
                .clone()
                .filter(|child| !updated_hashes.contains_key(&child.as_pointer()));
            let has_unprocessed_children = unprocessed_children.clone().next().is_some();
            if has_unprocessed_children {
                queue.push_front(term);
                for unprocessed_child in unprocessed_children {
                    queue.push_front(unprocessed_child)
                }
                continue;
            }
            let invalidated_child_depth = children
                .clone()
                .filter_map(|child| updated_hashes.get(&child.as_pointer()).copied())
                .max()
                .and_then(|depth| depth);
            let invalidated_depth = match invalidated_child_depth {
                Some(depth) => Some(depth + 1),
                None => {
                    if invalidated_terms.contains(&term_pointer) {
                        Some(0)
                    } else {
                        None
                    }
                }
            };
            updated_hashes.insert(term_pointer, invalidated_depth);
        }
        let mut updated_hashes = updated_hashes
            .into_iter()
            .filter_map(|(term_pointer, updated_hash)| {
                updated_hash.map(|depth| (term_pointer, depth))
            })
            .collect::<Vec<_>>();
        // Sort the list of invalidated terms topologically to ensure child terms are hashed before their parents
        updated_hashes.sort_by_key(|(_, depth)| *depth);
        updated_hashes
            .into_iter()
            .map(|(term_pointer, _)| term_pointer)
    };
    for term_pointer in updated_terms {
        let (hash_pointer, updated_hash) = {
            let arena = &*heap_snapshot;
            let term = ArenaRef::<Term, _>::new(arena, term_pointer);
            let hash_pointer = Term::get_hash_pointer(term_pointer);
            let updated_hash =
                term.read_value(|term| TermHasher::default().hash(term, &arena).finish());
            (hash_pointer, updated_hash)
        };
        let heap_offset = u32::from(hash_pointer) as usize;
        // Get the bytes to write at the given insertion offset
        // (WASM specifies that integers are encoded in little-endian format)
        let patch_bytes = u64::from(updated_hash).to_le_bytes();
        // Overwrite the bytes at the insertion offset
        // (note that this will require term hashes to be recomputed for any terms that reference this term)
        heap_snapshot[heap_offset + 0] = patch_bytes[0];
        heap_snapshot[heap_offset + 1] = patch_bytes[1];
        heap_snapshot[heap_offset + 2] = patch_bytes[2];
        heap_snapshot[heap_offset + 3] = patch_bytes[3];
        heap_snapshot[heap_offset + 4] = patch_bytes[4];
        heap_snapshot[heap_offset + 5] = patch_bytes[5];
        heap_snapshot[heap_offset + 6] = patch_bytes[6];
        heap_snapshot[heap_offset + 7] = patch_bytes[7];
    }
}

#[must_use]
fn update_indirect_call_arity_lookup_function(
    ast: &mut Module,
    export_name: &str,
    indirect_call_table_id: TableId,
    function_arities: impl IntoIterator<Item = (FunctionIndex, Arity)>,
) -> Result<(), WasmCompilerError> {
    // Collect a lookup table that maps indirect function call indices to their respective arities
    let function_arities = function_arities.into_iter().collect::<HashMap<_, _>>();
    // Locate the exported arity lookup function within the WASM module AST
    let indirect_call_arity_function_id = get_exported_function_id(ast, export_name)
        .ok_or_else(|| WasmCompilerError::IndirectFunctionCallArityLookupNotFound)?;
    // Introspect the WASM module AST to get the corresponding arity for each entry in the indirect call table
    let indirect_call_table_arities = {
        // Retrieve the indices of all prepopulated elements of the indirect call table
        let indirect_function_indices = ast
            .elements
            .iter()
            .filter_map(|elem| match &elem.kind {
                ElementKind::Active { table, offset } if *table == indirect_call_table_id => {
                    match offset {
                        InitExpr::Value(Value::I32(offset)) => Some(
                            elem.members
                                .iter()
                                .enumerate()
                                .map({
                                    let offset = *offset as u32;
                                    move |(index, value)| {
                                        (FunctionIndex::from(offset + (index as u32)), value)
                                    }
                                })
                                .filter_map(|(function_index, value)| match value {
                                    Some(_function_id) => Some(function_index),
                                    None => None,
                                }),
                        ),
                        _ => None,
                    }
                }
                _ => None,
            })
            .flatten();
        // Determine the number of indirect call table entries based on the largest function index
        let indirect_call_table_size = indirect_function_indices
            .map(u32::from)
            .max()
            .map(|index| index + 1)
            .unwrap_or(0);
        // Determine the arity for each of the entries in the indirect call table, filling any gaps with a default value
        (0..indirect_call_table_size)
            .map(FunctionIndex::from)
            .map(|function_index| {
                function_arities
                    .get(&function_index)
                    .copied()
                    .unwrap_or(Arity::eager(0, 0, false))
            })
            .collect::<Vec<_>>()
    };
    // Rewrite the existing arity lookup function with the updated function arities
    patch_indirect_call_arity_lookup_function(
        ast,
        indirect_call_arity_function_id,
        &indirect_call_table_arities,
    )
}

#[must_use]
fn patch_indirect_call_arity_lookup_function(
    ast: &mut Module,
    function_id: FunctionId,
    indirect_call_table_arities: &[Arity],
) -> Result<(), WasmCompilerError> {
    // Retrieve the arity lookup function for editing
    let (func_arg, mut func_body) = match &mut ast.funcs.get_mut(function_id).kind {
        walrus::FunctionKind::Local(function) => {
            let function_index_type = ValType::I32;
            let arity_value_type = [ValType::I32, ValType::I32];
            let function_type = ast.types.get(function.ty()).clone();
            let expected_function_params = &[function_index_type];
            let expected_function_results = &arity_value_type;
            if (function_type.params(), function_type.results())
                == (expected_function_params, expected_function_results)
            {
                function
                    .args
                    .get(0)
                    .copied()
                    .map(|arg| (arg, function.builder_mut().func_body()))
            } else {
                None
            }
        }
        _ => None,
    }
    .ok_or_else(|| WasmCompilerError::InvalidIndirectFunctionCallArityLookup)?;

    // Clear the existing function body implementation
    func_body.instrs_mut().clear();

    // Helper function for creating a branch of the arity lookup branch table
    fn build_arity_lookup_block(
        builder: &mut InstrSeqBuilder,
        arity: Arity,
        remaining: &[Arity],
        local_id: LocalId,
        mut parent_blocks: Vec<InstrSeqId>,
        default_block: InstrSeqId,
    ) {
        if remaining.is_empty() {
            // If all branches have been processed, embed the branch statement that selects the correct branch based on
            // the provided function index
            builder.block(None, |builder| {
                parent_blocks.push(builder.id());
                builder
                    // Get the function index from the provided local variable
                    .local_get(local_id)
                    // Break out of the block that corresponds to the function index,
                    // or the default block if an unknown function index was provided
                    .br_table(parent_blocks.into(), default_block);
            })
        } else {
            // There are still more branches to process, so create a control flow block (for the current entry to break
            // out of) and recurse within that block to process the remaining branches
            builder.block(None, |builder| {
                parent_blocks.push(builder.id());
                build_arity_lookup_block(
                    builder,
                    remaining[0],
                    &remaining[1..],
                    local_id,
                    parent_blocks,
                    default_block,
                );
            })
        }
        // Breaking out of this entry's block will cause the following instructions to be executed
        // Push the positional argument count onto the stack
        .i32_const({
            let num_positional_args = (arity.required().len() + arity.optional().len()) as u32;
            num_positional_args as i32
        })
        // Push the 'has variadic args' boolean onto the stack
        .i32_const({
            let has_variadic_args = if arity.variadic().is_some() {
                1u32
            } else {
                0u32
            };
            has_variadic_args as i32
        })
        // Now that the correct return values are on the stack,
        // the return statement will bail out of the arity lookup function regardless of which block we are currenly in
        .return_();
    }

    match indirect_call_table_arities.get(0).copied() {
        // If there were no provided function arities, no need to create a branch table
        None => &mut func_body,
        // Otherwise create a branch table that maps the provided function index to the corresponding arity
        Some(arity) => func_body.block(None, |builder| {
            build_arity_lookup_block(
                builder,
                arity,
                &indirect_call_table_arities[1..],
                func_arg,
                Vec::with_capacity(indirect_call_table_arities.len()),
                builder.id(),
            );
        }),
    }
    // Default arity return type (this will only be encountered if an unknown function index was provided)
    // Push the positional argument count onto the stack
    .i32_const({
        let num_positional_args = 0u32;
        num_positional_args as i32
    })
    // Push the 'has variadic args' boolean onto the stack
    .i32_const({
        let has_variadic_args = 0u32;
        has_variadic_args as i32
    });
    Ok(())
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
    compiled_function_term: ArenaPointer,
    function_index: FunctionIndex,
) {
    // Get the insertion offset of the placeholder target function ID to overwrite with the final value
    let target_uid_pointer = {
        let arena = &*heap_snapshot;
        let term = ArenaRef::<TypedTerm<BuiltinTerm>, _>::new(arena, compiled_function_term);
        term.as_inner().inner_pointer(|term| &term.uid)
    };
    let heap_offset = u32::from(target_uid_pointer) as usize;
    // Get the bytes to write at the given insertion offset
    // (WASM specifies that integers are encoded in little-endian format)
    let patch_bytes = u32::from(function_index).to_le_bytes();
    // Overwrite the bytes at the insertion offset
    // (note that this will require term hashes to be recomputed for any terms that reference this term)
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
            create_empty_list: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateEmptyList,
            )?,
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
            create_symbol: get_builtin_function(&exported_functions, RuntimeBuiltin::CreateSymbol)?,
            create_timestamp: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::CreateTimestamp,
            )?,
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
            get_boolean_value: get_builtin_function(
                &exported_functions,
                RuntimeBuiltin::GetBooleanValue,
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
            collect_constructor: get_stdlib_function(
                &exported_functions,
                stdlib::CollectConstructor.into(),
            )?,
            collect_hashmap: get_stdlib_function(
                &exported_functions,
                stdlib::CollectHashmap.into(),
            )?,
            collect_hashset: get_stdlib_function(
                &exported_functions,
                stdlib::CollectHashset.into(),
            )?,
            collect_list: get_stdlib_function(&exported_functions, stdlib::CollectList.into())?,
            collect_record: get_stdlib_function(&exported_functions, stdlib::CollectRecord.into())?,
            collect_signal: get_stdlib_function(&exported_functions, stdlib::CollectSignal.into())?,
            collect_string: get_stdlib_function(&exported_functions, stdlib::CollectString.into())?,
            collect_tree: get_stdlib_function(&exported_functions, stdlib::CollectTree.into())?,
            cons: get_stdlib_function(&exported_functions, stdlib::Cons.into())?,
            construct: get_stdlib_function(&exported_functions, stdlib::Construct.into())?,
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
            intersperse: get_stdlib_function(&exported_functions, stdlib::Intersperse.into())?,
            is_finite: get_stdlib_function(&exported_functions, stdlib::IsFinite.into())?,
            is_truthy: get_stdlib_function(&exported_functions, stdlib::IsTruthy.into())?,
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
            resolve_loader_results: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveLoaderResults.into(),
            )?,
            resolve_query_branch: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveQueryBranch.into(),
            )?,
            resolve_query_leaf: get_stdlib_function(
                &exported_functions,
                stdlib::ResolveQueryLeaf.into(),
            )?,
            resolve_record: get_stdlib_function(&exported_functions, stdlib::ResolveRecord.into())?,
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
    target: stdlib::Stdlib,
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

fn get_exported_function_id(module: &walrus::Module, export_name: &str) -> Option<FunctionId> {
    parse_exported_functions(module)
        .find(|(exported_name, _)| *exported_name == export_name)
        .map(|(_, function_id)| function_id)
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
        wasm_module: &[u8],
    ) -> Result<WasmInterpreter, InterpreterError> {
        let memory_name = "memory";
        let context = WasmContextBuilder::from_wasm(wasm_module, memory_name)?;
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
            [(&ModuleEntryPoint::from("foo"), entry_point)],
            RUNTIME_BYTES,
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
            [(&ModuleEntryPoint::from("foo"), entry_point)],
            RUNTIME_BYTES,
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

fn split_at_separator(separator: char, value: &str) -> Option<(&str, &str)> {
    let separator_index = value.find(separator)?;
    let (left, right) = value.split_at(separator_index);
    Some((left, &right[1..]))
}
