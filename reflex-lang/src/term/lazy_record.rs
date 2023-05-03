// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

use reflex::core::{
    transform_expression_list, transform_expression_list_values, ArgType, CompoundNode,
    DependencyList, DynamicState, Eagerness, Evaluate, EvaluationCache, EvaluationResult,
    Expression, ExpressionFactory, ExpressionListIter, ExpressionListType, GraphNode,
    HeapAllocator, Internable, LazyRecordTermType, Reducible, RefType, Rewritable, SerializeJson,
    StackOffset, StructPrototypeType, Substitutions,
};
use reflex_utils::json::is_empty_json_object;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LazyRecordTerm<T: Expression> {
    prototype: T::StructPrototype,
    values: T::ExpressionList,
    eagerness: Vec<ArgType>,
}

impl<T: Expression> std::hash::Hash for LazyRecordTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.prototype.keys().as_deref().id().hash(state);
        self.values.id().hash(state);
        self.eagerness.hash(state);
    }
}

impl<T: Expression> LazyRecordTerm<T> {
    pub fn new(
        prototype: T::StructPrototype,
        values: T::ExpressionList,
        eagerness: impl IntoIterator<Item = ArgType>,
    ) -> Self {
        Self {
            prototype,
            values,
            eagerness: eagerness.into_iter().collect(),
        }
    }
}
impl<T: Expression> LazyRecordTermType<T> for LazyRecordTerm<T> {
    type EagernessIterator<'a> = std::iter::Copied<std::slice::Iter<'a, ArgType>>
    where
        Self: 'a;
    fn prototype<'a>(&'a self) -> T::StructPrototypeRef<'a>
    where
        T::StructPrototype: 'a,
        T: 'a,
    {
        (&self.prototype).into()
    }
    fn values<'a>(&'a self) -> T::ExpressionListRef<'a>
    where
        T::ExpressionList: 'a,
        T: 'a,
    {
        (&self.values).into()
    }
    fn eagerness<'a>(&'a self) -> Self::EagernessIterator<'a>
    where
        Self: 'a,
    {
        self.eagerness.iter().copied()
    }
    fn get<'a>(&'a self, key: &T) -> Option<T::ExpressionRef<'a>>
    where
        T: 'a,
    {
        let index = self
            .prototype
            .keys()
            .as_deref()
            .iter()
            .position(|field_name| field_name.as_deref().id() == key.as_deref().id())?;
        self.values.get(index)
    }
}
impl<T: Expression> GraphNode for LazyRecordTerm<T> {
    fn size(&self) -> usize {
        1 + self.values.size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.values.capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.values.free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.values.count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.values.dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.values.has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        self.values
            .iter()
            .zip(self.eagerness.iter().copied())
            .all(|(value, eagerness)| match eagerness {
                ArgType::Lazy => true,
                ArgType::Eager | ArgType::Strict => value.as_deref().is_static(),
            })
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}
impl<T: Expression> CompoundNode<T> for LazyRecordTerm<T> {
    type Children<'a> = ExpressionListIter<'a, T>
        where
            T: 'a,
            Self: 'a;
    fn children<'a>(&'a self) -> Self::Children<'a>
    where
        T: 'a,
    {
        self.values.iter()
    }
}
impl<T: Expression + Rewritable<T>> Rewritable<T> for LazyRecordTerm<T> {
    fn substitute_static(
        &self,
        substitutions: &Substitutions<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        transform_expression_list(&self.values, allocator, |expression| {
            expression.substitute_static(substitutions, factory, allocator, cache)
        })
        .map(|fields| {
            factory.create_lazy_record_term(
                allocator.clone_struct_prototype((&self.prototype).into()),
                fields,
                self.eagerness.iter().copied(),
            )
        })
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
            transform_expression_list(&self.values, allocator, |expression| {
                expression.substitute_dynamic(deep, state, factory, allocator, cache)
            })
            .map(|fields| {
                factory.create_lazy_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    fields,
                    self.eagerness.iter().copied(),
                )
            })
        } else {
            None
        }
    }
    fn hoist_free_variables(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
    ) -> Option<T> {
        transform_expression_list(&self.values, allocator, |expression| {
            expression.hoist_free_variables(factory, allocator)
        })
        .map(|fields| {
            factory.create_lazy_record_term(
                allocator.clone_struct_prototype((&self.prototype).into()),
                fields,
                self.eagerness.iter().copied(),
            )
        })
    }
    fn normalize(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        transform_expression_list(&self.values, allocator, |expression| {
            expression.normalize(factory, allocator, cache)
        })
        .map(|fields| {
            if fields
                .iter()
                .zip(self.eagerness.iter())
                .all(|(field, eagerness)| {
                    field.as_deref().is_static()
                        && !(matches!(eagerness, ArgType::Strict)
                            && factory.match_signal_term(field.as_deref()).is_some())
                })
            {
                factory.create_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    fields,
                )
            } else {
                factory.create_lazy_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    fields,
                    self.eagerness.iter().copied(),
                )
            }
        })
    }
}
impl<T: Expression + Reducible<T>> Reducible<T> for LazyRecordTerm<T> {
    fn is_reducible(&self) -> bool {
        true
    }
    fn reduce(
        &self,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<T> {
        transform_expression_list_values(
            &self.values,
            |index, expression: &T| match self.eagerness.get(index) {
                Some(ArgType::Lazy) => None,
                _ => expression.reduce(factory, allocator, cache),
            },
            |_index, item| item.clone(),
        )
        .map(|values| allocator.create_list(values))
        .map(|fields| {
            if fields
                .iter()
                .zip(self.eagerness.iter())
                .all(|(field, eagerness)| {
                    field.as_deref().is_static()
                        && !(matches!(eagerness, ArgType::Strict)
                            && factory.match_signal_term(field.as_deref()).is_some())
                })
            {
                factory.create_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    fields,
                )
            } else {
                factory.create_lazy_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    fields,
                    self.eagerness.iter().copied(),
                )
            }
        })
    }
}
impl<T: Expression + Evaluate<T>> Evaluate<T> for LazyRecordTerm<T> {
    fn evaluate(
        &self,
        state: &impl DynamicState<T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Option<EvaluationResult<T>> {
        transform_expression_list_values(
            &self.values,
            |index, expression: &T| match self.eagerness.get(index) {
                Some(ArgType::Lazy) => None,
                _ => expression.evaluate(state, factory, allocator, cache),
            },
            |_index, expression: &T| {
                EvaluationResult::new(expression.clone(), DependencyList::default())
            },
        )
        .map(|results| {
            let (fields, dependencies): (Vec<_>, Vec<_>) = results
                .into_iter()
                .map(|result| result.into_parts())
                .unzip();
            let dependencies = dependencies
                .into_iter()
                .flatten()
                .collect::<DependencyList>();
            let value = if fields
                .iter()
                .zip(self.eagerness.iter())
                .all(|(field, eagerness)| {
                    field.is_static()
                        && !(matches!(eagerness, ArgType::Strict)
                            && factory.match_signal_term(field.as_deref()).is_some())
                }) {
                factory.create_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    allocator.create_list(fields),
                )
            } else {
                factory.create_lazy_record_term(
                    allocator.clone_struct_prototype((&self.prototype).into()),
                    allocator.create_list(fields),
                    self.eagerness.iter().copied(),
                )
            };
            EvaluationResult::new(value, dependencies)
        })
    }
}

impl<T: Expression> Internable for LazyRecordTerm<T> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        eager == Eagerness::Lazy && self.capture_depth() == 0
    }
}

impl<T: Expression> std::fmt::Display for LazyRecordTerm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.prototype.keys().as_deref().len() {
            0 => write!(f, "{{}}"),
            _ => write!(
                f,
                "{{ {} }}",
                self.prototype
                    .keys()
                    .as_deref()
                    .iter()
                    .zip(self.values.iter())
                    .map(|(key, value)| format!("{}: {}", key.as_deref(), value.as_deref()))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}
impl<T: Expression> SerializeJson for LazyRecordTerm<T> {
    fn to_json(&self) -> Result<JsonValue, String> {
        let fields = self
            .prototype
            .keys()
            .as_deref()
            .iter()
            .zip(self.values.iter())
            .map(|(key, value)| {
                let key = key.as_deref().to_json()?;
                let value = value.as_deref().to_json()?;
                match key {
                    JsonValue::String(key) => Ok((key, value)),
                    _ => Err(format!("Invalid JSON object key: {}", key.to_string())),
                }
            })
            .collect::<Result<JsonMap<_, _>, String>>()?;
        Ok(JsonValue::Object(fields))
    }

    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.prototype.keys().as_deref().len() != target.prototype.keys().as_deref().len() {
            return Err(format!(
                "Prototype has changed from {} to {}",
                self.prototype, target.prototype
            ));
        }
        let updates = JsonValue::Object(
            target
                .prototype
                .keys()
                .as_deref()
                .iter()
                .zip(target.values.iter())
                .map(|(key, new_value)| {
                    let previous_value = self.get(key.as_deref()).ok_or_else(|| {
                        format!(
                            "Prototype has changed, key {} not present in {}",
                            key.as_deref(),
                            self.prototype
                        )
                    })?;
                    Ok(previous_value
                        .as_deref()
                        .patch(new_value.as_deref())?
                        .map(|value_patch| (key, value_patch)))
                })
                .filter_map(|entry| entry.transpose()) // Filter out unchanged fields
                .map(|entry| {
                    entry.and_then(|(key, value)| match key.as_deref().to_json()? {
                        JsonValue::String(key) => Ok((key, value)),
                        _ => Err(format!("Invalid JSON object key: {}", key.as_deref())),
                    })
                })
                .collect::<Result<JsonMap<_, _>, _>>()?,
        );

        if is_empty_json_object(&updates) {
            Ok(None)
        } else {
            Ok(Some(updates))
        }
    }
}
