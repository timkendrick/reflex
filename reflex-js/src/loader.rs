// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    fs::{self},
    iter::once,
    marker::PhantomData,
    path::Path,
};

use derivative::Derivative;
use reflex::{
    core::{Expression, ExpressionFactory, HeapAllocator, ModuleLoader},
    loader::{
        get_module_filesystem_path, ChainedModuleLoader, ErrorFallbackModuleLoader,
        RecursiveModuleLoader, StaticModuleLoader,
    },
};

use crate::{
    globals::{builtin_globals, JsGlobalsBuiltin},
    parse_module,
    parser::JsParserBuiltin,
    Env,
};

pub fn create_module_loader<T: Expression + 'static>(
    env: Env<T>,
    custom_loader: impl ModuleLoader<Output = T> + 'static,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> RecursiveModuleLoader<T>
where
    T::Builtin: JsParserBuiltin,
{
    let factory = factory.clone();
    let allocator = allocator.clone();
    RecursiveModuleLoader::new(move |loader| {
        ChainedModuleLoader::new(
            JavaScriptModuleLoader::new(env, loader, factory, allocator),
            ChainedModuleLoader::new(custom_loader, ErrorFallbackModuleLoader::default()),
        )
    })
}

pub fn compose_module_loaders<
    T: Expression,
    T1: ModuleLoader<Output = T>,
    T2: ModuleLoader<Output = T>,
>(
    left: T1,
    right: T2,
) -> ChainedModuleLoader<T, T1, T2> {
    ChainedModuleLoader::new(left, right)
}

pub fn static_module_loader<T: Expression + 'static>(
    modules: impl IntoIterator<Item = (String, T)>,
) -> StaticModuleLoader<T> {
    StaticModuleLoader::new(modules)
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = "TLoader: Clone, TFactory: Clone, TAllocator: Clone"),
    Debug(
        bound = "TLoader: std::fmt::Debug, TFactory: std::fmt::Debug, TAllocator: std::fmt::Debug"
    )
)]
struct JavaScriptModuleLoader<
    T: Expression,
    TLoader: ModuleLoader<Output = T>,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    env: Env<T>,
    loader: TLoader,
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}

impl<
        T: Expression,
        TLoader: ModuleLoader<Output = T>,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > JavaScriptModuleLoader<T, TLoader, TFactory, TAllocator>
{
    pub fn new(env: Env<T>, loader: TLoader, factory: TFactory, allocator: TAllocator) -> Self {
        Self {
            env,
            factory,
            allocator,
            loader,
            _expression: PhantomData,
        }
    }
}

impl<
        T: Expression,
        TLoader: ModuleLoader<Output = T>,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > ModuleLoader for JavaScriptModuleLoader<T, TLoader, TFactory, TAllocator>
where
    T::Builtin: JsParserBuiltin,
{
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        if !import_path.ends_with(".js") {
            return None;
        }
        let target_path = get_module_filesystem_path(import_path, current_path);
        Some(match fs::read_to_string(&target_path) {
            Err(err) => Err(format!("{}", err)),
            Ok(source) => parse_module(
                &source,
                &self.env,
                &target_path,
                &self.loader,
                &self.factory,
                &self.allocator,
            )
            .map(|result| create_default_module_export(result, &self.factory, &self.allocator)),
        })
    }
}

pub fn create_js_env<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> Env<T>
where
    T::Builtin: JsGlobalsBuiltin,
{
    Env::new().with_globals(builtin_globals(factory, allocator))
}

pub fn create_default_module_export<T: Expression>(
    value: T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_record_term(
        allocator.create_struct_prototype(allocator.create_list(once(
            factory.create_string_term(allocator.create_string(String::from("default"))),
        ))),
        allocator.create_unit_list(value),
    )
}
