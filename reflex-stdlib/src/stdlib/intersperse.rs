// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use reflex::core::{
    uuid, Applicable, ArgType, Arity, EvaluationCache, Expression, ExpressionFactory,
    ExpressionListType, FunctionArity, HeapAllocator, ListTermType, RefType, Uid, Uuid,
};

pub struct Intersperse;
impl Intersperse {
    pub const UUID: Uuid = uuid!("85c212ff-c095-45de-b3b4-de628ad9e518");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity() -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Intersperse {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
impl<T: Expression> Applicable<T> for Intersperse {
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
        let separator = args.next().unwrap();
        if let Some(collection) = factory.match_list_term(&target) {
            let num_items = collection.items().as_deref().len();
            Ok(match num_items {
                0 => factory.create_list_term(allocator.create_empty_list()),
                1 => target,
                num_items => {
                    let list_items = collection.items();
                    let final_item = list_items
                        .as_deref()
                        .get(num_items - 1)
                        .map(|item| item.as_deref().clone());
                    let preceding_items = list_items
                        .as_deref()
                        .iter()
                        .map(|item| item.as_deref().clone())
                        .take(num_items - 1)
                        .flat_map(|item| [item, separator.clone()]);
                    let num_separators = num_items - 1;
                    factory.create_list_term(allocator.create_sized_list(
                        num_items + num_separators,
                        preceding_items.chain(final_item),
                    ))
                }
            })
        } else {
            Err(format!(
                "Expected (List, <any>), received ({}, {})",
                target, separator
            ))
        }
    }
}
