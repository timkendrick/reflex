// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn stdlib_and() {
    let scenario = StdlibAndStaticTruthyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndStaticFalsyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndStaticTruthyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndStaticFalsyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndDynamicTruthyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndDynamicFalsyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndDynamicTruthyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndDynamicFalsyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndUnresolvedStatefulConditionUnresolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndResolvedTruthyStatefulConditionUnresolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndResolvedTruthyStatefulConditionResolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndResolvedFalsyStatefulConditionUnresolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibAndResolvedFalsyStatefulConditionResolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct StdlibAndStaticTruthyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndStaticTruthyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_boolean_term(true),
                factory.create_int_term(3),
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

struct StdlibAndStaticFalsyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndStaticFalsyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_boolean_term(false),
                factory.create_int_term(3),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_boolean_term(false);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibAndStaticTruthyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndStaticTruthyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_boolean_term(true),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
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

struct StdlibAndStaticFalsyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndStaticFalsyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_boolean_term(false),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_boolean_term(false);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibAndDynamicTruthyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndDynamicTruthyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(true)),
                ),
                factory.create_int_term(3),
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

struct StdlibAndDynamicFalsyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndDynamicFalsyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(false)),
                ),
                factory.create_int_term(3),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_boolean_term(false);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibAndDynamicTruthyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndDynamicTruthyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(true)),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
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

struct StdlibAndDynamicFalsyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndDynamicFalsyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(factory.create_boolean_term(false)),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_boolean_term(false);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibAndUnresolvedStatefulConditionUnresolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndUnresolvedStatefulConditionUnresolvedStatefulConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                })),
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

struct StdlibAndResolvedTruthyStatefulConditionUnresolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndResolvedTruthyStatefulConditionUnresolvedStatefulConsequentScenario
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
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                })),
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
                    payload: factory.create_boolean_term(false),
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
                payload: factory.create_boolean_term(false),
                token: factory.create_nil_term(),
            }),
        ];
        (result, dependencies)
    }
}

struct StdlibAndResolvedTruthyStatefulConditionResolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndResolvedTruthyStatefulConditionResolvedStatefulConsequentScenario
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
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                }),
                factory.create_int_term(3),
            ),
        ]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                })),
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
                payload: factory.create_boolean_term(false),
                token: factory.create_nil_term(),
            }),
        ];
        (result, dependencies)
    }
}

struct StdlibAndResolvedFalsyStatefulConditionUnresolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndResolvedFalsyStatefulConditionUnresolvedStatefulConsequentScenario
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
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                })),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_boolean_term(false);
        let dependencies = vec![allocator.create_signal(SignalType::Custom {
            effect_type: factory.create_string_term(allocator.create_static_string("foo")),
            payload: factory.create_int_term(3),
            token: factory.create_nil_term(),
        })];
        (result, dependencies)
    }
}

struct StdlibAndResolvedFalsyStatefulConditionResolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibAndResolvedFalsyStatefulConditionResolvedStatefulConsequentScenario
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
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                }),
                factory.create_int_term(3),
            ),
        ]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::And),
            allocator.create_pair(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                    payload: factory.create_int_term(3),
                    token: factory.create_nil_term(),
                })),
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                    payload: factory.create_boolean_term(false),
                    token: factory.create_nil_term(),
                })),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_boolean_term(false);
        let dependencies = vec![allocator.create_signal(SignalType::Custom {
            effect_type: factory.create_string_term(allocator.create_static_string("foo")),
            payload: factory.create_int_term(3),
            token: factory.create_nil_term(),
        })];
        (result, dependencies)
    }
}
