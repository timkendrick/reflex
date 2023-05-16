// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use reflex::{
    core::{Expression, ExpressionFactory, HeapAllocator, ModuleLoader, Reducible, Rewritable},
    loader::{ChainedModuleLoader, StaticModuleLoader},
};
use reflex_graphql::{
    imports::{graphql_imports, GraphQlImportsBuiltin},
    GraphQlParserBuiltin,
};
use reflex_grpc::loader::{create_grpc_loader, GrpcModuleLoader};
use reflex_handlers::{
    imports::{handler_imports, HandlerImportsBuiltin},
    loader::{create_graphql_loader, GraphQlModuleLoader},
};
use reflex_js::{
    builtin_imports, compose_module_loaders, imports::JsImportsBuiltin, static_module_loader,
};
use reflex_parser::{create_parser, ParserBuiltin, Syntax, SyntaxParser};
use reflex_wasm::cli::compile::{compile_wasm_module, WasmCompilerError, WasmCompilerOptions};

#[derive(Debug, Clone)]
pub struct CompileEntryPointArg {
    /// Name of the exported WASM function
    export_name: String,
    /// Entry point definition
    root: CompileRootArg,
}

impl std::str::FromStr for CompileEntryPointArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match split_at_separator(':', s) {
            Some((export_name, root)) if export_name != "" => {
                CompileRootArg::from_str(root).map(|root| Self {
                    export_name: String::from(export_name),
                    root,
                })
            }
            _ => Err(format!("Missing entry point definition")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompileRootArg {
    Lisp(CompileLispRootArg),
    Json(CompileJsonRootArg),
    JavaScript(CompileJavaScriptRootArg),
}

impl std::str::FromStr for CompileRootArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match split_at_separator(':', s) {
            Some((entry_point_format, root)) if entry_point_format != "" => {
                match entry_point_format.to_lowercase().as_str() {
                    "sexpr" | "lisp" => CompileLispRootArg::from_str(root).map(Self::Lisp),
                    "javascript" | "js" => {
                        CompileJavaScriptRootArg::from_str(root).map(Self::JavaScript)
                    }
                    "json" => CompileJsonRootArg::from_str(root).map(Self::Json),
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
pub struct CompileLispRootArg {
    path: PathBuf,
}

impl std::str::FromStr for CompileLispRootArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompileJsonRootArg {
    path: PathBuf,
}

impl std::str::FromStr for CompileJsonRootArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompileJavaScriptRootArg {
    path: PathBuf,
}

impl std::str::FromStr for CompileJavaScriptRootArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
        })
    }
}

#[derive(Debug)]
pub enum ServerCompilerError {
    ParseError(PathBuf, String),
    CompilerError(WasmCompilerError),
}

impl std::error::Error for ServerCompilerError {}

impl std::fmt::Display for ServerCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(input_path, err) => write!(
                f,
                "Failed to parse input file {}: {err}",
                input_path.display()
            ),
            Self::CompilerError(err) => write!(f, "Failed to compile WASM output: {err}"),
        }
    }
}

pub fn parse_and_compile_module<T: Expression + 'static>(
    entry_points: impl IntoIterator<Item = CompileEntryPointArg>,
    module_loader: (impl ModuleLoader<Output = T> + Clone + 'static),
    env_vars: impl IntoIterator<Item = (String, String)>,
    runtime: &[u8],
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
    compiler_options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<Vec<u8>, ServerCompilerError>
where
    T::Builtin: ParserBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    let env = env_vars.into_iter().collect::<HashMap<_, _>>();
    let entry_points = entry_points
        .into_iter()
        .map({
            |entry_point| {
                let CompileEntryPointArg { export_name, root } = entry_point;
                compile_module_entry_point(&root, &env, module_loader.clone(), factory, allocator)
                    .map(|expression| (export_name, expression))
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
    .map_err(ServerCompilerError::CompilerError)
}

fn compile_module_entry_point<T: Expression + 'static>(
    root: &CompileRootArg,
    env_vars: &HashMap<String, String>,
    module_loader: impl ModuleLoader<Output = T> + 'static,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> Result<T, ServerCompilerError>
where
    T::Builtin: ParserBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    let env_vars = env_vars
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()));
    match root {
        CompileRootArg::Lisp(CompileLispRootArg { path }) => compile_generic_module_entry_point(
            path,
            Syntax::Lisp,
            env_vars,
            module_loader,
            factory,
            allocator,
        ),
        CompileRootArg::Json(CompileJsonRootArg { path }) => compile_generic_module_entry_point(
            path,
            Syntax::Json,
            env_vars,
            module_loader,
            factory,
            allocator,
        ),
        CompileRootArg::JavaScript(CompileJavaScriptRootArg { path }) => {
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
}

fn compile_generic_module_entry_point<T: Expression + 'static>(
    input_path: &Path,
    syntax: Syntax,
    env_vars: impl IntoIterator<Item = (String, String)>,
    module_loader: impl ModuleLoader<Output = T> + 'static,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> Result<T, ServerCompilerError>
where
    T::Builtin: ParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    // Parse the input file into an expression
    let source = std::fs::read_to_string(input_path).map_err(|err| {
        ServerCompilerError::ParseError(
            input_path.into(),
            format!("Failed to parse source file: {}", err),
        )
    })?;
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
        .map_err(|err| ServerCompilerError::ParseError(input_path.into(), err))?;
    Ok(expression)
}

pub fn create_loader<
    T: Expression + 'static,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    custom_imports: impl IntoIterator<Item = (String, T)>,
    factory: &TFactory,
    allocator: &TAllocator,
) -> ChainedModuleLoader<
    T,
    GraphQlModuleLoader<T, TFactory, TAllocator>,
    ChainedModuleLoader<T, GrpcModuleLoader<T, TFactory, TAllocator>, StaticModuleLoader<T>>,
>
where
    T::Builtin: JsImportsBuiltin + GraphQlImportsBuiltin + HandlerImportsBuiltin,
{
    let static_imports_loader = static_module_loader(
        builtin_imports(factory, allocator)
            .into_iter()
            .chain(handler_imports(factory, allocator))
            .chain(graphql_imports(factory, allocator))
            .map(|(key, value)| (String::from(key), value))
            .chain(custom_imports),
    );
    let graphql_loader = create_graphql_loader(factory, allocator);
    let grpc_loader = create_grpc_loader(factory, allocator);
    compose_module_loaders(
        graphql_loader,
        compose_module_loaders(grpc_loader, static_imports_loader),
    )
}

fn split_at_separator(separator: char, value: &str) -> Option<(&str, &str)> {
    let separator_index = value.find(separator)?;
    let (left, right) = value.split_at(separator_index);
    Some((left, &right[1..]))
}
