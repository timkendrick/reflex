// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::{
    core::{
        uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
        ExpressionListType, FunctionArity, HashmapTermType, HeapAllocator, ListTermType,
        RecordTermType, RefType, StructPrototypeType, Uid, Uuid,
    },
    hash::IntSet,
};

pub struct ResolveLoaderResults;
impl ResolveLoaderResults {
    pub const UUID: Uuid = uuid!("dd4517b0-722d-431f-89fd-612c76f7e694");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ResolveLoaderResults {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for ResolveLoaderResults {
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
        let keys = args.next().unwrap();
        let results = args.next().unwrap();
        let keys_list = factory.match_list_term(&keys);
        if let (Some(keys_list), Some(results_list)) =
            (keys_list, factory.match_list_term(&results))
        {
            let num_results = results_list.items().as_deref().len();
            let num_keys = keys_list.items().as_deref().len();
            if num_results == num_keys {
                Ok(results)
            } else {
                Err(format!(
                    "Expected {num_keys} results, received {num_results}"
                ))
            }
        } else if let (Some(keys_list), Some(results_record)) =
            (keys_list, factory.match_record_term(&results))
        {
            let expected_keys = keys_list.items();
            let num_expected_keys = expected_keys.as_deref().len();
            let result_prototype = results_record.prototype();
            let result_keys = result_prototype.as_deref().keys();
            let num_results = result_keys.as_deref().len();
            let ordered_results = expected_keys.as_deref().iter().fold(
                Ok(Vec::with_capacity(num_expected_keys)),
                |results, key| {
                    let mut results = results?;
                    match results_record.get(key.as_deref()) {
                        Some(value) => {
                            results.push(value.as_deref().clone());
                            Ok(results)
                        }
                        None => Err(format!("Missing result for key: {}", key.as_deref())),
                    }
                },
            )?;
            let first_unexpected_key = if num_results > num_expected_keys {
                let expected_keys_lookup = keys_list
                    .items()
                    .as_deref()
                    .iter()
                    .map(|item| item.as_deref().id())
                    .collect::<IntSet<_>>();
                let mut unexpected_keys = result_keys
                    .as_deref()
                    .iter()
                    .filter(|key| !expected_keys_lookup.contains(&key.as_deref().id()));
                unexpected_keys.next()
            } else {
                None
            };
            if let Some(key) = first_unexpected_key {
                Err(format!("Unexpected key: {}", key.as_deref()))
            } else {
                Ok(factory.create_list_term(allocator.create_list(ordered_results)))
            }
        } else if let (Some(keys_list), Some(results_hashmap)) =
            (keys_list, factory.match_hashmap_term(&results))
        {
            let expected_keys = keys_list.items();
            let num_expected_keys = expected_keys.as_deref().len();
            let result_keys = results_hashmap.keys();
            let num_results = result_keys.len();
            let ordered_results = expected_keys.as_deref().iter().fold(
                Ok(Vec::with_capacity(num_expected_keys)),
                |results, key| {
                    let mut results = results?;
                    match results_hashmap.get(key.as_deref()) {
                        Some(value) => {
                            results.push(value.as_deref().clone());
                            Ok(results)
                        }
                        None => Err(format!("Missing result for key: {}", key.as_deref())),
                    }
                },
            )?;
            let first_unexpected_key = if num_results > num_expected_keys {
                let expected_keys_lookup = expected_keys
                    .as_deref()
                    .iter()
                    .map(|item| item.as_deref().id())
                    .collect::<IntSet<_>>();
                let mut unexpected_keys =
                    result_keys.filter(|key| !expected_keys_lookup.contains(&key.as_deref().id()));
                unexpected_keys.next()
            } else {
                None
            };
            if let Some(key) = first_unexpected_key {
                Err(format!("Unexpected key: {}", key.as_deref()))
            } else {
                Ok(factory.create_list_term(allocator.create_list(ordered_results)))
            }
        } else {
            Err(format!(
                "Expected (List, List) or (Record, List) or (HashMap, List), received ({}, {})",
                results, keys
            ))
        }
    }
}
