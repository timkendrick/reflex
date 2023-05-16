// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{io::Write, iter::empty, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use reflex_lang::{allocator::DefaultAllocator, SharedTermFactory};
use reflex_parser::syntax::js::default_js_loaders;
use reflex_server::cli::compile::{parse_and_compile_module, CompileEntryPointArg};
use reflex_wasm::{
    cli::compile::{WasmCompilerOptions, WasmCompilerRuntimeOptions},
    compiler::CompilerOptions,
};

// Reflex WebAssembly compiler
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Named program entry points
    #[arg(short, long)]
    entry_point: Vec<CompileEntryPointArg>,
    /// Path to runtime library module
    #[arg(short, long)]
    runtime: PathBuf,
    /// Path to output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
    /// Whether to skip compile-time evaluation where applicable
    #[arg(long)]
    unoptimized: bool,
    #[arg(long)]
    /// Compile array items as lazily-evaluated expressions
    lazy_list_items: bool,
    /// Compile variable initializer values as lazily-evaluated expressions
    #[arg(long)]
    lazy_variable_initializers: bool,
    /// Wrap compiled lambdas in argument memoization wrappers
    #[arg(long)]
    memoize_lambdas: bool,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let runtime_path = &args.runtime;
    let entry_points = args.entry_point;
    let unoptimized = args.unoptimized;
    let factory = SharedTermFactory::<reflex_server::builtins::ServerBuiltins>::default();
    let allocator = DefaultAllocator::default();

    // Load the runtime library module
    let runtime_bytes =
        std::fs::read(&runtime_path).with_context(|| "Failed to load runtime library")?;

    let compiler_options = WasmCompilerOptions {
        compiler: CompilerOptions {
            lazy_list_items: args.lazy_list_items,
            lazy_variable_initializers: args.lazy_variable_initializers,
        },
        runtime: WasmCompilerRuntimeOptions {
            memoize_lambdas: args.memoize_lambdas,
        },
        ..Default::default()
    };

    // Parse the input file and compile to WASM
    let wasm_module = parse_and_compile_module(
        entry_points,
        default_js_loaders(empty(), &factory, &allocator),
        std::env::vars(),
        &runtime_bytes,
        &factory,
        &allocator,
        &compiler_options,
        unoptimized,
    )
    .with_context(|| "Failed to compile WebAssembly module")?;

    // Output compiled WASM module bytes
    match args.output {
        Some(name) => std::fs::write(&name, &wasm_module),
        None => std::io::stdout().write(&wasm_module).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
