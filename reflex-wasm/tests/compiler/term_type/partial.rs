// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn partial_application_term() {
    let scenario = PartialTermStaticArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = PartialTermDynamicArgsScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct PartialTermStaticArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for PartialTermStaticArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_partial_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_partial_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct PartialTermDynamicArgsScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for PartialTermDynamicArgsScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_partial_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-4)),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_partial_application_term(
            factory.create_builtin_term(stdlib::Add),
            allocator.create_pair(factory.create_int_term(3), factory.create_int_term(4)),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}
