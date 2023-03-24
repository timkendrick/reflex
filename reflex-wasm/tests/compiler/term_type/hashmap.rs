// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn hashmap_term() {
    let scenario = HashmapTermStaticEntriesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = HashmapTermDynamicEntriesScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
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
