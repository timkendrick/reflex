// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    Applicable, Arity, Builtin, EvaluationCache, Expression, ExpressionFactory, HeapAllocator, Uid,
    Uuid,
};
use reflex_macros::Matcher;
use strum_macros::{EnumDiscriminants, EnumIter};

pub mod abs;
pub mod add;
pub mod and;
pub mod apply;
pub mod car;
pub mod cdr;
pub mod ceil;
pub mod chain;
pub mod collect_constructor;
pub mod collect_hashmap;
pub mod collect_hashset;
pub mod collect_list;
pub mod collect_record;
pub mod collect_signal;
pub mod collect_string;
pub mod collect_tree;
pub mod cons;
pub mod divide;
pub mod effect;
pub mod ends_with;
pub mod eq;
pub mod equal;
pub mod filter;
pub mod flatten;
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
pub mod intersperse;
pub mod iterate;
pub mod js;
pub mod json;
pub mod keys;
pub mod length;
pub mod lt;
pub mod lte;
pub mod map;
pub mod max;
pub mod merge;
pub mod min;
pub mod multiply;
pub mod not;
pub mod or;
pub mod pow;
pub mod push;
pub mod push_front;
pub mod raise;
pub mod remainder;
pub mod replace;
pub mod resolve_args;
pub mod resolve_deep;
pub mod resolve_hashmap;
pub mod resolve_hashset;
pub mod resolve_list;
pub mod resolve_record;
pub mod resolve_tree;
pub mod round;
pub mod sequence;
pub mod set;
pub mod skip;
pub mod slice;
pub mod split;
pub mod starts_with;
pub mod subtract;
pub mod take;
pub mod unzip;
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
pub use collect_constructor::*;
pub use collect_hashmap::*;
pub use collect_hashset::*;
pub use collect_list::*;
pub use collect_record::*;
pub use collect_signal::*;
pub use collect_string::*;
pub use collect_tree::*;
pub use cons::*;
pub use divide::*;
pub use effect::*;
pub use ends_with::*;
pub use eq::*;
pub use equal::*;
pub use filter::*;
pub use flatten::*;
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
pub use intersperse::*;
pub use iterate::*;
pub use js::*;
pub use json::*;
pub use keys::*;
pub use length::*;
pub use lt::*;
pub use lte::*;
pub use map::*;
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
pub use raise::*;
pub use remainder::*;
pub use replace::*;
pub use resolve_args::*;
pub use resolve_deep::*;
pub use resolve_hashmap::*;
pub use resolve_hashset::*;
pub use resolve_list::*;
pub use resolve_record::*;
pub use resolve_tree::*;
pub use round::*;
pub use sequence::*;
pub use set::*;
pub use skip::*;
pub use slice::*;
pub use split::*;
pub use starts_with::*;
pub use subtract::*;
pub use take::*;
pub use unzip::*;
pub use values::*;
pub use zip::*;

#[derive(Matcher, PartialEq, Eq, Clone, Copy, Debug, EnumDiscriminants, EnumIter)]
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
    CollectConstructor(CollectConstructor),
    CollectHashmap(CollectHashmap),
    CollectHashset(CollectHashset),
    CollectList(CollectList),
    CollectRecord(CollectRecord),
    CollectSignal(CollectSignal),
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
    Filter(Filter),
    Flatten(Flatten),
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
    Intersperse(Intersperse),
    IsFinite(IsFinite),
    IsTruthy(IsTruthy),
    Iterate(Iterate),
    Keys(Keys),
    Length(Length),
    Log(Log),
    Lt(Lt),
    Lte(Lte),
    Map(Map),
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
    Raise(Raise),
    Remainder(Remainder),
    Replace(Replace),
    ResolveArgs(ResolveArgs),
    ResolveDeep(ResolveDeep),
    ResolveHashmap(ResolveHashmap),
    ResolveHashset(ResolveHashset),
    ResolveList(ResolveList),
    ResolveLoaderResults(ResolveLoaderResults),
    ResolveQueryBranch(ResolveQueryBranch),
    ResolveQueryLeaf(ResolveQueryLeaf),
    ResolveRecord(ResolveRecord),
    ResolveTree(ResolveTree),
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
    Unzip(Unzip),
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
            Stdlib::CollectConstructor(_) => StdlibDiscriminants::CollectConstructor as u32,
            Stdlib::CollectHashmap(_) => StdlibDiscriminants::CollectHashmap as u32,
            Stdlib::CollectHashset(_) => StdlibDiscriminants::CollectHashset as u32,
            Stdlib::CollectList(_) => StdlibDiscriminants::CollectList as u32,
            Stdlib::CollectRecord(_) => StdlibDiscriminants::CollectRecord as u32,
            Stdlib::CollectSignal(_) => StdlibDiscriminants::CollectSignal as u32,
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
            Stdlib::Filter(_) => StdlibDiscriminants::Filter as u32,
            Stdlib::Flatten(_) => StdlibDiscriminants::Flatten as u32,
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
            Stdlib::Intersperse(_) => StdlibDiscriminants::Intersperse as u32,
            Stdlib::IsFinite(_) => StdlibDiscriminants::IsFinite as u32,
            Stdlib::IsTruthy(_) => StdlibDiscriminants::IsTruthy as u32,
            Stdlib::Iterate(_) => StdlibDiscriminants::Iterate as u32,
            Stdlib::Keys(_) => StdlibDiscriminants::Keys as u32,
            Stdlib::Length(_) => StdlibDiscriminants::Length as u32,
            Stdlib::Log(_) => StdlibDiscriminants::Log as u32,
            Stdlib::Lt(_) => StdlibDiscriminants::Lt as u32,
            Stdlib::Lte(_) => StdlibDiscriminants::Lte as u32,
            Stdlib::Map(_) => StdlibDiscriminants::Map as u32,
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
            Stdlib::Raise(_) => StdlibDiscriminants::Raise as u32,
            Stdlib::Remainder(_) => StdlibDiscriminants::Remainder as u32,
            Stdlib::Replace(_) => StdlibDiscriminants::Replace as u32,
            Stdlib::ResolveArgs(_) => StdlibDiscriminants::ResolveArgs as u32,
            Stdlib::ResolveDeep(_) => StdlibDiscriminants::ResolveDeep as u32,
            Stdlib::ResolveHashmap(_) => StdlibDiscriminants::ResolveHashmap as u32,
            Stdlib::ResolveHashset(_) => StdlibDiscriminants::ResolveHashset as u32,
            Stdlib::ResolveList(_) => StdlibDiscriminants::ResolveList as u32,
            Stdlib::ResolveLoaderResults(_) => StdlibDiscriminants::ResolveLoaderResults as u32,
            Stdlib::ResolveQueryBranch(_) => StdlibDiscriminants::ResolveQueryBranch as u32,
            Stdlib::ResolveQueryLeaf(_) => StdlibDiscriminants::ResolveQueryLeaf as u32,
            Stdlib::ResolveRecord(_) => StdlibDiscriminants::ResolveRecord as u32,
            Stdlib::ResolveTree(_) => StdlibDiscriminants::ResolveTree as u32,
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
            Stdlib::Unzip(_) => StdlibDiscriminants::Unzip as u32,
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
            value if value == StdlibDiscriminants::CollectConstructor as u32 => {
                Ok(Self::CollectConstructor(CollectConstructor))
            }
            value if value == StdlibDiscriminants::CollectHashmap as u32 => {
                Ok(Self::CollectHashmap(CollectHashmap))
            }
            value if value == StdlibDiscriminants::CollectHashset as u32 => {
                Ok(Self::CollectHashset(CollectHashset))
            }
            value if value == StdlibDiscriminants::CollectList as u32 => {
                Ok(Self::CollectList(CollectList))
            }
            value if value == StdlibDiscriminants::CollectRecord as u32 => {
                Ok(Self::CollectRecord(CollectRecord))
            }
            value if value == StdlibDiscriminants::CollectSignal as u32 => {
                Ok(Self::CollectSignal(CollectSignal))
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
            value if value == StdlibDiscriminants::Filter as u32 => Ok(Self::Filter(Filter)),
            value if value == StdlibDiscriminants::Flatten as u32 => Ok(Self::Flatten(Flatten)),
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
            value if value == StdlibDiscriminants::Intersperse as u32 => {
                Ok(Self::Intersperse(Intersperse))
            }
            value if value == StdlibDiscriminants::IsFinite as u32 => Ok(Self::IsFinite(IsFinite)),
            value if value == StdlibDiscriminants::IsTruthy as u32 => Ok(Self::IsTruthy(IsTruthy)),
            value if value == StdlibDiscriminants::Iterate as u32 => Ok(Self::Iterate(Iterate)),
            value if value == StdlibDiscriminants::Keys as u32 => Ok(Self::Keys(Keys)),
            value if value == StdlibDiscriminants::Length as u32 => Ok(Self::Length(Length)),
            value if value == StdlibDiscriminants::Log as u32 => Ok(Self::Log(Log)),
            value if value == StdlibDiscriminants::Lt as u32 => Ok(Self::Lt(Lt)),
            value if value == StdlibDiscriminants::Lte as u32 => Ok(Self::Lte(Lte)),
            value if value == StdlibDiscriminants::Map as u32 => Ok(Self::Map(Map)),
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
            value if value == StdlibDiscriminants::Raise as u32 => Ok(Self::Raise(Raise)),
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
            value if value == StdlibDiscriminants::ResolveHashmap as u32 => {
                Ok(Self::ResolveHashmap(ResolveHashmap))
            }
            value if value == StdlibDiscriminants::ResolveHashset as u32 => {
                Ok(Self::ResolveHashset(ResolveHashset))
            }
            value if value == StdlibDiscriminants::ResolveList as u32 => {
                Ok(Self::ResolveList(ResolveList))
            }
            value if value == StdlibDiscriminants::ResolveLoaderResults as u32 => {
                Ok(Self::ResolveLoaderResults(ResolveLoaderResults))
            }
            value if value == StdlibDiscriminants::ResolveQueryBranch as u32 => {
                Ok(Self::ResolveQueryBranch(ResolveQueryBranch))
            }
            value if value == StdlibDiscriminants::ResolveQueryLeaf as u32 => {
                Ok(Self::ResolveQueryLeaf(ResolveQueryLeaf))
            }
            value if value == StdlibDiscriminants::ResolveRecord as u32 => {
                Ok(Self::ResolveRecord(ResolveRecord))
            }
            value if value == StdlibDiscriminants::ResolveTree as u32 => {
                Ok(Self::ResolveTree(ResolveTree))
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
            value if value == StdlibDiscriminants::Unzip as u32 => Ok(Self::Unzip(Unzip)),
            value if value == StdlibDiscriminants::Values as u32 => Ok(Self::Values(Values)),
            value if value == StdlibDiscriminants::Zip as u32 => Ok(Self::Zip(Zip)),
            _ => Err(()),
        }
    }
}

impl Stdlib {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Abs(_) => "Stdlib_Abs",
            Self::Accessor(_) => "Stdlib_Accessor",
            Self::Add(_) => "Stdlib_Add",
            Self::And(_) => "Stdlib_And",
            Self::Apply(_) => "Stdlib_Apply",
            Self::Car(_) => "Stdlib_Car",
            Self::Cdr(_) => "Stdlib_Cdr",
            Self::Ceil(_) => "Stdlib_Ceil",
            Self::Chain(_) => "Stdlib_Chain",
            Self::CollectConstructor(_) => "Stdlib_CollectConstructor",
            Self::CollectHashmap(_) => "Stdlib_CollectHashmap",
            Self::CollectHashset(_) => "Stdlib_CollectHashset",
            Self::CollectList(_) => "Stdlib_CollectList",
            Self::CollectRecord(_) => "Stdlib_CollectRecord",
            Self::CollectSignal(_) => "Stdlib_CollectSignal",
            Self::CollectString(_) => "Stdlib_CollectString",
            Self::CollectTree(_) => "Stdlib_CollectTree",
            Self::Cons(_) => "Stdlib_Cons",
            Self::Construct(_) => "Stdlib_Construct",
            Self::Debug(_) => "Stdlib_Debug",
            Self::DecrementVariable(_) => "Stdlib_DecrementVariable",
            Self::Divide(_) => "Stdlib_Divide",
            Self::Effect(_) => "Stdlib_Effect",
            Self::EndsWith(_) => "Stdlib_EndsWith",
            Self::Eq(_) => "Stdlib_Eq",
            Self::Equal(_) => "Stdlib_Equal",
            Self::Filter(_) => "Stdlib_Filter",
            Self::Flatten(_) => "Stdlib_Flatten",
            Self::Floor(_) => "Stdlib_Floor",
            Self::Fold(_) => "Stdlib_Fold",
            Self::FormatErrorMessage(_) => "Stdlib_FormatErrorMessage",
            Self::Get(_) => "Stdlib_Get",
            Self::GetVariable(_) => "Stdlib_GetVariable",
            Self::GraphQlResolver(_) => "Stdlib_GraphQlResolver",
            Self::Gt(_) => "Stdlib_Gt",
            Self::Gte(_) => "Stdlib_Gte",
            Self::Has(_) => "Stdlib_Has",
            Self::Hash(_) => "Stdlib_Hash",
            Self::Identity(_) => "Stdlib_Identity",
            Self::If(_) => "Stdlib_If",
            Self::IfError(_) => "Stdlib_IfError",
            Self::IfPending(_) => "Stdlib_IfPending",
            Self::IncrementVariable(_) => "Stdlib_IncrementVariable",
            Self::Intersperse(_) => "Stdlib_Intersperse",
            Self::IsFinite(_) => "Stdlib_IsFinite",
            Self::IsTruthy(_) => "Stdlib_IsTruthy",
            Self::Iterate(_) => "Stdlib_Iterate",
            Self::Keys(_) => "Stdlib_Keys",
            Self::Length(_) => "Stdlib_Length",
            Self::Log(_) => "Stdlib_Log",
            Self::Lt(_) => "Stdlib_Lt",
            Self::Lte(_) => "Stdlib_Lte",
            Self::Map(_) => "Stdlib_Map",
            Self::Max(_) => "Stdlib_Max",
            Self::Merge(_) => "Stdlib_Merge",
            Self::Min(_) => "Stdlib_Min",
            Self::Multiply(_) => "Stdlib_Multiply",
            Self::Not(_) => "Stdlib_Not",
            Self::Or(_) => "Stdlib_Or",
            Self::ParseDate(_) => "Stdlib_ParseDate",
            Self::ParseFloat(_) => "Stdlib_ParseFloat",
            Self::ParseInt(_) => "Stdlib_ParseInt",
            Self::ParseJson(_) => "Stdlib_ParseJson",
            Self::Pow(_) => "Stdlib_Pow",
            Self::Push(_) => "Stdlib_Push",
            Self::PushFront(_) => "Stdlib_PushFront",
            Self::Raise(_) => "Stdlib_Raise",
            Self::Remainder(_) => "Stdlib_Remainder",
            Self::Replace(_) => "Stdlib_Replace",
            Self::ResolveArgs(_) => "Stdlib_ResolveArgs",
            Self::ResolveDeep(_) => "Stdlib_ResolveDeep",
            Self::ResolveHashmap(_) => "Stdlib_ResolveHashmap",
            Self::ResolveHashset(_) => "Stdlib_ResolveHashset",
            Self::ResolveList(_) => "Stdlib_ResolveList",
            Self::ResolveLoaderResults(_) => "Stdlib_ResolveLoaderResults",
            Self::ResolveQueryBranch(_) => "Stdlib_ResolveQueryBranch",
            Self::ResolveQueryLeaf(_) => "Stdlib_ResolveQueryLeaf",
            Self::ResolveRecord(_) => "Stdlib_ResolveRecord",
            Self::ResolveTree(_) => "Stdlib_ResolveTree",
            Self::Round(_) => "Stdlib_Round",
            Self::Scan(_) => "Stdlib_Scan",
            Self::Sequence(_) => "Stdlib_Sequence",
            Self::Set(_) => "Stdlib_Set",
            Self::SetVariable(_) => "Stdlib_SetVariable",
            Self::Skip(_) => "Stdlib_Skip",
            Self::Slice(_) => "Stdlib_Slice",
            Self::Split(_) => "Stdlib_Split",
            Self::StartsWith(_) => "Stdlib_StartsWith",
            Self::StringifyJson(_) => "Stdlib_StringifyJson",
            Self::Subtract(_) => "Stdlib_Subtract",
            Self::Take(_) => "Stdlib_Take",
            Self::Throw(_) => "Stdlib_Throw",
            Self::ToRequest(_) => "Stdlib_ToRequest",
            Self::ToString(_) => "Stdlib_ToString",
            Self::Urlencode(_) => "Stdlib_Urlencode",
            Self::Unzip(_) => "Stdlib_Unzip",
            Self::Values(_) => "Stdlib_Values",
            Self::Zip(_) => "Stdlib_Zip",
        }
    }
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
            Self::CollectConstructor(inner) => inner.arity(),
            Self::CollectHashmap(inner) => inner.arity(),
            Self::CollectHashset(inner) => inner.arity(),
            Self::CollectList(inner) => inner.arity(),
            Self::CollectRecord(inner) => inner.arity(),
            Self::CollectSignal(inner) => inner.arity(),
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
            Self::Filter(inner) => inner.arity(),
            Self::Flatten(inner) => inner.arity(),
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
            Self::Intersperse(inner) => inner.arity(),
            Self::IsFinite(inner) => inner.arity(),
            Self::IsTruthy(inner) => inner.arity(),
            Self::Iterate(inner) => inner.arity(),
            Self::Keys(inner) => inner.arity(),
            Self::Length(inner) => inner.arity(),
            Self::Log(inner) => inner.arity(),
            Self::Lt(inner) => inner.arity(),
            Self::Lte(inner) => inner.arity(),
            Self::Map(inner) => inner.arity(),
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
            Self::Raise(inner) => inner.arity(),
            Self::Remainder(inner) => inner.arity(),
            Self::Replace(inner) => inner.arity(),
            Self::ResolveArgs(inner) => inner.arity(),
            Self::ResolveDeep(inner) => inner.arity(),
            Self::ResolveHashmap(inner) => inner.arity(),
            Self::ResolveHashset(inner) => inner.arity(),
            Self::ResolveList(inner) => inner.arity(),
            Self::ResolveLoaderResults(inner) => inner.arity(),
            Self::ResolveQueryBranch(inner) => inner.arity(),
            Self::ResolveQueryLeaf(inner) => inner.arity(),
            Self::ResolveRecord(inner) => inner.arity(),
            Self::ResolveTree(inner) => inner.arity(),
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
            Self::Unzip(inner) => inner.arity(),
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
            Self::CollectConstructor(inner) => inner.uid(),
            Self::CollectHashmap(inner) => inner.uid(),
            Self::CollectHashset(inner) => inner.uid(),
            Self::CollectList(inner) => inner.uid(),
            Self::CollectRecord(inner) => inner.uid(),
            Self::CollectSignal(inner) => inner.uid(),
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
            Self::Filter(inner) => inner.uid(),
            Self::Flatten(inner) => inner.uid(),
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
            Self::Intersperse(inner) => inner.uid(),
            Self::IsFinite(inner) => inner.uid(),
            Self::IsTruthy(inner) => inner.uid(),
            Self::Iterate(inner) => inner.uid(),
            Self::Keys(inner) => inner.uid(),
            Self::Length(inner) => inner.uid(),
            Self::Log(inner) => inner.uid(),
            Self::Lt(inner) => inner.uid(),
            Self::Lte(inner) => inner.uid(),
            Self::Map(inner) => inner.uid(),
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
            Self::Raise(inner) => inner.uid(),
            Self::Remainder(inner) => inner.uid(),
            Self::Replace(inner) => inner.uid(),
            Self::ResolveArgs(inner) => inner.uid(),
            Self::ResolveDeep(inner) => inner.uid(),
            Self::ResolveHashmap(inner) => inner.uid(),
            Self::ResolveHashset(inner) => inner.uid(),
            Self::ResolveList(inner) => inner.uid(),
            Self::ResolveLoaderResults(inner) => inner.uid(),
            Self::ResolveQueryBranch(inner) => inner.uid(),
            Self::ResolveQueryLeaf(inner) => inner.uid(),
            Self::ResolveRecord(inner) => inner.uid(),
            Self::ResolveTree(inner) => inner.uid(),
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
            Self::Unzip(inner) => inner.uid(),
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
        _cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        // Native WebAssembly functions cannot currently be evaluated outside a WebAssembly VM, but the `Builtin`  trait
        // interface does not provide a WebAssembly execution context to allow this.
        // We therefore need to ensure that WASM stdlib functions do not interfere with normalization of other nodes.
        // Note that this method should only be invoked during the pre-compile AST normalization phase,
        // which includes an explicit check that bails out if a builtin function returns an identical application term.
        Ok(factory.create_application_term(
            factory.create_builtin_term(*self),
            allocator.create_list(args),
        ))
    }
    fn should_parallelize<T: Expression<Builtin = Self> + Applicable<T>>(
        &self,
        _args: &[T],
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
            CollectConstructor::UUID => Ok(Self::CollectConstructor(CollectConstructor)),
            CollectHashmap::UUID => Ok(Self::CollectHashmap(CollectHashmap)),
            CollectHashset::UUID => Ok(Self::CollectHashset(CollectHashset)),
            CollectList::UUID => Ok(Self::CollectList(CollectList)),
            CollectRecord::UUID => Ok(Self::CollectRecord(CollectRecord)),
            CollectSignal::UUID => Ok(Self::CollectSignal(CollectSignal)),
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
            Filter::UUID => Ok(Self::Filter(Filter)),
            Flatten::UUID => Ok(Self::Flatten(Flatten)),
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
            Intersperse::UUID => Ok(Self::Intersperse(Intersperse)),
            IsFinite::UUID => Ok(Self::IsFinite(IsFinite)),
            IsTruthy::UUID => Ok(Self::IsTruthy(IsTruthy)),
            Iterate::UUID => Ok(Self::Iterate(Iterate)),
            Keys::UUID => Ok(Self::Keys(Keys)),
            Length::UUID => Ok(Self::Length(Length)),
            Log::UUID => Ok(Self::Log(Log)),
            Lt::UUID => Ok(Self::Lt(Lt)),
            Lte::UUID => Ok(Self::Lte(Lte)),
            Map::UUID => Ok(Self::Map(Map)),
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
            Raise::UUID => Ok(Self::Raise(Raise)),
            Remainder::UUID => Ok(Self::Remainder(Remainder)),
            Replace::UUID => Ok(Self::Replace(Replace)),
            ResolveArgs::UUID => Ok(Self::ResolveArgs(ResolveArgs)),
            ResolveDeep::UUID => Ok(Self::ResolveDeep(ResolveDeep)),
            ResolveHashmap::UUID => Ok(Self::ResolveHashmap(ResolveHashmap)),
            ResolveHashset::UUID => Ok(Self::ResolveHashset(ResolveHashset)),
            ResolveList::UUID => Ok(Self::ResolveList(ResolveList)),
            ResolveLoaderResults::UUID => Ok(Self::ResolveLoaderResults(ResolveLoaderResults)),
            ResolveQueryBranch::UUID => Ok(Self::ResolveQueryBranch(ResolveQueryBranch)),
            ResolveQueryLeaf::UUID => Ok(Self::ResolveQueryLeaf(ResolveQueryLeaf)),
            ResolveRecord::UUID => Ok(Self::ResolveRecord(ResolveRecord)),
            ResolveTree::UUID => Ok(Self::ResolveTree(ResolveTree)),
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
            Unzip::UUID => Ok(Self::Unzip(Unzip)),
            Values::UUID => Ok(Self::Values(Values)),
            Zip::UUID => Ok(Self::Zip(Zip)),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Stdlib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wasm::{:?}", StdlibDiscriminants::from(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdlib_function_indices() {
        assert_eq!(StdlibDiscriminants::Abs as u32, 0);
        assert_eq!(StdlibDiscriminants::Accessor as u32, 1);
        assert_eq!(StdlibDiscriminants::Add as u32, 2);
        assert_eq!(StdlibDiscriminants::And as u32, 3);
        assert_eq!(StdlibDiscriminants::Apply as u32, 4);
        assert_eq!(StdlibDiscriminants::Car as u32, 5);
        assert_eq!(StdlibDiscriminants::Cdr as u32, 6);
        assert_eq!(StdlibDiscriminants::Ceil as u32, 7);
        assert_eq!(StdlibDiscriminants::Chain as u32, 8);
        assert_eq!(StdlibDiscriminants::CollectConstructor as u32, 9);
        assert_eq!(StdlibDiscriminants::CollectHashmap as u32, 10);
        assert_eq!(StdlibDiscriminants::CollectHashset as u32, 11);
        assert_eq!(StdlibDiscriminants::CollectList as u32, 12);
        assert_eq!(StdlibDiscriminants::CollectRecord as u32, 13);
        assert_eq!(StdlibDiscriminants::CollectSignal as u32, 14);
        assert_eq!(StdlibDiscriminants::CollectString as u32, 15);
        assert_eq!(StdlibDiscriminants::CollectTree as u32, 16);
        assert_eq!(StdlibDiscriminants::Cons as u32, 17);
        assert_eq!(StdlibDiscriminants::Construct as u32, 18);
        assert_eq!(StdlibDiscriminants::Debug as u32, 19);
        assert_eq!(StdlibDiscriminants::DecrementVariable as u32, 20);
        assert_eq!(StdlibDiscriminants::Divide as u32, 21);
        assert_eq!(StdlibDiscriminants::Effect as u32, 22);
        assert_eq!(StdlibDiscriminants::EndsWith as u32, 23);
        assert_eq!(StdlibDiscriminants::Eq as u32, 24);
        assert_eq!(StdlibDiscriminants::Equal as u32, 25);
        assert_eq!(StdlibDiscriminants::Filter as u32, 26);
        assert_eq!(StdlibDiscriminants::Flatten as u32, 27);
        assert_eq!(StdlibDiscriminants::Floor as u32, 28);
        assert_eq!(StdlibDiscriminants::Fold as u32, 29);
        assert_eq!(StdlibDiscriminants::FormatErrorMessage as u32, 30);
        assert_eq!(StdlibDiscriminants::Get as u32, 31);
        assert_eq!(StdlibDiscriminants::GetVariable as u32, 32);
        assert_eq!(StdlibDiscriminants::GraphQlResolver as u32, 33);
        assert_eq!(StdlibDiscriminants::Gt as u32, 34);
        assert_eq!(StdlibDiscriminants::Gte as u32, 35);
        assert_eq!(StdlibDiscriminants::Has as u32, 36);
        assert_eq!(StdlibDiscriminants::Hash as u32, 37);
        assert_eq!(StdlibDiscriminants::Identity as u32, 38);
        assert_eq!(StdlibDiscriminants::If as u32, 39);
        assert_eq!(StdlibDiscriminants::IfError as u32, 40);
        assert_eq!(StdlibDiscriminants::IfPending as u32, 41);
        assert_eq!(StdlibDiscriminants::IncrementVariable as u32, 42);
        assert_eq!(StdlibDiscriminants::Intersperse as u32, 43);
        assert_eq!(StdlibDiscriminants::IsFinite as u32, 44);
        assert_eq!(StdlibDiscriminants::IsTruthy as u32, 45);
        assert_eq!(StdlibDiscriminants::Iterate as u32, 46);
        assert_eq!(StdlibDiscriminants::Keys as u32, 47);
        assert_eq!(StdlibDiscriminants::Length as u32, 48);
        assert_eq!(StdlibDiscriminants::Log as u32, 49);
        assert_eq!(StdlibDiscriminants::Lt as u32, 50);
        assert_eq!(StdlibDiscriminants::Lte as u32, 51);
        assert_eq!(StdlibDiscriminants::Map as u32, 52);
        assert_eq!(StdlibDiscriminants::Max as u32, 53);
        assert_eq!(StdlibDiscriminants::Merge as u32, 54);
        assert_eq!(StdlibDiscriminants::Min as u32, 55);
        assert_eq!(StdlibDiscriminants::Multiply as u32, 56);
        assert_eq!(StdlibDiscriminants::Not as u32, 57);
        assert_eq!(StdlibDiscriminants::Or as u32, 58);
        assert_eq!(StdlibDiscriminants::ParseDate as u32, 59);
        assert_eq!(StdlibDiscriminants::ParseFloat as u32, 60);
        assert_eq!(StdlibDiscriminants::ParseInt as u32, 61);
        assert_eq!(StdlibDiscriminants::ParseJson as u32, 62);
        assert_eq!(StdlibDiscriminants::Pow as u32, 63);
        assert_eq!(StdlibDiscriminants::Push as u32, 64);
        assert_eq!(StdlibDiscriminants::PushFront as u32, 65);
        assert_eq!(StdlibDiscriminants::Raise as u32, 66);
        assert_eq!(StdlibDiscriminants::Remainder as u32, 67);
        assert_eq!(StdlibDiscriminants::Replace as u32, 68);
        assert_eq!(StdlibDiscriminants::ResolveArgs as u32, 69);
        assert_eq!(StdlibDiscriminants::ResolveDeep as u32, 70);
        assert_eq!(StdlibDiscriminants::ResolveHashmap as u32, 71);
        assert_eq!(StdlibDiscriminants::ResolveHashset as u32, 72);
        assert_eq!(StdlibDiscriminants::ResolveList as u32, 73);
        assert_eq!(StdlibDiscriminants::ResolveLoaderResults as u32, 74);
        assert_eq!(StdlibDiscriminants::ResolveQueryBranch as u32, 75);
        assert_eq!(StdlibDiscriminants::ResolveQueryLeaf as u32, 76);
        assert_eq!(StdlibDiscriminants::ResolveRecord as u32, 77);
        assert_eq!(StdlibDiscriminants::ResolveTree as u32, 78);
        assert_eq!(StdlibDiscriminants::Round as u32, 79);
        assert_eq!(StdlibDiscriminants::Scan as u32, 80);
        assert_eq!(StdlibDiscriminants::Sequence as u32, 81);
        assert_eq!(StdlibDiscriminants::Set as u32, 82);
        assert_eq!(StdlibDiscriminants::SetVariable as u32, 83);
        assert_eq!(StdlibDiscriminants::Skip as u32, 84);
        assert_eq!(StdlibDiscriminants::Slice as u32, 85);
        assert_eq!(StdlibDiscriminants::Split as u32, 86);
        assert_eq!(StdlibDiscriminants::StartsWith as u32, 87);
        assert_eq!(StdlibDiscriminants::StringifyJson as u32, 88);
        assert_eq!(StdlibDiscriminants::Subtract as u32, 89);
        assert_eq!(StdlibDiscriminants::Take as u32, 90);
        assert_eq!(StdlibDiscriminants::Throw as u32, 91);
        assert_eq!(StdlibDiscriminants::ToRequest as u32, 92);
        assert_eq!(StdlibDiscriminants::ToString as u32, 93);
        assert_eq!(StdlibDiscriminants::Urlencode as u32, 94);
        assert_eq!(StdlibDiscriminants::Unzip as u32, 95);
        assert_eq!(StdlibDiscriminants::Values as u32, 96);
        assert_eq!(StdlibDiscriminants::Zip as u32, 97);
    }
}
