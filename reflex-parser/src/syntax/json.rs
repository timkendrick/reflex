// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{iter::once, marker::PhantomData, path::Path};

use derivative::Derivative;
use reflex::core::{Expression, ExpressionFactory, HeapAllocator, ModuleLoader};

use crate::SyntaxParser;

pub fn create_json_parser<
    T: Expression,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> JsonParser<T, TFactory, TAllocator> {
    let factory = factory.clone();
    let allocator = allocator.clone();
    JsonParser::new(factory, allocator)
}

pub fn json_loader<
    T: Expression,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> JsonModuleLoader<T, TFactory, TAllocator> {
    JsonModuleLoader::new(factory.clone(), allocator.clone())
}

#[derive(Derivative)]
#[derivative(Clone(bound = "TFactory: Clone, TAllocator: Clone"))]
pub struct JsonModuleLoader<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    JsonModuleLoader<T, TFactory, TAllocator>
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
    for JsonModuleLoader<T, TFactory, TAllocator>
{
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        if !import_path.ends_with(".json") {
            return None;
        }
        let file_path = current_path
            .parent()
            .map(|parent| parent.join(import_path))
            .unwrap_or_else(|| Path::new(import_path).to_path_buf());
        Some(
            load_json_module(&file_path, &self.factory, &self.allocator)
                .map(|value| create_default_module_export(value, &self.factory, &self.allocator)),
        )
    }
}

fn load_json_module<T: Expression>(
    path: &Path,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> Result<T, String> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => Ok(source),
        Err(error) => Err(format!("{}", error)),
    }?;
    reflex_json::parse(&source, factory, allocator)
}

fn create_default_module_export<T: Expression>(
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

pub struct JsonParser<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>> {
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    JsonParser<T, TFactory, TAllocator>
{
    pub fn new(factory: TFactory, allocator: TAllocator) -> Self {
        Self {
            factory,
            allocator,
            _expression: PhantomData,
        }
    }
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>> SyntaxParser<T>
    for JsonParser<T, TFactory, TAllocator>
{
    fn parse(&self, input: &str) -> Result<T, String> {
        reflex_json::parse(input, &self.factory, &self.allocator)
    }
}
