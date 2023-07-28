// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    uuid, Applicable, ArgType, Arity, Builtin, EvaluationCache, Expression, ExpressionFactory,
    ExpressionListType, FunctionArity, HeapAllocator, RecordTermType, RefType, Uid, Uuid,
};

use crate::stdlib::{Apply, CollectList};

pub struct ResolveRecord;
impl ResolveRecord {
    pub const UUID: Uuid = uuid!("0e580200-7a85-415b-ba8e-ac854dc51ec7");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ResolveRecord {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for ResolveRecord
where
    T::Builtin: Builtin + From<Apply> + From<CollectList>,
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
        if let Some(value) = factory.match_record_term(&target) {
            let has_dynamic_values = value
                .values()
                .as_deref()
                .iter()
                .map(|item| item.as_deref().clone())
                .any(|item| !item.is_static());
            if !has_dynamic_values {
                Ok(target)
            } else {
                Ok(factory.create_application_term(
                    factory.create_builtin_term(Apply),
                    allocator.create_pair(
                        factory.create_constructor_term(
                            allocator.clone_struct_prototype(value.prototype()),
                        ),
                        factory.create_application_term(
                            factory.create_builtin_term(CollectList),
                            allocator.clone_list(value.values()),
                        ),
                    ),
                ))
            }
        } else {
            Err(format!("Expected <struct>, received {}", target))
        }
    }
}
