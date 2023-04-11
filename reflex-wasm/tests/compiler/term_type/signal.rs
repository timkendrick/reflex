// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn signal_term() {
    let scenario = CustomSignalTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = PendingSignalTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ErrorSignalTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = CombinedSignalTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct CustomSignalTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for CustomSignalTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
            SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            },
        )]))
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
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct PendingSignalTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for PendingSignalTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_signal_term(
            allocator.create_signal_list([allocator.create_signal(SignalType::Pending)]),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_signal_term(
            allocator.create_signal_list([allocator.create_signal(SignalType::Pending)]),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ErrorSignalTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ErrorSignalTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
            SignalType::Error {
                payload: factory.create_string_term(allocator.create_static_string("foo")),
            },
        )]))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result =
            factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
                SignalType::Error {
                    payload: factory.create_string_term(allocator.create_static_string("foo")),
                },
            )]));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct CombinedSignalTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for CombinedSignalTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_signal_term(allocator.create_signal_list([
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
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            }),
        ]))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_signal_term(allocator.create_signal_list([
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
            allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            }),
        ]));
        let dependencies = Default::default();
        (result, dependencies)
    }
}
