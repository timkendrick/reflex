// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::path::Path;

use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_graphql::imports::{graphql_imports, GraphQlImportsBuiltin};
use reflex_grpc::loader::create_grpc_loader;
use reflex_handlers::{
    imports::{handler_imports, HandlerImportsBuiltin},
    loader::graphql_loader,
};
use reflex_js::{builtin_imports, imports::JsImportsBuiltin, static_module_loader};

pub fn create_loader<'a, T: Expression + 'static>(
    custom_imports: impl IntoIterator<Item = (String, T)>,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> impl Fn(&str, &Path) -> Option<Result<T, String>>
where
    T::Builtin: JsImportsBuiltin + GraphQlImportsBuiltin + HandlerImportsBuiltin,
{
    let static_imports_loader = static_module_loader(
        builtin_imports(factory, allocator)
            .into_iter()
            .chain(handler_imports(factory, allocator))
            .chain(graphql_imports(factory, allocator))
            .map(|(key, value)| (String::from(key), value))
            .chain(custom_imports),
    );
    let graphql_loader = graphql_loader(factory, allocator);
    let grpc_loader = create_grpc_loader(factory, allocator);
    move |import_path, module_path| {
        None.or_else(|| graphql_loader(import_path, module_path))
            .or_else(|| grpc_loader(import_path, module_path))
            .or_else(|| static_imports_loader(import_path, module_path))
    }
}
