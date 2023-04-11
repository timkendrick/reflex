// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    cache::SubstitutionCache,
    core::{
        create_record, ConditionType, Expression, ExpressionFactory, HeapAllocator, Reducible,
        Rewritable, SignalType, StateCache,
    },
};

const EVENT_TYPE_ENV: &'static str = "reflex::env";

pub fn create_env_args_accessor<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T::Signal {
    allocator.create_signal(SignalType::Custom {
        effect_type: factory.create_string_term(allocator.create_static_string(EVENT_TYPE_ENV)),
        payload: factory.create_list_term(allocator.create_empty_list()),
        token: factory.create_nil_term(),
    })
}

pub fn inject_env_vars<'a, T: Expression + Rewritable<T> + Reducible<T>>(
    expression: T,
    vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    let env_accessor = create_env_args_accessor(factory, allocator);
    let env_values = create_record(
        vars.into_iter().map(|(key, value)| {
            (
                factory.create_string_term(allocator.create_string(key)),
                factory.create_string_term(allocator.create_string(value)),
            )
        }),
        factory,
        allocator,
    );
    expression
        .substitute_dynamic(
            true,
            &StateCache::from_iter([(env_accessor.id(), env_values)]),
            factory,
            allocator,
            &mut SubstitutionCache::new(),
        )
        .unwrap_or(expression)
}
