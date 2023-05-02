// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{io::Write, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use reflex_wasm::cli::snapshot::{capture_heap_snapshot, inline_heap_snapshot, MemorySnapshot};

// Reflex WebAssembly memory snapshot tool
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to input WASM module
    #[arg(short, long)]
    input: PathBuf,

    /// Name of exported WASM memory
    #[arg(short, long)]
    memory_name: String,

    /// Whether to inline global variables as constants
    #[arg(short = 'g', long)]
    inline_globals: bool,

    /// Heap snapshot to inline into the WASM module (defaults to initial VM memory snapshot)
    #[arg(short, long)]
    snapshot: Option<PathBuf>,

    /// Path to output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let Args {
        input: input_path,
        memory_name,
        inline_globals,
        output: output_path,
        snapshot,
    } = args;

    // Load the WASM module
    let wasm_bytes = std::fs::read(&input_path).with_context(|| "Failed to load input module")?;

    // Capture the heap snapshot if one was not provided
    let snapshot = match snapshot {
        Some(snapshot_path) => std::fs::read(&snapshot_path)
            .map(MemorySnapshot::from_bytes)
            .with_context(|| "Failed to load heap snapshot"),
        None => capture_heap_snapshot(&wasm_bytes, &memory_name, inline_globals)
            .with_context(|| "Failed to capture initial heap snapshot"),
    }?;

    // Inline the heap snapshot into the WASM module source
    let output_bytes = inline_heap_snapshot(&wasm_bytes, &memory_name, snapshot)?;

    // Output .wasm file contents
    match output_path {
        Some(name) => std::fs::write(&name, output_bytes),
        None => std::io::stdout().write(&output_bytes).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
