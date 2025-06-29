// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::ops::Deref;

use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    FunctionArity, HeapAllocator, RefType, SignalType, StringTermType, StringValue, Uid, Uuid,
};
use reflex_stdlib::Stdlib;

pub const EFFECT_TYPE_VARIABLE_GET: &'static str = "reflex::variable::get";

pub fn is_variable_get_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| {
            effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_VARIABLE_GET
        })
        .unwrap_or(false)
}

pub fn create_variable_get_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_VARIABLE_GET))
}

pub const EFFECT_TYPE_VARIABLE_SET: &'static str = "reflex::variable::set";

pub fn is_variable_set_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| {
            effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_VARIABLE_SET
        })
        .unwrap_or(false)
}

pub fn create_variable_set_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_VARIABLE_SET))
}

pub const EFFECT_TYPE_VARIABLE_INCREMENT: &'static str = "reflex::variable::increment";

pub fn is_variable_increment_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| {
            effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_VARIABLE_INCREMENT
        })
        .unwrap_or(false)
}

pub fn create_variable_increment_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_VARIABLE_INCREMENT))
}

pub const EFFECT_TYPE_VARIABLE_DECREMENT: &'static str = "reflex::variable::decrement";

pub fn is_variable_decrement_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| {
            effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_VARIABLE_DECREMENT
        })
        .unwrap_or(false)
}

pub fn create_variable_decrement_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_VARIABLE_DECREMENT))
}

pub struct GetVariable;
impl GetVariable {
    pub const UUID: Uuid = uuid!("fb7bbe51-fa38-4c79-a361-c90607db2736");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for GetVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for GetVariable
where
    T::Builtin: From<Stdlib>,
{
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
        let state_token = args.next().unwrap();
        let initial_value = args.next().unwrap();
        if let Some(_) = factory.match_symbol_term(&state_token) {
            Ok(
                factory.create_effect_term(allocator.create_signal(SignalType::Custom {
                    effect_type: factory.create_string_term(
                        allocator.create_static_string(EFFECT_TYPE_VARIABLE_GET),
                    ),
                    payload:
                        factory.create_list_term(allocator.create_pair(state_token, initial_value)),
                    token: factory.create_nil_term(),
                })),
            )
        } else {
            Err(format!(
                "Expected (Symbol, <any>), received ({}, {})",
                state_token, initial_value
            ))
        }
    }
}

pub struct SetVariable;
impl SetVariable {
    pub const UUID: Uuid = uuid!("c7a1b8e5-1045-4ce7-bd1a-4125bdb8d647");
    const ARITY: FunctionArity<3, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for SetVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for SetVariable
where
    T::Builtin: From<Stdlib>,
{
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
        let state_token = args.next().unwrap();
        let value = args.next().unwrap();
        let update_token = args.next().unwrap();
        match (
            factory.match_symbol_term(&state_token),
            factory.match_symbol_term(&update_token),
        ) {
            (Some(_), Some(_)) => Ok(factory.create_effect_term(allocator.create_signal(
                SignalType::Custom {
                    effect_type: factory.create_string_term(
                        allocator.create_static_string(EFFECT_TYPE_VARIABLE_SET),
                    ),
                    payload: factory.create_list_term(allocator.create_pair(state_token, value)),
                    token: update_token,
                },
            ))),
            _ => Err(format!(
                "Expected (Symbol, <any>, Symbol), received ({}, {}, {})",
                state_token, value, update_token
            )),
        }
    }
}

pub struct IncrementVariable;
impl IncrementVariable {
    pub const UUID: Uuid = uuid!("c73a5dfa-61fb-4a19-956e-752f34526718");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for IncrementVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for IncrementVariable
where
    T::Builtin: From<Stdlib>,
{
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
        let state_token = args.next().unwrap();
        let update_token = args.next().unwrap();
        match (
            factory.match_symbol_term(&state_token),
            factory.match_symbol_term(&update_token),
        ) {
            (Some(_), Some(_)) => Ok(factory.create_effect_term(allocator.create_signal(
                SignalType::Custom {
                    effect_type: factory.create_string_term(
                        allocator.create_static_string(EFFECT_TYPE_VARIABLE_INCREMENT),
                    ),
                    payload: factory.create_list_term(allocator.create_unit_list(state_token)),
                    token: update_token,
                },
            ))),
            _ => Err(format!(
                "Expected (Symbol, Symbol), received ({}, {})",
                state_token, update_token
            )),
        }
    }
}

pub struct DecrementVariable;
impl DecrementVariable {
    pub const UUID: Uuid = uuid!("75b1d997-91ff-43e6-bbb6-b3f9d6c47a34");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for DecrementVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for DecrementVariable
where
    T::Builtin: From<Stdlib>,
{
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
        let state_token = args.next().unwrap();
        let update_token = args.next().unwrap();
        match (
            factory.match_symbol_term(&state_token),
            factory.match_symbol_term(&update_token),
        ) {
            (Some(_), Some(_)) => Ok(factory.create_effect_term(allocator.create_signal(
                SignalType::Custom {
                    effect_type: factory.create_string_term(
                        allocator.create_static_string(EFFECT_TYPE_VARIABLE_DECREMENT),
                    ),
                    payload: factory.create_list_term(allocator.create_unit_list(state_token)),
                    token: update_token,
                },
            ))),
            _ => Err(format!(
                "Expected (Symbol, Symbol), received ({}, {})",
                state_token, update_token
            )),
        }
    }
}
