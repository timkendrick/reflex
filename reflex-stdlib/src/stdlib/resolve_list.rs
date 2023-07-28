// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    ExpressionListType, FunctionArity, HeapAllocator, ListTermType, RefType, Uid, Uuid,
};

use crate::stdlib::CollectList;

pub struct ResolveList;
impl ResolveList {
    pub const UUID: Uuid = uuid!("6d324a63-2138-41ad-8775-ad4931043700");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ResolveList {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for ResolveList
where
    T::Builtin: From<CollectList>,
{
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
        let target = args.next().unwrap();
        if let Some(value) = factory.match_list_term(&target) {
            let has_dynamic_values = value
                .items()
                .as_deref()
                .iter()
                .any(|item| !item.as_deref().is_static());
            if !has_dynamic_values {
                Ok(target)
            } else {
                Ok(factory.create_application_term(
                    factory.create_builtin_term(CollectList),
                    allocator.create_list(
                        value
                            .items()
                            .as_deref()
                            .iter()
                            .map(|item| item.as_deref().clone()),
                    ),
                ))
            }
        } else {
            Err(format!("Expected List, received {}", target))
        }
    }
}
