// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    ExpressionListType, FunctionArity, HeapAllocator, ListTermType, RefType, Uid, Uuid,
};

pub struct ConstructRecord;
impl ConstructRecord {
    pub const UUID: Uuid = uuid!("f3a1b7ad-fe7d-444b-adf3-6945332e03b7");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ConstructRecord {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for ConstructRecord {
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
        let keys = args.next().unwrap();
        let values = args.next().unwrap();
        let (num_keys, keys) = match factory.match_list_term(&keys) {
            Some(keys) => {
                let num_keys = keys.items().as_deref().len();
                Ok((
                    num_keys,
                    allocator.create_list(
                        keys.items()
                            .as_deref()
                            .iter()
                            .map(|item| item.as_deref().clone()),
                    ),
                ))
            }
            None => Err(format!("Invalid property keys: {}", keys)),
        }?;
        match factory.match_list_term(&values) {
            Some(values) => {
                if values.items().as_deref().len() != num_keys {
                    Err(format!(
                        "Invalid property entries: received {} keys and {} values",
                        num_keys,
                        values.items().as_deref().len(),
                    ))
                } else {
                    Ok(factory.create_record_term(
                        allocator.create_struct_prototype(keys),
                        allocator.clone_list(values.items()),
                    ))
                }
            }
            None => Err(format!("Invalid property values: {}", values)),
        }
    }
}
