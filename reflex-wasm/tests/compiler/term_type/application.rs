// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn application_term_builtin_target() {
    let scenario = ApplicationTermBuiltinStaticArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermBuiltinSingleDynamicArgScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermBuiltinMultipleDynamicArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermBuiltinSingleSignalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermBuiltinMultipleSignalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermBuiltinNestedSignalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn application_term_variadic_target() {
    let scenario = ApplicationTermVariadicStaticArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermVariadicDynamicArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermVariadicUnresolvedSignalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermVariadicPartiallyResolvedSignalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermVariadicFullyResolvedSignalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermNestedVariadicBuiltinCallsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn application_term_lambda_target() {
    let scenario = ApplicationTermNullaryLambdaScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermUnaryLambdaScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermPolyadicLambdaScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn application_term_constructor_target() {
    let scenario = ApplicationTermConstructorScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn application_term_dynamic_target() {
    let scenario = ApplicationTermDynamicScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn application_term_partial_target() {
    let scenario = ApplicationTermPartialBuiltinScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermPartialVariadicScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermPartialLambdaScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermPartialConstructorScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermNestedPartialsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ApplicationTermPartialDynamicScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct ApplicationTermBuiltinStaticArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermBuiltinStaticArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3 + 4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermBuiltinSingleDynamicArgScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermBuiltinSingleDynamicArgScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Add),
                    allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
                ),
                factory.create_int_term(5),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3 + 4 + 5);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermBuiltinMultipleDynamicArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for ApplicationTermBuiltinMultipleDynamicArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Add),
                    allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Add),
                    allocator.create_pair(factory.create_int_term(5), factory.create_int_term(6)),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3 + 4 + 5 + 6);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermBuiltinSingleSignalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermBuiltinSingleSignalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_int_term(3),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                )),
            ),
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

struct ApplicationTermBuiltinMultipleSignalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermBuiltinMultipleSignalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                )),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_signal_term(allocator.create_signal_list([
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
        ]));
        let dependencies = vec![
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
        ];
        (result, dependencies)
    }
}

struct ApplicationTermBuiltinNestedSignalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermBuiltinNestedSignalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                )),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Add),
                    allocator.create_pair(
                        factory.create_effect_term(allocator.create_signal(
                            SignalType::Custom(
                                factory.create_string_term(allocator.create_static_string("bar")),
                            ),
                            factory.create_int_term(4),
                            factory.create_nil_term(),
                        )),
                        factory.create_effect_term(allocator.create_signal(
                            SignalType::Custom(
                                factory.create_string_term(allocator.create_static_string("baz")),
                            ),
                            factory.create_int_term(5),
                            factory.create_nil_term(),
                        )),
                    ),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_signal_term(allocator.create_signal_list([
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            ),
        ]));
        let dependencies = vec![
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            ),
        ];
        (result, dependencies)
    }
}

struct ApplicationTermVariadicStaticArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermVariadicStaticArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::CollectList),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermVariadicDynamicArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermVariadicDynamicArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::CollectList),
            allocator.create_triple(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_int_term(3)),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_int_term(4)),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_int_term(5)),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermVariadicUnresolvedSignalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for ApplicationTermVariadicUnresolvedSignalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::CollectList),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("baz")),
                    ),
                    factory.create_int_term(5),
                    factory.create_nil_term(),
                )),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_signal_term(allocator.create_signal_list([
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            ),
        ]));
        let dependencies = vec![
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            ),
        ];
        (result, dependencies)
    }
}

struct ApplicationTermVariadicPartiallyResolvedSignalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for ApplicationTermVariadicPartiallyResolvedSignalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(3),
            ),
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(4),
            ),
        ]
    }
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::CollectList),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("baz")),
                    ),
                    factory.create_int_term(5),
                    factory.create_nil_term(),
                )),
            ),
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
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            )]));
        let dependencies = vec![
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            ),
        ];
        (result, dependencies)
    }
}

struct ApplicationTermVariadicFullyResolvedSignalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for ApplicationTermVariadicFullyResolvedSignalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(3),
            ),
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(4),
            ),
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("baz")),
                    ),
                    factory.create_int_term(5),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(5),
            ),
        ]
    }
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::CollectList),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                )),
                factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("baz")),
                    ),
                    factory.create_int_term(5),
                    factory.create_nil_term(),
                )),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = vec![
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
                factory.create_nil_term(),
            ),
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
                factory.create_int_term(5),
                factory.create_nil_term(),
            ),
        ];
        (result, dependencies)
    }
}

struct ApplicationTermNestedVariadicBuiltinCallsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for ApplicationTermNestedVariadicBuiltinCallsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_string_term(allocator.create_static_string("bar")),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::CollectString),
                allocator.create_pair(
                    factory.create_string_term(allocator.create_static_string("foo")),
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::ToString),
                        allocator.create_unit_list(factory.create_variable_term(0)),
                    ),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_string_term(allocator.create_static_string("foobar"));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermNullaryLambdaScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermNullaryLambdaScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_lambda_term(0, factory.create_int_term(3)),
            allocator.create_empty_list(),
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

struct ApplicationTermUnaryLambdaScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermUnaryLambdaScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_lambda_term(1, factory.create_variable_term(0)),
            allocator.create_unit_list(factory.create_int_term(3)),
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

struct ApplicationTermPolyadicLambdaScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermPolyadicLambdaScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_lambda_term(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Multiply),
                    allocator.create_pair(
                        factory.create_variable_term(0),
                        factory.create_application_term(
                            factory.create_builtin_term(stdlib::Subtract),
                            allocator.create_pair(
                                factory.create_variable_term(2),
                                factory.create_variable_term(1),
                            ),
                        ),
                    ),
                ),
            ),
            allocator.create_triple(
                factory.create_int_term(10),
                factory.create_int_term(3),
                factory.create_int_term(2),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(2 * (10 - 3));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermConstructorScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermConstructorScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_constructor_term(allocator.create_struct_prototype(
                allocator.create_triple(
                    factory.create_string_term(allocator.create_static_string("foo")),
                    factory.create_string_term(allocator.create_static_string("bar")),
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermDynamicScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermDynamicScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Identity),
                allocator.create_unit_list(factory.create_lambda_term(
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Subtract),
                        allocator.create_pair(
                            factory.create_variable_term(0),
                            factory.create_variable_term(1),
                        ),
                    ),
                )),
            ),
            allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4 - 3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermPartialBuiltinScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermPartialBuiltinScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_builtin_term(stdlib::Subtract),
                allocator.create_unit_list(factory.create_int_term(3)),
            ),
            allocator.create_unit_list(factory.create_int_term(4)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3 - 4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermPartialVariadicScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermPartialVariadicScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_builtin_term(stdlib::CollectList),
                allocator.create_unit_list(factory.create_int_term(3)),
            ),
            allocator.create_pair(factory.create_int_term(4), factory.create_int_term(5)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermPartialLambdaScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermPartialLambdaScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_lambda_term(
                    3,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Multiply),
                        allocator.create_pair(
                            factory.create_variable_term(0),
                            factory.create_application_term(
                                factory.create_builtin_term(stdlib::Subtract),
                                allocator.create_pair(
                                    factory.create_variable_term(2),
                                    factory.create_variable_term(1),
                                ),
                            ),
                        ),
                    ),
                ),
                allocator.create_pair(factory.create_int_term(10), factory.create_int_term(3)),
            ),
            allocator.create_unit_list(factory.create_int_term(2)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(2 * (10 - 3));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermPartialConstructorScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermPartialConstructorScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_constructor_term(allocator.create_struct_prototype(
                    allocator.create_list([
                        factory.create_string_term(allocator.create_static_string("foo")),
                        factory.create_string_term(allocator.create_static_string("bar")),
                        factory.create_string_term(allocator.create_static_string("baz")),
                        factory.create_string_term(allocator.create_static_string("qux")),
                    ]),
                )),
                allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
            ),
            allocator.create_pair(factory.create_int_term(5), factory.create_int_term(6)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_list([
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
                factory.create_string_term(allocator.create_static_string("qux")),
            ])),
            allocator.create_list([
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
                factory.create_int_term(6),
            ]),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermPartialDynamicScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermPartialDynamicScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_lambda_term(
                        2,
                        factory.create_application_term(
                            factory.create_builtin_term(stdlib::Subtract),
                            allocator.create_pair(
                                factory.create_variable_term(0),
                                factory.create_variable_term(1),
                            ),
                        ),
                    )),
                ),
                allocator.create_unit_list(factory.create_int_term(3)),
            ),
            allocator.create_unit_list(factory.create_int_term(4)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4 - 3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ApplicationTermNestedPartialsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ApplicationTermNestedPartialsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_partial_application_term(
                    factory.create_partial_application_term(
                        factory.create_constructor_term(allocator.create_struct_prototype(
                            allocator.create_list([
                                factory.create_string_term(allocator.create_static_string("foo")),
                                factory.create_string_term(allocator.create_static_string("bar")),
                                factory.create_string_term(allocator.create_static_string("baz")),
                                factory.create_string_term(allocator.create_static_string("qux")),
                            ]),
                        )),
                        allocator.create_unit_list(factory.create_int_term(3)),
                    ),
                    allocator.create_unit_list(factory.create_int_term(4)),
                ),
                allocator.create_unit_list(factory.create_int_term(5)),
            ),
            allocator.create_unit_list(factory.create_int_term(6)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_list([
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
                factory.create_string_term(allocator.create_static_string("qux")),
            ])),
            allocator.create_list([
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
                factory.create_int_term(6),
            ]),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}
