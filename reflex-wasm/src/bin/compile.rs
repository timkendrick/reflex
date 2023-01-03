// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{cell::RefCell, rc::Rc};

use anyhow::Context;
use clap::Parser;
use reflex::core::Eagerness;
use reflex_wasm::{
    allocator::{ArenaAllocator, VecAllocator},
    compiler::{generate_link_table, CompileWasm, CompilerError, ModuleLinkTable, RuntimeBuiltin},
    interpreter::InterpreterError,
    term_type::{IntTerm, TermType},
    ArenaRef, Term, TermPointer,
};
use walrus::{self, FunctionId, ValType};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the input file
    #[arg(index = 1)]
    input: String,

    /// Name of the output file
    #[arg(short, long)]
    output: Option<String>,

    /// Name of the exported function
    #[arg(short, long)]
    entrypoint: String,
}

fn create_5_expression() -> Result<
    (
        impl for<'a> ArenaAllocator<Slice<'a> = &'a [u8]>,
        TermPointer,
    ),
    InterpreterError,
> {
    let mut arena = VecAllocator::default();

    let int5 = arena.allocate(Term::new(TermType::Int(IntTerm::from(5)), &arena));

    Ok((arena, int5))
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let runtime_path = &args.input;

    let entry_point_name = &args.entrypoint;

    let mut module = walrus::Module::from_file(&runtime_path)?;

    let link_table =
        generate_link_table(&module.exports).with_context(|| "Failed to generate link table")?;

    let (arena, expression) =
        create_5_expression().with_context(|| "Failed to create node factory expression")?;

    let arena = Rc::new(RefCell::new(arena));
    let expression = ArenaRef::new(Rc::clone(&arena), expression);

    let wasm = compile_entry_point(&mut module, expression, &entry_point_name, &link_table)
        .with_context(|| "Failed to compile entry point function")?;

    std::fs::write(args.output.as_ref().unwrap_or(runtime_path), wasm)?;
    Ok(())
}

fn compile_entry_point<A: ArenaAllocator + Clone>(
    module: &mut walrus::Module,
    expression: ArenaRef<Term, A>,
    export_name: &str,
    link_table: &ModuleLinkTable,
) -> Result<Vec<u8>, CompilerError> {
    let compiled = expression.compile(
        Eagerness::Eager,
        &mut Default::default(),
        &Default::default(),
    )?;

    let temp_local = module.locals.add(ValType::I32);

    let linked = compiled.link_instrs(temp_local, &link_table);
    let evaluate_function = link_table.get(&RuntimeBuiltin::Evaluate).unwrap().clone();

    let node_factory = create_node_factory_function(module, linked);
    let entry_point = create_entry_point_function(module, node_factory, evaluate_function);

    module.exports.add(&export_name, entry_point);

    Ok(module.emit_wasm())
}

fn create_node_factory_function(
    module: &mut walrus::Module,
    instructions: impl IntoIterator<Item = walrus::ir::Instr>,
) -> FunctionId {
    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[walrus::ValType::I32]);

    instructions
        .into_iter()
        .fold(&mut builder.func_body(), |acc, next| acc.instr(next));

    builder.finish(vec![], &mut module.funcs)
}

fn create_entry_point_function(
    module: &mut walrus::Module,
    node_factory: FunctionId,
    evaluate_function: FunctionId,
) -> FunctionId {
    let state = module.locals.add(walrus::ValType::I32);
    let mut builder = walrus::FunctionBuilder::new(
        &mut module.types,
        &[walrus::ValType::I32],
        &[walrus::ValType::I32, walrus::ValType::I32],
    );

    builder
        .func_body()
        .call(node_factory)
        .local_get(state)
        .call(evaluate_function);

    builder.finish(vec![state], &mut module.funcs)
}

#[cfg(test)]
mod tests {
    const RUNTIME_BYTES: &[u8] = include_bytes!("../../build/runtime.wasm");

    use std::ops::{Deref, DerefMut};

    use reflex_wasm::{
        interpreter::{mocks::add_import_stubs, WasmContextBuilder, WasmInterpreter},
        stdlib::{Add, Stdlib},
        term_type::{ApplicationTerm, BuiltinTerm, ListTerm},
    };

    use super::*;

    #[test]
    fn simple_int_factory() {
        let mut module = walrus::Module::from_buffer(RUNTIME_BYTES).unwrap();

        let link_table = generate_link_table(&module.exports)
            .with_context(|| "Failed to generate link table")
            .unwrap();

        let (arena, expression) = create_5_expression()
            .with_context(|| "Failed to create node factory expression")
            .unwrap();

        let arena = Rc::new(RefCell::new(arena));
        let expression = ArenaRef::new(Rc::clone(&arena), expression);

        let wasm = compile_entry_point(&mut module, expression, "test_entrypoint_1", &link_table)
            .with_context(|| "Failed to compile entry point function")
            .unwrap();

        let interpreter: WasmInterpreter =
            add_import_stubs(WasmContextBuilder::from_wasm(&wasm, "memory").unwrap())
                .unwrap()
                .build()
                .unwrap()
                .into();

        let state = TermPointer::null();

        let interpreter = Rc::new(RefCell::new(interpreter));
        let result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .execute("test_entrypoint_1", state)
            .unwrap()
            .bind(Rc::clone(&interpreter));

        let expected_result = ArenaRef::<Term, _>::new(
            Rc::clone(&interpreter),
            interpreter
                .deref()
                .borrow_mut()
                .deref_mut()
                .allocate(Term::new(TermType::Int(IntTerm { value: 5 }), &interpreter)),
        );

        assert_eq!(result.result(), expected_result);
        assert!(result.dependencies().is_none());
    }

    #[test]
    fn deep_add() {
        let mut module = walrus::Module::from_buffer(RUNTIME_BYTES).unwrap();

        let link_table = generate_link_table(&module.exports)
            .with_context(|| "Failed to generate link table")
            .unwrap();

        let mut arena = VecAllocator::default();

        let mut current = arena.allocate(Term::new(TermType::Int(IntTerm::from(1)), &arena));

        let add = arena.allocate(Term::new(
            TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
            &arena,
        ));

        for i in 2..=100 {
            let new_int = arena.allocate(Term::new(TermType::Int(IntTerm::from(i)), &arena));

            let list = ListTerm::allocate([current, new_int], &mut arena);

            current = arena.allocate(Term::new(
                TermType::Application(ApplicationTerm {
                    target: add,
                    args: list,
                }),
                &arena,
            ));
        }

        let arena = Rc::new(RefCell::new(arena));
        let expression = ArenaRef::new(Rc::clone(&arena), current);

        let wasm = compile_entry_point(&mut module, expression, "test_deep_sum", &link_table)
            .with_context(|| "Failed to compile entry point function")
            .unwrap();

        let state = TermPointer::null();

        let interpreter: WasmInterpreter =
            add_import_stubs(WasmContextBuilder::from_wasm(&wasm, "memory").unwrap())
                .unwrap()
                .build()
                .unwrap()
                .into();

        let interpreter = Rc::new(RefCell::new(interpreter));
        let result = interpreter
            .deref()
            .borrow_mut()
            .deref_mut()
            .execute("test_deep_sum", state)
            .unwrap()
            .bind(Rc::clone(&interpreter));

        let expected_result = ArenaRef::<Term, _>::new(
            Rc::clone(&interpreter),
            interpreter
                .deref()
                .borrow_mut()
                .deref_mut()
                .allocate(Term::new(
                    TermType::Int(IntTerm { value: 5050 }),
                    &interpreter,
                )),
        );

        assert_eq!(result.result(), expected_result);
        assert!(result.dependencies().is_none());
    }
}
