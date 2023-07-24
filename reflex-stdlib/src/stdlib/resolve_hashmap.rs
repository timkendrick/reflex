// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    FunctionArity, HashmapTermType, HeapAllocator, RefType, Uid, Uuid,
};

use crate::stdlib::CollectHashMap;

pub struct ResolveHashMap;
impl ResolveHashMap {
    pub const UUID: Uuid = uuid!("15ccc11f-31e9-4f8d-abb9-cf9dff0b26bd");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ResolveHashMap {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for ResolveHashMap
where
    T::Builtin: From<CollectHashMap>,
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
        if let Some(value) = factory.match_hashmap_term(&target) {
            let keys_are_atomic = value.keys().all(|key| key.as_deref().is_atomic());
            let values_are_atomic = value.values().all(|value| value.as_deref().is_atomic());
            if keys_are_atomic && values_are_atomic {
                Ok(target)
            } else {
                Ok(factory.create_application_term(
                    factory.create_builtin_term(CollectHashMap),
                    allocator.create_sized_list(
                        value.keys().len() + value.values().len(),
                        value
                            .keys()
                            .zip(value.values())
                            .flat_map(|(key, value)| [key, value])
                            .map(|item| item.as_deref().clone()),
                    ),
                ))
            }
        } else {
            Err(format!("Expected HashMap, received {}", target))
        }
    }
}
