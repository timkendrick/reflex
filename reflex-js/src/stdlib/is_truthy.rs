use std::ops::Deref;

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, BooleanTermType, EvaluationCache, Expression,
    ExpressionFactory, FloatTermType, FunctionArity, HeapAllocator, IntTermType, RefType,
    StringTermType, StringValue, Uid, Uuid,
};

pub struct IsTruthy;
impl IsTruthy {
    pub const UUID: Uuid = uuid!("c3304698-2617-437c-b2d5-e0990182b9df");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for IsTruthy {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for IsTruthy {
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
        _allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        let mut args = args.into_iter();
        let value = args.next().unwrap();
        if let Some(_) = factory.match_nil_term(&value) {
            Ok(factory.create_boolean_term(false))
        } else if let Some(value) = factory.match_boolean_term(&value) {
            Ok(factory.create_boolean_term(value.value()))
        } else if let Some(value) = factory.match_int_term(&value) {
            Ok(factory.create_boolean_term(value.value() != 0))
        } else if let Some(value) = factory.match_float_term(&value) {
            Ok(factory.create_boolean_term(value.value() != 0.0))
        } else if let Some(value) = factory.match_string_term(&value) {
            Ok(factory.create_boolean_term(value.value().as_deref().as_str().deref() != ""))
        } else {
            Ok(factory.create_boolean_term(true))
        }
    }
}
