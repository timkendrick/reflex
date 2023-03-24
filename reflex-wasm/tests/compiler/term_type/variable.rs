// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn variable_term() {
    let scenario = VariableTermLocalVariableScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = VariableTermAliasedVariableScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = VariableTermUnaryLambdaArgScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = VariableTermPolyadicLambdaArgScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct VariableTermLocalVariableScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for VariableTermLocalVariableScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
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

struct VariableTermAliasedVariableScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for VariableTermAliasedVariableScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
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

struct VariableTermUnaryLambdaArgScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for VariableTermUnaryLambdaArgScenario
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

struct VariableTermPolyadicLambdaArgScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for VariableTermPolyadicLambdaArgScenario
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
