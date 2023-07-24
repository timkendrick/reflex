// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    deduplicate_hashmap_entries, uuid, Applicable, ArgType, Arity, EvaluationCache, Expression,
    ExpressionFactory, FunctionArity, HeapAllocator, Uid, Uuid,
};

pub struct CollectRecord;
impl CollectRecord {
    pub const UUID: Uuid = uuid!("1e56a1aa-4804-4e24-abfe-6660be015dcd");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Strict),
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for CollectRecord {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for CollectRecord {
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
        let num_args = args.len();
        let (keys, values) = args
            .fold(
                Ok((
                    Vec::with_capacity(num_args / 2),
                    Vec::with_capacity(num_args / 2),
                )),
                |state, arg| match state {
                    Ok((keys, values)) => Err((keys, values, arg)),
                    Err((mut keys, mut values, key)) => {
                        keys.push(key);
                        values.push(arg);
                        Ok((keys, values))
                    }
                },
            )
            .map_err(|(_keys, _values, key)| {
                format!(
                    "Expected <key1>, <value1>, <key2>, <value2>..., received trailing key: {key}"
                )
            })?;
        let (keys, values) = deduplicate_hashmap_entries(&keys, &values)
            .map(|entries| entries.into_iter().unzip())
            .unwrap_or((keys, values));
        Ok(factory.create_record_term(
            allocator.create_struct_prototype(allocator.create_list(keys)),
            allocator.create_list(values),
        ))
    }
}
