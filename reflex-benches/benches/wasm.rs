// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::ops::Rem;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use reflex::cache::SubstitutionCache;
use reflex::core::{evaluate, InstructionPointer, StateCache};

use reflex_interpreter::compiler::{
    hash_compiled_program, Compiler, CompilerMode, CompilerOptions,
};
use reflex_interpreter::{DefaultInterpreterCache, InterpreterOptions};
use reflex_lang::allocator::DefaultAllocator;
use reflex_lang::{self, CachedSharedTerm, ExpressionList, SharedTermFactory};
use reflex_lisp::parse;
use reflex_wasm::allocator::ArenaAllocator;

use reflex_wasm::stdlib::{Add, Stdlib};
use reflex_wasm::*;
use reflex_wasm::{
    interpreter::{InterpreterError, WasmContextBuilder, WasmInterpreter},
    term_type::*,
};

criterion_group!(benches, simple_addition_benchmark, deep_addition_benchmark);
criterion_main!(benches);

fn simple_addition_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Simple");

    group.bench_function("Wasm", |b| {
        b.iter_batched(
            || {
                let mut interpreter = initialize_interpreter_context(RUNTIME_BYTES).unwrap();
                let (input, state) = generate_3_plus_5_wasm(&mut interpreter);
                (interpreter, input, state)
            },
            |(mut interpreter, input, state)| interpreter.interpret(input, state),
            criterion::BatchSize::PerIteration,
        );
    });

    group.bench_function("Rust", |b| {
        b.iter(|| execute_rust_benchmark(black_box(generate_3_plus_5_rust)))
    });

    group.finish();
}

fn deep_addition_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deep");

    for i in (100..=1000).step_by(100) {
        group.bench_with_input(BenchmarkId::new("Wasm", i), &i, |b, i| {
            b.iter_batched(
                || {
                    let mut interpreter = initialize_interpreter_context(RUNTIME_BYTES).unwrap();
                    let (input, state) = generate_deep_add_wasm(&mut interpreter, *i);
                    (interpreter, input, state)
                },
                |(mut interpreter, input, state)| interpreter.interpret(input, state),
                criterion::BatchSize::PerIteration,
            );
        });
        group.bench_with_input(BenchmarkId::new("Rust interpreted", i), &i, |b, i| {
            b.iter_batched(
                || {
                    let mut factory = SharedTermFactory::<reflex_stdlib::Stdlib>::default();
                    let mut allocator = DefaultAllocator::default();
                    let cache = SubstitutionCache::new();
                    let (input, state) = generate_deep_add_rust(&mut factory, &mut allocator, *i);
                    (input, state, factory, allocator, cache)
                },
                |(input, state, factory, allocator, mut cache)| {
                    evaluate(&input, &state, &factory, &allocator, &mut cache)
                },
                criterion::BatchSize::PerIteration,
            );
        });

        group.bench_with_input(BenchmarkId::new("Rust bytecode", i), &i, |b, i| {
            b.iter_batched(
                || {
                    let mut factory = SharedTermFactory::<reflex_stdlib::Stdlib>::default();
                    let mut allocator = DefaultAllocator::default();
                    let (expression, state) =
                        generate_deep_add_rust(&mut factory, &mut allocator, *i);

                    let program = Compiler::new(
                        CompilerOptions {
                            debug: false,
                            hoist_free_variables: false,
                            inline_static_data: false,
                            normalize: false,
                        },
                        None,
                    )
                    .compile(&expression, CompilerMode::Expression, &factory, &allocator)
                    .unwrap();
                    let state_id = 0;
                    let options = InterpreterOptions {
                        debug_instructions: false,
                        debug_stack: false,
                        call_stack_size: None,
                        variable_stack_size: None,
                    };

                    (program, state, factory, allocator, state_id, options)
                },
                |(program, state, factory, allocator, state_id, options)| {
                    let mut cache = DefaultInterpreterCache::default();
                    let entry_point = InstructionPointer::default();
                    let cache_key = hash_compiled_program(&program, &entry_point);
                    reflex_interpreter::execute(
                        cache_key,
                        &program,
                        entry_point,
                        state_id,
                        &state,
                        &factory,
                        &allocator,
                        &options,
                        &mut cache,
                    )
                },
                criterion::BatchSize::PerIteration,
            )
        });
    }

    group.finish()
}

const RUNTIME_BYTES: &'static [u8] = include_bytes!("../../reflex-wasm/build/runtime.cwasm");

fn execute_rust_benchmark(
    query_gen: impl Fn(
        &mut SharedTermFactory<reflex_stdlib::Stdlib>,
        &mut DefaultAllocator<CachedSharedTerm<reflex_stdlib::Stdlib>>,
    ) -> (
        CachedSharedTerm<reflex_stdlib::Stdlib>,
        StateCache<CachedSharedTerm<reflex_stdlib::Stdlib>>,
    ),
) {
    let mut factory = SharedTermFactory::<reflex_stdlib::Stdlib>::default();
    let mut allocator = DefaultAllocator::default();
    let mut cache = SubstitutionCache::new();
    let (expression, state) = query_gen(&mut factory, &mut allocator);
    evaluate(&expression, &state, &factory, &allocator, &mut cache);
}

fn generate_3_plus_5_wasm(interpreter: &mut WasmInterpreter) -> (TermPointer, TermPointer) {
    let int3 = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(3)), interpreter));

    let int2 = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(2)), interpreter));

    let add = interpreter.allocate(Term::new(
        TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
        interpreter,
    ));

    let list = ListTerm::allocate([int3, int2], interpreter);

    let application = interpreter.allocate(Term::new(
        TermType::Application(ApplicationTerm {
            target: add,
            args: list,
        }),
        interpreter,
    ));

    let state = HashmapTerm::allocate(std::iter::empty(), interpreter);

    (application, state)
}

fn generate_deep_add_wasm(
    interpreter: &mut WasmInterpreter,
    depth: i32,
) -> (TermPointer, TermPointer) {
    let mut current = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(1)), interpreter));

    let add = interpreter.allocate(Term::new(
        TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))),
        interpreter,
    ));

    for i in 2..=depth {
        let new_int = interpreter.allocate(Term::new(TermType::Int(IntTerm::from(i)), interpreter));

        let list = ListTerm::allocate([current, new_int], interpreter);

        current = interpreter.allocate(Term::new(
            TermType::Application(ApplicationTerm {
                target: add,
                args: list,
            }),
            interpreter,
        ));
    }

    let state = HashmapTerm::allocate(std::iter::empty(), interpreter);

    (current, state)
}

fn generate_deep_add_rust(
    _factory: &mut SharedTermFactory<reflex_stdlib::Stdlib>,
    _allocator: &mut DefaultAllocator<CachedSharedTerm<reflex_stdlib::Stdlib>>,
    depth: i32,
) -> (
    CachedSharedTerm<reflex_stdlib::Stdlib>,
    StateCache<CachedSharedTerm<reflex_stdlib::Stdlib>>,
) {
    use reflex_lang::term::{ApplicationTerm, BuiltinTerm, IntTerm, Term};

    let mut current = CachedSharedTerm::new(Term::Int(IntTerm::new(1)));
    let add = CachedSharedTerm::new(Term::Builtin(BuiltinTerm::new(reflex_stdlib::Stdlib::Add)));
    for i in 2..=depth {
        let new_int = CachedSharedTerm::new(Term::Int(IntTerm::new(i)));

        let list = ExpressionList::new([current, new_int]);

        current = CachedSharedTerm::new(Term::Application(ApplicationTerm::new(add.clone(), list)));
    }

    let state = StateCache::default();

    (current, state)
}

fn generate_3_plus_5_rust(
    factory: &mut SharedTermFactory<reflex_stdlib::Stdlib>,
    allocator: &mut DefaultAllocator<CachedSharedTerm<reflex_stdlib::Stdlib>>,
) -> (
    CachedSharedTerm<reflex_stdlib::Stdlib>,
    StateCache<CachedSharedTerm<reflex_stdlib::Stdlib>>,
) {
    let state = StateCache::default();
    let expression = parse("(+ 3 5)", factory, allocator).unwrap();
    (expression, state)
}

fn initialize_interpreter_context(wasm: &[u8]) -> Result<WasmInterpreter, InterpreterError> {
    WasmContextBuilder::from_cwasm(wasm)?
        .add_import("Math", "remainder", |a: f64, b: f64| a.rem(b))?
        .add_import("Math", "pow", |a: f64, b: f64| a.powf(b))?
        .add_import("Date", "parse", |_: u32, _: u32| 0u64)?
        .add_import("Date", "toISOString", |_: u64, _: u32| 0u32)?
        .add_import("Number", "toString", |_: f64, _: u32| 0u32)?
        .build()
        .map(WasmInterpreter::from)
}
