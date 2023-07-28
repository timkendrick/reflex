// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    FunctionArity, HashsetTermType, HeapAllocator, RefType, Uid, Uuid,
};

use crate::stdlib::CollectHashSet;

pub struct ResolveHashSet;
impl ResolveHashSet {
    pub const UUID: Uuid = uuid!("8c3c802c-8092-4dbf-9d5f-393b8144d39e");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ResolveHashSet {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for ResolveHashSet
where
    T::Builtin: From<CollectHashSet>,
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
        if let Some(value) = factory.match_hashset_term(&target) {
            let has_dynamic_values = value.values().any(|item| !item.as_deref().is_static());
            if !has_dynamic_values {
                Ok(target)
            } else {
                Ok(factory.create_application_term(
                    factory.create_builtin_term(CollectHashSet),
                    allocator.create_list(value.values().map(|item| item.as_deref().clone())),
                ))
            }
        } else {
            Err(format!("Expected HashSet, received {}", target))
        }
    }
}
