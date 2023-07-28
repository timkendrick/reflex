// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::convert::{TryFrom, TryInto};

use reflex::core::Uuid;
use reflex::core::{
    Applicable, Arity, Builtin, EvaluationCache, Expression, ExpressionFactory, HeapAllocator, Uid,
};
use reflex_stdlib::stdlib;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum LispBuiltins {
    Stdlib(stdlib::Stdlib),
    Lisp(crate::stdlib::Stdlib),
}
impl From<stdlib::Stdlib> for LispBuiltins {
    fn from(builtin: stdlib::Stdlib) -> Self {
        LispBuiltins::Stdlib(builtin)
    }
}
impl From<crate::stdlib::Stdlib> for LispBuiltins {
    fn from(target: crate::stdlib::Stdlib) -> Self {
        LispBuiltins::Lisp(target)
    }
}
impl LispBuiltins {
    pub fn entries() -> impl Iterator<Item = Self> {
        stdlib::Stdlib::entries()
            .map(Self::Stdlib)
            .chain(crate::stdlib::Stdlib::entries().map(Self::Lisp))
    }
}
impl Uid for LispBuiltins {
    fn uid(&self) -> reflex::core::Uuid {
        match self {
            LispBuiltins::Stdlib(term) => term.uid(),
            LispBuiltins::Lisp(term) => term.uid(),
        }
    }
}
impl Builtin for LispBuiltins {
    fn arity(&self) -> Arity {
        match self {
            LispBuiltins::Stdlib(term) => term.arity(),
            LispBuiltins::Lisp(term) => term.arity(),
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
            LispBuiltins::Stdlib(term) => term.apply(args, factory, allocator, cache),
            LispBuiltins::Lisp(term) => term.apply(args, factory, allocator, cache),
        }
    }
    fn should_parallelize<T: Expression<Builtin = Self> + Applicable<T>>(
        &self,
        args: &[T],
    ) -> bool {
        match self {
            LispBuiltins::Stdlib(term) => term.should_parallelize(args),
            LispBuiltins::Lisp(term) => term.should_parallelize(args),
        }
    }
}
impl std::fmt::Display for LispBuiltins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdlib(target) => std::fmt::Display::fmt(target, f),
            Self::Lisp(target) => std::fmt::Display::fmt(target, f),
        }
    }
}
impl TryFrom<Uuid> for LispBuiltins {
    type Error = ();
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        TryInto::<stdlib::Stdlib>::try_into(value)
            .map(Self::Stdlib)
            .or_else(|_| TryInto::<crate::stdlib::Stdlib>::try_into(value).map(Self::Lisp))
    }
}

impl From<stdlib::Abs> for LispBuiltins {
    fn from(value: stdlib::Abs) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Add> for LispBuiltins {
    fn from(value: stdlib::Add) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::And> for LispBuiltins {
    fn from(value: stdlib::And) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Apply> for LispBuiltins {
    fn from(value: stdlib::Apply) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Ceil> for LispBuiltins {
    fn from(value: stdlib::Ceil) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Chain> for LispBuiltins {
    fn from(value: stdlib::Chain) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectConstructor> for LispBuiltins {
    fn from(value: stdlib::CollectConstructor) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectHashMap> for LispBuiltins {
    fn from(value: stdlib::CollectHashMap) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectHashSet> for LispBuiltins {
    fn from(value: stdlib::CollectHashSet) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectList> for LispBuiltins {
    fn from(value: stdlib::CollectList) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectRecord> for LispBuiltins {
    fn from(value: stdlib::CollectRecord) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectSignal> for LispBuiltins {
    fn from(value: stdlib::CollectSignal) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::CollectString> for LispBuiltins {
    fn from(value: stdlib::CollectString) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Contains> for LispBuiltins {
    fn from(value: stdlib::Contains) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Divide> for LispBuiltins {
    fn from(value: stdlib::Divide) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Effect> for LispBuiltins {
    fn from(value: stdlib::Effect) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::EndsWith> for LispBuiltins {
    fn from(value: stdlib::EndsWith) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Eq> for LispBuiltins {
    fn from(value: stdlib::Eq) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Equal> for LispBuiltins {
    fn from(value: stdlib::Equal) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Filter> for LispBuiltins {
    fn from(value: stdlib::Filter) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Flatten> for LispBuiltins {
    fn from(value: stdlib::Flatten) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Floor> for LispBuiltins {
    fn from(value: stdlib::Floor) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Get> for LispBuiltins {
    fn from(value: stdlib::Get) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Gt> for LispBuiltins {
    fn from(value: stdlib::Gt) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Gte> for LispBuiltins {
    fn from(value: stdlib::Gte) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Hash> for LispBuiltins {
    fn from(value: stdlib::Hash) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::If> for LispBuiltins {
    fn from(value: stdlib::If) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::IfError> for LispBuiltins {
    fn from(value: stdlib::IfError) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::IfPending> for LispBuiltins {
    fn from(value: stdlib::IfPending) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Insert> for LispBuiltins {
    fn from(value: stdlib::Insert) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Intersperse> for LispBuiltins {
    fn from(value: stdlib::Intersperse) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Keys> for LispBuiltins {
    fn from(value: stdlib::Keys) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Length> for LispBuiltins {
    fn from(value: stdlib::Length) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Lt> for LispBuiltins {
    fn from(value: stdlib::Lt) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Lte> for LispBuiltins {
    fn from(value: stdlib::Lte) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Map> for LispBuiltins {
    fn from(value: stdlib::Map) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Max> for LispBuiltins {
    fn from(value: stdlib::Max) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Merge> for LispBuiltins {
    fn from(value: stdlib::Merge) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Min> for LispBuiltins {
    fn from(value: stdlib::Min) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Multiply> for LispBuiltins {
    fn from(value: stdlib::Multiply) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Not> for LispBuiltins {
    fn from(value: stdlib::Not) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Or> for LispBuiltins {
    fn from(value: stdlib::Or) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Pow> for LispBuiltins {
    fn from(value: stdlib::Pow) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Push> for LispBuiltins {
    fn from(value: stdlib::Push) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::PushFront> for LispBuiltins {
    fn from(value: stdlib::PushFront) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Raise> for LispBuiltins {
    fn from(value: stdlib::Raise) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Fold> for LispBuiltins {
    fn from(value: stdlib::Fold) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Remainder> for LispBuiltins {
    fn from(value: stdlib::Remainder) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Replace> for LispBuiltins {
    fn from(value: stdlib::Replace) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveArgs> for LispBuiltins {
    fn from(value: stdlib::ResolveArgs) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveDeep> for LispBuiltins {
    fn from(value: stdlib::ResolveDeep) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveHashMap> for LispBuiltins {
    fn from(value: stdlib::ResolveHashMap) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveHashSet> for LispBuiltins {
    fn from(value: stdlib::ResolveHashSet) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveList> for LispBuiltins {
    fn from(value: stdlib::ResolveList) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::ResolveRecord> for LispBuiltins {
    fn from(value: stdlib::ResolveRecord) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Round> for LispBuiltins {
    fn from(value: stdlib::Round) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Sequence> for LispBuiltins {
    fn from(value: stdlib::Sequence) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Slice> for LispBuiltins {
    fn from(value: stdlib::Slice) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Split> for LispBuiltins {
    fn from(value: stdlib::Split) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::StartsWith> for LispBuiltins {
    fn from(value: stdlib::StartsWith) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Subtract> for LispBuiltins {
    fn from(value: stdlib::Subtract) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Unzip> for LispBuiltins {
    fn from(value: stdlib::Unzip) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Values> for LispBuiltins {
    fn from(value: stdlib::Values) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}
impl From<stdlib::Zip> for LispBuiltins {
    fn from(value: stdlib::Zip) -> Self {
        Self::from(stdlib::Stdlib::from(value))
    }
}

impl From<crate::stdlib::Car> for LispBuiltins {
    fn from(value: crate::stdlib::Car) -> Self {
        Self::from(crate::stdlib::Stdlib::from(value))
    }
}
impl From<crate::stdlib::Cdr> for LispBuiltins {
    fn from(value: crate::stdlib::Cdr) -> Self {
        Self::from(crate::stdlib::Stdlib::from(value))
    }
}
impl From<crate::stdlib::Cons> for LispBuiltins {
    fn from(value: crate::stdlib::Cons) -> Self {
        Self::from(crate::stdlib::Stdlib::from(value))
    }
}
