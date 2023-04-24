// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{io::Write, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use reflex_wasm::cli::entry_point::add_module_entry_point;

// Reflex WebAssembly entry point function generator tool
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to input WASM module
    #[arg(short, long)]
    input: PathBuf,

    /// Heap pointer to expression to evaluate
    #[arg(short, long)]
    entry_point: u32,

    /// Heap pointer to state object to use for evaluation
    #[arg(short, long)]
    state: Option<u32>,

    /// Name of exported entry point function
    #[arg(short = 'n', long)]
    export_name: String,

    /// Path to output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let Args {
        input: input_path,
        output: output_path,
        entry_point: expression_pointer,
        state: state_pointer,
        export_name,
    } = args;

    // Load the WASM module
    let wasm_bytes = std::fs::read(&input_path).with_context(|| "Failed to load input module")?;

    // Inline the heap snapshot into the WASM module source
    let output_bytes =
        add_module_entry_point(&wasm_bytes, &export_name, expression_pointer, state_pointer)?;

    // Output .wasm file contents
    match output_path {
        Some(name) => std::fs::write(&name, output_bytes),
        None => std::io::stdout().write(&output_bytes).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
