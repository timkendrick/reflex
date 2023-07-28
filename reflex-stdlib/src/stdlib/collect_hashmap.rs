// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    deduplicate_hashmap_entries, uuid, Applicable, ArgType, Arity, EvaluationCache, Expression,
    ExpressionFactory, FunctionArity, HeapAllocator, Uid, Uuid,
};

pub struct CollectHashMap;
impl CollectHashMap {
    pub const UUID: Uuid = uuid!("4a9b9c32-8597-4a23-875e-d320f187cf7a");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Strict),
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for CollectHashMap {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for CollectHashMap {
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
        let num_args = args.len();
        let entries = args
            .fold(
                Ok(Vec::with_capacity(num_args / 2)),
                |state, arg| match state {
                    Ok(entries) => Err((entries, arg)),
                    Err((mut entries, key)) => {
                        entries.push((key, arg));
                        Ok(entries)
                    }
                },
            )
            .map_err(|(_entries, key)| {
                format!(
                    "Expected <key1>, <value1>, <key2>, <value2>..., received trailing key: {key}"
                )
            })?;
        // FIXME: prevent unnecessary vector allocations
        let (keys, values): (Vec<_>, Vec<_>) = entries
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .unzip();
        let entries = deduplicate_hashmap_entries(&keys, &values).unwrap_or(entries);
        Ok(factory.create_hashmap_term(entries))
    }
}
