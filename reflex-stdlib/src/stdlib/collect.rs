// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    deduplicate_hashmap_entries, deduplicate_hashset_entries, match_typed_expression_list, uuid,
    Applicable, ArgType, Arity, ConditionListType, EvaluationCache, Expression, ExpressionFactory,
    FunctionArity, HeapAllocator, RefType, SignalTermType, Uid, Uuid,
};

use crate::Get;

pub struct CollectList;
impl CollectList {
    pub const UUID: Uuid = uuid!("c99b0901-f996-4887-9403-c2f123b779b0");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Strict),
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for CollectList {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for CollectList {
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
        Ok(factory.create_list_term(allocator.create_list(args)))
    }
}

pub struct CollectHashSet;
impl CollectHashSet {
    pub const UUID: Uuid = uuid!("941b9298-62d3-47e1-a909-d4252df2c935");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Strict),
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for CollectHashSet {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for CollectHashSet {
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
        let values = args.collect::<Vec<_>>();
        let deduplicated_values = match deduplicate_hashset_entries(&values) {
            Some(values) => values,
            None => values,
        };
        Ok(factory.create_hashset_term(deduplicated_values))
    }
}

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
impl<T: Expression> Applicable<T> for CollectHashMap
where
    T::Builtin: From<Get>,
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

pub struct CollectSignal;
impl CollectSignal {
    pub const UUID: Uuid = uuid!("d4ea9c07-b2da-4da7-89c0-352eb5abd35f");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Eager),
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for CollectSignal {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for CollectSignal {
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
        let args = args.into_iter().collect::<Vec<_>>();
        let signals = match_typed_expression_list(
            args.iter(),
            |arg| factory.match_signal_term(arg).cloned(),
            |arg| format!("Expected <signal>, received {arg}"),
        )?
        .into_iter()
        .fold(Vec::new(), |mut results, term| {
            results.extend(
                term.signals()
                    .as_deref()
                    .iter()
                    .map(|signal| signal.as_deref().clone()),
            );
            results
        });
        Ok(factory.create_signal_term(allocator.create_signal_list(signals)))
    }
}
