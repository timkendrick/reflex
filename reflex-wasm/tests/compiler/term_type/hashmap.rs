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
fn hashmap_term() {
    let scenario = HashmapTermStaticEntriesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_hashmap_result(actual, expected);

    let scenario = HashmapTermDynamicEntriesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_hashmap_result(actual, expected);
}

fn assert_hashmap_result<A: Arena + Clone>(
    actual: WasmTestScenarioResult<A>,
    expected: WasmTestScenarioResult<A>,
) {
    let hashmap_result = actual.result.as_hashmap_term().unwrap().as_inner();
    let expected_result = expected.result.as_hashmap_term().unwrap().as_inner();
    assert_eq!(hashmap_result.num_entries(), expected_result.num_entries());
    let hashmap_entries = hashmap_result.keys().zip(hashmap_result.values());
    let expected_entries = expected_result.keys().zip(expected_result.values());
    for (key, value) in hashmap_entries {
        let expected_value = expected_entries
            .clone()
            .find(|(expected_key, _)| expected_key.id() == key.id())
            .map(|(_, value)| value)
            .unwrap();
        assert_eq!(value, expected_value);
    }
}

struct HashmapTermStaticEntriesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for HashmapTermStaticEntriesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_hashmap_term([
            (
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_int_term(3),
            ),
            (
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_int_term(4),
            ),
            (
                factory.create_string_term(allocator.create_static_string("baz")),
                factory.create_int_term(5),
            ),
        ])
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_hashmap_term([
            (
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_int_term(3),
            ),
            (
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_int_term(4),
            ),
            (
                factory.create_string_term(allocator.create_static_string("baz")),
                factory.create_int_term(5),
            ),
        ]);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct HashmapTermDynamicEntriesScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for HashmapTermDynamicEntriesScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_hashmap_term([
            (
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
                ),
            ),
            (
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(
                        factory.create_string_term(allocator.create_static_string("bar")),
                    ),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-4)),
                ),
            ),
            (
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Identity),
                    allocator.create_unit_list(
                        factory.create_string_term(allocator.create_static_string("baz")),
                    ),
                ),
                factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-5)),
                ),
            ),
        ])
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_hashmap_term([
            (
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_int_term(3),
            ),
            (
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_int_term(4),
            ),
            (
                factory.create_string_term(allocator.create_static_string("baz")),
                factory.create_int_term(5),
            ),
        ]);
        let dependencies = Default::default();
        (result, dependencies)
    }
}
