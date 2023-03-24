// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_wasm::stdlib;

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn constructor_term() {
    let scenario = ConstructorTermScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct ConstructorTermScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ConstructorTermScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_constructor_term(allocator.create_struct_prototype(allocator.create_triple(
            factory.create_string_term(allocator.create_static_string("foo")),
            factory.create_string_term(allocator.create_static_string("bar")),
            factory.create_string_term(allocator.create_static_string("baz")),
        )))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_constructor_term(allocator.create_struct_prototype(
            allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            ),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}
