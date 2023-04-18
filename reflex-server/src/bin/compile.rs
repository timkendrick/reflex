// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{io::Write, iter::empty, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use reflex_lang::{allocator::DefaultAllocator, SharedTermFactory};
use reflex_parser::Syntax;
use reflex_server::cli::compile::{create_loader, parse_and_compile_module};
use reflex_wasm::{
    cli::compile::{WasmCompilerMode, WasmCompilerOptions},
    compiler::CompilerOptions,
};

// Reflex WebAssembly compiler
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to program entry point
    entry_point: PathBuf,
    /// Input file syntax
    #[clap(long, default_value = "javascript")]
    syntax: Syntax,
    /// Name of the exported WASM function
    #[arg(short, long)]
    export_name: String,
    /// Path to runtime library module
    #[arg(short, long)]
    runtime: PathBuf,
    /// Path to output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
    /// Whether to precompile the resulting module with the Cranelift compiler
    #[arg(long)]
    precompile: bool,
    /// Whether to skip compile-time evaluation where applicable
    #[arg(long)]
    unoptimized: bool,
    #[arg(long)]
    /// Compile array items as lazily-evaluated expressions
    lazy_list_items: bool,
    /// Compile record field values as lazily-evaluated expressions
    #[arg(long)]
    lazy_record_values: bool,
    /// Compile variable initializer values as lazily-evaluated expressions
    #[arg(long)]
    lazy_variable_initializers: bool,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let runtime_path = &args.runtime;
    let input_path = args.entry_point;
    let syntax = args.syntax;
    let export_name = args.export_name;
    let compiler_mode = if args.precompile {
        WasmCompilerMode::Cranelift
    } else {
        WasmCompilerMode::Wasm
    };
    let unoptimized = args.unoptimized;
    let factory = SharedTermFactory::<reflex_server::builtins::ServerBuiltins>::default();
    let allocator = DefaultAllocator::default();

    // Load the runtime library module
    let runtime_bytes =
        std::fs::read(&runtime_path).with_context(|| "Failed to load runtime library")?;

    // Read the input file
    let source =
        std::fs::read_to_string(&input_path).with_context(|| "Failed to read input file")?;

    let mut compiler_options = WasmCompilerOptions {
        compiler: CompilerOptions {
            lazy_record_values: args.lazy_record_values,
            lazy_list_items: args.lazy_list_items,
            lazy_variable_initializers: args.lazy_variable_initializers,
        },
        ..Default::default()
    };
    if !unoptimized {
        // wasm-opt doesn't currently support block params
        compiler_options.generator.disable_block_params = true;
    }

    // Parse the input file and compile to WASM
    let wasm_module = parse_and_compile_module(
        &source,
        syntax,
        &input_path,
        create_loader(empty(), &factory, &allocator),
        std::env::vars(),
        &export_name,
        &runtime_bytes,
        &factory,
        &allocator,
        compiler_mode,
        &compiler_options,
        unoptimized,
    )
    .with_context(|| "Failed to compile WebAssembly module")?;

    // Output compiled WASM module bytes
    let output_bytes = wasm_module.as_bytes();
    match args.output {
        Some(name) => std::fs::write(&name, output_bytes),
        None => std::io::stdout().write(&output_bytes).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
