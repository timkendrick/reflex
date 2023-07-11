// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{marker::PhantomData, path::Path, str::FromStr};

use reflex::{
    core::{Builtin, Expression, ExpressionFactory, HeapAllocator, Reducible, Rewritable},
    env::inject_env_vars,
};
use reflex_graphql::imports::GraphQlImportsBuiltin;
use reflex_grpc::loader::GrpcLoaderBuiltin;
use reflex_handlers::imports::HandlerImportsBuiltin;
use reflex_js::{globals::JsGlobalsBuiltin, imports::JsImportsBuiltin, JsParserBuiltin};
use reflex_lisp::LispParserBuiltin;
use reflex_macros::blanket_trait;
use syntax::{
    js::{
        create_js_module_parser, create_js_script_parser, JavaScriptModuleParser,
        JavaScriptScriptParser,
    },
    json::{create_json_parser, JsonParser},
    sexpr::{create_sexpr_parser, LispParser},
};

pub mod syntax {
    pub mod js;
    pub mod json;
    pub mod sexpr;
}

blanket_trait!(
    pub trait ParserBuiltin:
        Builtin
        + JsParserBuiltin
        + JsGlobalsBuiltin
        + JsImportsBuiltin
        + HandlerImportsBuiltin
        + GraphQlImportsBuiltin
        + GrpcLoaderBuiltin
        + LispParserBuiltin
    {
    }
);

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Syntax {
    JavaScript,
    Json,
    Lisp,
}
impl FromStr for Syntax {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "javascript" | "js" => Ok(Self::JavaScript),
            "json" => Ok(Self::Json),
            "sexpr" | "lisp" => Ok(Self::Lisp),
            _ => Err(anyhow::anyhow!("Unknown syntax: {}", input)),
        }
    }
}

pub trait SyntaxParser<T: Expression> {
    fn parse(&self, input: &str) -> Result<T, String>;
}
impl<T, F> SyntaxParser<T> for F
where
    T: Expression,
    F: Fn(&str) -> Result<T, String>,
{
    fn parse(&self, input: &str) -> Result<T, String> {
        self(input)
    }
}

pub type DefaultModuleLoader<T> = Box<dyn Fn(&str, &Path) -> Option<Result<T, String>> + 'static>;

pub fn create_parser<
    T: Expression + 'static,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    syntax: Syntax,
    entry_path: Option<&Path>,
    module_loader: Option<impl Fn(&str, &Path) -> Option<Result<T, String>> + 'static>,
    env_vars: impl IntoIterator<Item = (String, String)>,
    factory: &TFactory,
    allocator: &TAllocator,
) -> impl SyntaxParser<T>
where
    T::Builtin: ParserBuiltin,
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    let parser = match (syntax, entry_path) {
        (Syntax::JavaScript, None) => {
            PolyglotSyntaxParser::JavaScriptScript(create_js_script_parser(factory, allocator))
        }
        (Syntax::JavaScript, Some(entry_path)) => PolyglotSyntaxParser::JavaScriptModule(
            create_js_module_parser(entry_path, module_loader, factory, allocator),
        ),
        (Syntax::Json, _) => PolyglotSyntaxParser::Json(create_json_parser(factory, allocator)),
        (Syntax::Lisp, _) => PolyglotSyntaxParser::Lisp(create_sexpr_parser(factory, allocator)),
    };
    GenericSyntaxParser::new(parser, env_vars, factory.clone(), allocator.clone())
}

struct GenericSyntaxParser<
    T: Expression,
    TInner: SyntaxParser<T>,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    parser: TInner,
    env_vars: Vec<(String, String)>,
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}
impl<
        T: Expression,
        TInner: SyntaxParser<T>,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > GenericSyntaxParser<T, TInner, TFactory, TAllocator>
{
    fn new(
        parser: TInner,
        env_vars: impl IntoIterator<Item = (String, String)>,
        factory: TFactory,
        allocator: TAllocator,
    ) -> Self {
        Self {
            parser,
            env_vars: env_vars.into_iter().collect(),
            factory,
            allocator,
            _expression: PhantomData,
        }
    }
}
impl<
        T: Expression,
        TInner: SyntaxParser<T>,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > SyntaxParser<T> for GenericSyntaxParser<T, TInner, TFactory, TAllocator>
where
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T>,
{
    fn parse(&self, input: &str) -> Result<T, String> {
        self.parser.parse(input).map(|expression| {
            inject_env_vars(
                expression,
                self.env_vars
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
                &self.factory,
                &self.allocator,
            )
        })
    }
}

enum PolyglotSyntaxParser<
    T: Expression,
    TLoader: Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    JavaScriptScript(JavaScriptScriptParser<T, TFactory, TAllocator>),
    JavaScriptModule(JavaScriptModuleParser<T, TLoader, TFactory, TAllocator>),
    Json(JsonParser<T, TFactory, TAllocator>),
    Lisp(LispParser<T, TFactory, TAllocator>),
}

impl<
        T: Expression,
        TLoader: Fn(&str, &Path) -> Option<Result<T, String>> + 'static,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
    > SyntaxParser<T> for PolyglotSyntaxParser<T, TLoader, TFactory, TAllocator>
where
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T>,
    T::Builtin: ParserBuiltin,
{
    fn parse(&self, input: &str) -> Result<T, String> {
        match self {
            Self::JavaScriptScript(inner) => inner.parse(input),
            Self::JavaScriptModule(inner) => inner.parse(input),
            Self::Json(inner) => inner.parse(input),
            Self::Lisp(inner) => inner.parse(input),
        }
    }
}
