// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{cell::RefCell, io::Write, rc::Rc};

use anyhow::{bail, Context};
use clap::Parser;
use reflex::{
    core::Eagerness,
    hash::{HashId, IntMap},
};
use reflex_wasm::{
    allocator::{ArenaAllocator, ArenaIterator, VecAllocator},
    compiler::{
        generate_link_table, CompileWasm, CompilerError, CompilerState, ModuleLinkTable,
        RuntimeBuiltin, SerializerState,
    },
    interpreter::{mocks::add_import_stubs, InterpreterError, WasmContextBuilder, WasmInterpreter},
    term_type::{IntTerm, TermType},
    ArenaArrayIter, ArenaRef, Term, TermPointer,
};
use reflex_wasm::{IntoArenaRefIter, IntoArenaRefIterator};
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

    // Load .wasm file into wasmtime

    let interpreter: WasmInterpreter =
        add_import_stubs(WasmContextBuilder::from_path(runtime_path, "memory")?)?
            .build()?
            .into();

    let interpreter = Rc::new(RefCell::new(interpreter));

    // Load contents of "memory" into a SerializerState

    let iter: IntoArenaRefIter<Term, _, ArenaIterator<_, Term>> =
        ArenaIterator::new(interpreter.clone(), TermPointer::from(4)).as_arena_ref(&interpreter);
    let allocated_terms: IntMap<HashId, TermPointer> = iter
        .inspect(|aref| println!("{aref:?}"))
        .map(|arena_ref| (arena_ref.read_value(|term| term.id()), arena_ref.pointer))
        .collect();

    let mut compiler_state = CompilerState {
        serializer_state: SerializerState { allocated_terms },
        destination_arena: interpreter.clone(),
    };

    // Compile the expression, serializing all interned expressions

    let (arena, five_expression) = create_5_expression()?;

    let arena = Rc::new(RefCell::new(arena));

    let target_expr: ArenaRef<Term, _> = ArenaRef::new(arena.clone(), five_expression);

    let compiled_expr =
        target_expr.compile(Eagerness::Lazy, &mut compiler_state, &Default::default())?;

    // Load .wasm file into walrus

    let mut module = walrus::Module::from_file(&runtime_path)?;
    let link_table =
        generate_link_table(&module.exports).with_context(|| "Failed to generate link table")?;

    // Clear the _initialize method

    let init_function = module
        .funcs
        .get_mut(*link_table.get(&RuntimeBuiltin::Initialize).unwrap());

    let init_function = match &mut init_function.kind {
        walrus::FunctionKind::Local(func) => func,
        _ => bail!("_initialize was not found as a local function"),
    };

    init_function.builder_mut().func_body().instrs_mut().clear();

    // Replace "memory" section with serialized data

    let data = module.data.iter().next().unwrap().id();
    let vec = &mut module.data.get_mut(data).value;

    vec.clear();
    vec.extend(
        interpreter
            .borrow()
            .as_slice(TermPointer::from(0), interpreter.len())
            .iter(),
    );

    // Add compiled export entry point

    let temp_local = module.locals.add(ValType::I32);

    let linked = compiled_expr.link_instrs(temp_local, &link_table);
    let evaluate_function = link_table.get(&RuntimeBuiltin::Evaluate).unwrap().clone();

    let node_factory = create_node_factory_function(&mut module, linked);
    let entry_point = create_entry_point_function(&mut module, node_factory, evaluate_function);

    module.exports.add(&entry_point_name, entry_point);

    // Output .wasm file

    let output_wasm = module.emit_wasm();

    match args.output {
        Some(name) => std::fs::write(&name, output_wasm)?,
        None => std::io::stdout().write(&output_wasm).map(|_| ())?,
    }

    Ok(())
}

fn compile_entry_point<A: ArenaAllocator + Clone, DestA: ArenaAllocator>(
    module: &mut walrus::Module,
    expression: ArenaRef<Term, A>,
    export_name: &str,
    state: &mut CompilerState<DestA>,
    link_table: &ModuleLinkTable,
) -> Result<Vec<u8>, CompilerError> {
    let compiled = expression.compile(Eagerness::Lazy, state, &Default::default())?;

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

        let wasm = compile_entry_point(
            &mut module,
            expression,
            "test_entrypoint_1",
            &mut CompilerState {
                serializer_state: Default::default(),
                destination_arena: arena.clone(),
            },
            &link_table,
        )
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

        let wasm = compile_entry_point(
            &mut module,
            expression,
            "test_deep_sum",
            &mut CompilerState {
                serializer_state: Default::default(),
                destination_arena: arena.clone(),
            },
            &link_table,
        )
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
