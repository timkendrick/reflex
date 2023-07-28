// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{
    collections::{HashMap, HashSet},
    iter::once,
};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::{
    cache::NoopCache,
    core::{
        Applicable, ApplicationTermType, Arity, CompoundNode, DependencyList, DynamicState,
        EvaluationCache, Expression, ExpressionFactory, ExpressionListType, GraphNode,
        HeapAllocator, LambdaTermType, RefType, Rewritable, ScopeOffset, SerializeJson,
        StackOffset, Substitutions, VariableTermType,
    },
};

use crate::term::variable::should_inline_value;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LambdaTerm<T: Expression> {
    num_args: StackOffset,
    body: T,
}

impl<T: Expression> std::hash::Hash for LambdaTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.num_args.hash(state);
        self.body.id().hash(state);
    }
}

impl<T: Expression> LambdaTerm<T> {
    pub fn new(num_args: StackOffset, body: T) -> Self {
        Self { num_args, body }
    }
}
impl<T: Expression> LambdaTermType<T> for LambdaTerm<T> {
    fn num_args(&self) -> StackOffset {
        self.num_args
    }
    fn body<'a>(&'a self) -> T::ExpressionRef<'a>
    where
        T: 'a,
    {
        (&self.body).into()
    }
}
impl<T: Expression> GraphNode for LambdaTerm<T> {
    fn size(&self) -> usize {
        1 + self.body.size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.body.capture_depth().saturating_sub(self.num_args)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let num_args = self.num_args;
        self.body
            .free_variables()
            .into_iter()
            .filter_map(|offset| {
                if offset < num_args {
                    None
                } else {
                    Some(offset - num_args)
                }
            })
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.body.count_variable_usages(offset + self.num_args)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.body.dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.body.has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}
impl<T: Expression> CompoundNode<T> for LambdaTerm<T> {
    type Children<'a> = std::iter::Once<T::ExpressionRef<'a>>
        where
            T: 'a,
            Self: 'a;
    fn children<'a>(&'a self) -> Self::Children<'a>
    where
        T: 'a,
    {
        once((&self.body).into())
    }
}
impl<T: Expression + Rewritable<T>> Rewritable<T> for LambdaTerm<T> {
    fn substitute_static(
        &self,
        substitutions: &Substitutions<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        self.body
            .substitute_static(
                &substitutions.offset(self.num_args),
                factory,
                allocator,
                cache,
            )
            .map(|body| factory.create_lambda_term(self.num_args, body))
    }
    fn substitute_dynamic(
        &self,
        deep: bool,
        state: &impl DynamicState<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        if deep {
            self.body
                .substitute_dynamic(deep, state, factory, allocator, cache)
                .map(|body| factory.create_lambda_term(self.num_args, body))
        } else {
            None
        }
    }
    fn hoist_free_variables(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
    ) -> Option<T> {
        let free_variables = self.free_variables().into_iter().collect::<Vec<_>>();
        let hoisted_body = self.body.hoist_free_variables(factory, allocator);
        if free_variables.is_empty() {
            hoisted_body.map(|body| factory.create_lambda_term(self.num_args, body))
        } else {
            let mut stack_offsets = free_variables;
            stack_offsets.sort();
            let substitutions = stack_offsets
                .iter()
                .enumerate()
                .map(|(arg_offset, target_offset)| {
                    (*target_offset, factory.create_variable_term(arg_offset))
                })
                .collect::<Vec<_>>();
            let arg_values = stack_offsets
                .into_iter()
                .rev()
                .map(|offset| factory.create_variable_term(offset));
            let substitutions = Substitutions::named(&substitutions, None).offset(self.num_args);
            let body = hoisted_body.as_ref().unwrap_or(&self.body);
            match body.substitute_static(
                &substitutions,
                factory,
                allocator,
                &mut NoopCache::default(),
            ) {
                Some(body) => Some(factory.create_partial_application_term(
                    factory.create_lambda_term(arg_values.len() + self.num_args, body),
                    allocator.create_list(arg_values),
                )),
                None => hoisted_body.map(|body| factory.create_lambda_term(self.num_args, body)),
            }
        }
    }
    fn normalize(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        let normalized_body = self.body.normalize(factory, allocator, cache);
        let eta_reduced_body = apply_eta_reduction(
            normalized_body.as_ref().unwrap_or(&self.body),
            self.num_args,
            factory,
        );
        eta_reduced_body
            .and_then(|eta_reduced_body| {
                eta_reduced_body
                    .as_deref()
                    .normalize(factory, allocator, cache)
                    .or_else(|| Some(eta_reduced_body.as_deref().clone()))
            })
            .or_else(|| normalized_body.map(|body| factory.create_lambda_term(self.num_args, body)))
    }
}
impl<T: Expression + Rewritable<T>> Applicable<T> for LambdaTerm<T> {
    fn arity(&self) -> Option<Arity> {
        Some(Arity::lazy(self.num_args, 0, false))
    }
    fn apply(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        let num_args = self.num_args;
        if num_args == 0 || self.body.capture_depth() == 0 {
            Ok(self.body.clone())
        } else {
            let substitutions = args
                .into_iter()
                .take(num_args)
                .enumerate()
                .map(|(index, arg)| ((num_args - index - 1), arg))
                .collect::<Vec<_>>();
            let substitutions =
                Substitutions::named(&substitutions, Some(ScopeOffset::Unwrap(num_args)));
            Ok(self
                .body
                .substitute_static(&substitutions, factory, allocator, cache)
                .unwrap_or_else(|| self.body.clone()))
        }
    }
    fn should_parallelize(&self, _args: &[T]) -> bool {
        false
    }
}
impl<T: Expression> std::fmt::Display for LambdaTerm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function:{}>", self.num_args)
    }
}

impl<T: Expression> SerializeJson for LambdaTerm<T> {
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

pub(crate) fn inline_lambda_arg_values<T: Expression + Rewritable<T>>(
    target: &T::LambdaTerm,
    substitutions: impl IntoIterator<Item = (StackOffset, T)>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    cache: &mut impl EvaluationCache<T>,
) -> Option<T> {
    let num_args = target.num_args();
    // Collect a list of substitutions sorted by scope offset, starting with the lowest scope offset
    // (i.e. sorting the substituted arguments from rightmost to leftmost)
    let substitutions = {
        let mut substituted_args = substitutions
            .into_iter()
            // Filter out any invalid substitutions
            .filter(|(offset, _arg_value)| *offset < num_args)
            // Deduplicate any duplicate offsets
            .collect::<HashMap<_, _>>()
            .into_iter()
            .collect::<Vec<_>>();
        // Order by increasing scope offset
        substituted_args.sort_by_key(|(offset, _)| *offset);
        substituted_args
    };
    if substitutions.is_empty() {
        return None;
    }
    // Keep track of how many arguments will remain after the substitution
    let num_remaining_args = num_args - substitutions.len();
    // Iterate over each injected argument, replacing all references within the function body with the provided value
    // Note that we modify the function body incrementally, one substitution at a time, starting with the lowest scope
    // offset within the function body (i.e. the rightmost substituted argument) and iterating through the substituted
    // arguments in order of increasig scope offset
    let body = substitutions.into_iter().enumerate().fold(
        target.body().as_deref().clone(),
        |body, (substitution_index, (offset, arg_value))| {
            // Keep track of how many arguments have already been removed from the the work-in-progress lambda due to
            // previous iterations (one argument per iteration)
            let num_removed_args = substitution_index;
            // Get the current offset within the work-in-progress function body for the argument we are replacing
            // (this diverges from the provided offset as more of the arguments are inlined)
            let target_offset = offset - num_removed_args;
            // If the value can be inlined, or if the argument is unused, or if there is only a single reference to the
            // argument within the function body, we replace any references to the argument with the value itself.
            // Otherwise if the value cannot be inlined, and there are multiple references to the argument within the
            // function body, we create a lexically-scoped variable within the function body that is initialized to the
            // argument value, and replace any references to the argument with references to the scoped variable
            let should_inline = should_inline_value(&arg_value, factory)
                || body.count_variable_usages(target_offset) <= 1;
            // The provided argument value is being moved inside the lambda, so before injecting the it into the
            // function body we need to increase any variable scope offsets present within the argument value itself
            // by the number of arguments remaining in the work-in-progress lambda, to reflect the argument value having
            // been moved into the deeper scope
            let replacement = arg_value
                .substitute_static(
                    &Substitutions::increase_scope_offset(num_args - (num_removed_args + 1), 0),
                    factory,
                    allocator,
                    cache,
                )
                .unwrap_or(arg_value);
            if should_inline {
                // If the value can be inlined, or if the argument is unused, or if there is only a single reference to
                // the argument within the function body, replace any references to the argument with the value itself
                // (the scope is unwrapped on all other nodes of at least the target depth, to account for the fact that
                // an argument has been removed from the lambda)
                let substituted_body = body
                    .substitute_static(
                        &Substitutions::named(
                            &vec![(target_offset, replacement)],
                            Some(ScopeOffset::Unwrap(1)),
                        ),
                        factory,
                        allocator,
                        cache,
                    )
                    .unwrap_or(body);
                substituted_body
            } else {
                // Otherwise if the value cannot be inlined, replace the argument with a lexically-scoped variable
                // declaration within the function body that is initialized to the argument value, and replace any
                // references to the argument with references to the scoped variable.
                // Increment all scope offsets within the function body to account for them being nested within a new
                // variable scope that we are about to create
                let shifted_body = body
                    .substitute_static(
                        &Substitutions::increase_scope_offset(1, 0),
                        factory,
                        allocator,
                        cache,
                    )
                    .unwrap_or(body);
                // Given that we have just incremented all the scope offsets in the function body, we must also
                // increment the target offset that we are looking to substitute
                let target_offset = target_offset + 1;
                // Replace all references to the target variable within the function body with a reference to the new
                // lexical scope variable that we are about to create
                // (the scope is unwrapped on all other nodes of at least the target depth, to account for the fact that
                // an argument has been removed from the lambda)
                let substituted_body = shifted_body
                    .substitute_static(
                        &Substitutions::named(
                            &vec![(target_offset, factory.create_variable_term(0))],
                            Some(ScopeOffset::Unwrap(1)),
                        ),
                        factory,
                        allocator,
                        cache,
                    )
                    .unwrap_or(shifted_body);
                // Return the newly-wrapped function body
                factory.create_let_term(replacement, substituted_body)
            }
        },
    );
    Some(factory.create_lambda_term(num_remaining_args, body))
}

fn apply_eta_reduction<'a, T: Expression>(
    body: &'a T,
    num_args: StackOffset,
    factory: &'a impl ExpressionFactory<T>,
) -> Option<T::ExpressionRef<'a>> {
    match factory.match_application_term(body) {
        Some(term)
            if term.target().as_deref().capture_depth() == 0
                && term.args().as_deref().len() <= num_args
                && term
                    .args()
                    .as_deref()
                    .iter()
                    .enumerate()
                    .all(
                        |(index, arg)| match factory.match_variable_term(arg.as_deref()) {
                            Some(term) => term.offset() == num_args - index - 1,
                            _ => false,
                        },
                    ) =>
        {
            Some(term.target())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reflex::{
        cache::SubstitutionCache,
        core::{
            evaluate, DependencyList, EvaluationResult, ExpressionFactory, HeapAllocator,
            Rewritable, StateCache,
        },
    };
    use reflex_stdlib::{Add, Floor, Stdlib};

    use crate::{allocator::DefaultAllocator, SharedTermFactory};

    use super::*;

    #[test]
    fn inline_lambda_args() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::default();
        let target = {
            "
            (lambda (foo bar baz qux)
                (+ (+ (+ foo bar) (+ baz baz)) qux))
            ";
            factory.create_lambda_term(
                4,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(3),
                                        factory.create_variable_term(2),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(1),
                                        factory.create_variable_term(1),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_variable_term(0),
                    ),
                ),
            )
        };
        let term = factory.match_lambda_term(&target).unwrap();
        {
            let substitutions = [];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = None;
            assert_eq!(result, expected);
        }
        {
            let substitutions = [(0, factory.create_int_term(0))];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(2),
                                        factory.create_variable_term(1),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(0),
                                        factory.create_variable_term(0),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_int_term(0),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [(3, factory.create_int_term(3))];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_int_term(3),
                                        factory.create_variable_term(2),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(1),
                                        factory.create_variable_term(1),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_variable_term(0),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [(1, factory.create_int_term(1))];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(2),
                                        factory.create_variable_term(1),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_int_term(1),
                                        factory.create_int_term(1),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_variable_term(0),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [
                (0, factory.create_int_term(0)),
                (1, factory.create_int_term(1)),
                (2, factory.create_int_term(2)),
                (3, factory.create_int_term(3)),
            ];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                0,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_int_term(3),
                                        factory.create_int_term(2),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_int_term(1),
                                        factory.create_int_term(1),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_int_term(0),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [
                (1, factory.create_variable_term(123)),
                (2, factory.create_variable_term(234)),
            ];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                2,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(1),
                                        factory.create_variable_term(236),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(125),
                                        factory.create_variable_term(125),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_variable_term(0),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(Floor),
                    allocator.create_unit_list(factory.create_int_term(3)),
                ),
            )];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                3,
                factory.create_application_term(
                    factory.create_builtin_term(Add),
                    allocator.create_pair(
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_application_term(
                                            factory.create_builtin_term(Floor),
                                            allocator.create_unit_list(factory.create_int_term(3)),
                                        ),
                                        factory.create_variable_term(2),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(1),
                                        factory.create_variable_term(1),
                                    ),
                                ),
                            ),
                        ),
                        factory.create_variable_term(0),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [(
                1,
                factory.create_application_term(
                    factory.create_builtin_term(Floor),
                    allocator.create_unit_list(factory.create_int_term(1)),
                ),
            )];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                3,
                factory.create_let_term(
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_int_term(1)),
                    ),
                    factory.create_application_term(
                        factory.create_builtin_term(Add),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(3),
                                            factory.create_variable_term(2),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(0),
                                            factory.create_variable_term(0),
                                        ),
                                    ),
                                ),
                            ),
                            factory.create_variable_term(1),
                        ),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [
                (
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_int_term(0)),
                    ),
                ),
                (
                    1,
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_int_term(1)),
                    ),
                ),
                (
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_int_term(2)),
                    ),
                ),
                (
                    3,
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_int_term(3)),
                    ),
                ),
            ];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected =
                Some(factory.create_lambda_term(
                    0,
                    factory.create_let_term(
                        factory.create_application_term(
                            factory.create_builtin_term(Floor),
                            allocator.create_unit_list(factory.create_int_term(1)),
                        ),
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_application_term(
                                                    factory.create_builtin_term(Floor),
                                                    allocator.create_unit_list(
                                                        factory.create_int_term(3),
                                                    ),
                                                ),
                                                factory.create_application_term(
                                                    factory.create_builtin_term(Floor),
                                                    allocator.create_unit_list(
                                                        factory.create_int_term(2),
                                                    ),
                                                ),
                                            ),
                                        ),
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_variable_term(0),
                                                factory.create_variable_term(0),
                                            ),
                                        ),
                                    ),
                                ),
                                factory.create_application_term(
                                    factory.create_builtin_term(Floor),
                                    allocator.create_unit_list(factory.create_int_term(0)),
                                ),
                            ),
                        ),
                    ),
                ));
            assert_eq!(result, expected);
        }
        {
            let substitutions = [(
                1,
                factory.create_application_term(
                    factory.create_builtin_term(Floor),
                    allocator.create_unit_list(factory.create_variable_term(123)),
                ),
            )];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(factory.create_lambda_term(
                3,
                factory.create_let_term(
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_variable_term(126)),
                    ),
                    factory.create_application_term(
                        factory.create_builtin_term(Add),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(3),
                                            factory.create_variable_term(2),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(0),
                                            factory.create_variable_term(0),
                                        ),
                                    ),
                                ),
                            ),
                            factory.create_variable_term(1),
                        ),
                    ),
                ),
            ));
            assert_eq!(result, expected);
        }
        {
            let target = {
                "
                (lambda (five four three two one zero)
                    (+ (+ (+ (+ (+ (+ five five) (+ four four)) (+ three three)) (+ two two)) (+ one one)) (+ zero zero)))
                ";
                factory.create_lambda_term(
                    6,
                    factory.create_application_term(
                        factory.create_builtin_term(Add),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_application_term(
                                                factory.create_builtin_term(Add),
                                                allocator.create_pair(
                                                    factory.create_application_term(
                                                        factory.create_builtin_term(Add),
                                                        allocator.create_pair(
                                                            factory.create_application_term(
                                                                factory.create_builtin_term(Add),
                                                                allocator.create_pair(
                                                                    factory.create_variable_term(5),
                                                                    factory.create_variable_term(5),
                                                                ),
                                                            ),
                                                            factory.create_application_term(
                                                                factory.create_builtin_term(Add),
                                                                allocator.create_pair(
                                                                    factory.create_variable_term(4),
                                                                    factory.create_variable_term(4),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                    factory.create_application_term(
                                                        factory.create_builtin_term(Add),
                                                        allocator.create_pair(
                                                            factory.create_variable_term(3),
                                                            factory.create_variable_term(3),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                            factory.create_application_term(
                                                factory.create_builtin_term(Add),
                                                allocator.create_pair(
                                                    factory.create_variable_term(2),
                                                    factory.create_variable_term(2),
                                                ),
                                            ),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(1),
                                            factory.create_variable_term(1),
                                        ),
                                    ),
                                ),
                            ),
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_variable_term(0),
                                    factory.create_variable_term(0),
                                ),
                            ),
                        ),
                    ),
                )
            };
            let term = factory.match_lambda_term(&target).unwrap();
            let substitutions = [
                (1, factory.create_variable_term(123)),
                (
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_variable_term(123)),
                    ),
                ),
                (
                    4,
                    factory.create_application_term(
                        factory.create_builtin_term(Floor),
                        allocator.create_unit_list(factory.create_variable_term(456)),
                    ),
                ),
                (5, factory.create_variable_term(567)),
            ];
            let result = inline_lambda_arg_values(
                term,
                substitutions,
                &factory,
                &allocator,
                &mut SubstitutionCache::new(),
            );
            let expected = Some(
                factory.create_lambda_term(
                    2,
                    factory.create_let_term(
                        factory.create_application_term(
                            factory.create_builtin_term(Floor),
                            allocator.create_unit_list(factory.create_variable_term(458)),
                        ),
                        factory.create_let_term(
                            factory.create_application_term(
                                factory.create_builtin_term(Floor),
                                allocator.create_unit_list(factory.create_variable_term(126)),
                            ),
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_application_term(
                                                factory.create_builtin_term(Add),
                                                allocator.create_pair(
                                                    factory.create_application_term(
                                                        factory.create_builtin_term(Add),
                                                        allocator.create_pair(
                                                            factory.create_application_term(
                                                                factory.create_builtin_term(Add),
                                                                allocator.create_pair(
                                                                    factory.create_application_term(
                                                                        factory.create_builtin_term(Add),
                                                                        allocator.create_pair(
                                                                            factory.create_variable_term(571),
                                                                            factory.create_variable_term(571),
                                                                        ),
                                                                    ),
                                                                    factory.create_application_term(
                                                                        factory.create_builtin_term(Add),
                                                                        allocator.create_pair(
                                                                            factory.create_variable_term(1),
                                                                            factory.create_variable_term(1),
                                                                        ),
                                                                    ),
                                                                ),
                                                            ),
                                                            factory.create_application_term(
                                                                factory.create_builtin_term(Add),
                                                                allocator.create_pair(
                                                                    factory.create_variable_term(3),
                                                                    factory.create_variable_term(3),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                    factory.create_application_term(
                                                        factory.create_builtin_term(Add),
                                                        allocator.create_pair(
                                                            factory.create_variable_term(0),
                                                            factory.create_variable_term(0),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                            factory.create_application_term(
                                                factory.create_builtin_term(Add),
                                                allocator.create_pair(
                                                    factory.create_variable_term(127),
                                                    factory.create_variable_term(127),
                                                ),
                                            ),
                                        ),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(2),
                                            factory.create_variable_term(2),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            );
            assert_eq!(expected, result);
        }
    }

    #[test]
    fn hoist_lambda_variables() {
        let factory = SharedTermFactory::<Stdlib>::default();
        let allocator = DefaultAllocator::default();

        let expression = {
            // (lambda (two one zero)
            //     (lambda ()
            //         (+ (+ zero one) two)))
            factory.create_lambda_term(
                3,
                factory.create_lambda_term(
                    0,
                    factory.create_application_term(
                        factory.create_builtin_term(Add),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_variable_term(0),
                                    factory.create_variable_term(1),
                                ),
                            ),
                            factory.create_variable_term(2),
                        ),
                    ),
                ),
            )
        };
        let result = expression.hoist_free_variables(&factory, &allocator);
        assert_eq!(
            result,
            Some(factory.create_lambda_term(
                3,
                factory.create_partial_application_term(
                    factory.create_lambda_term(
                        3,
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_variable_term(0),
                                        factory.create_variable_term(1),
                                    ),
                                ),
                                factory.create_variable_term(2),
                            ),
                        ),
                    ),
                    allocator.create_list(vec![
                        factory.create_variable_term(2),
                        factory.create_variable_term(1),
                        factory.create_variable_term(0),
                    ]),
                )
            )),
        );
        let expression = factory.create_application_term(
            factory.create_application_term(
                result.unwrap(),
                allocator.create_list(vec![
                    factory.create_int_term(1),
                    factory.create_int_term(2),
                    factory.create_int_term(3),
                ]),
            ),
            allocator.create_empty_list(),
        );
        assert_eq!(
            evaluate(
                &expression,
                &StateCache::default(),
                &factory,
                &allocator,
                &mut SubstitutionCache::new()
            ),
            EvaluationResult::new(factory.create_int_term(1 + 2 + 3), DependencyList::empty())
        );

        let expression = {
            // (lambda (three two one)
            //     (lambda (zero)
            //         (+ (+ (+ zero one) two) three)))";
            factory.create_lambda_term(
                3,
                factory.create_lambda_term(
                    1,
                    factory.create_application_term(
                        factory.create_builtin_term(Add),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_builtin_term(Add),
                                allocator.create_pair(
                                    factory.create_application_term(
                                        factory.create_builtin_term(Add),
                                        allocator.create_pair(
                                            factory.create_variable_term(0),
                                            factory.create_variable_term(1),
                                        ),
                                    ),
                                    factory.create_variable_term(2),
                                ),
                            ),
                            factory.create_variable_term(3),
                        ),
                    ),
                ),
            )
        };
        let result = expression.hoist_free_variables(&factory, &allocator);
        assert_eq!(
            result,
            Some(factory.create_lambda_term(
                3,
                factory.create_partial_application_term(
                    factory.create_lambda_term(
                        4,
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_variable_term(0),
                                                factory.create_variable_term(1),
                                            ),
                                        ),
                                        factory.create_variable_term(2),
                                    ),
                                ),
                                factory.create_variable_term(3),
                            ),
                        ),
                    ),
                    allocator.create_list(vec![
                        factory.create_variable_term(2),
                        factory.create_variable_term(1),
                        factory.create_variable_term(0),
                    ]),
                ),
            )),
        );
        let expression = factory.create_application_term(
            factory.create_application_term(
                result.unwrap(),
                allocator.create_list(vec![
                    factory.create_int_term(1),
                    factory.create_int_term(2),
                    factory.create_int_term(3),
                ]),
            ),
            allocator.create_list(vec![factory.create_int_term(4)]),
        );
        assert_eq!(
            evaluate(
                &expression,
                &StateCache::default(),
                &factory,
                &allocator,
                &mut SubstitutionCache::new()
            ),
            EvaluationResult::new(
                factory.create_int_term(1 + 2 + 3 + 4),
                DependencyList::empty()
            )
        );

        let expression = {
            // (lambda (five four three)
            //     (lambda (two)
            //         (lambda (one zero)
            //             (+ (+ (+ (+ (+ zero one) two) three) four) five))))";
            factory.create_lambda_term(
                3,
                factory.create_lambda_term(
                    1,
                    factory.create_lambda_term(
                        2,
                        factory.create_application_term(
                            factory.create_builtin_term(Add),
                            allocator.create_pair(
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_application_term(
                                                    factory.create_builtin_term(Add),
                                                    allocator.create_pair(
                                                        factory.create_application_term(
                                                            factory.create_builtin_term(Add),
                                                            allocator.create_pair(
                                                                factory.create_variable_term(0),
                                                                factory.create_variable_term(1),
                                                            ),
                                                        ),
                                                        factory.create_variable_term(2),
                                                    ),
                                                ),
                                                factory.create_variable_term(3),
                                            ),
                                        ),
                                        factory.create_variable_term(4),
                                    ),
                                ),
                                factory.create_variable_term(5),
                            ),
                        ),
                    ),
                ),
            )
        };
        let result = expression.hoist_free_variables(&factory, &allocator);
        assert_eq!(
            result,
            Some(factory.create_lambda_term(
                3,
                factory.create_partial_application_term(
                    factory.create_lambda_term(
                        4,
                        factory.create_partial_application_term(
                            factory.create_lambda_term(
                                6,
                                factory.create_application_term(
                                    factory.create_builtin_term(Add),
                                    allocator.create_pair(
                                        factory.create_application_term(
                                            factory.create_builtin_term(Add),
                                            allocator.create_pair(
                                                factory.create_application_term(
                                                    factory.create_builtin_term(Add),
                                                    allocator.create_pair(
                                                        factory.create_application_term(
                                                            factory.create_builtin_term(Add),
                                                            allocator.create_pair(
                                                                factory.create_application_term(
                                                                    factory.create_builtin_term(Add),
                                                                    allocator.create_pair(
                                                                        factory.create_variable_term(0),
                                                                        factory.create_variable_term(1),
                                                                    ),
                                                                ),
                                                                factory.create_variable_term(2),
                                                            ),
                                                        ),
                                                        factory.create_variable_term(3),
                                                    ),
                                                ),
                                                factory.create_variable_term(4),
                                            ),
                                        ),
                                        factory.create_variable_term(5),
                                    ),
                                ),
                            ),
                            allocator.create_list(vec![
                                factory.create_variable_term(3),
                                factory.create_variable_term(2),
                                factory.create_variable_term(1),
                                factory.create_variable_term(0),
                            ]),
                        ),
                    ),
                    allocator.create_list(vec![
                        factory.create_variable_term(2),
                        factory.create_variable_term(1),
                        factory.create_variable_term(0),
                    ])
                )
            )),
        );
        let expression = factory.create_application_term(
            factory.create_application_term(
                factory.create_application_term(
                    result.unwrap(),
                    allocator.create_list(vec![
                        factory.create_int_term(1),
                        factory.create_int_term(2),
                        factory.create_int_term(3),
                    ]),
                ),
                allocator.create_list(vec![factory.create_int_term(4)]),
            ),
            allocator.create_list(vec![factory.create_int_term(5), factory.create_int_term(6)]),
        );
        assert_eq!(
            evaluate(
                &expression,
                &StateCache::default(),
                &factory,
                &allocator,
                &mut SubstitutionCache::new()
            ),
            EvaluationResult::new(
                factory.create_int_term(1 + 2 + 3 + 4 + 5 + 6),
                DependencyList::empty()
            )
        );
    }
}
