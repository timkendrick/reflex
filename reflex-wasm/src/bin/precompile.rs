// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{io::Write, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use wasmtime::Engine;

// Cranelift precompiler tool for WebAssembly modules
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to input WASM module
    input: PathBuf,

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
    } = args;

    // Load the WASM module
    let wasm_bytes = std::fs::read(&input_path).with_context(|| "Failed to load input module")?;
    let engine = Engine::default();
    let output_bytes = engine.precompile_module(&wasm_bytes)?;

    // Output precompiled Cranelift output
    match output_path {
        Some(name) => std::fs::write(&name, &output_bytes),
        None => std::io::stdout().write(&output_bytes).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
