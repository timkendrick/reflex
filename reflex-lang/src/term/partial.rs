// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{collections::HashSet, iter::once};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::core::{
    transform_expression_list, Applicable, ArgType, Arity, CompoundNode, DependencyList,
    DynamicState, EvaluationCache, Expression, ExpressionFactory, ExpressionListIter,
    ExpressionListType, GraphNode, HeapAllocator, LambdaTermType, PartialApplicationTermType,
    RefType, Rewritable, SerializeJson, StackOffset, Substitutions,
};

use crate::term::lambda::inline_lambda_arg_values;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct PartialApplicationTerm<T: Expression> {
    target: T,
    args: T::ExpressionList,
}

impl<T: Expression> std::hash::Hash for PartialApplicationTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.target.id().hash(state);
        self.args.id().hash(state);
    }
}

impl<T: Expression> PartialApplicationTerm<T> {
    pub fn new(target: T, args: T::ExpressionList) -> Self {
        Self { target, args }
    }
}
impl<T: Expression> PartialApplicationTermType<T> for PartialApplicationTerm<T> {
    fn target<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        (&self.target).into()
    }
    fn args<'a>(&'a self) -> T::ExpressionListRef<'a>
    where
        T::ExpressionList: 'a,
        T: 'a,
    {
        (&self.args).into()
    }
}
impl<T: Expression + Applicable<T>> GraphNode for PartialApplicationTerm<T> {
    fn size(&self) -> usize {
        1 + self.target.size() + self.args.size()
    }
    fn capture_depth(&self) -> StackOffset {
        let target_depth = self.target.capture_depth();
        let arg_depth = self.args.capture_depth();
        target_depth.max(arg_depth)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let mut target_free_variables = self.target.free_variables();
        let args_free_variables = self.args.free_variables();
        if target_free_variables.is_empty() {
            args_free_variables
        } else if args_free_variables.is_empty() {
            target_free_variables
        } else {
            target_free_variables.extend(args_free_variables);
            target_free_variables
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.target.count_variable_usages(offset) + self.args.count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        let target_dependencies = self.target.dynamic_dependencies(deep);
        if deep {
            target_dependencies.union(self.args.dynamic_dependencies(deep))
        } else {
            let eager_args = self.target.arity().map(|arity| {
                self.args
                    .iter()
                    .zip(arity.iter())
                    .filter_map(|(arg, arg_type)| match arg_type {
                        ArgType::Strict | ArgType::Eager => Some(arg),
                        _ => None,
                    })
            });
            match eager_args {
                None => target_dependencies,
                Some(args) => args.fold(target_dependencies, |acc, arg| {
                    acc.union(arg.as_deref().dynamic_dependencies(deep))
                }),
            }
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.target.has_dynamic_dependencies(deep)
            || (if deep {
                self.args.has_dynamic_dependencies(deep)
            } else {
                let eager_args = self.target.arity().map(|arity| {
                    self.args.iter().zip(arity.iter()).filter_map(
                        |(arg, arg_type)| match arg_type {
                            ArgType::Strict | ArgType::Eager => Some(arg),
                            _ => None,
                        },
                    )
                });
                match eager_args {
                    None => false,
                    Some(mut args) => args.any(|arg| arg.as_deref().has_dynamic_dependencies(deep)),
                }
            })
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.target.is_atomic() && self.args.is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}
impl<T: Expression> CompoundNode<T> for PartialApplicationTerm<T> {
    type Children<'a> = std::iter::Chain<std::iter::Once<T::ExpressionRef<'a>>, ExpressionListIter<'a, T>>
        where
            T: 'a,
            Self: 'a;
    fn children<'a>(&'a self) -> Self::Children<'a>
    where
        T: 'a,
    {
        once((&self.target).into()).chain(self.args.iter())
    }
}
impl<T: Expression + Rewritable<T>> Rewritable<T> for PartialApplicationTerm<T> {
    fn substitute_static(
        &self,
        substitutions: &Substitutions<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        let target = self
            .target
            .substitute_static(substitutions, factory, allocator, cache);
        let args = transform_expression_list(&self.args, allocator, |arg| {
            arg.substitute_static(substitutions, factory, allocator, cache)
        });
        if target.is_none() && args.is_none() {
            return None;
        }
        let target = target.unwrap_or_else(|| self.target.clone());
        let args = args.unwrap_or_else(|| allocator.clone_list((&self.args).into()));
        Some(factory.create_partial_application_term(target, args))
    }
    fn substitute_dynamic(
        &self,
        deep: bool,
        state: &impl DynamicState<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        let target = self
            .target
            .substitute_dynamic(deep, state, factory, allocator, cache);
        let args = transform_expression_list(&self.args, allocator, |arg| {
            arg.substitute_dynamic(deep, state, factory, allocator, cache)
        });
        if target.is_none() && args.is_none() {
            return None;
        }
        let target = target.unwrap_or_else(|| self.target.clone());
        let args = args.unwrap_or_else(|| allocator.clone_list((&self.args).into()));
        Some(factory.create_partial_application_term(target, args))
    }
    fn hoist_free_variables(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
    ) -> Option<T> {
        let hoisted_target = self.target.hoist_free_variables(factory, allocator);
        let hoisted_args = transform_expression_list(&self.args, allocator, |arg| {
            arg.hoist_free_variables(factory, allocator)
        });
        if hoisted_target.is_none() && hoisted_args.is_none() {
            None
        } else {
            Some(factory.create_partial_application_term(
                hoisted_target.unwrap_or_else(|| self.target.clone()),
                hoisted_args.unwrap_or_else(|| allocator.clone_list((&self.args).into())),
            ))
        }
    }
    fn normalize(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        let normalized_target = self.target.normalize(factory, allocator, cache);
        if self.args.len() == 0 {
            return normalized_target.or_else(|| Some(self.target.clone()));
        }
        let normalized_args = transform_expression_list(&self.args, allocator, |arg| {
            arg.normalize(factory, allocator, cache)
        });
        let target = normalized_target.as_ref().unwrap_or(&self.target);
        let args = normalized_args.as_ref().unwrap_or(&self.args);
        if let Some(partial) = factory.match_partial_application_term(target) {
            factory
                .create_partial_application_term(
                    partial.target().as_deref().clone(),
                    allocator.create_sized_list(
                        partial.args().as_deref().len() + args.len(),
                        partial
                            .args()
                            .as_deref()
                            .iter()
                            .map(|item| item.as_deref().clone())
                            .chain(args.iter().map(|item| item.as_deref().clone())),
                    ),
                )
                .normalize(factory, allocator, cache)
        } else {
            normalize_partial_application(target, args, factory, allocator, cache).or_else(|| {
                if normalized_target.is_some() || normalized_args.is_some() {
                    Some(factory.create_partial_application_term(
                        normalized_target.unwrap_or_else(|| self.target.clone()),
                        normalized_args.unwrap_or_else(|| self.args.clone()),
                    ))
                } else {
                    None
                }
            })
        }
    }
}
impl<T: Expression + Applicable<T>> Applicable<T> for PartialApplicationTerm<T> {
    fn arity(&self) -> Option<Arity> {
        match self.target.arity() {
            None => None,
            Some(arity) => Some(arity.partial(self.args.len())),
        }
    }
    fn apply(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        self.target.apply(
            self.args
                .iter()
                .map(|item| item.as_deref().clone())
                .chain(args.into_iter())
                .collect::<Vec<_>>() // Required to prevent infinite type recursion
                .into_iter(),
            factory,
            allocator,
            cache,
        )
    }
    fn should_parallelize(&self, _args: &[T]) -> bool {
        false
    }
}

fn normalize_partial_application<T: Expression + Rewritable<T>>(
    target: &T,
    args: &T::ExpressionList,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    cache: &mut impl EvaluationCache<T>,
) -> Option<T> {
    if let Some(lambda) = factory.match_lambda_term(target) {
        normalize_partial_lambda_application(target, lambda, args, factory, allocator, cache)
    } else {
        None
    }
}

fn normalize_partial_lambda_application<T: Expression + Rewritable<T>>(
    target: &T,
    lambda: &T::LambdaTerm,
    args: &T::ExpressionList,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    cache: &mut impl EvaluationCache<T>,
) -> Option<T> {
    if args.len() == 0 {
        Some(target.clone())
    } else {
        let num_args = lambda.num_args();
        let (inlined_args, remaining_args): (Vec<_>, Vec<_>) = args
            .iter()
            .map(|item| item.as_deref().clone())
            .take(num_args)
            .enumerate()
            .map(|(index, arg)| (num_args - index - 1, arg))
            .partition({
                // Prevent currently-unreferenced free variable scopes from being pulled into partial application targets
                let existing_capture_depth = target.capture_depth();
                move |(_offset, arg)| arg.capture_depth() <= existing_capture_depth
            });
        if inlined_args.is_empty() {
            return None;
        }
        let remaining_args = allocator.create_list(remaining_args.into_iter().map(|(_, arg)| arg));
        inline_lambda_arg_values(lambda, inlined_args, factory, allocator, cache)
            .map(|target| {
                if remaining_args.len() == 0 {
                    target
                        .normalize(factory, allocator, cache)
                        .unwrap_or(target)
                } else {
                    let target = target
                        .normalize(factory, allocator, cache)
                        .unwrap_or(target);
                    factory.create_partial_application_term(target, remaining_args)
                }
            })
            .or_else(|| {
                if args.len() > num_args {
                    Some(
                        factory.create_partial_application_term(
                            target.clone(),
                            allocator.create_list(
                                args.iter()
                                    .take(num_args)
                                    .map(|item| item.as_deref().clone()),
                            ),
                        ),
                    )
                } else {
                    None
                }
            })
    }
}

impl<T: Expression> std::fmt::Display for PartialApplicationTerm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<partial:{}:{}>", self.args.len(), self.target)
    }
}

impl<T: Expression> SerializeJson for PartialApplicationTerm<T> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

#[cfg(test)]
mod tests {
    use reflex::{
        cache::SubstitutionCache,
        core::{ExpressionFactory, HeapAllocator, Rewritable},
    };
    use reflex_stdlib::{Add, Get, Stdlib, Subtract};

    use crate::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};

    #[test]
    fn normalize_fully_applied_partial() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::<CachedSharedTerm<Stdlib>>::default();

        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(0, factory.create_int_term(3)),
            allocator.create_empty_list(),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_lambda_term(0, factory.create_int_term(3)))
        );

        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(1, factory.create_variable_term(0)),
            allocator.create_empty_list(),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_lambda_term(1, factory.create_variable_term(0)))
        );

        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(1, factory.create_variable_term(0)),
            allocator.create_unit_list(factory.create_variable_term(0)),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(result, None);

        let expression = factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_lambda_term(1, factory.create_variable_term(0)),
                allocator.create_unit_list(factory.create_int_term(3)),
            ),
            allocator.create_empty_list(),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(result, Some(factory.create_int_term(3)));

        let expression = factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_lambda_term(0, factory.create_int_term(3)),
                allocator.create_empty_list(),
            ),
            allocator.create_empty_list(),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(result, Some(factory.create_int_term(3)));

        let expression = factory.create_application_term(
            factory.create_partial_application_term(
                factory.create_lambda_term(1, factory.create_variable_term(0)),
                allocator.create_empty_list(),
            ),
            allocator.create_unit_list(factory.create_int_term(3)),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(result, Some(factory.create_int_term(3)));
    }

    #[test]
    fn normalize_fully_specified_partial_application() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::<CachedSharedTerm<Stdlib>>::default();
        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(Subtract),
                    allocator.create_pair(
                        factory.create_variable_term(2),
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_variable_term(1),
                                factory.create_variable_term(0),
                            ),
                        ),
                    ),
                ),
            ),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_int_term(4),
                factory.create_int_term(5),
            ),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_lambda_term(0, factory.create_int_term(3 - (4 + 5))))
        );
    }

    #[test]
    fn normalize_partially_specified_partial_application() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::<CachedSharedTerm<Stdlib>>::default();
        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(
                4,
                factory.create_application_term(
                    factory.create_builtin_term(Subtract),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Subtract),
                            allocator.create_pair(
                                factory.create_variable_term(3),
                                factory.create_variable_term(2),
                            ),
                        ),
                        factory.create_application_term(
                            factory.create_builtin_term(Subtract),
                            allocator.create_pair(
                                factory.create_variable_term(1),
                                factory.create_variable_term(0),
                            ),
                        ),
                    ),
                ),
            ),
            allocator.create_triple(
                factory.create_int_term(3),
                factory.create_variable_term(123),
                factory.create_int_term(4),
            ),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_partial_application_term(
                factory.create_lambda_term(
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(Subtract),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Subtract),
                                allocator.create_pair(
                                    factory.create_int_term(3),
                                    factory.create_variable_term(1),
                                ),
                            ),
                            factory.create_application_term(
                                factory.create_builtin_term(Subtract),
                                allocator.create_pair(
                                    factory.create_int_term(4),
                                    factory.create_variable_term(0),
                                ),
                            ),
                        ),
                    ),
                ),
                allocator.create_unit_list(factory.create_variable_term(123)),
            )),
        );
    }

    #[test]
    fn normalize_reused_primitive_partial_application_args() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::<CachedSharedTerm<Stdlib>>::default();
        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(
                5,
                factory.create_application_term(
                    factory.create_variable_term(4),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_variable_term(4),
                            allocator.create_pair(
                                factory.create_variable_term(3),
                                factory.create_variable_term(2),
                            ),
                        ),
                        factory.create_application_term(
                            factory.create_variable_term(4),
                            allocator.create_pair(
                                factory.create_variable_term(1),
                                factory.create_variable_term(0),
                            ),
                        ),
                    ),
                ),
            ),
            allocator.create_list([
                factory.create_builtin_term(Add),
                factory.create_int_term(3),
                factory.create_variable_term(123),
                factory.create_int_term(4),
                factory.create_variable_term(456),
            ]),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_partial_application_term(
                factory.create_lambda_term(
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(Add),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_int_term(3),
                                    factory.create_variable_term(1),
                                ),
                            ),
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_int_term(4),
                                    factory.create_variable_term(0),
                                ),
                            ),
                        ),
                    ),
                ),
                allocator.create_pair(
                    factory.create_variable_term(123),
                    factory.create_variable_term(456),
                ),
            ))
        );
    }

    #[test]
    fn normalize_reused_complex_partial_application_args() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::<CachedSharedTerm<Stdlib>>::default();
        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(
                6,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Get),
                            allocator.create_pair(
                                factory.create_variable_term(5),
                                factory.create_application_term(
                                    factory.create_builtin_term(Get),
                                    allocator.create_pair(
                                        factory.create_variable_term(4),
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_variable_term(3),
                                                factory.create_variable_term(2),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_application_term(
                            factory.create_builtin_term(Get),
                            allocator.create_pair(
                                factory.create_variable_term(5),
                                factory.create_application_term(
                                    factory.create_builtin_term(Get),
                                    allocator.create_pair(
                                        factory.create_variable_term(4),
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_variable_term(1),
                                                factory.create_variable_term(0),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
            allocator.create_list([
                factory.create_list_term(allocator.create_triple(
                    factory.create_int_term(0),
                    factory.create_int_term(1),
                    factory.create_int_term(2),
                )),
                factory.create_list_term(allocator.create_triple(
                    factory.create_int_term(3),
                    factory.create_int_term(4),
                    factory.create_int_term(5),
                )),
                factory.create_int_term(6),
                factory.create_variable_term(0),
                factory.create_int_term(7),
                factory.create_variable_term(1),
            ]),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_partial_application_term(
                factory.create_lambda_term(
                    2,
                    factory.create_let_term(
                        factory.create_list_term(allocator.create_triple(
                            factory.create_int_term(0),
                            factory.create_int_term(1),
                            factory.create_int_term(2),
                        )),
                        factory.create_let_term(
                            factory.create_list_term(allocator.create_triple(
                                factory.create_int_term(3),
                                factory.create_int_term(4),
                                factory.create_int_term(5),
                            )),
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_application_term(
                                        factory.create_builtin_term(Get),
                                        allocator.create_pair(
                                            factory.create_variable_term(1),
                                            factory.create_application_term(
                                                factory.create_builtin_term(Get),
                                                allocator.create_pair(
                                                    factory.create_variable_term(0),
                                                    factory.create_application_term(
                                                        factory.create_builtin_term(Add),
                                                        allocator.create_pair(
                                                            factory.create_int_term(6),
                                                            factory.create_variable_term(3),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(Get),
                                        allocator.create_pair(
                                            factory.create_variable_term(1),
                                            factory.create_application_term(
                                                factory.create_builtin_term(Get),
                                                allocator.create_pair(
                                                    factory.create_variable_term(0),
                                                    factory.create_application_term(
                                                        factory.create_builtin_term(Add),
                                                        allocator.create_pair(
                                                            factory.create_int_term(7),
                                                            factory.create_variable_term(2),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            )
                        ),
                    ),
                ),
                allocator.create_pair(
                    factory.create_variable_term(0),
                    factory.create_variable_term(1),
                ),
            ))
        );
    }

    #[test]
    fn normalize_higher_order_partial_application_arg_usages() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::<CachedSharedTerm<Stdlib>>::default();
        let expression = factory.create_partial_application_term(
            factory.create_lambda_term(
                1,
                factory.create_application_term(
                    factory.create_lambda_term(
                        1,
                        factory.create_application_term(
                            factory.create_variable_term(1),
                            allocator.create_unit_list(factory.create_int_term(3)),
                        ),
                    ),
                    allocator.create_unit_list(factory.create_boolean_term(false)),
                ),
            ),
            allocator
                .create_unit_list(factory.create_lambda_term(1, factory.create_boolean_term(true))),
        );
        let result = expression.normalize(&factory, &allocator, &mut SubstitutionCache::new());
        assert_eq!(
            result,
            Some(factory.create_lambda_term(0, factory.create_boolean_term(true)))
        );
    }
}
