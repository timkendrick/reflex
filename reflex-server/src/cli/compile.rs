// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::path::{Path, PathBuf};

use reflex::core::{
    Expression, ExpressionFactory, HeapAllocator, ModuleLoader, Reducible, Rewritable,
};
use reflex_parser::{create_parser, ParserBuiltin, Syntax, SyntaxParser};
use reflex_wasm::cli::compile::{compile_wasm_module, WasmCompilerError, WasmCompilerOptions};

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
    module_loader: impl ModuleLoader<Output = T> + 'static,
    env_vars: impl IntoIterator<Item = (String, String)>,
    export_name: &str,
    runtime: &[u8],
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
    compiler_options: &WasmCompilerOptions,
    unoptimized: bool,
) -> Result<Vec<u8>, ServerCompilerError>
where
    T::Builtin: ParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    // Parse the input file into an expression
    let parser = create_parser(
        syntax,
        Some(input_path),
        module_loader,
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
        compiler_options,
        unoptimized,
    )
    .map_err(ServerCompilerError::CompilerError)
}
