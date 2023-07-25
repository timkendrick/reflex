// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, BooleanTermType, EvaluationCache, Expression,
    ExpressionFactory, FunctionArity, HeapAllocator, Uid, Uuid,
};

pub struct If;
impl If {
    pub const UUID: Uuid = uuid!("9c8fc3a1-2d55-420e-bf81-3098932f8cf0");
    const ARITY: FunctionArity<3, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for If {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for If {
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
        allocator: &impl HeapAllocator<T>,
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        let condition = args.next().unwrap();
        let consequent = args.next().unwrap();
        let alternate = args.next().unwrap();
        if is_truthy(&condition, factory) {
            Ok(factory.create_application_term(consequent, allocator.create_empty_list()))
        } else {
            Ok(factory.create_application_term(alternate, allocator.create_empty_list()))
        }
    }
}

pub fn is_truthy<T: Expression>(value: &T, factory: &impl ExpressionFactory<T>) -> bool {
    match factory.match_boolean_term(value) {
        Some(term) => term.value(),
        _ => match factory.match_nil_term(value) {
            Some(_) => false,
            _ => true,
        },
    }
}
