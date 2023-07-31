// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn lambda_term() {
    let scenario = NullaryLambdaTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = UnaryLambdaTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = PolyadicLambdaScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct NullaryLambdaTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for NullaryLambdaTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_lambda_term(0, factory.create_int_term(3)),
            factory.create_application_term(
                factory.create_variable_term(0),
                allocator.create_empty_list(),
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

struct UnaryLambdaTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for UnaryLambdaTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_lambda_term(1, factory.create_variable_term(0)),
            factory.create_application_term(
                factory.create_variable_term(0),
                allocator.create_unit_list(factory.create_int_term(3)),
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

struct PolyadicLambdaScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for PolyadicLambdaScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
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
            factory.create_application_term(
                factory.create_variable_term(0),
                allocator.create_triple(
                    factory.create_int_term(10),
                    factory.create_int_term(3),
                    factory.create_int_term(2),
                ),
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
