// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{ArgType, Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::{compiler::CompilerOptions, stdlib};

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn record_term() {
    let scenario = RecordTermStaticValuesLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermStaticValuesEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermStaticValuesStrictScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermDynamicValuesEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermDynamicValuesStrictScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermDynamicValuesEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermSignalValuesLazyScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermSignalValuesEagerScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = RecordTermSignalValuesStrictScenario;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct RecordTermStaticValuesLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermStaticValuesLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Lazy,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermStaticValuesEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermStaticValuesEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Eager,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermStaticValuesStrictScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermStaticValuesStrictScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Strict,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermDynamicValuesLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermDynamicValuesLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Lazy,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermDynamicValuesEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermDynamicValuesEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Eager,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermDynamicValuesStrictScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermDynamicValuesStrictScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Strict,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermSignalValuesLazyScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermSignalValuesLazyScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Lazy,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermSignalValuesEagerScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermSignalValuesEagerScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Eager,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
                factory.create_lazy_result_term(
                    factory.create_signal_term(allocator.create_signal_list([
                        allocator.create_signal(SignalType::Custom {
                            effect_type:
                                factory.create_string_term(allocator.create_static_string("foo")),
                            payload: factory.create_int_term(3),
                            token: factory.create_nil_term(),
                        }),
                    ])),
                    allocator.create_signal_list([allocator.create_signal(SignalType::Custom {
                        effect_type: factory
                            .create_string_term(allocator.create_static_string("foo")),
                        payload: factory.create_int_term(3),
                        token: factory.create_nil_term(),
                    })]),
                ),
                factory.create_lazy_result_term(
                    factory.create_signal_term(allocator.create_signal_list([
                        allocator.create_signal(SignalType::Custom {
                            effect_type:
                                factory.create_string_term(allocator.create_static_string("bar")),
                            payload: factory.create_int_term(4),
                            token: factory.create_nil_term(),
                        }),
                    ])),
                    allocator.create_signal_list([allocator.create_signal(SignalType::Custom {
                        effect_type: factory
                            .create_string_term(allocator.create_static_string("bar")),
                        payload: factory.create_int_term(4),
                        token: factory.create_nil_term(),
                    })]),
                ),
                factory.create_lazy_result_term(
                    factory.create_signal_term(allocator.create_signal_list([
                        allocator.create_signal(SignalType::Custom {
                            effect_type:
                                factory.create_string_term(allocator.create_static_string("baz")),
                            payload: factory.create_int_term(5),
                            token: factory.create_nil_term(),
                        }),
                    ])),
                    allocator.create_signal_list([allocator.create_signal(SignalType::Custom {
                        effect_type: factory
                            .create_string_term(allocator.create_static_string("baz")),
                        payload: factory.create_int_term(5),
                        token: factory.create_nil_term(),
                    })]),
                ),
            ),
        );
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct RecordTermSignalValuesStrictScenario;

impl<T, TFactory> WasmTestScenario<T, TFactory> for RecordTermSignalValuesStrictScenario
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: ArgType::Strict,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_triple(
                factory.create_string_term(allocator.create_static_string("foo")),
                factory.create_string_term(allocator.create_static_string("bar")),
                factory.create_string_term(allocator.create_static_string("baz")),
            )),
            allocator.create_triple(
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
            ),
        )
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
