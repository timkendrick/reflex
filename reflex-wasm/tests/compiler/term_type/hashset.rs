// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, NodeId};
use reflex_wasm::{allocator::Arena, stdlib};

use crate::{
    compiler::runner::{run_scenario, WasmTestScenarioResult},
    WasmTestScenario,
};

#[test]
fn hashset_term() {
    let scenario = HashsetTermStaticValuesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_hashset_result(actual, expected);

    let scenario = HashsetTermDynamicValuesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_hashset_result(actual, expected);
}

fn assert_hashset_result<A: Arena + Clone>(
    actual: WasmTestScenarioResult<A>,
    expected: WasmTestScenarioResult<A>,
) {
    let hashset_result = actual.result.as_hashset_term().unwrap().as_inner();
    let expected_result = expected.result.as_hashset_term().unwrap().as_inner();
    assert_eq!(hashset_result.num_values(), expected_result.num_values());
    let hashset_values = hashset_result.values();
    let expected_values = expected_result.values();
    for value in hashset_values {
        let expected_value = expected_values
            .clone()
            .find(|expected_value| expected_value.id() == value.id())
            .unwrap();
        assert_eq!(value, expected_value);
    }
}

struct HashsetTermStaticValuesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for HashsetTermStaticValuesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_hashset_term([
            factory.create_string_term(allocator.create_static_string("foo")),
            factory.create_string_term(allocator.create_static_string("bar")),
            factory.create_string_term(allocator.create_static_string("baz")),
        ])
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_hashset_term([
            factory.create_string_term(allocator.create_static_string("foo")),
            factory.create_string_term(allocator.create_static_string("bar")),
            factory.create_string_term(allocator.create_static_string("baz")),
        ]);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct HashsetTermDynamicValuesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for HashsetTermDynamicValuesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_hashset_term([
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Identity),
                allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Identity),
                allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Identity),
                allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("baz")),
                ),
            ),
        ])
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_hashset_term([
            factory.create_string_term(allocator.create_static_string("foo")),
            factory.create_string_term(allocator.create_static_string("bar")),
            factory.create_string_term(allocator.create_static_string("baz")),
        ]);
        let dependencies = Default::default();
        (result, dependencies)
    }
}
