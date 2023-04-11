// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::path::{Path, PathBuf};

use reflex::core::{Expression, ExpressionFactory, HeapAllocator, Reducible, Rewritable};
use reflex_graphql::imports::{graphql_imports, GraphQlImportsBuiltin};
use reflex_grpc::loader::create_grpc_loader;
use reflex_handlers::{
    imports::{handler_imports, HandlerImportsBuiltin},
    loader::graphql_loader,
};
use reflex_js::{builtin_imports, imports::JsImportsBuiltin, static_module_loader};
use reflex_parser::{create_parser, ParserBuiltin, Syntax, SyntaxParser};
use reflex_wasm::cli::compile::{
    compile_wasm_module, WasmCompilerError, WasmCompilerMode, WasmCompilerOptions, WasmProgram,
};

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
    source: &str,
    syntax: Syntax,
    input_path: &Path,
    module_loader: impl Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
    env_vars: impl IntoIterator<Item = (String, String)>,
    export_name: &str,
    runtime: &[u8],
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
    compiler_mode: WasmCompilerMode,
    compiler_options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<WasmProgram, ServerCompilerError>
where
    T::Builtin: ParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    // Parse the input file into an expression
    let parser = create_parser(
        syntax,
        Some(input_path),
        Some(module_loader),
        env_vars,
        factory,
        allocator,
    );
    let expression = parser
        .parse(source)
        .map_err(|err| ServerCompilerError::ParseError(input_path.into(), err))?;
    compile_wasm_module(
        &expression,
        export_name,
        runtime,
        factory,
        allocator,
        compiler_mode,
        compiler_options,
        unoptimized,
    )
    .map_err(ServerCompilerError::CompilerError)
}

pub fn create_loader<'a, T: Expression + 'static>(
    custom_imports: impl IntoIterator<Item = (String, T)>,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> impl Fn(&str, &Path) -> Option<Result<T, String>>
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
    let graphql_loader = graphql_loader(factory, allocator);
    let grpc_loader = create_grpc_loader(factory, allocator);
    move |import_path, module_path| {
        None.or_else(|| graphql_loader(import_path, module_path))
            .or_else(|| grpc_loader(import_path, module_path))
            .or_else(|| static_imports_loader(import_path, module_path))
    }
}
