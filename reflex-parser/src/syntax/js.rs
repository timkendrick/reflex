// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    iter::empty,
    path::{Path, PathBuf},
};

use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_graphql::imports::{graphql_imports, GraphQlImportsBuiltin};
use reflex_grpc::loader::{create_grpc_loader, GrpcLoaderBuiltin};
use reflex_handlers::{
    imports::{handler_imports, HandlerImportsBuiltin},
    loader::graphql_loader,
};
use reflex_js::{
    builtin_imports, compose_module_loaders, create_js_env, create_module_loader,
    globals::JsGlobalsBuiltin, imports::JsImportsBuiltin, static_module_loader, Env,
    JsParserBuiltin,
};

use crate::{syntax::json::json_loader, ParserBuiltin, SyntaxParser};

pub fn default_js_loaders<'a, T: Expression + 'static>(
    imports: impl IntoIterator<Item = (String, T)>,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> impl Fn(&str, &Path) -> Option<Result<T, String>> + 'static
where
    T::Builtin: JsParserBuiltin
        + JsGlobalsBuiltin
        + JsImportsBuiltin
        + HandlerImportsBuiltin
        + GraphQlImportsBuiltin
        + GrpcLoaderBuiltin,
{
    compose_module_loaders(
        static_module_loader(
            imports
                .into_iter()
                .chain(builtin_imports(factory, allocator))
                .chain(handler_imports(factory, allocator))
                .chain(graphql_imports(factory, allocator)),
        ),
        compose_module_loaders(
            json_loader(factory, allocator),
            compose_module_loaders(
                graphql_loader(factory, allocator),
                create_grpc_loader(factory, allocator),
            ),
        ),
    )
}

pub fn create_js_script_parser<
    T: Expression,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> JavaScriptScriptParser<T, TFactory, TAllocator>
where
    T::Builtin: JsParserBuiltin + JsGlobalsBuiltin,
{
    let env = create_js_env(factory, allocator);
    let factory = factory.clone();
    let allocator = allocator.clone();
    JavaScriptScriptParser::new(env, factory, allocator)
}

pub fn create_js_module_parser<
    T: Expression + 'static,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    path: &Path,
    module_loader: Option<impl Fn(&str, &Path) -> Option<Result<T, String>> + 'static>,
    factory: &TFactory,
    allocator: &TAllocator,
) -> JavaScriptModuleParser<
    T,
    impl Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
    TFactory,
    TAllocator,
>
where
    T::Builtin: ParserBuiltin,
{
    let env = create_js_env(factory, allocator);
    let loader = create_module_loader(
        env.clone(),
        Some({
            let default_loader = default_js_loaders(empty(), factory, allocator);
            move |input: &str, path: &Path| {
                if let Some(custom_loader) = module_loader.as_ref() {
                    custom_loader(input, path)
                } else {
                    default_loader(input, path)
                }
            }
        }),
        factory,
        allocator,
    );
    let factory = factory.clone();
    let allocator = allocator.clone();
    let path = path.to_owned();
    JavaScriptModuleParser::new(path, loader, env, factory, allocator)
}

pub struct JavaScriptScriptParser<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    env: Env<T>,
    factory: TFactory,
    allocator: TAllocator,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    JavaScriptScriptParser<T, TFactory, TAllocator>
{
    pub fn new(env: Env<T>, factory: TFactory, allocator: TAllocator) -> Self {
        Self {
            env,
            factory,
            allocator,
        }
    }
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>> SyntaxParser<T>
    for JavaScriptScriptParser<T, TFactory, TAllocator>
where
    T::Builtin: JsParserBuiltin,
{
    fn parse(&self, input: &str) -> Result<T, String> {
        reflex_js::parse(input, &self.env, &self.factory, &self.allocator)
    }
}

pub struct JavaScriptModuleParser<
    T: Expression,
    TLoader: Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    path: PathBuf,
    loader: TLoader,
    env: Env<T>,
    factory: TFactory,
    allocator: TAllocator,
}

impl<
        T: Expression,
        TLoader: Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > JavaScriptModuleParser<T, TLoader, TFactory, TAllocator>
{
    pub fn new(
        path: PathBuf,
        loader: TLoader,
        env: Env<T>,
        factory: TFactory,
        allocator: TAllocator,
    ) -> Self {
        Self {
            path,
            loader,
            env,
            factory,
            allocator,
        }
    }
}

impl<
        T: Expression,
        TLoader: Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > SyntaxParser<T> for JavaScriptModuleParser<T, TLoader, TFactory, TAllocator>
where
    T::Builtin: JsParserBuiltin,
{
    fn parse(&self, input: &str) -> Result<T, String> {
        reflex_js::parse_module(
            input,
            &self.env,
            &self.path,
            &self.loader,
            &self.factory,
            &self.allocator,
        )
    }
}
