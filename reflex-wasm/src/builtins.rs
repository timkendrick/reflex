// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::convert::{TryFrom, TryInto};

use reflex::core::{
    Applicable, Arity, Builtin, EvaluationCache, Expression, ExpressionFactory, HeapAllocator, Uid,
    Uuid,
};
use reflex_stdlib::stdlib;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum WasmCompilerBuiltins {
    Stdlib(stdlib::Stdlib),
    Json(reflex_json::stdlib::Stdlib),
    Js(reflex_js::stdlib::Stdlib),
    Handlers(reflex_handlers::stdlib::Stdlib),
    GraphQl(reflex_graphql::stdlib::Stdlib),
}
impl From<stdlib::Stdlib> for WasmCompilerBuiltins {
    fn from(target: stdlib::Stdlib) -> Self {
        WasmCompilerBuiltins::Stdlib(target)
    }
}
impl From<reflex_json::stdlib::Stdlib> for WasmCompilerBuiltins {
    fn from(target: reflex_json::stdlib::Stdlib) -> Self {
        WasmCompilerBuiltins::Json(target)
    }
}
impl From<reflex_js::stdlib::Stdlib> for WasmCompilerBuiltins {
    fn from(target: reflex_js::stdlib::Stdlib) -> Self {
        WasmCompilerBuiltins::Js(target)
    }
}
impl From<reflex_handlers::stdlib::Stdlib> for WasmCompilerBuiltins {
    fn from(target: reflex_handlers::stdlib::Stdlib) -> Self {
        WasmCompilerBuiltins::Handlers(target)
    }
}
impl From<reflex_graphql::stdlib::Stdlib> for WasmCompilerBuiltins {
    fn from(target: reflex_graphql::stdlib::Stdlib) -> Self {
        WasmCompilerBuiltins::GraphQl(target)
    }
}
impl Uid for WasmCompilerBuiltins {
    fn uid(&self) -> reflex::core::Uuid {
        match self {
            WasmCompilerBuiltins::Stdlib(term) => term.uid(),
            WasmCompilerBuiltins::Json(term) => term.uid(),
            WasmCompilerBuiltins::Js(term) => term.uid(),
            WasmCompilerBuiltins::Handlers(term) => term.uid(),
            WasmCompilerBuiltins::GraphQl(term) => term.uid(),
        }
    }
}
impl TryFrom<Uuid> for WasmCompilerBuiltins {
    type Error = ();
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        TryInto::<stdlib::Stdlib>::try_into(value)
            .map(Self::Stdlib)
            .or_else(|_| TryInto::<reflex_json::stdlib::Stdlib>::try_into(value).map(Self::Json))
            .or_else(|_| TryInto::<reflex_js::stdlib::Stdlib>::try_into(value).map(Self::Js))
            .or_else(|_| {
                TryInto::<reflex_handlers::stdlib::Stdlib>::try_into(value).map(Self::Handlers)
            })
            .or_else(|_| {
                TryInto::<reflex_graphql::stdlib::Stdlib>::try_into(value).map(Self::GraphQl)
            })
    }
}
impl Builtin for WasmCompilerBuiltins {
    fn arity(&self) -> Arity {
        match self {
            WasmCompilerBuiltins::Stdlib(term) => term.arity(),
            WasmCompilerBuiltins::Json(term) => term.arity(),
            WasmCompilerBuiltins::Js(term) => term.arity(),
            WasmCompilerBuiltins::Handlers(term) => term.arity(),
            WasmCompilerBuiltins::GraphQl(term) => term.arity(),
        }
    }
    fn apply<T: Expression<Builtin = Self> + Applicable<T>>(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        match self {
            WasmCompilerBuiltins::Stdlib(term) => term.apply(args, factory, allocator, cache),
            WasmCompilerBuiltins::Json(term) => term.apply(args, factory, allocator, cache),
            WasmCompilerBuiltins::Js(term) => term.apply(args, factory, allocator, cache),
            WasmCompilerBuiltins::Handlers(term) => term.apply(args, factory, allocator, cache),
            WasmCompilerBuiltins::GraphQl(term) => term.apply(args, factory, allocator, cache),
        }
    }
    fn should_parallelize<T: Expression<Builtin = Self> + Applicable<T>>(
        &self,
        args: &[T],
    ) -> bool {
        match self {
            WasmCompilerBuiltins::Stdlib(term) => term.should_parallelize(args),
            WasmCompilerBuiltins::Json(term) => term.should_parallelize(args),
            WasmCompilerBuiltins::Js(term) => term.should_parallelize(args),
            WasmCompilerBuiltins::Handlers(term) => term.should_parallelize(args),
            WasmCompilerBuiltins::GraphQl(term) => term.should_parallelize(args),
        }
    }
}
impl std::fmt::Display for WasmCompilerBuiltins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdlib(target) => std::fmt::Display::fmt(target, f),
            Self::Json(target) => std::fmt::Display::fmt(target, f),
            Self::Js(target) => std::fmt::Display::fmt(target, f),
            Self::Handlers(target) => std::fmt::Display::fmt(target, f),
            Self::GraphQl(target) => std::fmt::Display::fmt(target, f),
        }
    }
}

impl From<WasmCompilerBuiltins> for crate::stdlib::Stdlib {
    fn from(value: WasmCompilerBuiltins) -> Self {
        match value {
            WasmCompilerBuiltins::Stdlib(inner) => inner.into(),
            WasmCompilerBuiltins::Json(inner) => inner.into(),
            WasmCompilerBuiltins::Js(inner) => inner.into(),
            WasmCompilerBuiltins::Handlers(inner) => inner.into(),
            WasmCompilerBuiltins::GraphQl(inner) => inner.into(),
        }
    }
}

impl From<stdlib::Abs> for WasmCompilerBuiltins {
    fn from(value: stdlib::Abs) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Add> for WasmCompilerBuiltins {
    fn from(value: stdlib::Add) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::And> for WasmCompilerBuiltins {
    fn from(value: stdlib::And) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Apply> for WasmCompilerBuiltins {
    fn from(value: stdlib::Apply) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Car> for WasmCompilerBuiltins {
    fn from(value: stdlib::Car) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Cdr> for WasmCompilerBuiltins {
    fn from(value: stdlib::Cdr) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Ceil> for WasmCompilerBuiltins {
    fn from(value: stdlib::Ceil) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Chain> for WasmCompilerBuiltins {
    fn from(value: stdlib::Chain) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectConstructor> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectConstructor) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectHashMap> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectHashMap) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectHashSet> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectHashSet) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectList> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectList) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectRecord> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectRecord) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectSignal> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectSignal) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectString> for WasmCompilerBuiltins {
    fn from(value: stdlib::CollectString) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Cons> for WasmCompilerBuiltins {
    fn from(value: stdlib::Cons) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Contains> for WasmCompilerBuiltins {
    fn from(value: stdlib::Contains) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Divide> for WasmCompilerBuiltins {
    fn from(value: stdlib::Divide) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Effect> for WasmCompilerBuiltins {
    fn from(value: stdlib::Effect) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::EndsWith> for WasmCompilerBuiltins {
    fn from(value: stdlib::EndsWith) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Eq> for WasmCompilerBuiltins {
    fn from(value: stdlib::Eq) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Equal> for WasmCompilerBuiltins {
    fn from(value: stdlib::Equal) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Filter> for WasmCompilerBuiltins {
    fn from(value: stdlib::Filter) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Flatten> for WasmCompilerBuiltins {
    fn from(value: stdlib::Flatten) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Floor> for WasmCompilerBuiltins {
    fn from(value: stdlib::Floor) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Get> for WasmCompilerBuiltins {
    fn from(value: stdlib::Get) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Gt> for WasmCompilerBuiltins {
    fn from(value: stdlib::Gt) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Gte> for WasmCompilerBuiltins {
    fn from(value: stdlib::Gte) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Hash> for WasmCompilerBuiltins {
    fn from(value: stdlib::Hash) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::If> for WasmCompilerBuiltins {
    fn from(value: stdlib::If) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::IfError> for WasmCompilerBuiltins {
    fn from(value: stdlib::IfError) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::IfPending> for WasmCompilerBuiltins {
    fn from(value: stdlib::IfPending) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Insert> for WasmCompilerBuiltins {
    fn from(value: stdlib::Insert) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Intersperse> for WasmCompilerBuiltins {
    fn from(value: stdlib::Intersperse) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Keys> for WasmCompilerBuiltins {
    fn from(value: stdlib::Keys) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Length> for WasmCompilerBuiltins {
    fn from(value: stdlib::Length) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Lt> for WasmCompilerBuiltins {
    fn from(value: stdlib::Lt) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Lte> for WasmCompilerBuiltins {
    fn from(value: stdlib::Lte) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Map> for WasmCompilerBuiltins {
    fn from(value: stdlib::Map) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Max> for WasmCompilerBuiltins {
    fn from(value: stdlib::Max) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Merge> for WasmCompilerBuiltins {
    fn from(value: stdlib::Merge) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Min> for WasmCompilerBuiltins {
    fn from(value: stdlib::Min) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Multiply> for WasmCompilerBuiltins {
    fn from(value: stdlib::Multiply) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Not> for WasmCompilerBuiltins {
    fn from(value: stdlib::Not) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Or> for WasmCompilerBuiltins {
    fn from(value: stdlib::Or) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Pow> for WasmCompilerBuiltins {
    fn from(value: stdlib::Pow) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Push> for WasmCompilerBuiltins {
    fn from(value: stdlib::Push) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::PushFront> for WasmCompilerBuiltins {
    fn from(value: stdlib::PushFront) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Raise> for WasmCompilerBuiltins {
    fn from(value: stdlib::Raise) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Reduce> for WasmCompilerBuiltins {
    fn from(value: stdlib::Reduce) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Remainder> for WasmCompilerBuiltins {
    fn from(value: stdlib::Remainder) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Replace> for WasmCompilerBuiltins {
    fn from(value: stdlib::Replace) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveArgs> for WasmCompilerBuiltins {
    fn from(value: stdlib::ResolveArgs) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveDeep> for WasmCompilerBuiltins {
    fn from(value: stdlib::ResolveDeep) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveHashMap> for WasmCompilerBuiltins {
    fn from(value: stdlib::ResolveHashMap) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveHashSet> for WasmCompilerBuiltins {
    fn from(value: stdlib::ResolveHashSet) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveRecord> for WasmCompilerBuiltins {
    fn from(value: stdlib::ResolveRecord) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveList> for WasmCompilerBuiltins {
    fn from(value: stdlib::ResolveList) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Round> for WasmCompilerBuiltins {
    fn from(value: stdlib::Round) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Sequence> for WasmCompilerBuiltins {
    fn from(value: stdlib::Sequence) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Slice> for WasmCompilerBuiltins {
    fn from(value: stdlib::Slice) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Split> for WasmCompilerBuiltins {
    fn from(value: stdlib::Split) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::StartsWith> for WasmCompilerBuiltins {
    fn from(value: stdlib::StartsWith) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Subtract> for WasmCompilerBuiltins {
    fn from(value: stdlib::Subtract) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Unzip> for WasmCompilerBuiltins {
    fn from(value: stdlib::Unzip) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Values> for WasmCompilerBuiltins {
    fn from(value: stdlib::Values) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Zip> for WasmCompilerBuiltins {
    fn from(value: stdlib::Zip) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}

impl From<reflex_json::stdlib::JsonDeserialize> for WasmCompilerBuiltins {
    fn from(value: reflex_json::stdlib::JsonDeserialize) -> Self {
        Self::from(reflex_json::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_json::stdlib::JsonSerialize> for WasmCompilerBuiltins {
    fn from(value: reflex_json::stdlib::JsonSerialize) -> Self {
        Self::from(reflex_json::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_js::stdlib::Accessor> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::Accessor) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::Construct> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::Construct) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::EncodeUriComponent> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::EncodeUriComponent) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::FormatErrorMessage> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::FormatErrorMessage) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::IsFinite> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::IsFinite) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::Log> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::Log) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::LogArgs> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::LogArgs) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ParseDate> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::ParseDate) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ParseFloat> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::ParseFloat) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ParseInt> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::ParseInt) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::Throw> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::Throw) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ToString> for WasmCompilerBuiltins {
    fn from(value: reflex_js::stdlib::ToString) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_handlers::stdlib::ResolveLoaderResults> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::ResolveLoaderResults) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::Scan> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::Scan) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::ToRequest> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::ToRequest) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::GetVariable> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::GetVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::SetVariable> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::SetVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::IncrementVariable> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::IncrementVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::DecrementVariable> for WasmCompilerBuiltins {
    fn from(value: reflex_handlers::stdlib::DecrementVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_graphql::stdlib::CollectQueryListItems> for WasmCompilerBuiltins {
    fn from(value: reflex_graphql::stdlib::CollectQueryListItems) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_graphql::stdlib::DynamicQueryBranch> for WasmCompilerBuiltins {
    fn from(value: reflex_graphql::stdlib::DynamicQueryBranch) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_graphql::stdlib::FlattenDeep> for WasmCompilerBuiltins {
    fn from(value: reflex_graphql::stdlib::FlattenDeep) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_graphql::stdlib::GraphQlResolver> for WasmCompilerBuiltins {
    fn from(value: reflex_graphql::stdlib::GraphQlResolver) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
