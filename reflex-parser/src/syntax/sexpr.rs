// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::marker::PhantomData;

use reflex::core::{Expression, ExpressionFactory, HeapAllocator, Rewritable};
use reflex_lisp::LispParserBuiltin;

use crate::SyntaxParser;

pub fn create_sexpr_parser<
    T: Expression,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> LispParser<T, TFactory, TAllocator>
where
    T::Builtin: LispParserBuiltin,
{
    let factory = factory.clone();
    let allocator = allocator.clone();
    LispParser::new(factory, allocator)
}

pub struct LispParser<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>> {
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    LispParser<T, TFactory, TAllocator>
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
    for LispParser<T, TFactory, TAllocator>
where
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T>,
    T::Builtin: LispParserBuiltin,
{
    fn parse(&self, input: &str) -> Result<T, String> {
        match reflex_lisp::parse(input, &self.factory, &self.allocator) {
            Ok(result) => Ok(result),
            Err(error) => Err(format!("{}", error)),
        }
    }
}
