// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn stdlib_or() {
    let scenario = StdlibOrStaticTruthyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrStaticFalsyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrStaticTruthyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrStaticFalsyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrDynamicTruthyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrDynamicFalsyConditionStaticConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrDynamicTruthyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrDynamicFalsyConditionDynamicConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrUnresolvedStatefulConditionUnresolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrResolvedTruthyStatefulConditionUnresolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrResolvedTruthyStatefulConditionResolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrResolvedFalsyStatefulConditionUnresolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = StdlibOrResolvedFalsyStatefulConditionResolvedStatefulConsequentScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct StdlibOrStaticTruthyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrStaticTruthyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_boolean_term(true);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrStaticFalsyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrStaticFalsyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrStaticTruthyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrStaticTruthyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_boolean_term(true);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrStaticFalsyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrStaticFalsyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrDynamicTruthyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrDynamicTruthyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_boolean_term(true);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrDynamicFalsyConditionStaticConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrDynamicFalsyConditionStaticConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrDynamicTruthyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrDynamicTruthyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_boolean_term(true);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrDynamicFalsyConditionDynamicConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrDynamicFalsyConditionDynamicConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_int_term(3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct StdlibOrUnresolvedStatefulConditionUnresolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrUnresolvedStatefulConditionUnresolvedStatefulConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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

struct StdlibOrResolvedTruthyStatefulConditionUnresolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrResolvedTruthyStatefulConditionUnresolvedStatefulConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![(
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            factory.create_boolean_term(true),
        )]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_boolean_term(true);
        let dependencies = vec![allocator.create_signal(
            SignalType::Custom(factory.create_string_term(allocator.create_static_string("foo"))),
            factory.create_int_term(3),
            factory.create_nil_term(),
        )];
        (result, dependencies)
    }
}

struct StdlibOrResolvedTruthyStatefulConditionResolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrResolvedTruthyStatefulConditionResolvedStatefulConsequentScenario
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
                factory.create_boolean_term(true),
            ),
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(3),
            ),
        ]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_boolean_term(true);
        let dependencies = vec![allocator.create_signal(
            SignalType::Custom(factory.create_string_term(allocator.create_static_string("foo"))),
            factory.create_int_term(3),
            factory.create_nil_term(),
        )];
        (result, dependencies)
    }
}

struct StdlibOrResolvedFalsyStatefulConditionUnresolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrResolvedFalsyStatefulConditionUnresolvedStatefulConsequentScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![(
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            factory.create_boolean_term(false),
        )]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result =
            factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
                factory.create_int_term(4),
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
        ];
        (result, dependencies)
    }
}

struct StdlibOrResolvedFalsyStatefulConditionResolvedStatefulConsequentScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory>
    for StdlibOrResolvedFalsyStatefulConditionResolvedStatefulConsequentScenario
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
                factory.create_boolean_term(false),
            ),
            (
                allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                    factory.create_int_term(4),
                    factory.create_nil_term(),
                ),
                factory.create_int_term(3),
            ),
        ]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_application_term(
            factory.create_builtin_term(stdlib::Or),
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
        let result = factory.create_int_term(3);
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
