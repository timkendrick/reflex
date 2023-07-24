// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    match_typed_expression_list, uuid, Applicable, ArgType, Arity, ConditionListType,
    EvaluationCache, Expression, ExpressionFactory, FunctionArity, HeapAllocator, RefType,
    SignalTermType, Uid, Uuid,
};

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
