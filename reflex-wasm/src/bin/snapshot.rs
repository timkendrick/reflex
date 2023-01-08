// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use reflex_wasm::snapshot::inline_heap_snapshot;

// Reflex WebAssembly memory snapshot tool
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to input WASM module
    #[arg(short, long)]
    input: String,

    /// Name of exported WASM memory
    #[arg(short, long)]
    memory_name: String,

    /// Path to output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let Args {
        input: input_path,
        memory_name,
        output: output_path,
    } = args;

    // Load the WASM module
    let wasm_bytes = std::fs::read(&input_path).with_context(|| "Failed to load input module")?;

    // Inline the initial interpreter heap snapshot into the WASM module source
    let output_bytes = inline_heap_snapshot(&wasm_bytes, &memory_name)?;

    // Output .wasm file contents
    match output_path {
        Some(name) => std::fs::write(&name, output_bytes),
        None => std::io::stdout().write(&output_bytes).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
