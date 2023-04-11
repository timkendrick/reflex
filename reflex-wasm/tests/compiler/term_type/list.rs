// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::{compiler::CompilerOptions, stdlib};

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn list_term() {
    let scenario = ListTermStaticItemsLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ListTermStaticItemsEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ListTermDynamicItemsLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ListTermDynamicItemsEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ListTermSignalItemsLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = ListTermSignalItemsEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct ListTermStaticItemsLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ListTermStaticItemsLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_list_items: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ListTermStaticItemsEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ListTermStaticItemsEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_list_items: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ListTermDynamicItemsLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ListTermDynamicItemsLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_list_items: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_list_term(allocator.create_triple(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-3)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-4)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-5)),
            ),
        ))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-3)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-4)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-5)),
            ),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ListTermDynamicItemsEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ListTermDynamicItemsEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_list_items: false,
            ..Default::default()
        }
    }
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_list_term(allocator.create_triple(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-3)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-4)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-5)),
            ),
        ))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_int_term(3),
            factory.create_int_term(4),
            factory.create_int_term(5),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ListTermSignalItemsLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ListTermSignalItemsLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_list_items: true,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_list_term(allocator.create_triple(
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            })),
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                payload: factory.create_int_term(4),
                token: factory.create_nil_term(),
            })),
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            })),
        ))
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_list_term(allocator.create_triple(
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            })),
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                payload: factory.create_int_term(4),
                token: factory.create_nil_term(),
            })),
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            })),
        ));
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct ListTermSignalItemsEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for ListTermSignalItemsEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_list_items: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_list_term(allocator.create_triple(
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("foo")),
                payload: factory.create_int_term(3),
                token: factory.create_nil_term(),
            })),
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("bar")),
                payload: factory.create_int_term(4),
                token: factory.create_nil_term(),
            })),
            factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                effect_type: factory.create_string_term(allocator.create_static_string("baz")),
                payload: factory.create_int_term(5),
                token: factory.create_nil_term(),
            })),
        ))
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
        let dependencies = vec![
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
        ];
        (result, dependencies)
    }
}
