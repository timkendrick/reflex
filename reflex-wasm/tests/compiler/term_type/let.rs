// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::{compiler::CompilerOptions, stdlib};

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn let_term() {
    let scenario = LetTermLocalVariableLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermLocalVariableEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermAliasedVariableLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermAliasedVariableEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermNestedVariablesLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermNestedVariablesEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermSignalInitializerLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LetTermSignalInitializerEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct LetTermLocalVariableLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermLocalVariableLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, _allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(factory.create_int_term(3), factory.create_variable_term(0))
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermLocalVariableEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermLocalVariableEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, _allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(factory.create_int_term(3), factory.create_variable_term(0))
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermAliasedVariableLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermAliasedVariableLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, _allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_int_term(3),
            factory.create_let_term(
                factory.create_variable_term(0),
                factory.create_variable_term(0),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermAliasedVariableEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermAliasedVariableEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, _allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_int_term(3),
            factory.create_let_term(
                factory.create_variable_term(0),
                factory.create_variable_term(0),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermNestedVariablesLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermNestedVariablesLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_int_term(3),
            factory.create_let_term(
                factory.create_int_term(4),
                factory.create_let_term(
                    factory.create_int_term(5),
                    factory.create_let_term(
                        factory.create_variable_term(2),
                        factory.create_let_term(
                            factory.create_variable_term(2),
                            factory.create_let_term(
                                factory.create_variable_term(2),
                                factory.create_application_term(
                                    factory.create_builtin_term(stdlib::Subtract),
                                    allocator.create_pair(
                                        factory.create_variable_term(0),
                                        factory.create_variable_term(2),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(5 - 3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermNestedVariablesEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermNestedVariablesEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_int_term(3),
            factory.create_let_term(
                factory.create_int_term(4),
                factory.create_let_term(
                    factory.create_int_term(5),
                    factory.create_let_term(
                        factory.create_variable_term(2),
                        factory.create_let_term(
                            factory.create_variable_term(2),
                            factory.create_let_term(
                                factory.create_variable_term(2),
                                factory.create_application_term(
                                    factory.create_builtin_term(stdlib::Subtract),
                                    allocator.create_pair(
                                        factory.create_variable_term(0),
                                        factory.create_variable_term(2),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(5 - 3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermSignalInitializerLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermSignalInitializerLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_effect_term(allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            )),
            factory.create_int_term(3),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LetTermSignalInitializerEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LetTermSignalInitializerEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_effect_term(allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            )),
            factory.create_int_term(3),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result =
            factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            )]));
        let dependencies = vec![allocator.create_signal(
            SignalType::Custom(factory.create_string_term(allocator.create_static_string("foo"))),
            factory.create_int_term(3),
            factory.create_nil_term(),
        )];
        (result, dependencies)
    }
}
