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
            SignalType::Custom(factory.create_string_term(allocator.create_static_string("foo"))),
            factory.create_int_term(3),
            factory.create_nil_term(),
        )]))
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
        factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
            SignalType::Pending,
            factory.create_nil_term(),
            factory.create_nil_term(),
        )]))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result =
            factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
                SignalType::Pending,
                factory.create_nil_term(),
                factory.create_nil_term(),
            )]));
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
            SignalType::Error,
            factory.create_string_term(allocator.create_static_string("foo")),
            factory.create_nil_term(),
        )]))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result =
            factory.create_signal_term(allocator.create_signal_list([allocator.create_signal(
                SignalType::Error,
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_nil_term(),
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
        ]))
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
        let dependencies = Default::default();
        (result, dependencies)
    }
}
