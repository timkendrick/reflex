// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    ExpressionListType, FunctionArity, HeapAllocator, ListTermType, RefType, Uid, Uuid,
};
use reflex_stdlib::CollectList;

pub struct FlattenDeep;
impl FlattenDeep {
    pub const UUID: Uuid = uuid!("52299fbb-a84b-488e-81ef-98c439869f74");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for FlattenDeep {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for FlattenDeep
where
    T::Builtin: From<CollectList> + From<FlattenDeep>,
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
        let target = args.next().unwrap();
        if is_scalar_value(&target, factory) {
            Ok(target)
        } else if let Some(list) = factory.match_list_term(&target) {
            Ok(factory.create_application_term(
                factory.create_builtin_term(CollectList),
                allocator.create_list(
                    list.items()
                        .as_deref()
                        .iter()
                        .map(|item| item.as_deref().clone())
                        .map(|item| {
                            factory.create_application_term(
                                factory.create_builtin_term(FlattenDeep),
                                allocator.create_unit_list(item),
                            )
                        }),
                ),
            ))
        } else {
            // Assume the leaf definition is a lazy thunk
            let leaf = factory.create_application_term(target, allocator.create_empty_list());
            Ok(factory.create_application_term(
                factory.create_builtin_term(FlattenDeep),
                allocator.create_unit_list(leaf),
            ))
        }
    }
}

fn is_scalar_value<T: Expression>(expression: &T, factory: &impl ExpressionFactory<T>) -> bool {
    factory.match_nil_term(expression).is_some()
        || factory.match_boolean_term(expression).is_some()
        || factory.match_int_term(expression).is_some()
        || factory.match_float_term(expression).is_some()
        || factory.match_string_term(expression).is_some()
}
