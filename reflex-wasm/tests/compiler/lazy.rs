// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, SignalType};
use reflex_wasm::{compiler::CompilerOptions, stdlib};

use crate::{compiler::runner::run_scenario, WasmTestScenario};

#[test]
fn lazy_expressions() {
    let scenario = LazyRecordFieldStaticValue;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LazyRecordFieldDynamicValue;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LazyRecordFieldUnresolvedSignalValue;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LazyRecordFieldUnusedSignalValue;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LazyRecordFieldCapturedLocals;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);

    let scenario = LazyRecordFieldNestedCapturedLocals;
    let (actual, expected) = run_scenario(&scenario).unwrap();
    assert_eq!(actual, expected);
}

struct LazyRecordFieldStaticValue;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldStaticValue
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_record_term(
                allocator.create_struct_prototype(allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("foo")),
                )),
                allocator.create_unit_list(factory.create_int_term(3)),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Get),
                allocator.create_pair(
                    factory.create_variable_term(0),
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
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

struct LazyRecordFieldDynamicValue;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldDynamicValue
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_record_term(
                allocator.create_struct_prototype(allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("foo")),
                )),
                allocator.create_unit_list(factory.create_application_term(
                    factory.create_builtin_term(stdlib::Abs),
                    allocator.create_unit_list(factory.create_int_term(-3)),
                )),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Get),
                allocator.create_pair(
                    factory.create_variable_term(0),
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
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

struct LazyRecordFieldCapturedLocals;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldCapturedLocals
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-3)),
            ),
            factory.create_let_term(
                factory.create_nil_term(),
                factory.create_let_term(
                    factory.create_nil_term(),
                    factory.create_let_term(
                        factory.create_application_term(
                            factory.create_builtin_term(stdlib::Abs),
                            allocator.create_unit_list(factory.create_int_term(-4)),
                        ),
                        factory.create_let_term(
                            factory.create_nil_term(),
                            factory.create_let_term(
                                factory.create_nil_term(),
                                factory.create_let_term(
                                    factory.create_record_term(
                                        allocator.create_struct_prototype(
                                            allocator.create_unit_list(factory.create_string_term(
                                                allocator.create_static_string("foo"),
                                            )),
                                        ),
                                        allocator.create_unit_list(
                                            factory.create_application_term(
                                                factory.create_builtin_term(stdlib::Subtract),
                                                allocator.create_pair(
                                                    factory.create_variable_term(2),
                                                    factory.create_variable_term(5),
                                                ),
                                            ),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(stdlib::Get),
                                        allocator.create_pair(
                                            factory.create_variable_term(0),
                                            factory.create_string_term(
                                                allocator.create_static_string("foo"),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4 - 3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LazyRecordFieldNestedCapturedLocals;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldNestedCapturedLocals
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Abs),
                allocator.create_unit_list(factory.create_int_term(-3)),
            ),
            factory.create_let_term(
                factory.create_nil_term(),
                factory.create_let_term(
                    factory.create_nil_term(),
                    factory.create_let_term(
                        factory.create_application_term(
                            factory.create_builtin_term(stdlib::Abs),
                            allocator.create_unit_list(factory.create_int_term(-4)),
                        ),
                        factory.create_let_term(
                            factory.create_nil_term(),
                            factory.create_let_term(
                                factory.create_nil_term(),
                                factory.create_let_term(
                                    factory.create_record_term(
                                        allocator.create_struct_prototype(
                                            allocator.create_unit_list(factory.create_string_term(
                                                allocator.create_static_string("foo"),
                                            )),
                                        ),
                                        allocator.create_unit_list(
                                            factory.create_let_term(
                                                factory.create_record_term(
                                                    allocator.create_struct_prototype(
                                                        allocator.create_unit_list(
                                                            factory.create_string_term(
                                                                allocator
                                                                    .create_static_string("bar"),
                                                            ),
                                                        ),
                                                    ),
                                                    allocator.create_unit_list(
                                                        factory.create_application_term(
                                                            factory.create_builtin_term(
                                                                stdlib::Subtract,
                                                            ),
                                                            allocator.create_pair(
                                                                factory.create_variable_term(2),
                                                                factory.create_variable_term(5),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                                factory.create_application_term(
                                                    factory.create_builtin_term(stdlib::Get),
                                                    allocator.create_pair(
                                                        factory.create_variable_term(0),
                                                        factory.create_string_term(
                                                            allocator.create_static_string("bar"),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(stdlib::Get),
                                        allocator.create_pair(
                                            factory.create_variable_term(0),
                                            factory.create_string_term(
                                                allocator.create_static_string("foo"),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        _allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(4 - 3);
        let dependencies = Default::default();
        (result, dependencies)
    }
}

struct LazyRecordFieldUnresolvedSignalValue;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldUnresolvedSignalValue
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_record_term(
                allocator.create_struct_prototype(allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("foo")),
                )),
                allocator.create_unit_list(factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                ))),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Get),
                allocator.create_pair(
                    factory.create_variable_term(0),
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
            ),
        )
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
        let dependencies = vec![allocator.create_signal(
            SignalType::Custom(factory.create_string_term(allocator.create_static_string("foo"))),
            factory.create_int_term(3),
            factory.create_nil_term(),
        )];
        (result, dependencies)
    }
}

struct LazyRecordFieldResolvedSignalValue;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldResolvedSignalValue
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        vec![(
            allocator.create_signal(
                SignalType::Custom(
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
                factory.create_int_term(3),
                factory.create_nil_term(),
            ),
            factory.create_int_term(3),
        )]
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_record_term(
                allocator.create_struct_prototype(allocator.create_unit_list(
                    factory.create_string_term(allocator.create_static_string("foo")),
                )),
                allocator.create_unit_list(factory.create_effect_term(allocator.create_signal(
                    SignalType::Custom(
                        factory.create_string_term(allocator.create_static_string("foo")),
                    ),
                    factory.create_int_term(3),
                    factory.create_nil_term(),
                ))),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Get),
                allocator.create_pair(
                    factory.create_variable_term(0),
                    factory.create_string_term(allocator.create_static_string("foo")),
                ),
            ),
        )
    }

    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>) {
        let result = factory.create_int_term(3);
        let dependencies = vec![allocator.create_signal(
            SignalType::Custom(factory.create_string_term(allocator.create_static_string("foo"))),
            factory.create_int_term(3),
            factory.create_nil_term(),
        )];
        (result, dependencies)
    }
}

struct LazyRecordFieldUnusedSignalValue;

impl<T, TFactory> WasmTestScenario<T, TFactory> for LazyRecordFieldUnusedSignalValue
where
    T: Expression<Builtin = stdlib::Stdlib>,
    TFactory: ExpressionFactory<T>,
{
    fn options(&self) -> CompilerOptions {
        CompilerOptions {
            lazy_record_values: true,
            lazy_variable_initializers: false,
            ..Default::default()
        }
    }

    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T {
        factory.create_let_term(
            factory.create_record_term(
                allocator.create_struct_prototype(allocator.create_pair(
                    factory.create_string_term(allocator.create_static_string("foo")),
                    factory.create_string_term(allocator.create_static_string("bar")),
                )),
                allocator.create_pair(
                    factory.create_effect_term(allocator.create_signal(
                        SignalType::Custom(
                            factory.create_string_term(allocator.create_static_string("foo")),
                        ),
                        factory.create_int_term(3),
                        factory.create_nil_term(),
                    )),
                    factory.create_int_term(3),
                ),
            ),
            factory.create_application_term(
                factory.create_builtin_term(stdlib::Get),
                allocator.create_pair(
                    factory.create_variable_term(0),
                    factory.create_string_term(allocator.create_static_string("bar")),
                ),
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
