// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn stdlib_if() {
    let scenario = StdlibIfStaticTruthyConditionStaticBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfStaticFalsyConditionStaticBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfStaticTruthyConditionDynamicBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfStaticFalsyConditionDynamicBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfDynamicTruthyConditionStaticBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfDynamicFalsyConditionStaticBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfDynamicTruthyConditionDynamicBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfDynamicFalsyConditionDynamicBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfUnresolvedStatefulConditionUnresolvedStatefulBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfResolvedTruthyStatefulConditionUnresolvedStatefulBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfResolvedTruthyStatefulConditionResolvedStatefulBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfResolvedFalsyStatefulConditionUnresolvedStatefulBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfResolvedFalsyStatefulConditionResolvedStatefulBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfMultipleConditionalArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfCapturingTruthyBranchScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfCapturingFalsyBranchScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfCapturingBothBranchesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibIfDynamicBranchFactoriesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct StdlibIfStaticTruthyConditionStaticBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfStaticTruthyConditionStaticBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_boolean_term(true),
                factory.create_lambda_term(0, factory.create_int_term(3)),
                factory.create_lambda_term(0, factory.create_int_term(4)),
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

struct StdlibIfStaticFalsyConditionStaticBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfStaticFalsyConditionStaticBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_boolean_term(false),
                factory.create_lambda_term(0, factory.create_int_term(3)),
                factory.create_lambda_term(0, factory.create_int_term(4)),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfStaticTruthyConditionDynamicBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfStaticTruthyConditionDynamicBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_boolean_term(true),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-3)),
                    ),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-4)),
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
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfStaticFalsyConditionDynamicBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfStaticFalsyConditionDynamicBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_boolean_term(false),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-3)),
                    ),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-4)),
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
        let result = factory.create_int_term(4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfDynamicTruthyConditionStaticBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfDynamicTruthyConditionStaticBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(true)),
                ),
                factory.create_lambda_term(0, factory.create_int_term(3)),
                factory.create_lambda_term(0, factory.create_int_term(4)),
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

struct StdlibIfDynamicFalsyConditionStaticBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfDynamicFalsyConditionStaticBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(false)),
                ),
                factory.create_lambda_term(0, factory.create_int_term(3)),
                factory.create_lambda_term(0, factory.create_int_term(4)),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfDynamicTruthyConditionDynamicBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfDynamicTruthyConditionDynamicBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(true)),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-3)),
                    ),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-4)),
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
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfDynamicFalsyConditionDynamicBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfDynamicFalsyConditionDynamicBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(false)),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-3)),
                    ),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(stdlib::Abs),
                        allocator.create_unit_list(factory.create_int_term(-4)),
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
        let result = factory.create_int_term(4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfUnresolvedStatefulConditionUnresolvedStatefulBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfUnresolvedStatefulConditionUnresolvedStatefulBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("bar")),
                        payload: factory.create_int_term(4),
                        token: factory.create_nil_term(),
                    })),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("baz")),
                        payload: factory.create_int_term(5),
                        token: factory.create_nil_term(),
                    })),
                ),
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
                SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                },
            )]));
        let dependencies = vec![allocator.create_signal(SignalType::Custom {
            effect_type: factory.create_string_term(allocator.create_static_string("foo")),
            payload: factory.create_int_term(3),
            token: factory.create_nil_term(),
        })];
        (result, dependencies)
    }
}

struct StdlibIfResolvedTruthyStatefulConditionUnresolvedStatefulBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfResolvedTruthyStatefulConditionUnresolvedStatefulBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![(
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            }),
            factory.create_boolean_term(true),
        )]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("bar")),
                        payload: factory.create_int_term(4),
                        token: factory.create_nil_term(),
                    })),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("baz")),
                        payload: factory.create_int_term(5),
                        token: factory.create_nil_term(),
                    })),
                ),
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
                SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_int_term(4),
                    token: factory.create_nil_term(),
                },
            )]));
        let dependencies = vec![
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            }),
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                payload: factory.create_int_term(4),
                token: factory.create_nil_term(),
            }),
        ];
        (result, dependencies)
    }
}

struct StdlibIfResolvedTruthyStatefulConditionResolvedStatefulBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfResolvedTruthyStatefulConditionResolvedStatefulBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![
            (
                allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                }),
                factory.create_boolean_term(true),
            ),
            (
                allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_int_term(4),
                    token: factory.create_nil_term(),
                }),
                factory.create_int_term(3),
            ),
            (
                allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                    payload: factory.create_int_term(5),
                    token: factory.create_nil_term(),
                }),
                factory.create_int_term(4),
            ),
        ]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("bar")),
                        payload: factory.create_int_term(4),
                        token: factory.create_nil_term(),
                    })),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("baz")),
                        payload: factory.create_int_term(5),
                        token: factory.create_nil_term(),
                    })),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = vec![
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            }),
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                payload: factory.create_int_term(4),
                token: factory.create_nil_term(),
            }),
        ];
        (result, dependencies)
    }
}

struct StdlibIfResolvedFalsyStatefulConditionUnresolvedStatefulBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfResolvedFalsyStatefulConditionUnresolvedStatefulBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![(
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            }),
            factory.create_boolean_term(false),
        )]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("bar")),
                        payload: factory.create_int_term(4),
                        token: factory.create_nil_term(),
                    })),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("baz")),
                        payload: factory.create_int_term(5),
                        token: factory.create_nil_term(),
                    })),
                ),
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
                SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                    payload: factory.create_int_term(5),
                    token: factory.create_nil_term(),
                },
            )]));
        let dependencies = vec![
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            }),
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            }),
        ];
        (result, dependencies)
    }
}

struct StdlibIfResolvedFalsyStatefulConditionResolvedStatefulBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibIfResolvedFalsyStatefulConditionResolvedStatefulBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![
            (
                allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                }),
                factory.create_boolean_term(false),
            ),
            (
                allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_int_term(4),
                    token: factory.create_nil_term(),
                }),
                factory.create_int_term(3),
            ),
            (
                allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                    payload: factory.create_int_term(5),
                    token: factory.create_nil_term(),
                }),
                factory.create_int_term(4),
            ),
        ]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("bar")),
                        payload: factory.create_int_term(4),
                        token: factory.create_nil_term(),
                    })),
                ),
                factory.create_lambda_term(
                    0,
                    factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                        effect_type:
                            factory.create_string_term(allocator.create_static_string("baz")),
                        payload: factory.create_int_term(5),
                        token: factory.create_nil_term(),
                    })),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4);
        let dependencies = vec![
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            }),
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            }),
        ];
        (result, dependencies)
    }
}

struct StdlibIfMultipleConditionalArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for StdlibIfMultipleConditionalArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::If),
                    allocator.create_triple(
                        factory.create_boolean_term(true),
                        factory.create_lambda_term(0, factory.create_int_term(3)),
                        factory.create_lambda_term(0, factory.create_int_term(4)),
                    ),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::If),
                    allocator.create_triple(
                        factory.create_boolean_term(false),
                        factory.create_lambda_term(0, factory.create_int_term(3)),
                        factory.create_lambda_term(0, factory.create_int_term(4)),
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
        let result = factory.create_int_term(3 + 4);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfCapturingTruthyBranchScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for StdlibIfCapturingTruthyBranchScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        (0..=9).rev().fold(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::If),
                allocator.create_triple(
                    factory.create_boolean_term(true),
                    factory.create_partial_application_term(
                        factory.create_lambda_term(
                            3,
                            factory.create_application_term(
                                factory.create_builtin_term(stdlib::Subtract),
                                allocator.create_pair(
                                    factory.create_variable_term(1),
                                    factory.create_variable_term(2),
                                ),
                            ),
                        ),
                        allocator.create_list([
                            factory.create_variable_term(5),
                            factory.create_variable_term(3),
                            factory.create_variable_term(1),
                        ]),
                    ),
                    factory.create_lambda_term(0, factory.create_int_term(3)),
                ),
            ),
            |expression, index| {
                factory.create_let_term(factory.create_int_term(fib(index)), expression)
            },
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(fib(9 - 3) - fib(9 - 5));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfCapturingFalsyBranchScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for StdlibIfCapturingFalsyBranchScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        (0..=9).rev().fold(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::If),
                allocator.create_triple(
                    factory.create_boolean_term(false),
                    factory.create_lambda_term(0, factory.create_int_term(3)),
                    factory.create_partial_application_term(
                        factory.create_lambda_term(
                            3,
                            factory.create_application_term(
                                factory.create_builtin_term(stdlib::Subtract),
                                allocator.create_pair(
                                    factory.create_variable_term(1),
                                    factory.create_variable_term(2),
                                ),
                            ),
                        ),
                        allocator.create_list([
                            factory.create_variable_term(5),
                            factory.create_variable_term(3),
                            factory.create_variable_term(1),
                        ]),
                    ),
                ),
            ),
            |expression, index| {
                factory.create_let_term(factory.create_int_term(fib(index)), expression)
            },
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(fib(9 - 3) - fib(9 - 5));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfCapturingBothBranchesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for StdlibIfCapturingBothBranchesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        (0..=9).rev().fold(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::If),
                allocator.create_triple(
                    factory.create_boolean_term(false),
                    factory.create_partial_application_term(
                        factory.create_lambda_term(
                            3,
                            factory.create_application_term(
                                factory.create_builtin_term(stdlib::Subtract),
                                allocator.create_pair(
                                    factory.create_variable_term(1),
                                    factory.create_variable_term(2),
                                ),
                            ),
                        ),
                        allocator.create_list([
                            factory.create_variable_term(6),
                            factory.create_variable_term(3),
                            factory.create_variable_term(1),
                        ]),
                    ),
                    factory.create_partial_application_term(
                        factory.create_lambda_term(
                            3,
                            factory.create_application_term(
                                factory.create_builtin_term(stdlib::Subtract),
                                allocator.create_pair(
                                    factory.create_variable_term(1),
                                    factory.create_variable_term(2),
                                ),
                            ),
                        ),
                        allocator.create_list([
                            factory.create_variable_term(5),
                            factory.create_variable_term(3),
                            factory.create_variable_term(1),
                        ]),
                    ),
                ),
            ),
            |expression, index| {
                factory.create_let_term(factory.create_int_term(fib(index)), expression)
            },
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(fib(9 - 3) - fib(9 - 5));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibIfDynamicBranchFactoriesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for StdlibIfDynamicBranchFactoriesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::If),
            allocator.create_triple(
                factory.create_boolean_term(true),
                factory.create_application_term(
                    factory.create_lambda_term(
                        1,
                        factory.create_partial_application_term(
                            factory.create_lambda_term(1, factory.create_variable_term(0)),
                            allocator.create_unit_list(factory.create_variable_term(0)),
                        ),
                    ),
                    allocator.create_unit_list(factory.create_int_term(3)),
                ),
                factory.create_application_term(
                    factory.create_lambda_term(
                        1,
                        factory.create_partial_application_term(
                            factory.create_lambda_term(1, factory.create_variable_term(0)),
                            allocator.create_unit_list(factory.create_variable_term(0)),
                        ),
                    ),
                    allocator.create_unit_list(factory.create_int_term(4)),
                ),
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

fn fib(n: i64) -> i64 {
    if n < 2 {
        n
    } else {
        let mut n1 = 1;
        let mut n2 = 0;
        let mut n = n - 2;
        while n > 0 {
            let value = n1 + n2;
            n2 = n1;
            n1 = value;
            n -= 1;
        }
        let value = n1 + n2;
        value
    }
}
