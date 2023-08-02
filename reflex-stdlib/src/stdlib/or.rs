// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, BooleanTermType, EvaluationCache, Expression,
    ExpressionFactory, FunctionArity, HeapAllocator, Uid, Uuid,
};

pub struct Or;
impl Or {
    pub const UUID: Uuid = uuid!("b4798927-f64e-4835-962e-e3ff1fbe3153");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Or {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for Or {
    fn arity(&self) -> Option<Arity> {
        Some(Self::arity())
    }
    fn should_parallelize(&self, _args: &[T]) -> bool {
        false
    }
    fn apply(
        &self,
        mut args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        _allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        let left = args.next().unwrap();
        let right = args.next().unwrap();
        if let (Some(left_term), Some(right_term)) = (
            factory.match_boolean_term(&left),
            factory.match_boolean_term(&right),
        ) {
            let left_value = left_term.value();
            let right_value = right_term.value();
            Ok(factory.create_boolean_term(left_value || right_value))
        } else {
            Err(format!(
                "Expected (Boolean, Boolean), received ({}, {})",
                left, right
            ))
        }
    }
}
