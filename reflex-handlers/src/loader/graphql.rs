// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{fs, iter::once, marker::PhantomData, path::Path};

use derivative::Derivative;
use reflex::core::{
    create_record, Builtin, Expression, ExpressionFactory, HeapAllocator, ModuleLoader,
};
use reflex_graphql::graphql_parser::{self, schema::Document};
use reflex_stdlib::{CollectList, Contains, Effect, Get, If, ResolveDeep};

use crate::actor::graphql::EFFECT_TYPE_GRAPHQL;

pub trait GraphQlLoaderBuiltin:
    Builtin
    + From<CollectList>
    + From<Contains>
    + From<Effect>
    + From<Get>
    + From<If>
    + From<ResolveDeep>
{
}
impl<T> GraphQlLoaderBuiltin for T where
    T: Builtin
        + From<CollectList>
        + From<Contains>
        + From<Effect>
        + From<Get>
        + From<If>
        + From<ResolveDeep>
{
}

pub fn create_graphql_loader<
    T: Expression,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> GraphQlModuleLoader<T, TFactory, TAllocator>
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    GraphQlModuleLoader::new(factory.clone(), allocator.clone())
}

#[derive(Derivative)]
#[derivative(Clone(bound = "TFactory: Clone, TAllocator: Clone"))]
pub struct GraphQlModuleLoader<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    GraphQlModuleLoader<T, TFactory, TAllocator>
{
    fn new(factory: TFactory, allocator: TAllocator) -> Self {
        Self {
            factory,
            allocator,
            _expression: PhantomData,
        }
    }
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>> ModuleLoader
    for GraphQlModuleLoader<T, TFactory, TAllocator>
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        if !import_path.ends_with(".graphql") {
            return None;
        }
        let schema_path = current_path
            .parent()
            .map(|parent| parent.join(import_path))
            .unwrap_or_else(|| Path::new(import_path).to_path_buf());
        Some(load_graphql_module(
            &schema_path,
            &self.factory,
            &self.allocator,
        ))
    }
}

fn load_graphql_module<T: Expression>(
    path: &Path,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> Result<T, String>
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    let source = match fs::read_to_string(path) {
        Ok(source) => Ok(source),
        Err(error) => Err(format!("{}", error)),
    }?;
    match graphql_parser::parse_schema(&source) {
        Ok(schema) => Ok(create_graphql_module(&schema, factory, allocator)),
        Err(error) => Err(format!("{}", error)),
    }
}

fn create_graphql_module<'a: 'src, 'src, T: Expression>(
    schema: &'a Document<&'src str>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    create_default_export(
        create_graphql_client_constructor(schema, factory, allocator),
        factory,
        allocator,
    )
}

fn create_graphql_client_constructor<'a: 'src, 'src, T: Expression>(
    schema: &'a Document<&'src str>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    factory.create_lambda_term(
        1,
        factory.create_application_term(
            create_graphql_client_instance(schema, factory, allocator),
            allocator.create_unit_list(get_struct_field(
                factory.create_variable_term(0),
                factory.create_string_term(allocator.create_static_string("url")),
                factory,
                allocator,
            )),
        ),
    )
}

fn create_graphql_client_instance<'a: 'src, 'src, T: Expression>(
    _schema: &'a Document<&'src str>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    factory.create_lambda_term(
        1,
        create_record(
            [(
                factory.create_string_term(allocator.create_static_string("execute")),
                factory.create_lambda_term(
                    1,
                    factory.create_application_term(
                        factory.create_builtin_term(Effect),
                        allocator.create_triple(
                            factory.create_string_term(
                                allocator.create_static_string(EFFECT_TYPE_GRAPHQL),
                            ),
                            factory.create_application_term(
                                factory.create_builtin_term(CollectList),
                                allocator.create_list([
                                    factory.create_variable_term(1),
                                    get_struct_field(
                                        factory.create_variable_term(0),
                                        factory.create_string_term(
                                            allocator.create_static_string("query"),
                                        ),
                                        factory,
                                        allocator,
                                    ),
                                    get_optional_record_field(
                                        factory.create_variable_term(0),
                                        factory.create_string_term(
                                            allocator.create_static_string("operationName"),
                                        ),
                                        factory.create_nil_term(),
                                        factory,
                                        allocator,
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(ResolveDeep),
                                        allocator.create_unit_list(get_optional_record_field(
                                            factory.create_variable_term(0),
                                            factory.create_string_term(
                                                allocator.create_static_string("variables"),
                                            ),
                                            factory.create_nil_term(),
                                            factory,
                                            allocator,
                                        )),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(ResolveDeep),
                                        allocator.create_unit_list(get_optional_record_field(
                                            factory.create_variable_term(0),
                                            factory.create_string_term(
                                                allocator.create_static_string("extensions"),
                                            ),
                                            factory.create_nil_term(),
                                            factory,
                                            allocator,
                                        )),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(ResolveDeep),
                                        allocator.create_unit_list(get_optional_record_field(
                                            factory.create_variable_term(0),
                                            factory.create_string_term(
                                                allocator.create_static_string("headers"),
                                            ),
                                            factory.create_nil_term(),
                                            factory,
                                            allocator,
                                        )),
                                    ),
                                ]),
                            ),
                            get_optional_record_field(
                                factory.create_variable_term(0),
                                factory.create_string_term(allocator.create_static_string("token")),
                                factory.create_nil_term(),
                                factory,
                                allocator,
                            ),
                        ),
                    ),
                ),
            )],
            factory,
            allocator,
        ),
    )
}

fn create_default_export<T: Expression>(
    value: T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    create_record(
        once((
            factory.create_string_term(allocator.create_static_string("default")),
            value,
        )),
        factory,
        allocator,
    )
}

fn get_struct_field<T: Expression>(
    target: T,
    field: T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    factory.create_application_term(
        factory.create_builtin_term(Get),
        allocator.create_pair(target, field),
    )
}

fn get_optional_record_field<T: Expression>(
    target: T,
    field: T,
    fallback: T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: GraphQlLoaderBuiltin,
{
    factory.create_application_term(
        factory.create_builtin_term(If),
        allocator.create_triple(
            factory.create_application_term(
                factory.create_builtin_term(Contains),
                allocator.create_pair(target.clone(), field.clone()),
            ),
            factory.create_lambda_term(
                0,
                factory.create_application_term(
                    factory.create_builtin_term(Get),
                    allocator.create_pair(target, field),
                ),
            ),
            factory.create_lambda_term(0, fallback),
        ),
    )
}
