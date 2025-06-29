// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::ops::Deref;

use reflex::core::{
    create_error_expression, uuid, Applicable, ArgType, Arity, EvaluationCache, Expression,
    ExpressionFactory, ExpressionListType, FunctionArity, HeapAllocator, ListTermType,
    RecordTermType, RefType, SignalType, StringTermType, StringValue, Uid, Uuid,
};

pub struct Throw;
impl Throw {
    pub const UUID: Uuid = uuid!("fb9bef4b-da7a-46ef-af03-50ed2984274c");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Throw {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for Throw {
    fn arity(&self) -> Option<Arity> {
        Some(Self::arity())
    }
    fn should_parallelize(&self, _args: &[T]) -> bool {
        false
    }
    fn apply(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        let mut args = args.into_iter();
        let error = args.next().unwrap();
        if !error.is_atomic() {
            return Err(String::from(
                "Thrown exceptions cannot contain dynamic values",
            ));
        }
        let exception = if let Some(errors) = parse_aggregate_error(&error, factory, allocator) {
            factory.create_signal_term(
                allocator.create_signal_list(
                    errors
                        .iter()
                        .map(|item| item.as_deref().clone())
                        .map(|payload| allocator.create_signal(SignalType::Error { payload })),
                ),
            )
        } else {
            create_error_expression(error, factory, allocator)
        };
        Ok(exception)
    }
}

fn parse_aggregate_error<T: Expression>(
    target: &T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> Option<T::ExpressionList> {
    factory.match_record_term(target).and_then(|target| {
        target
            .as_deref()
            .get(&factory.create_string_term(allocator.create_static_string("name")))
            .and_then(|term| {
                let error_type = term.as_deref();
                factory.match_string_term(error_type).and_then(|name| {
                    if name.value().as_deref().as_str().deref() == "AggregateError" {
                        target
                            .get(
                                &factory
                                    .create_string_term(allocator.create_static_string("errors")),
                            )
                            .and_then(|term| {
                                let errors = term.as_deref();
                                factory
                                    .match_list_term(errors)
                                    .map(|errors| errors.items().as_deref().clone())
                            })
                    } else {
                        None
                    }
                })
            })
    })
}
