// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, ConditionListType, ConditionType, EvaluationCache,
    Expression, ExpressionFactory, FunctionArity, HeapAllocator, RefType, SignalTermType,
    SignalType, Uid, Uuid,
};

pub struct IfError;
impl IfError {
    pub const UUID: Uuid = uuid!("ca015984-8f33-49c5-b821-aca8fb122ee8");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Eager, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for IfError {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for IfError {
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
        let handler = args.next().unwrap();
        if let Some(signal) = factory.match_signal_term(&target) {
            let (error_payloads, other_signals) = signal
                .signals()
                .as_deref()
                .iter()
                .map(|item| item.as_deref().clone())
                .fold(
                    (Vec::new(), Vec::new()),
                    |(mut error_payloads, mut other_signals), signal| {
                        if let SignalType::Error { payload, .. } = signal.signal_type() {
                            error_payloads.push(payload);
                        } else {
                            other_signals.push(signal);
                        }
                        (error_payloads, other_signals)
                    },
                );
            if error_payloads.is_empty() {
                Ok(target.clone())
            } else if !other_signals.is_empty() {
                Ok(factory.create_signal_term(allocator.create_signal_list(other_signals)))
            } else {
                Ok(factory.create_application_term(
                    handler,
                    allocator.create_unit_list(
                        factory.create_list_term(allocator.create_list(error_payloads)),
                    ),
                ))
            }
        } else {
            Ok(target)
        }
    }
}
