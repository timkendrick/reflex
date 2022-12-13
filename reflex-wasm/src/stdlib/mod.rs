// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    Applicable, Arity, Builtin, EvaluationCache, Expression, ExpressionFactory, HeapAllocator, Uid,
    Uuid,
};
use reflex_macros::Matcher;
use strum_macros::EnumDiscriminants;

pub mod abs;
pub mod add;
pub mod and;
pub mod apply;
pub mod car;
pub mod cdr;
pub mod ceil;
pub mod chain;
pub mod collect_hashmap;
pub mod collect_hashset;
pub mod collect_list;
pub mod collect_string;
pub mod collect_tree;
pub mod cons;
pub mod divide;
pub mod effect;
pub mod ends_with;
pub mod eq;
pub mod equal;
pub mod floor;
pub mod fold;
pub mod get;
pub mod graphql;
pub mod gt;
pub mod gte;
pub mod handlers;
pub mod has;
pub mod hash;
pub mod identity;
pub mod r#if;
pub mod if_error;
pub mod if_pending;
pub mod iterate;
pub mod js;
pub mod json;
pub mod keys;
pub mod length;
pub mod lt;
pub mod lte;
pub mod max;
pub mod merge;
pub mod min;
pub mod multiply;
pub mod not;
pub mod or;
pub mod pow;
pub mod push;
pub mod push_front;
pub mod remainder;
pub mod replace;
pub mod resolve_args;
pub mod resolve_deep;
pub mod resolve_shallow;
pub mod round;
pub mod sequence;
pub mod server;
pub mod set;
pub mod skip;
pub mod slice;
pub mod split;
pub mod starts_with;
pub mod subtract;
pub mod take;
pub mod values;
pub mod zip;

pub use abs::*;
pub use add::*;
pub use and::*;
pub use apply::*;
pub use car::*;
pub use cdr::*;
pub use ceil::*;
pub use chain::*;
pub use collect_hashmap::*;
pub use collect_hashset::*;
pub use collect_list::*;
pub use collect_string::*;
pub use collect_tree::*;
pub use cons::*;
pub use divide::*;
pub use effect::*;
pub use ends_with::*;
pub use eq::*;
pub use equal::*;
pub use floor::*;
pub use fold::*;
pub use format_error_message::*;
pub use get::*;
pub use graphql::*;
pub use gt::*;
pub use gte::*;
pub use handlers::*;
pub use has::*;
pub use hash::*;
pub use identity::*;
pub use if_error::*;
pub use if_pending::*;
pub use iterate::*;
pub use js::*;
pub use json::*;
pub use keys::*;
pub use length::*;
pub use lt::*;
pub use lte::*;
pub use max::*;
pub use merge::*;
pub use min::*;
pub use multiply::*;
pub use not::*;
pub use or::*;
pub use pow::*;
pub use push::*;
pub use push_front::*;
pub use r#if::*;
pub use remainder::*;
pub use replace::*;
pub use resolve_args::*;
pub use resolve_deep::*;
pub use resolve_shallow::*;
pub use round::*;
pub use sequence::*;
pub use server::*;
pub use set::*;
pub use skip::*;
pub use slice::*;
pub use split::*;
pub use starts_with::*;
pub use subtract::*;
pub use take::*;
pub use values::*;
pub use zip::*;

#[derive(Matcher, PartialEq, Eq, Clone, Copy, Debug, EnumDiscriminants)]
pub enum Stdlib {
    Abs(Abs),
    Accessor(Accessor),
    Add(Add),
    And(And),
    Apply(Apply),
    Car(Car),
    Cdr(Cdr),
    Ceil(Ceil),
    Chain(Chain),
    CollectHashmap(CollectHashmap),
    CollectHashset(CollectHashset),
    CollectList(CollectList),
    CollectString(CollectString),
    CollectTree(CollectTree),
    Cons(Cons),
    Construct(Construct),
    Debug(Debug),
    DecrementVariable(DecrementVariable),
    Divide(Divide),
    Effect(Effect),
    EndsWith(EndsWith),
    Eq(Eq),
    Equal(Equal),
    Floor(Floor),
    Fold(Fold),
    FormatErrorMessage(FormatErrorMessage),
    Get(Get),
    GetVariable(GetVariable),
    GraphQlResolver(GraphQlResolver),
    Gt(Gt),
    Gte(Gte),
    Has(Has),
    Hash(Hash),
    Identity(Identity),
    If(If),
    IfError(IfError),
    IfPending(IfPending),
    IncrementVariable(IncrementVariable),
    IsFinite(IsFinite),
    Iterate(Iterate),
    Keys(Keys),
    Length(Length),
    Log(Log),
    Lt(Lt),
    Lte(Lte),
    Max(Max),
    Merge(Merge),
    Min(Min),
    Multiply(Multiply),
    Not(Not),
    Or(Or),
    ParseDate(ParseDate),
    ParseFloat(ParseFloat),
    ParseInt(ParseInt),
    ParseJson(ParseJson),
    Pow(Pow),
    Push(Push),
    PushFront(PushFront),
    Remainder(Remainder),
    Replace(Replace),
    ResolveArgs(ResolveArgs),
    ResolveDeep(ResolveDeep),
    ResolveQueryBranch(ResolveQueryBranch),
    ResolveQueryLeaf(ResolveQueryLeaf),
    ResolveShallow(ResolveShallow),
    Round(Round),
    Scan(Scan),
    Sequence(Sequence),
    Set(Set),
    SetVariable(SetVariable),
    Skip(Skip),
    Slice(Slice),
    Split(Split),
    StartsWith(StartsWith),
    StringifyJson(StringifyJson),
    Subtract(Subtract),
    Take(Take),
    Throw(Throw),
    ToRequest(ToRequest),
    ToString(ToString),
    Urlencode(Urlencode),
    Values(Values),
    Zip(Zip),
}

impl std::hash::Hash for Stdlib {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl From<Stdlib> for u32 {
    fn from(value: Stdlib) -> Self {
        match value {
            Stdlib::Abs(_) => StdlibDiscriminants::Abs as u32,
            Stdlib::Accessor(_) => StdlibDiscriminants::Accessor as u32,
            Stdlib::Add(_) => StdlibDiscriminants::Add as u32,
            Stdlib::And(_) => StdlibDiscriminants::And as u32,
            Stdlib::Apply(_) => StdlibDiscriminants::Apply as u32,
            Stdlib::Car(_) => StdlibDiscriminants::Car as u32,
            Stdlib::Cdr(_) => StdlibDiscriminants::Cdr as u32,
            Stdlib::Ceil(_) => StdlibDiscriminants::Ceil as u32,
            Stdlib::Chain(_) => StdlibDiscriminants::Chain as u32,
            Stdlib::CollectHashmap(_) => StdlibDiscriminants::CollectHashmap as u32,
            Stdlib::CollectHashset(_) => StdlibDiscriminants::CollectHashset as u32,
            Stdlib::CollectList(_) => StdlibDiscriminants::CollectList as u32,
            Stdlib::CollectString(_) => StdlibDiscriminants::CollectString as u32,
            Stdlib::CollectTree(_) => StdlibDiscriminants::CollectTree as u32,
            Stdlib::Cons(_) => StdlibDiscriminants::Cons as u32,
            Stdlib::Construct(_) => StdlibDiscriminants::Construct as u32,
            Stdlib::Debug(_) => StdlibDiscriminants::Debug as u32,
            Stdlib::DecrementVariable(_) => StdlibDiscriminants::DecrementVariable as u32,
            Stdlib::Divide(_) => StdlibDiscriminants::Divide as u32,
            Stdlib::Effect(_) => StdlibDiscriminants::Effect as u32,
            Stdlib::EndsWith(_) => StdlibDiscriminants::EndsWith as u32,
            Stdlib::Eq(_) => StdlibDiscriminants::Eq as u32,
            Stdlib::Equal(_) => StdlibDiscriminants::Equal as u32,
            Stdlib::Floor(_) => StdlibDiscriminants::Floor as u32,
            Stdlib::Fold(_) => StdlibDiscriminants::Fold as u32,
            Stdlib::FormatErrorMessage(_) => StdlibDiscriminants::FormatErrorMessage as u32,
            Stdlib::Get(_) => StdlibDiscriminants::Get as u32,
            Stdlib::GetVariable(_) => StdlibDiscriminants::GetVariable as u32,
            Stdlib::GraphQlResolver(_) => StdlibDiscriminants::GraphQlResolver as u32,
            Stdlib::Gt(_) => StdlibDiscriminants::Gt as u32,
            Stdlib::Gte(_) => StdlibDiscriminants::Gte as u32,
            Stdlib::Has(_) => StdlibDiscriminants::Has as u32,
            Stdlib::Hash(_) => StdlibDiscriminants::Hash as u32,
            Stdlib::Identity(_) => StdlibDiscriminants::Identity as u32,
            Stdlib::If(_) => StdlibDiscriminants::If as u32,
            Stdlib::IfError(_) => StdlibDiscriminants::IfError as u32,
            Stdlib::IfPending(_) => StdlibDiscriminants::IfPending as u32,
            Stdlib::IncrementVariable(_) => StdlibDiscriminants::IncrementVariable as u32,
            Stdlib::IsFinite(_) => StdlibDiscriminants::IsFinite as u32,
            Stdlib::Iterate(_) => StdlibDiscriminants::Iterate as u32,
            Stdlib::Keys(_) => StdlibDiscriminants::Keys as u32,
            Stdlib::Length(_) => StdlibDiscriminants::Length as u32,
            Stdlib::Log(_) => StdlibDiscriminants::Log as u32,
            Stdlib::Lt(_) => StdlibDiscriminants::Lt as u32,
            Stdlib::Lte(_) => StdlibDiscriminants::Lte as u32,
            Stdlib::Max(_) => StdlibDiscriminants::Max as u32,
            Stdlib::Merge(_) => StdlibDiscriminants::Merge as u32,
            Stdlib::Min(_) => StdlibDiscriminants::Min as u32,
            Stdlib::Multiply(_) => StdlibDiscriminants::Multiply as u32,
            Stdlib::Not(_) => StdlibDiscriminants::Not as u32,
            Stdlib::Or(_) => StdlibDiscriminants::Or as u32,
            Stdlib::ParseDate(_) => StdlibDiscriminants::ParseDate as u32,
            Stdlib::ParseFloat(_) => StdlibDiscriminants::ParseFloat as u32,
            Stdlib::ParseInt(_) => StdlibDiscriminants::ParseInt as u32,
            Stdlib::ParseJson(_) => StdlibDiscriminants::ParseJson as u32,
            Stdlib::Pow(_) => StdlibDiscriminants::Pow as u32,
            Stdlib::Push(_) => StdlibDiscriminants::Push as u32,
            Stdlib::PushFront(_) => StdlibDiscriminants::PushFront as u32,
            Stdlib::Remainder(_) => StdlibDiscriminants::Remainder as u32,
            Stdlib::Replace(_) => StdlibDiscriminants::Replace as u32,
            Stdlib::ResolveArgs(_) => StdlibDiscriminants::ResolveArgs as u32,
            Stdlib::ResolveDeep(_) => StdlibDiscriminants::ResolveDeep as u32,
            Stdlib::ResolveQueryBranch(_) => StdlibDiscriminants::ResolveQueryBranch as u32,
            Stdlib::ResolveQueryLeaf(_) => StdlibDiscriminants::ResolveQueryLeaf as u32,
            Stdlib::ResolveShallow(_) => StdlibDiscriminants::ResolveShallow as u32,
            Stdlib::Round(_) => StdlibDiscriminants::Round as u32,
            Stdlib::Scan(_) => StdlibDiscriminants::Scan as u32,
            Stdlib::Sequence(_) => StdlibDiscriminants::Sequence as u32,
            Stdlib::Set(_) => StdlibDiscriminants::Set as u32,
            Stdlib::SetVariable(_) => StdlibDiscriminants::SetVariable as u32,
            Stdlib::Skip(_) => StdlibDiscriminants::Skip as u32,
            Stdlib::Slice(_) => StdlibDiscriminants::Slice as u32,
            Stdlib::Split(_) => StdlibDiscriminants::Split as u32,
            Stdlib::StartsWith(_) => StdlibDiscriminants::StartsWith as u32,
            Stdlib::StringifyJson(_) => StdlibDiscriminants::StringifyJson as u32,
            Stdlib::Subtract(_) => StdlibDiscriminants::Subtract as u32,
            Stdlib::Take(_) => StdlibDiscriminants::Take as u32,
            Stdlib::Throw(_) => StdlibDiscriminants::Throw as u32,
            Stdlib::ToRequest(_) => StdlibDiscriminants::ToRequest as u32,
            Stdlib::ToString(_) => StdlibDiscriminants::ToString as u32,
            Stdlib::Urlencode(_) => StdlibDiscriminants::Urlencode as u32,
            Stdlib::Values(_) => StdlibDiscriminants::Values as u32,
            Stdlib::Zip(_) => StdlibDiscriminants::Zip as u32,
        }
    }
}
impl TryFrom<u32> for Stdlib {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            value if value == StdlibDiscriminants::Abs as u32 => Ok(Self::Abs(Abs)),
            value if value == StdlibDiscriminants::Accessor as u32 => Ok(Self::Accessor(Accessor)),
            value if value == StdlibDiscriminants::Add as u32 => Ok(Self::Add(Add)),
            value if value == StdlibDiscriminants::And as u32 => Ok(Self::And(And)),
            value if value == StdlibDiscriminants::Apply as u32 => Ok(Self::Apply(Apply)),
            value if value == StdlibDiscriminants::Car as u32 => Ok(Self::Car(Car)),
            value if value == StdlibDiscriminants::Cdr as u32 => Ok(Self::Cdr(Cdr)),
            value if value == StdlibDiscriminants::Ceil as u32 => Ok(Self::Ceil(Ceil)),
            value if value == StdlibDiscriminants::Chain as u32 => Ok(Self::Chain(Chain)),
            value if value == StdlibDiscriminants::CollectHashmap as u32 => {
                Ok(Self::CollectHashmap(CollectHashmap))
            }
            value if value == StdlibDiscriminants::CollectHashset as u32 => {
                Ok(Self::CollectHashset(CollectHashset))
            }
            value if value == StdlibDiscriminants::CollectList as u32 => {
                Ok(Self::CollectList(CollectList))
            }
            value if value == StdlibDiscriminants::CollectString as u32 => {
                Ok(Self::CollectString(CollectString))
            }
            value if value == StdlibDiscriminants::CollectTree as u32 => {
                Ok(Self::CollectTree(CollectTree))
            }
            value if value == StdlibDiscriminants::Cons as u32 => Ok(Self::Cons(Cons)),
            value if value == StdlibDiscriminants::Construct as u32 => {
                Ok(Self::Construct(Construct))
            }
            value if value == StdlibDiscriminants::Debug as u32 => Ok(Self::Debug(Debug)),
            value if value == StdlibDiscriminants::DecrementVariable as u32 => {
                Ok(Self::DecrementVariable(DecrementVariable))
            }
            value if value == StdlibDiscriminants::Divide as u32 => Ok(Self::Divide(Divide)),
            value if value == StdlibDiscriminants::Effect as u32 => Ok(Self::Effect(Effect)),
            value if value == StdlibDiscriminants::EndsWith as u32 => Ok(Self::EndsWith(EndsWith)),
            value if value == StdlibDiscriminants::Eq as u32 => Ok(Self::Eq(Eq)),
            value if value == StdlibDiscriminants::Equal as u32 => Ok(Self::Equal(Equal)),
            value if value == StdlibDiscriminants::Floor as u32 => Ok(Self::Floor(Floor)),
            value if value == StdlibDiscriminants::Fold as u32 => Ok(Self::Fold(Fold)),
            value if value == StdlibDiscriminants::FormatErrorMessage as u32 => {
                Ok(Self::FormatErrorMessage(FormatErrorMessage))
            }
            value if value == StdlibDiscriminants::Get as u32 => Ok(Self::Get(Get)),
            value if value == StdlibDiscriminants::GetVariable as u32 => {
                Ok(Self::GetVariable(GetVariable))
            }
            value if value == StdlibDiscriminants::GraphQlResolver as u32 => {
                Ok(Self::GraphQlResolver(GraphQlResolver))
            }
            value if value == StdlibDiscriminants::Gt as u32 => Ok(Self::Gt(Gt)),
            value if value == StdlibDiscriminants::Gte as u32 => Ok(Self::Gte(Gte)),
            value if value == StdlibDiscriminants::Has as u32 => Ok(Self::Has(Has)),
            value if value == StdlibDiscriminants::Hash as u32 => Ok(Self::Hash(Hash)),
            value if value == StdlibDiscriminants::Identity as u32 => Ok(Self::Identity(Identity)),
            value if value == StdlibDiscriminants::If as u32 => Ok(Self::If(If)),
            value if value == StdlibDiscriminants::IfError as u32 => Ok(Self::IfError(IfError)),
            value if value == StdlibDiscriminants::IfPending as u32 => {
                Ok(Self::IfPending(IfPending))
            }
            value if value == StdlibDiscriminants::IncrementVariable as u32 => {
                Ok(Self::IncrementVariable(IncrementVariable))
            }
            value if value == StdlibDiscriminants::IsFinite as u32 => Ok(Self::IsFinite(IsFinite)),
            value if value == StdlibDiscriminants::Iterate as u32 => Ok(Self::Iterate(Iterate)),
            value if value == StdlibDiscriminants::Keys as u32 => Ok(Self::Keys(Keys)),
            value if value == StdlibDiscriminants::Length as u32 => Ok(Self::Length(Length)),
            value if value == StdlibDiscriminants::Log as u32 => Ok(Self::Log(Log)),
            value if value == StdlibDiscriminants::Lt as u32 => Ok(Self::Lt(Lt)),
            value if value == StdlibDiscriminants::Lte as u32 => Ok(Self::Lte(Lte)),
            value if value == StdlibDiscriminants::Max as u32 => Ok(Self::Max(Max)),
            value if value == StdlibDiscriminants::Merge as u32 => Ok(Self::Merge(Merge)),
            value if value == StdlibDiscriminants::Min as u32 => Ok(Self::Min(Min)),
            value if value == StdlibDiscriminants::Multiply as u32 => Ok(Self::Multiply(Multiply)),
            value if value == StdlibDiscriminants::Not as u32 => Ok(Self::Not(Not)),
            value if value == StdlibDiscriminants::Or as u32 => Ok(Self::Or(Or)),
            value if value == StdlibDiscriminants::ParseDate as u32 => {
                Ok(Self::ParseDate(ParseDate))
            }
            value if value == StdlibDiscriminants::ParseFloat as u32 => {
                Ok(Self::ParseFloat(ParseFloat))
            }
            value if value == StdlibDiscriminants::ParseInt as u32 => Ok(Self::ParseInt(ParseInt)),
            value if value == StdlibDiscriminants::ParseJson as u32 => {
                Ok(Self::ParseJson(ParseJson))
            }
            value if value == StdlibDiscriminants::Pow as u32 => Ok(Self::Pow(Pow)),
            value if value == StdlibDiscriminants::Push as u32 => Ok(Self::Push(Push)),
            value if value == StdlibDiscriminants::PushFront as u32 => {
                Ok(Self::PushFront(PushFront))
            }
            value if value == StdlibDiscriminants::Remainder as u32 => {
                Ok(Self::Remainder(Remainder))
            }
            value if value == StdlibDiscriminants::Replace as u32 => Ok(Self::Replace(Replace)),
            value if value == StdlibDiscriminants::ResolveArgs as u32 => {
                Ok(Self::ResolveArgs(ResolveArgs))
            }
            value if value == StdlibDiscriminants::ResolveDeep as u32 => {
                Ok(Self::ResolveDeep(ResolveDeep))
            }
            value if value == StdlibDiscriminants::ResolveQueryBranch as u32 => {
                Ok(Self::ResolveQueryBranch(ResolveQueryBranch))
            }
            value if value == StdlibDiscriminants::ResolveQueryLeaf as u32 => {
                Ok(Self::ResolveQueryLeaf(ResolveQueryLeaf))
            }
            value if value == StdlibDiscriminants::ResolveShallow as u32 => {
                Ok(Self::ResolveShallow(ResolveShallow))
            }
            value if value == StdlibDiscriminants::Round as u32 => Ok(Self::Round(Round)),
            value if value == StdlibDiscriminants::Scan as u32 => Ok(Self::Scan(Scan)),
            value if value == StdlibDiscriminants::Sequence as u32 => Ok(Self::Sequence(Sequence)),
            value if value == StdlibDiscriminants::Set as u32 => Ok(Self::Set(Set)),
            value if value == StdlibDiscriminants::SetVariable as u32 => {
                Ok(Self::SetVariable(SetVariable))
            }
            value if value == StdlibDiscriminants::Skip as u32 => Ok(Self::Skip(Skip)),
            value if value == StdlibDiscriminants::Slice as u32 => Ok(Self::Slice(Slice)),
            value if value == StdlibDiscriminants::Split as u32 => Ok(Self::Split(Split)),
            value if value == StdlibDiscriminants::StartsWith as u32 => {
                Ok(Self::StartsWith(StartsWith))
            }
            value if value == StdlibDiscriminants::StringifyJson as u32 => {
                Ok(Self::StringifyJson(StringifyJson))
            }
            value if value == StdlibDiscriminants::Subtract as u32 => Ok(Self::Subtract(Subtract)),
            value if value == StdlibDiscriminants::Take as u32 => Ok(Self::Take(Take)),
            value if value == StdlibDiscriminants::Throw as u32 => Ok(Self::Throw(Throw)),
            value if value == StdlibDiscriminants::ToRequest as u32 => {
                Ok(Self::ToRequest(ToRequest))
            }
            value if value == StdlibDiscriminants::ToString as u32 => Ok(Self::ToString(ToString)),
            value if value == StdlibDiscriminants::Urlencode as u32 => {
                Ok(Self::Urlencode(Urlencode))
            }
            value if value == StdlibDiscriminants::Values as u32 => Ok(Self::Values(Values)),
            value if value == StdlibDiscriminants::Zip as u32 => Ok(Self::Zip(Zip)),
            _ => Err(()),
        }
    }
}
impl Stdlib {
    pub fn arity(&self) -> Arity {
        match self {
            Self::Abs(inner) => inner.arity(),
            Self::Accessor(inner) => inner.arity(),
            Self::Add(inner) => inner.arity(),
            Self::And(inner) => inner.arity(),
            Self::Apply(inner) => inner.arity(),
            Self::Car(inner) => inner.arity(),
            Self::Cdr(inner) => inner.arity(),
            Self::Ceil(inner) => inner.arity(),
            Self::Chain(inner) => inner.arity(),
            Self::CollectHashmap(inner) => inner.arity(),
            Self::CollectHashset(inner) => inner.arity(),
            Self::CollectList(inner) => inner.arity(),
            Self::CollectString(inner) => inner.arity(),
            Self::CollectTree(inner) => inner.arity(),
            Self::Cons(inner) => inner.arity(),
            Self::Construct(inner) => inner.arity(),
            Self::Debug(inner) => inner.arity(),
            Self::DecrementVariable(inner) => inner.arity(),
            Self::Divide(inner) => inner.arity(),
            Self::Effect(inner) => inner.arity(),
            Self::EndsWith(inner) => inner.arity(),
            Self::Eq(inner) => inner.arity(),
            Self::Equal(inner) => inner.arity(),
            Self::Floor(inner) => inner.arity(),
            Self::Fold(inner) => inner.arity(),
            Self::FormatErrorMessage(inner) => inner.arity(),
            Self::Get(inner) => inner.arity(),
            Self::GetVariable(inner) => inner.arity(),
            Self::GraphQlResolver(inner) => inner.arity(),
            Self::Gt(inner) => inner.arity(),
            Self::Gte(inner) => inner.arity(),
            Self::Has(inner) => inner.arity(),
            Self::Hash(inner) => inner.arity(),
            Self::Identity(inner) => inner.arity(),
            Self::If(inner) => inner.arity(),
            Self::IfError(inner) => inner.arity(),
            Self::IfPending(inner) => inner.arity(),
            Self::IncrementVariable(inner) => inner.arity(),
            Self::IsFinite(inner) => inner.arity(),
            Self::Iterate(inner) => inner.arity(),
            Self::Keys(inner) => inner.arity(),
            Self::Length(inner) => inner.arity(),
            Self::Log(inner) => inner.arity(),
            Self::Lt(inner) => inner.arity(),
            Self::Lte(inner) => inner.arity(),
            Self::Max(inner) => inner.arity(),
            Self::Merge(inner) => inner.arity(),
            Self::Min(inner) => inner.arity(),
            Self::Multiply(inner) => inner.arity(),
            Self::Not(inner) => inner.arity(),
            Self::Or(inner) => inner.arity(),
            Self::ParseDate(inner) => inner.arity(),
            Self::ParseFloat(inner) => inner.arity(),
            Self::ParseInt(inner) => inner.arity(),
            Self::ParseJson(inner) => inner.arity(),
            Self::Pow(inner) => inner.arity(),
            Self::Push(inner) => inner.arity(),
            Self::PushFront(inner) => inner.arity(),
            Self::Remainder(inner) => inner.arity(),
            Self::Replace(inner) => inner.arity(),
            Self::ResolveArgs(inner) => inner.arity(),
            Self::ResolveDeep(inner) => inner.arity(),
            Self::ResolveQueryBranch(inner) => inner.arity(),
            Self::ResolveQueryLeaf(inner) => inner.arity(),
            Self::ResolveShallow(inner) => inner.arity(),
            Self::Round(inner) => inner.arity(),
            Self::Scan(inner) => inner.arity(),
            Self::Sequence(inner) => inner.arity(),
            Self::Set(inner) => inner.arity(),
            Self::SetVariable(inner) => inner.arity(),
            Self::Skip(inner) => inner.arity(),
            Self::Slice(inner) => inner.arity(),
            Self::Split(inner) => inner.arity(),
            Self::StartsWith(inner) => inner.arity(),
            Self::StringifyJson(inner) => inner.arity(),
            Self::Subtract(inner) => inner.arity(),
            Self::Take(inner) => inner.arity(),
            Self::Throw(inner) => inner.arity(),
            Self::ToRequest(inner) => inner.arity(),
            Self::ToString(inner) => inner.arity(),
            Self::Urlencode(inner) => inner.arity(),
            Self::Values(inner) => inner.arity(),
            Self::Zip(inner) => inner.arity(),
        }
    }
    pub fn uid(&self) -> Uuid {
        match self {
            Self::Abs(inner) => inner.uid(),
            Self::Accessor(inner) => inner.uid(),
            Self::Add(inner) => inner.uid(),
            Self::And(inner) => inner.uid(),
            Self::Apply(inner) => inner.uid(),
            Self::Car(inner) => inner.uid(),
            Self::Cdr(inner) => inner.uid(),
            Self::Ceil(inner) => inner.uid(),
            Self::Chain(inner) => inner.uid(),
            Self::CollectHashmap(inner) => inner.uid(),
            Self::CollectHashset(inner) => inner.uid(),
            Self::CollectList(inner) => inner.uid(),
            Self::CollectString(inner) => inner.uid(),
            Self::CollectTree(inner) => inner.uid(),
            Self::Cons(inner) => inner.uid(),
            Self::Construct(inner) => inner.uid(),
            Self::Debug(inner) => inner.uid(),
            Self::DecrementVariable(inner) => inner.uid(),
            Self::Divide(inner) => inner.uid(),
            Self::Effect(inner) => inner.uid(),
            Self::EndsWith(inner) => inner.uid(),
            Self::Eq(inner) => inner.uid(),
            Self::Equal(inner) => inner.uid(),
            Self::Floor(inner) => inner.uid(),
            Self::Fold(inner) => inner.uid(),
            Self::FormatErrorMessage(inner) => inner.uid(),
            Self::Get(inner) => inner.uid(),
            Self::GetVariable(inner) => inner.uid(),
            Self::GraphQlResolver(inner) => inner.uid(),
            Self::Gt(inner) => inner.uid(),
            Self::Gte(inner) => inner.uid(),
            Self::Has(inner) => inner.uid(),
            Self::Hash(inner) => inner.uid(),
            Self::Identity(inner) => inner.uid(),
            Self::If(inner) => inner.uid(),
            Self::IfError(inner) => inner.uid(),
            Self::IfPending(inner) => inner.uid(),
            Self::IncrementVariable(inner) => inner.uid(),
            Self::IsFinite(inner) => inner.uid(),
            Self::Iterate(inner) => inner.uid(),
            Self::Keys(inner) => inner.uid(),
            Self::Length(inner) => inner.uid(),
            Self::Log(inner) => inner.uid(),
            Self::Lt(inner) => inner.uid(),
            Self::Lte(inner) => inner.uid(),
            Self::Max(inner) => inner.uid(),
            Self::Merge(inner) => inner.uid(),
            Self::Min(inner) => inner.uid(),
            Self::Multiply(inner) => inner.uid(),
            Self::Not(inner) => inner.uid(),
            Self::Or(inner) => inner.uid(),
            Self::ParseDate(inner) => inner.uid(),
            Self::ParseFloat(inner) => inner.uid(),
            Self::ParseInt(inner) => inner.uid(),
            Self::ParseJson(inner) => inner.uid(),
            Self::Pow(inner) => inner.uid(),
            Self::Push(inner) => inner.uid(),
            Self::PushFront(inner) => inner.uid(),
            Self::Remainder(inner) => inner.uid(),
            Self::Replace(inner) => inner.uid(),
            Self::ResolveArgs(inner) => inner.uid(),
            Self::ResolveDeep(inner) => inner.uid(),
            Self::ResolveQueryBranch(inner) => inner.uid(),
            Self::ResolveQueryLeaf(inner) => inner.uid(),
            Self::ResolveShallow(inner) => inner.uid(),
            Self::Round(inner) => inner.uid(),
            Self::Scan(inner) => inner.uid(),
            Self::Sequence(inner) => inner.uid(),
            Self::Set(inner) => inner.uid(),
            Self::SetVariable(inner) => inner.uid(),
            Self::Skip(inner) => inner.uid(),
            Self::Slice(inner) => inner.uid(),
            Self::Split(inner) => inner.uid(),
            Self::StartsWith(inner) => inner.uid(),
            Self::StringifyJson(inner) => inner.uid(),
            Self::Subtract(inner) => inner.uid(),
            Self::Take(inner) => inner.uid(),
            Self::Throw(inner) => inner.uid(),
            Self::ToRequest(inner) => inner.uid(),
            Self::ToString(inner) => inner.uid(),
            Self::Urlencode(inner) => inner.uid(),
            Self::Values(inner) => inner.uid(),
            Self::Zip(inner) => inner.uid(),
        }
    }
}

impl Builtin for Stdlib {
    fn arity(&self) -> Arity {
        self.arity()
    }
    fn apply<T: Expression<Builtin = Self> + Applicable<T>>(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        Err(format!(
            "Unable to apply native WebAssembly function: {:?}",
            self
        ))
    }
    fn should_parallelize<T: Expression<Builtin = Self> + Applicable<T>>(
        &self,
        args: &[T],
    ) -> bool {
        false
    }
}
impl Uid for Stdlib {
    fn uid(&self) -> Uuid {
        self.uid()
    }
}

impl TryFrom<Uuid> for Stdlib {
    type Error = ();
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        match value {
            Abs::UUID => Ok(Self::Abs(Abs)),
            Accessor::UUID => Ok(Self::Accessor(Accessor)),
            Add::UUID => Ok(Self::Add(Add)),
            And::UUID => Ok(Self::And(And)),
            Apply::UUID => Ok(Self::Apply(Apply)),
            Car::UUID => Ok(Self::Car(Car)),
            Cdr::UUID => Ok(Self::Cdr(Cdr)),
            Ceil::UUID => Ok(Self::Ceil(Ceil)),
            Chain::UUID => Ok(Self::Chain(Chain)),
            CollectHashmap::UUID => Ok(Self::CollectHashmap(CollectHashmap)),
            CollectHashset::UUID => Ok(Self::CollectHashset(CollectHashset)),
            CollectList::UUID => Ok(Self::CollectList(CollectList)),
            CollectString::UUID => Ok(Self::CollectString(CollectString)),
            CollectTree::UUID => Ok(Self::CollectTree(CollectTree)),
            Cons::UUID => Ok(Self::Cons(Cons)),
            Construct::UUID => Ok(Self::Construct(Construct)),
            Debug::UUID => Ok(Self::Debug(Debug)),
            DecrementVariable::UUID => Ok(Self::DecrementVariable(DecrementVariable)),
            Divide::UUID => Ok(Self::Divide(Divide)),
            Effect::UUID => Ok(Self::Effect(Effect)),
            EndsWith::UUID => Ok(Self::EndsWith(EndsWith)),
            Eq::UUID => Ok(Self::Eq(Eq)),
            Equal::UUID => Ok(Self::Equal(Equal)),
            Floor::UUID => Ok(Self::Floor(Floor)),
            Fold::UUID => Ok(Self::Fold(Fold)),
            FormatErrorMessage::UUID => Ok(Self::FormatErrorMessage(FormatErrorMessage)),
            Get::UUID => Ok(Self::Get(Get)),
            GetVariable::UUID => Ok(Self::GetVariable(GetVariable)),
            GraphQlResolver::UUID => Ok(Self::GraphQlResolver(GraphQlResolver)),
            Gt::UUID => Ok(Self::Gt(Gt)),
            Gte::UUID => Ok(Self::Gte(Gte)),
            Has::UUID => Ok(Self::Has(Has)),
            Hash::UUID => Ok(Self::Hash(Hash)),
            Identity::UUID => Ok(Self::Identity(Identity)),
            If::UUID => Ok(Self::If(If)),
            IfError::UUID => Ok(Self::IfError(IfError)),
            IfPending::UUID => Ok(Self::IfPending(IfPending)),
            IncrementVariable::UUID => Ok(Self::IncrementVariable(IncrementVariable)),
            IsFinite::UUID => Ok(Self::IsFinite(IsFinite)),
            Iterate::UUID => Ok(Self::Iterate(Iterate)),
            Keys::UUID => Ok(Self::Keys(Keys)),
            Length::UUID => Ok(Self::Length(Length)),
            Log::UUID => Ok(Self::Log(Log)),
            Lt::UUID => Ok(Self::Lt(Lt)),
            Lte::UUID => Ok(Self::Lte(Lte)),
            Max::UUID => Ok(Self::Max(Max)),
            Merge::UUID => Ok(Self::Merge(Merge)),
            Min::UUID => Ok(Self::Min(Min)),
            Multiply::UUID => Ok(Self::Multiply(Multiply)),
            Not::UUID => Ok(Self::Not(Not)),
            Or::UUID => Ok(Self::Or(Or)),
            ParseDate::UUID => Ok(Self::ParseDate(ParseDate)),
            ParseFloat::UUID => Ok(Self::ParseFloat(ParseFloat)),
            ParseInt::UUID => Ok(Self::ParseInt(ParseInt)),
            ParseJson::UUID => Ok(Self::ParseJson(ParseJson)),
            Pow::UUID => Ok(Self::Pow(Pow)),
            Push::UUID => Ok(Self::Push(Push)),
            PushFront::UUID => Ok(Self::PushFront(PushFront)),
            Remainder::UUID => Ok(Self::Remainder(Remainder)),
            Replace::UUID => Ok(Self::Replace(Replace)),
            ResolveArgs::UUID => Ok(Self::ResolveArgs(ResolveArgs)),
            ResolveDeep::UUID => Ok(Self::ResolveDeep(ResolveDeep)),
            ResolveQueryBranch::UUID => Ok(Self::ResolveQueryBranch(ResolveQueryBranch)),
            ResolveQueryLeaf::UUID => Ok(Self::ResolveQueryLeaf(ResolveQueryLeaf)),
            ResolveShallow::UUID => Ok(Self::ResolveShallow(ResolveShallow)),
            Round::UUID => Ok(Self::Round(Round)),
            Scan::UUID => Ok(Self::Scan(Scan)),
            Sequence::UUID => Ok(Self::Sequence(Sequence)),
            Set::UUID => Ok(Self::Set(Set)),
            SetVariable::UUID => Ok(Self::SetVariable(SetVariable)),
            Skip::UUID => Ok(Self::Skip(Skip)),
            Slice::UUID => Ok(Self::Slice(Slice)),
            Split::UUID => Ok(Self::Split(Split)),
            StartsWith::UUID => Ok(Self::StartsWith(StartsWith)),
            StringifyJson::UUID => Ok(Self::StringifyJson(StringifyJson)),
            Subtract::UUID => Ok(Self::Subtract(Subtract)),
            Take::UUID => Ok(Self::Take(Take)),
            Throw::UUID => Ok(Self::Throw(Throw)),
            ToRequest::UUID => Ok(Self::ToRequest(ToRequest)),
            ToString::UUID => Ok(Self::ToString(ToString)),
            Urlencode::UUID => Ok(Self::Urlencode(Urlencode)),
            Values::UUID => Ok(Self::Values(Values)),
            Zip::UUID => Ok(Self::Zip(Zip)),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Stdlib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<wasm:{:?}>", self)
    }
}
