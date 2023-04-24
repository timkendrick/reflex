// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use reflex_wasm::{
    allocator::Arena, cli::compile::WasmProgram, interpreter::WasmInterpreter, ArenaPointer,
};

// Reflex WebAssembly standalone WASM module interpretr
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to input WASM module
    input: PathBuf,

    /// Name of exported entry point function (defaults to WASI `"_start" `function)
    #[arg(short, long)]
    entry_point: Option<String>,

    /// Whether the provided WebAssembly module has been precompiled via Cranelift
    #[arg(short, long)]
    precompiled: bool,

    /// Whether to display human-readable output
    #[arg(short, long)]
    formatted: bool,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let Args {
        input: input_path,
        entry_point,
        precompiled,
        formatted,
    } = args;

    // Load the WASM module
    let wasm_bytes = std::fs::read(&input_path).with_context(|| "Failed to load input module")?;
    let wasm_module = if precompiled {
        WasmProgram::from_cwasm(wasm_bytes)
    } else {
        WasmProgram::from_wasm(wasm_bytes)
    };
    let mut interpreter = WasmInterpreter::instantiate(&wasm_module, "memory")
        .with_context(|| "Failed to instantiate WebAssembly interpreter")?;
    let (result, dependencies) = interpreter
        .call::<(), (u32, u32)>(
            entry_point
                .as_ref()
                .map(|entry_point| entry_point.as_str())
                .unwrap_or("_start"),
            (),
        )
        .with_context(|| "Failed to execute entry point function")?;
    match formatted {
        false => {
            println!("{result}\n{dependencies}");
            Ok(())
        }
        true => {
            let output_start_offset = interpreter
                .call::<(), u32>("getAllocatorOffset", ())
                .with_context(|| "Failed to retrieve allocator offset")?;
            let output_end_offset = interpreter
                .call::<(u32, u32), u32>("debug", (result, output_start_offset))
                .with_context(|| "Failed to format result")?;
            let output_length = (output_end_offset - output_start_offset) as usize;
            let output_bytes =
                interpreter.as_slice(ArenaPointer::from(output_start_offset), output_length);
            let output_string = String::from_utf8(output_bytes.into())
                .with_context(|| "Failed to decode output string")?;
            println!("{output_string}");
            Ok(())
        }
    }
}
