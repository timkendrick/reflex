// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{cell::RefCell, io::Write, rc::Rc};

use anyhow::{Context, Result};
use clap::Parser;
use reflex_wasm::{
    allocator::{ArenaAllocator, VecAllocator},
    compile::compile_module,
    term_type::{IntTerm, TermType},
    ArenaRef, Term,
};

// Reflex WebAssembly compiler
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Path to runtime library module
    #[arg(short, long)]
    runtime: String,

    /// Name of the exported WASM function
    #[arg(short, long)]
    export_name: String,

    /// Path to output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    // Parse CLI args
    let args = Args::parse();
    let runtime_path = &args.runtime;
    let entry_point_name = args.export_name;

    // Load the runtime library module
    let runtime_bytes =
        std::fs::read(&runtime_path).with_context(|| "Failed to load runtime library")?;

    // Create a dummy expression
    // TODO: allow compiling user-provided expressions
    let mut arena = VecAllocator::default();
    let term_pointer = arena.allocate(Term::new(TermType::Int(IntTerm::from(5)), &arena));
    let arena = Rc::new(RefCell::new(arena));
    let expression: ArenaRef<Term, _> = ArenaRef::new(arena.clone(), term_pointer);

    // Compile the expression into a WASM module
    let output_bytes = compile_module([(entry_point_name, expression)], &runtime_bytes)
        .with_context(|| "Failed to compile WASM module")?;

    // Output .wasm file contents
    match args.output {
        Some(name) => std::fs::write(&name, output_bytes),
        None => std::io::stdout().write(&output_bytes).map(|_| ()),
    }
    .with_context(|| "Failed to write output file")
}
