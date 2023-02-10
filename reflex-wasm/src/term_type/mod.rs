// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, marker::PhantomData};

use reflex::{
    core::{
        Arity, DependencyList, Eagerness, Expression, GraphNode, Internable, NodeId, SerializeJson,
        StackOffset,
    },
    hash::HashId,
};
use serde_json::Value as JsonValue;
use strum_macros::EnumDiscriminants;

mod application;
mod boolean;
mod builtin;
mod cell;
mod condition;
mod constructor;
mod date;
mod effect;
mod float;
mod hashmap;
mod hashset;
mod int;
mod iterator;
mod lambda;
mod r#let;
mod list;
mod nil;
mod partial;
mod pointer;
mod record;
mod signal;
mod string;
mod symbol;
mod tree;
mod variable;

pub use application::*;
pub use boolean::*;
pub use builtin::*;
pub use cell::*;
pub use condition::*;
pub use constructor::*;
pub use date::*;
pub use effect::*;
pub use float::*;
pub use hashmap::*;
pub use hashset::*;
pub use int::*;
pub use iterator::*;
pub use lambda::*;
pub use list::*;
pub use nil::*;
pub use partial::*;
pub use pointer::*;
pub use r#let::*;
pub use record::*;
pub use signal::*;
pub use string::*;
pub use symbol::*;
pub use tree::*;
pub use variable::*;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    ArenaPointer, ArenaRef, PointerIter, Term,
};

const TERM_TYPE_DISCRIMINANT_SIZE: usize = std::mem::size_of::<u32>();

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[repr(C)]
pub enum TermType {
    Application(ApplicationTerm),
    Boolean(BooleanTerm),
    Builtin(BuiltinTerm),
    Cell(CellTerm),
    Condition(ConditionTerm),
    Constructor(ConstructorTerm),
    Date(DateTerm),
    Effect(EffectTerm),
    Float(FloatTerm),
    Hashmap(HashmapTerm),
    Hashset(HashsetTerm),
    Int(IntTerm),
    Lambda(LambdaTerm),
    Let(LetTerm),
    List(ListTerm),
    Nil(NilTerm),
    Partial(PartialTerm),
    Pointer(PointerTerm),
    Record(RecordTerm),
    Signal(SignalTerm),
    String(StringTerm),
    Symbol(SymbolTerm),
    Tree(TreeTerm),
    Variable(VariableTerm),
    EmptyIterator(EmptyIteratorTerm),
    EvaluateIterator(EvaluateIteratorTerm),
    FilterIterator(FilterIteratorTerm),
    FlattenIterator(FlattenIteratorTerm),
    HashmapKeysIterator(HashmapKeysIteratorTerm),
    HashmapValuesIterator(HashmapValuesIteratorTerm),
    IntegersIterator(IntegersIteratorTerm),
    IntersperseIterator(IntersperseIteratorTerm),
    MapIterator(MapIteratorTerm),
    OnceIterator(OnceIteratorTerm),
    RangeIterator(RangeIteratorTerm),
    RepeatIterator(RepeatIteratorTerm),
    SkipIterator(SkipIteratorTerm),
    TakeIterator(TakeIteratorTerm),
    ZipIterator(ZipIteratorTerm),
}
impl TryFrom<u32> for TermTypeDiscriminants {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            value if value == Self::Application as u32 => Ok(Self::Application),
            value if value == Self::Boolean as u32 => Ok(Self::Boolean),
            value if value == Self::Builtin as u32 => Ok(Self::Builtin),
            value if value == Self::Cell as u32 => Ok(Self::Cell),
            value if value == Self::Condition as u32 => Ok(Self::Condition),
            value if value == Self::Constructor as u32 => Ok(Self::Constructor),
            value if value == Self::Date as u32 => Ok(Self::Date),
            value if value == Self::Effect as u32 => Ok(Self::Effect),
            value if value == Self::Float as u32 => Ok(Self::Float),
            value if value == Self::Hashmap as u32 => Ok(Self::Hashmap),
            value if value == Self::Hashset as u32 => Ok(Self::Hashset),
            value if value == Self::Int as u32 => Ok(Self::Int),
            value if value == Self::Lambda as u32 => Ok(Self::Lambda),
            value if value == Self::Let as u32 => Ok(Self::Let),
            value if value == Self::List as u32 => Ok(Self::List),
            value if value == Self::Nil as u32 => Ok(Self::Nil),
            value if value == Self::Partial as u32 => Ok(Self::Partial),
            value if value == Self::Pointer as u32 => Ok(Self::Pointer),
            value if value == Self::Record as u32 => Ok(Self::Record),
            value if value == Self::Signal as u32 => Ok(Self::Signal),
            value if value == Self::String as u32 => Ok(Self::String),
            value if value == Self::Symbol as u32 => Ok(Self::Symbol),
            value if value == Self::Tree as u32 => Ok(Self::Tree),
            value if value == Self::Variable as u32 => Ok(Self::Variable),
            value if value == Self::EmptyIterator as u32 => Ok(Self::EmptyIterator),
            value if value == Self::EvaluateIterator as u32 => Ok(Self::EvaluateIterator),
            value if value == Self::FilterIterator as u32 => Ok(Self::FilterIterator),
            value if value == Self::FlattenIterator as u32 => Ok(Self::FlattenIterator),
            value if value == Self::HashmapKeysIterator as u32 => Ok(Self::HashmapKeysIterator),
            value if value == Self::HashmapValuesIterator as u32 => Ok(Self::HashmapValuesIterator),
            value if value == Self::IntegersIterator as u32 => Ok(Self::IntegersIterator),
            value if value == Self::IntersperseIterator as u32 => Ok(Self::IntersperseIterator),
            value if value == Self::MapIterator as u32 => Ok(Self::MapIterator),
            value if value == Self::OnceIterator as u32 => Ok(Self::OnceIterator),
            value if value == Self::RangeIterator as u32 => Ok(Self::RangeIterator),
            value if value == Self::RepeatIterator as u32 => Ok(Self::RepeatIterator),
            value if value == Self::SkipIterator as u32 => Ok(Self::SkipIterator),
            value if value == Self::TakeIterator as u32 => Ok(Self::TakeIterator),
            value if value == Self::ZipIterator as u32 => Ok(Self::ZipIterator),
            _ => Err(()),
        }
    }
}
impl TermSize for TermType {
    fn size_of(&self) -> usize {
        let discriminant_size = TERM_TYPE_DISCRIMINANT_SIZE;
        let value_size = match self {
            Self::Application(term) => term.size_of(),
            Self::Boolean(term) => term.size_of(),
            Self::Builtin(term) => term.size_of(),
            Self::Cell(term) => term.size_of(),
            Self::Condition(term) => term.size_of(),
            Self::Constructor(term) => term.size_of(),
            Self::Date(term) => term.size_of(),
            Self::Effect(term) => term.size_of(),
            Self::Float(term) => term.size_of(),
            Self::Hashmap(term) => term.size_of(),
            Self::Hashset(term) => term.size_of(),
            Self::Int(term) => term.size_of(),
            Self::Lambda(term) => term.size_of(),
            Self::Let(term) => term.size_of(),
            Self::List(term) => term.size_of(),
            Self::Nil(term) => term.size_of(),
            Self::Partial(term) => term.size_of(),
            Self::Pointer(term) => term.size_of(),
            Self::Record(term) => term.size_of(),
            Self::Signal(term) => term.size_of(),
            Self::String(term) => term.size_of(),
            Self::Symbol(term) => term.size_of(),
            Self::Tree(term) => term.size_of(),
            Self::Variable(term) => term.size_of(),
            Self::EmptyIterator(term) => term.size_of(),
            Self::EvaluateIterator(term) => term.size_of(),
            Self::FilterIterator(term) => term.size_of(),
            Self::FlattenIterator(term) => term.size_of(),
            Self::HashmapKeysIterator(term) => term.size_of(),
            Self::HashmapValuesIterator(term) => term.size_of(),
            Self::IntegersIterator(term) => term.size_of(),
            Self::IntersperseIterator(term) => term.size_of(),
            Self::MapIterator(term) => term.size_of(),
            Self::OnceIterator(term) => term.size_of(),
            Self::RangeIterator(term) => term.size_of(),
            Self::RepeatIterator(term) => term.size_of(),
            Self::SkipIterator(term) => term.size_of(),
            Self::TakeIterator(term) => term.size_of(),
            Self::ZipIterator(term) => term.size_of(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for TermType {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        match self {
            Self::Application(term) => hasher
                .write_u8(TermTypeDiscriminants::Application as u8)
                .hash(term, arena),
            Self::Boolean(term) => hasher
                .write_u8(TermTypeDiscriminants::Boolean as u8)
                .hash(term, arena),
            Self::Builtin(term) => hasher
                .write_u8(TermTypeDiscriminants::Builtin as u8)
                .hash(term, arena),
            Self::Cell(term) => hasher
                .write_u8(TermTypeDiscriminants::Cell as u8)
                .hash(term, arena),
            Self::Condition(term) => hasher
                .write_u8(TermTypeDiscriminants::Condition as u8)
                .hash(term, arena),
            Self::Constructor(term) => hasher
                .write_u8(TermTypeDiscriminants::Constructor as u8)
                .hash(term, arena),
            Self::Date(term) => hasher
                .write_u8(TermTypeDiscriminants::Date as u8)
                .hash(term, arena),
            Self::Effect(term) => hasher
                .write_u8(TermTypeDiscriminants::Effect as u8)
                .hash(term, arena),
            Self::Float(term) => hasher
                .write_u8(TermTypeDiscriminants::Float as u8)
                .hash(term, arena),
            Self::Hashmap(term) => hasher
                .write_u8(TermTypeDiscriminants::Hashmap as u8)
                .hash(term, arena),
            Self::Hashset(term) => hasher
                .write_u8(TermTypeDiscriminants::Hashset as u8)
                .hash(term, arena),
            Self::Int(term) => hasher
                .write_u8(TermTypeDiscriminants::Int as u8)
                .hash(term, arena),
            Self::Lambda(term) => hasher
                .write_u8(TermTypeDiscriminants::Lambda as u8)
                .hash(term, arena),
            Self::Let(term) => hasher
                .write_u8(TermTypeDiscriminants::Let as u8)
                .hash(term, arena),
            Self::List(term) => hasher
                .write_u8(TermTypeDiscriminants::List as u8)
                .hash(term, arena),
            Self::Nil(term) => hasher
                .write_u8(TermTypeDiscriminants::Nil as u8)
                .hash(term, arena),
            Self::Partial(term) => hasher
                .write_u8(TermTypeDiscriminants::Partial as u8)
                .hash(term, arena),
            Self::Pointer(term) => hasher
                .write_u8(TermTypeDiscriminants::Pointer as u8)
                .hash(term, arena),
            Self::Record(term) => hasher
                .write_u8(TermTypeDiscriminants::Record as u8)
                .hash(term, arena),
            Self::Signal(term) => hasher
                .write_u8(TermTypeDiscriminants::Signal as u8)
                .hash(term, arena),
            Self::String(term) => hasher
                .write_u8(TermTypeDiscriminants::String as u8)
                .hash(term, arena),
            Self::Symbol(term) => hasher
                .write_u8(TermTypeDiscriminants::Symbol as u8)
                .hash(term, arena),
            Self::Tree(term) => hasher
                .write_u8(TermTypeDiscriminants::Tree as u8)
                .hash(term, arena),
            Self::Variable(term) => hasher
                .write_u8(TermTypeDiscriminants::Variable as u8)
                .hash(term, arena),
            Self::EmptyIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::EmptyIterator as u8)
                .hash(term, arena),
            Self::EvaluateIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::EvaluateIterator as u8)
                .hash(term, arena),
            Self::FilterIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::FilterIterator as u8)
                .hash(term, arena),
            Self::FlattenIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::FlattenIterator as u8)
                .hash(term, arena),
            Self::HashmapKeysIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::HashmapKeysIterator as u8)
                .hash(term, arena),
            Self::HashmapValuesIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::HashmapValuesIterator as u8)
                .hash(term, arena),
            Self::IntegersIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::IntegersIterator as u8)
                .hash(term, arena),
            Self::IntersperseIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::IntersperseIterator as u8)
                .hash(term, arena),
            Self::MapIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::MapIterator as u8)
                .hash(term, arena),
            Self::OnceIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::OnceIterator as u8)
                .hash(term, arena),
            Self::RangeIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::RangeIterator as u8)
                .hash(term, arena),
            Self::RepeatIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::RepeatIterator as u8)
                .hash(term, arena),
            Self::SkipIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::SkipIterator as u8)
                .hash(term, arena),
            Self::TakeIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::TakeIterator as u8)
                .hash(term, arena),
            Self::ZipIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::ZipIterator as u8)
                .hash(term, arena),
        }
    }
}

pub enum TermPointerIterator {
    Application(ApplicationTermPointerIter),
    Boolean(BooleanTermPointerIter),
    Builtin(BuiltinTermPointerIter),
    Cell(CellTermPointerIter),
    Condition(ConditionTermPointerIter),
    Constructor(ConstructorTermPointerIter),
    Date(DateTermPointerIter),
    Effect(EffectTermPointerIter),
    Float(FloatTermPointerIter),
    Hashmap(HashmapTermPointerIter),
    Hashset(HashsetTermPointerIter),
    Int(IntTermPointerIter),
    Lambda(LambdaTermPointerIter),
    Let(LetTermPointerIter),
    List(ListTermPointerIter),
    Nil(NilTermPointerIter),
    Partial(PartialTermPointerIter),
    Pointer(PointerTermPointerIter),
    Record(RecordTermPointerIter),
    Signal(SignalTermPointerIter),
    String(StringTermPointerIter),
    Symbol(SymbolTermPointerIter),
    Tree(TreeTermPointerIter),
    Variable(VariableTermPointerIter),
    EmptyIterator(EmptyIteratorTermPointerIter),
    EvaluateIterator(EvaluateIteratorTermPointerIter),
    FilterIterator(FilterIteratorTermPointerIter),
    FlattenIterator(FlattenIteratorTermPointerIter),
    HashmapKeysIterator(HashmapKeysIteratorTermPointerIter),
    HashmapValuesIterator(HashmapValuesIteratorTermPointerIter),
    IntegersIterator(IntegersIteratorTermPointerIter),
    IntersperseIterator(IntersperseIteratorTermPointerIter),
    MapIterator(MapIteratorTermPointerIter),
    OnceIterator(OnceIteratorTermPointerIter),
    RangeIterator(RangeIteratorTermPointerIter),
    RepeatIterator(RepeatIteratorTermPointerIter),
    SkipIterator(SkipIteratorTermPointerIter),
    TakeIterator(TakeIteratorTermPointerIter),
    ZipIterator(ZipIteratorTermPointerIter),
}

impl Iterator for TermPointerIterator {
    type Item = ArenaPointer;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TermPointerIterator::Application(inner) => inner.next(),
            TermPointerIterator::Boolean(inner) => inner.next(),
            TermPointerIterator::Builtin(inner) => inner.next(),
            TermPointerIterator::Cell(inner) => inner.next(),
            TermPointerIterator::Condition(inner) => inner.next(),
            TermPointerIterator::Constructor(inner) => inner.next(),
            TermPointerIterator::Date(inner) => inner.next(),
            TermPointerIterator::Effect(inner) => inner.next(),
            TermPointerIterator::Float(inner) => inner.next(),
            TermPointerIterator::Hashmap(inner) => inner.next(),
            TermPointerIterator::Hashset(inner) => inner.next(),
            TermPointerIterator::Int(inner) => inner.next(),
            TermPointerIterator::Lambda(inner) => inner.next(),
            TermPointerIterator::Let(inner) => inner.next(),
            TermPointerIterator::List(inner) => inner.next(),
            TermPointerIterator::Nil(inner) => inner.next(),
            TermPointerIterator::Partial(inner) => inner.next(),
            TermPointerIterator::Pointer(inner) => inner.next(),
            TermPointerIterator::Record(inner) => inner.next(),
            TermPointerIterator::Signal(inner) => inner.next(),
            TermPointerIterator::String(inner) => inner.next(),
            TermPointerIterator::Symbol(inner) => inner.next(),
            TermPointerIterator::Tree(inner) => inner.next(),
            TermPointerIterator::Variable(inner) => inner.next(),
            TermPointerIterator::EmptyIterator(inner) => inner.next(),
            TermPointerIterator::EvaluateIterator(inner) => inner.next(),
            TermPointerIterator::FilterIterator(inner) => inner.next(),
            TermPointerIterator::FlattenIterator(inner) => inner.next(),
            TermPointerIterator::HashmapKeysIterator(inner) => inner.next(),
            TermPointerIterator::HashmapValuesIterator(inner) => inner.next(),
            TermPointerIterator::IntegersIterator(inner) => inner.next(),
            TermPointerIterator::IntersperseIterator(inner) => inner.next(),
            TermPointerIterator::MapIterator(inner) => inner.next(),
            TermPointerIterator::OnceIterator(inner) => inner.next(),
            TermPointerIterator::RangeIterator(inner) => inner.next(),
            TermPointerIterator::RepeatIterator(inner) => inner.next(),
            TermPointerIterator::SkipIterator(inner) => inner.next(),
            TermPointerIterator::TakeIterator(inner) => inner.next(),
            TermPointerIterator::ZipIterator(inner) => inner.next(),
        }
    }
}

impl<A: Arena + Clone> PointerIter for ArenaRef<Term, A> {
    type Iter<'a> = TermPointerIterator
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => TermPointerIterator::Application(
                self.as_typed_term::<ApplicationTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Boolean => {
                TermPointerIterator::Boolean(self.as_typed_term::<BooleanTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Builtin => {
                TermPointerIterator::Builtin(self.as_typed_term::<BuiltinTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Cell => {
                TermPointerIterator::Cell(self.as_typed_term::<CellTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Condition => TermPointerIterator::Condition(
                self.as_typed_term::<ConditionTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Constructor => TermPointerIterator::Constructor(
                self.as_typed_term::<ConstructorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Date => {
                TermPointerIterator::Date(self.as_typed_term::<DateTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Effect => {
                TermPointerIterator::Effect(self.as_typed_term::<EffectTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Float => {
                TermPointerIterator::Float(self.as_typed_term::<FloatTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Hashmap => {
                TermPointerIterator::Hashmap(self.as_typed_term::<HashmapTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Hashset => {
                TermPointerIterator::Hashset(self.as_typed_term::<HashsetTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Int => {
                TermPointerIterator::Int(self.as_typed_term::<IntTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Lambda => {
                TermPointerIterator::Lambda(self.as_typed_term::<LambdaTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Let => {
                TermPointerIterator::Let(self.as_typed_term::<LetTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::List => TermPointerIterator::List(PointerIter::iter(
                &self.as_typed_term::<ListTerm>().as_inner(),
            )),
            TermTypeDiscriminants::Nil => {
                TermPointerIterator::Nil(self.as_typed_term::<NilTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Partial => {
                TermPointerIterator::Partial(self.as_typed_term::<PartialTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Pointer => {
                TermPointerIterator::Pointer(self.as_typed_term::<PointerTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Record => {
                TermPointerIterator::Record(self.as_typed_term::<RecordTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Signal => {
                TermPointerIterator::Signal(self.as_typed_term::<SignalTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::String => {
                TermPointerIterator::String(self.as_typed_term::<StringTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Symbol => {
                TermPointerIterator::Symbol(self.as_typed_term::<SymbolTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Tree => TermPointerIterator::Tree(PointerIter::iter(
                &self.as_typed_term::<TreeTerm>().as_inner(),
            )),
            TermTypeDiscriminants::Variable => TermPointerIterator::Variable(
                self.as_typed_term::<VariableTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::EmptyIterator => TermPointerIterator::EmptyIterator(
                self.as_typed_term::<EmptyIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::EvaluateIterator => TermPointerIterator::EvaluateIterator(
                self.as_typed_term::<EvaluateIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::FilterIterator => TermPointerIterator::FilterIterator(
                self.as_typed_term::<FilterIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::FlattenIterator => TermPointerIterator::FlattenIterator(
                self.as_typed_term::<FlattenIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::HashmapKeysIterator => TermPointerIterator::HashmapKeysIterator(
                self.as_typed_term::<HashmapKeysIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::HashmapValuesIterator => {
                TermPointerIterator::HashmapValuesIterator(
                    self.as_typed_term::<HashmapValuesIteratorTerm>()
                        .as_inner()
                        .iter(),
                )
            }
            TermTypeDiscriminants::IntegersIterator => TermPointerIterator::IntegersIterator(
                self.as_typed_term::<IntegersIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::IntersperseIterator => TermPointerIterator::IntersperseIterator(
                self.as_typed_term::<IntersperseIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::MapIterator => TermPointerIterator::MapIterator(
                self.as_typed_term::<MapIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::OnceIterator => TermPointerIterator::OnceIterator(
                self.as_typed_term::<OnceIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::RangeIterator => TermPointerIterator::RangeIterator(
                self.as_typed_term::<RangeIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::RepeatIterator => TermPointerIterator::RepeatIterator(
                self.as_typed_term::<RepeatIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::SkipIterator => TermPointerIterator::SkipIterator(
                self.as_typed_term::<SkipIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::TakeIterator => TermPointerIterator::TakeIterator(
                self.as_typed_term::<TakeIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::ZipIterator => TermPointerIterator::ZipIterator(
                self.as_typed_term::<ZipIteratorTerm>().as_inner().iter(),
            ),
        }
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<Term, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .should_intern(eager),
        }
    }
}

impl<'a> Into<Option<&'a ApplicationTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ApplicationTerm> {
        match self {
            TermType::Application(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a BooleanTerm>> for &'a TermType {
    fn into(self) -> Option<&'a BooleanTerm> {
        match self {
            TermType::Boolean(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a BuiltinTerm>> for &'a TermType {
    fn into(self) -> Option<&'a BuiltinTerm> {
        match self {
            TermType::Builtin(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a CellTerm>> for &'a TermType {
    fn into(self) -> Option<&'a CellTerm> {
        match self {
            TermType::Cell(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ConditionTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ConditionTerm> {
        match self {
            TermType::Condition(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ConstructorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ConstructorTerm> {
        match self {
            TermType::Constructor(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a DateTerm>> for &'a TermType {
    fn into(self) -> Option<&'a DateTerm> {
        match self {
            TermType::Date(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a EffectTerm>> for &'a TermType {
    fn into(self) -> Option<&'a EffectTerm> {
        match self {
            TermType::Effect(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a FloatTerm>> for &'a TermType {
    fn into(self) -> Option<&'a FloatTerm> {
        match self {
            TermType::Float(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashmapTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashmapTerm> {
        match self {
            TermType::Hashmap(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashsetTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashsetTerm> {
        match self {
            TermType::Hashset(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a IntTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IntTerm> {
        match self {
            TermType::Int(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a LambdaTerm>> for &'a TermType {
    fn into(self) -> Option<&'a LambdaTerm> {
        match self {
            TermType::Lambda(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a LetTerm>> for &'a TermType {
    fn into(self) -> Option<&'a LetTerm> {
        match self {
            TermType::Let(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ListTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ListTerm> {
        match self {
            TermType::List(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a NilTerm>> for &'a TermType {
    fn into(self) -> Option<&'a NilTerm> {
        match self {
            TermType::Nil(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a PartialTerm>> for &'a TermType {
    fn into(self) -> Option<&'a PartialTerm> {
        match self {
            TermType::Partial(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a PointerTerm>> for &'a TermType {
    fn into(self) -> Option<&'a PointerTerm> {
        match self {
            TermType::Pointer(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a RecordTerm>> for &'a TermType {
    fn into(self) -> Option<&'a RecordTerm> {
        match self {
            TermType::Record(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a SignalTerm>> for &'a TermType {
    fn into(self) -> Option<&'a SignalTerm> {
        match self {
            TermType::Signal(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a StringTerm>> for &'a TermType {
    fn into(self) -> Option<&'a StringTerm> {
        match self {
            TermType::String(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a SymbolTerm>> for &'a TermType {
    fn into(self) -> Option<&'a SymbolTerm> {
        match self {
            TermType::Symbol(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a TreeTerm>> for &'a TermType {
    fn into(self) -> Option<&'a TreeTerm> {
        match self {
            TermType::Tree(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a VariableTerm>> for &'a TermType {
    fn into(self) -> Option<&'a VariableTerm> {
        match self {
            TermType::Variable(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a EmptyIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a EmptyIteratorTerm> {
        match self {
            TermType::EmptyIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a EvaluateIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a EvaluateIteratorTerm> {
        match self {
            TermType::EvaluateIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a FilterIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a FilterIteratorTerm> {
        match self {
            TermType::FilterIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a FlattenIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a FlattenIteratorTerm> {
        match self {
            TermType::FlattenIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashmapKeysIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashmapKeysIteratorTerm> {
        match self {
            TermType::HashmapKeysIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashmapValuesIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashmapValuesIteratorTerm> {
        match self {
            TermType::HashmapValuesIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a IntegersIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IntegersIteratorTerm> {
        match self {
            TermType::IntegersIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a IntersperseIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IntersperseIteratorTerm> {
        match self {
            TermType::IntersperseIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a MapIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a MapIteratorTerm> {
        match self {
            TermType::MapIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a OnceIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a OnceIteratorTerm> {
        match self {
            TermType::OnceIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a RangeIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a RangeIteratorTerm> {
        match self {
            TermType::RangeIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a RepeatIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a RepeatIteratorTerm> {
        match self {
            TermType::RepeatIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a SkipIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a SkipIteratorTerm> {
        match self {
            TermType::SkipIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a TakeIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a TakeIteratorTerm> {
        match self {
            TermType::TakeIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ZipIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ZipIteratorTerm> {
        match self {
            TermType::ZipIterator(term) => Some(term),
            _ => None,
        }
    }
}

impl<A: Arena + Clone> ArenaRef<Term, A> {
    pub fn arity(&self) -> Option<Arity> {
        match &self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Builtin => {
                self.as_typed_term::<BuiltinTerm>().as_inner().arity()
            }
            TermTypeDiscriminants::Constructor => {
                Some(self.as_typed_term::<ConstructorTerm>().as_inner().arity())
            }
            TermTypeDiscriminants::Lambda => {
                Some(self.as_typed_term::<LambdaTerm>().as_inner().arity())
            }
            TermTypeDiscriminants::Partial => {
                self.as_typed_term::<PartialTerm>().as_inner().arity()
            }
            _ => None,
        }
    }
}

impl<A: Arena + Clone> Eq for ArenaRef<Term, A> {}
impl<A: Arena + Clone> PartialEq for ArenaRef<Term, A> {
    fn eq(&self, other: &Self) -> bool {
        if self.read_value(|term| term.header.hash) != other.read_value(|term| term.header.hash) {
            return false;
        }
        match (
            self.read_value(|term| term.type_id()),
            other.read_value(|term| term.type_id()),
        ) {
            (TermTypeDiscriminants::Application, TermTypeDiscriminants::Application) => {
                self.as_typed_term::<ApplicationTerm>().as_inner()
                    == other.as_typed_term::<ApplicationTerm>().as_inner()
            }
            (TermTypeDiscriminants::Boolean, TermTypeDiscriminants::Boolean) => {
                self.as_typed_term::<BooleanTerm>().as_inner()
                    == other.as_typed_term::<BooleanTerm>().as_inner()
            }
            (TermTypeDiscriminants::Builtin, TermTypeDiscriminants::Builtin) => {
                self.as_typed_term::<BuiltinTerm>().as_inner()
                    == other.as_typed_term::<BuiltinTerm>().as_inner()
            }
            (TermTypeDiscriminants::Cell, TermTypeDiscriminants::Cell) => {
                self.as_typed_term::<CellTerm>().as_inner()
                    == other.as_typed_term::<CellTerm>().as_inner()
            }
            (TermTypeDiscriminants::Condition, TermTypeDiscriminants::Condition) => {
                self.as_typed_term::<ConditionTerm>().as_inner()
                    == other.as_typed_term::<ConditionTerm>().as_inner()
            }
            (TermTypeDiscriminants::Constructor, TermTypeDiscriminants::Constructor) => {
                self.as_typed_term::<ConstructorTerm>().as_inner()
                    == other.as_typed_term::<ConstructorTerm>().as_inner()
            }
            (TermTypeDiscriminants::Date, TermTypeDiscriminants::Date) => {
                self.as_typed_term::<DateTerm>().as_inner()
                    == other.as_typed_term::<DateTerm>().as_inner()
            }
            (TermTypeDiscriminants::Effect, TermTypeDiscriminants::Effect) => {
                self.as_typed_term::<EffectTerm>().as_inner()
                    == other.as_typed_term::<EffectTerm>().as_inner()
            }
            (TermTypeDiscriminants::Float, TermTypeDiscriminants::Float) => {
                self.as_typed_term::<FloatTerm>().as_inner()
                    == other.as_typed_term::<FloatTerm>().as_inner()
            }
            (TermTypeDiscriminants::Hashmap, TermTypeDiscriminants::Hashmap) => {
                self.as_typed_term::<HashmapTerm>().as_inner()
                    == other.as_typed_term::<HashmapTerm>().as_inner()
            }
            (TermTypeDiscriminants::Hashset, TermTypeDiscriminants::Hashset) => {
                self.as_typed_term::<HashsetTerm>().as_inner()
                    == other.as_typed_term::<HashsetTerm>().as_inner()
            }
            (TermTypeDiscriminants::Int, TermTypeDiscriminants::Int) => {
                self.as_typed_term::<IntTerm>().as_inner()
                    == other.as_typed_term::<IntTerm>().as_inner()
            }
            (TermTypeDiscriminants::Lambda, TermTypeDiscriminants::Lambda) => {
                self.as_typed_term::<LambdaTerm>().as_inner()
                    == other.as_typed_term::<LambdaTerm>().as_inner()
            }
            (TermTypeDiscriminants::Let, TermTypeDiscriminants::Let) => {
                self.as_typed_term::<LetTerm>().as_inner()
                    == other.as_typed_term::<LetTerm>().as_inner()
            }
            (TermTypeDiscriminants::List, TermTypeDiscriminants::List) => {
                self.as_typed_term::<ListTerm>().as_inner()
                    == other.as_typed_term::<ListTerm>().as_inner()
            }
            (TermTypeDiscriminants::Nil, TermTypeDiscriminants::Nil) => {
                self.as_typed_term::<NilTerm>().as_inner()
                    == other.as_typed_term::<NilTerm>().as_inner()
            }
            (TermTypeDiscriminants::Partial, TermTypeDiscriminants::Partial) => {
                self.as_typed_term::<PartialTerm>().as_inner()
                    == other.as_typed_term::<PartialTerm>().as_inner()
            }
            (TermTypeDiscriminants::Pointer, TermTypeDiscriminants::Pointer) => {
                self.as_typed_term::<PointerTerm>().as_inner()
                    == other.as_typed_term::<PointerTerm>().as_inner()
            }
            (TermTypeDiscriminants::Record, TermTypeDiscriminants::Record) => {
                self.as_typed_term::<RecordTerm>().as_inner()
                    == other.as_typed_term::<RecordTerm>().as_inner()
            }
            (TermTypeDiscriminants::Signal, TermTypeDiscriminants::Signal) => {
                self.as_typed_term::<SignalTerm>().as_inner()
                    == other.as_typed_term::<SignalTerm>().as_inner()
            }
            (TermTypeDiscriminants::String, TermTypeDiscriminants::String) => {
                self.as_typed_term::<StringTerm>().as_inner()
                    == other.as_typed_term::<StringTerm>().as_inner()
            }
            (TermTypeDiscriminants::Symbol, TermTypeDiscriminants::Symbol) => {
                self.as_typed_term::<SymbolTerm>().as_inner()
                    == other.as_typed_term::<SymbolTerm>().as_inner()
            }
            (TermTypeDiscriminants::Tree, TermTypeDiscriminants::Tree) => {
                self.as_typed_term::<TreeTerm>().as_inner()
                    == other.as_typed_term::<TreeTerm>().as_inner()
            }
            (TermTypeDiscriminants::Variable, TermTypeDiscriminants::Variable) => {
                self.as_typed_term::<VariableTerm>().as_inner()
                    == other.as_typed_term::<VariableTerm>().as_inner()
            }
            (TermTypeDiscriminants::EmptyIterator, TermTypeDiscriminants::EmptyIterator) => {
                self.as_typed_term::<EmptyIteratorTerm>().as_inner()
                    == other.as_typed_term::<EmptyIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::EvaluateIterator, TermTypeDiscriminants::EvaluateIterator) => {
                self.as_typed_term::<EvaluateIteratorTerm>().as_inner()
                    == other.as_typed_term::<EvaluateIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::FilterIterator, TermTypeDiscriminants::FilterIterator) => {
                self.as_typed_term::<FilterIteratorTerm>().as_inner()
                    == other.as_typed_term::<FilterIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::FlattenIterator, TermTypeDiscriminants::FlattenIterator) => {
                self.as_typed_term::<FlattenIteratorTerm>().as_inner()
                    == other.as_typed_term::<FlattenIteratorTerm>().as_inner()
            }
            (
                TermTypeDiscriminants::HashmapKeysIterator,
                TermTypeDiscriminants::HashmapKeysIterator,
            ) => {
                self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner()
                    == other.as_typed_term::<HashmapKeysIteratorTerm>().as_inner()
            }
            (
                TermTypeDiscriminants::HashmapValuesIterator,
                TermTypeDiscriminants::HashmapValuesIterator,
            ) => {
                self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner()
                    == other
                        .as_typed_term::<HashmapValuesIteratorTerm>()
                        .as_inner()
            }
            (TermTypeDiscriminants::IntegersIterator, TermTypeDiscriminants::IntegersIterator) => {
                self.as_typed_term::<IntegersIteratorTerm>().as_inner()
                    == other.as_typed_term::<IntegersIteratorTerm>().as_inner()
            }
            (
                TermTypeDiscriminants::IntersperseIterator,
                TermTypeDiscriminants::IntersperseIterator,
            ) => {
                self.as_typed_term::<IntersperseIteratorTerm>().as_inner()
                    == other.as_typed_term::<IntersperseIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::MapIterator, TermTypeDiscriminants::MapIterator) => {
                self.as_typed_term::<MapIteratorTerm>().as_inner()
                    == other.as_typed_term::<MapIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::OnceIterator, TermTypeDiscriminants::OnceIterator) => {
                self.as_typed_term::<OnceIteratorTerm>().as_inner()
                    == other.as_typed_term::<OnceIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::RangeIterator, TermTypeDiscriminants::RangeIterator) => {
                self.as_typed_term::<RangeIteratorTerm>().as_inner()
                    == other.as_typed_term::<RangeIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::RepeatIterator, TermTypeDiscriminants::RepeatIterator) => {
                self.as_typed_term::<RepeatIteratorTerm>().as_inner()
                    == other.as_typed_term::<RepeatIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::SkipIterator, TermTypeDiscriminants::SkipIterator) => {
                self.as_typed_term::<SkipIteratorTerm>().as_inner()
                    == other.as_typed_term::<SkipIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::TakeIterator, TermTypeDiscriminants::TakeIterator) => {
                self.as_typed_term::<TakeIteratorTerm>().as_inner()
                    == other.as_typed_term::<TakeIteratorTerm>().as_inner()
            }
            (TermTypeDiscriminants::ZipIterator, TermTypeDiscriminants::ZipIterator) => {
                self.as_typed_term::<ZipIteratorTerm>().as_inner()
                    == other.as_typed_term::<ZipIteratorTerm>().as_inner()
            }
            _ => false,
        }
    }
}

pub type WasmExpression<A> = ArenaRef<Term, A>;

impl<A: Arena + Clone> Expression for ArenaRef<Term, A> {
    type String = ArenaRef<TypedTerm<StringTerm>, A>;
    type Builtin = Stdlib;
    type Signal = ArenaRef<TypedTerm<ConditionTerm>, A>;
    type SignalList = ArenaRef<TypedTerm<TreeTerm>, A>;
    type StructPrototype = ArenaRef<TypedTerm<ListTerm>, A>;
    type ExpressionList = ArenaRef<TypedTerm<ListTerm>, A>;
    type NilTerm = ArenaRef<TypedTerm<NilTerm>, A>;
    type BooleanTerm = ArenaRef<TypedTerm<BooleanTerm>, A>;
    type IntTerm = ArenaRef<TypedTerm<IntTerm>, A>;
    type FloatTerm = ArenaRef<TypedTerm<FloatTerm>, A>;
    type StringTerm = ArenaRef<TypedTerm<StringTerm>, A>;
    type SymbolTerm = ArenaRef<TypedTerm<SymbolTerm>, A>;
    type VariableTerm = ArenaRef<TypedTerm<VariableTerm>, A>;
    type EffectTerm = ArenaRef<TypedTerm<EffectTerm>, A>;
    type LetTerm = ArenaRef<TypedTerm<LetTerm>, A>;
    type LambdaTerm = ArenaRef<TypedTerm<LambdaTerm>, A>;
    type ApplicationTerm = ArenaRef<TypedTerm<ApplicationTerm>, A>;
    type PartialApplicationTerm = ArenaRef<TypedTerm<PartialTerm>, A>;
    // FIXME: implement recursive term type
    type RecursiveTerm = ArenaRef<TypedTerm<NilTerm>, A>;
    type BuiltinTerm = ArenaRef<TypedTerm<BuiltinTerm>, A>;
    // FIXME: remove compiled function term
    type CompiledFunctionTerm = ArenaRef<TypedTerm<NilTerm>, A>;
    type RecordTerm = ArenaRef<TypedTerm<RecordTerm>, A>;
    type ConstructorTerm = ArenaRef<TypedTerm<ConstructorTerm>, A>;
    type ListTerm = ArenaRef<TypedTerm<ListTerm>, A>;
    type HashmapTerm = ArenaRef<TypedTerm<HashmapTerm>, A>;
    type HashsetTerm = ArenaRef<TypedTerm<HashsetTerm>, A>;
    type SignalTerm = ArenaRef<TypedTerm<SignalTerm>, A>;

    type StringRef<'a> = ArenaRef<TypedTerm<StringTerm>, A> where Self: 'a;
    type SignalRef<'a> = ArenaRef<TypedTerm<ConditionTerm>, A> where Self::Signal: 'a, Self: 'a;
    type StructPrototypeRef<'a> = ArenaRef<TypedTerm<ListTerm>, A> where Self::StructPrototype: 'a, Self: 'a;
    type SignalListRef<'a> = ArenaRef<TypedTerm<TreeTerm>, A> where Self::SignalList: 'a, Self: 'a;
    type ExpressionListRef<'a> = ArenaRef<TypedTerm<ListTerm>, A> where Self::ExpressionList: 'a, Self: 'a;
    type ExpressionRef<'a> = ArenaRef<Term, A> where Self: 'a;
}

impl<A: Arena + Clone> NodeId for ArenaRef<Term, A> {
    fn id(&self) -> HashId {
        self.read_value(|term| term.id())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<Term, A> {
    fn size(&self) -> usize {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                self.as_typed_term::<ApplicationTerm>().as_inner().size()
            }
            TermTypeDiscriminants::Boolean => self.as_typed_term::<BooleanTerm>().as_inner().size(),
            TermTypeDiscriminants::Builtin => self.as_typed_term::<BuiltinTerm>().as_inner().size(),
            TermTypeDiscriminants::Cell => self.as_typed_term::<CellTerm>().as_inner().size(),
            TermTypeDiscriminants::Condition => {
                self.as_typed_term::<ConditionTerm>().as_inner().size()
            }
            TermTypeDiscriminants::Constructor => {
                self.as_typed_term::<ConstructorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::Date => self.as_typed_term::<DateTerm>().as_inner().size(),
            TermTypeDiscriminants::Effect => self.as_typed_term::<EffectTerm>().as_inner().size(),
            TermTypeDiscriminants::Float => self.as_typed_term::<FloatTerm>().as_inner().size(),
            TermTypeDiscriminants::Hashmap => self.as_typed_term::<HashmapTerm>().as_inner().size(),
            TermTypeDiscriminants::Hashset => self.as_typed_term::<HashsetTerm>().as_inner().size(),
            TermTypeDiscriminants::Int => self.as_typed_term::<IntTerm>().as_inner().size(),
            TermTypeDiscriminants::Lambda => self.as_typed_term::<LambdaTerm>().as_inner().size(),
            TermTypeDiscriminants::Let => self.as_typed_term::<LetTerm>().as_inner().size(),
            TermTypeDiscriminants::List => self.as_typed_term::<ListTerm>().as_inner().size(),
            TermTypeDiscriminants::Nil => self.as_typed_term::<NilTerm>().as_inner().size(),
            TermTypeDiscriminants::Partial => self.as_typed_term::<PartialTerm>().as_inner().size(),
            TermTypeDiscriminants::Pointer => self.as_typed_term::<PointerTerm>().as_inner().size(),
            TermTypeDiscriminants::Record => self.as_typed_term::<RecordTerm>().as_inner().size(),
            TermTypeDiscriminants::Signal => self.as_typed_term::<SignalTerm>().as_inner().size(),
            TermTypeDiscriminants::String => self.as_typed_term::<StringTerm>().as_inner().size(),
            TermTypeDiscriminants::Symbol => self.as_typed_term::<SymbolTerm>().as_inner().size(),
            TermTypeDiscriminants::Tree => self.as_typed_term::<TreeTerm>().as_inner().size(),
            TermTypeDiscriminants::Variable => {
                self.as_typed_term::<VariableTerm>().as_inner().size()
            }
            TermTypeDiscriminants::EmptyIterator => {
                self.as_typed_term::<EmptyIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .size(),
            TermTypeDiscriminants::FilterIterator => {
                self.as_typed_term::<FilterIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .size(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .size(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .size(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .size(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .size(),
            TermTypeDiscriminants::MapIterator => {
                self.as_typed_term::<MapIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::OnceIterator => {
                self.as_typed_term::<OnceIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::RangeIterator => {
                self.as_typed_term::<RangeIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::RepeatIterator => {
                self.as_typed_term::<RepeatIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::SkipIterator => {
                self.as_typed_term::<SkipIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::TakeIterator => {
                self.as_typed_term::<TakeIteratorTerm>().as_inner().size()
            }
            TermTypeDiscriminants::ZipIterator => {
                self.as_typed_term::<ZipIteratorTerm>().as_inner().size()
            }
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Cell => {
                self.as_typed_term::<CellTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Date => {
                self.as_typed_term::<DateTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Float => {
                self.as_typed_term::<FloatTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Int => {
                self.as_typed_term::<IntTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Let => {
                self.as_typed_term::<LetTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::List => {
                self.as_typed_term::<ListTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Nil => {
                self.as_typed_term::<NilTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::Tree => {
                self.as_typed_term::<TreeTerm>().as_inner().capture_depth()
            }
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .capture_depth(),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .capture_depth(),
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Cell => {
                self.as_typed_term::<CellTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Date => {
                self.as_typed_term::<DateTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Int => {
                self.as_typed_term::<IntTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Let => {
                self.as_typed_term::<LetTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::List => {
                self.as_typed_term::<ListTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::Nil => {
                self.as_typed_term::<NilTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::Tree => {
                self.as_typed_term::<TreeTerm>().as_inner().free_variables()
            }
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .free_variables(),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .free_variables(),
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .count_variable_usages(offset),
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .dynamic_dependencies(deep),
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .has_dynamic_dependencies(deep),
        }
    }
    fn is_static(&self) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::Boolean => {
                self.as_typed_term::<BooleanTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Builtin => {
                self.as_typed_term::<BuiltinTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Cell => self.as_typed_term::<CellTerm>().as_inner().is_static(),
            TermTypeDiscriminants::Condition => {
                self.as_typed_term::<ConditionTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::Date => self.as_typed_term::<DateTerm>().as_inner().is_static(),
            TermTypeDiscriminants::Effect => {
                self.as_typed_term::<EffectTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Float => {
                self.as_typed_term::<FloatTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Hashmap => {
                self.as_typed_term::<HashmapTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Hashset => {
                self.as_typed_term::<HashsetTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Int => self.as_typed_term::<IntTerm>().as_inner().is_static(),
            TermTypeDiscriminants::Lambda => {
                self.as_typed_term::<LambdaTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Let => self.as_typed_term::<LetTerm>().as_inner().is_static(),
            TermTypeDiscriminants::List => self.as_typed_term::<ListTerm>().as_inner().is_static(),
            TermTypeDiscriminants::Nil => self.as_typed_term::<NilTerm>().as_inner().is_static(),
            TermTypeDiscriminants::Partial => {
                self.as_typed_term::<PartialTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Pointer => {
                self.as_typed_term::<PointerTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Record => {
                self.as_typed_term::<RecordTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Signal => {
                self.as_typed_term::<SignalTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::String => {
                self.as_typed_term::<StringTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Symbol => {
                self.as_typed_term::<SymbolTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::Tree => self.as_typed_term::<TreeTerm>().as_inner().is_static(),
            TermTypeDiscriminants::Variable => {
                self.as_typed_term::<VariableTerm>().as_inner().is_static()
            }
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .is_static(),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .is_static(),
        }
    }
    fn is_atomic(&self) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::Boolean => {
                self.as_typed_term::<BooleanTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Builtin => {
                self.as_typed_term::<BuiltinTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Cell => self.as_typed_term::<CellTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::Condition => {
                self.as_typed_term::<ConditionTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::Date => self.as_typed_term::<DateTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::Effect => {
                self.as_typed_term::<EffectTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Float => {
                self.as_typed_term::<FloatTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Hashmap => {
                self.as_typed_term::<HashmapTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Hashset => {
                self.as_typed_term::<HashsetTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Int => self.as_typed_term::<IntTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::Lambda => {
                self.as_typed_term::<LambdaTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Let => self.as_typed_term::<LetTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::List => self.as_typed_term::<ListTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::Nil => self.as_typed_term::<NilTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::Partial => {
                self.as_typed_term::<PartialTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Pointer => {
                self.as_typed_term::<PointerTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Record => {
                self.as_typed_term::<RecordTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Signal => {
                self.as_typed_term::<SignalTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::String => {
                self.as_typed_term::<StringTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Symbol => {
                self.as_typed_term::<SymbolTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::Tree => self.as_typed_term::<TreeTerm>().as_inner().is_atomic(),
            TermTypeDiscriminants::Variable => {
                self.as_typed_term::<VariableTerm>().as_inner().is_atomic()
            }
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .is_atomic(),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .is_static(),
        }
    }
    fn is_complex(&self) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::Boolean => {
                self.as_typed_term::<BooleanTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Builtin => {
                self.as_typed_term::<BuiltinTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Cell => self.as_typed_term::<CellTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::Date => self.as_typed_term::<DateTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::Effect => {
                self.as_typed_term::<EffectTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Float => {
                self.as_typed_term::<FloatTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Hashmap => {
                self.as_typed_term::<HashmapTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Hashset => {
                self.as_typed_term::<HashsetTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Int => self.as_typed_term::<IntTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::Lambda => {
                self.as_typed_term::<LambdaTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Let => self.as_typed_term::<LetTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::List => self.as_typed_term::<ListTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::Nil => self.as_typed_term::<NilTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::Partial => {
                self.as_typed_term::<PartialTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Pointer => {
                self.as_typed_term::<PointerTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Record => {
                self.as_typed_term::<RecordTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Signal => {
                self.as_typed_term::<SignalTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::String => {
                self.as_typed_term::<StringTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Symbol => {
                self.as_typed_term::<SymbolTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::Tree => self.as_typed_term::<TreeTerm>().as_inner().is_complex(),
            TermTypeDiscriminants::Variable => {
                self.as_typed_term::<VariableTerm>().as_inner().is_complex()
            }
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .is_complex(),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .is_complex(),
        }
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<Term, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                self.as_typed_term::<ApplicationTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Boolean => {
                self.as_typed_term::<BooleanTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Builtin => {
                self.as_typed_term::<BuiltinTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Cell => self.as_typed_term::<CellTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Condition => {
                self.as_typed_term::<ConditionTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Constructor => {
                self.as_typed_term::<ConstructorTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Date => self.as_typed_term::<DateTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Effect => {
                self.as_typed_term::<EffectTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Float => self.as_typed_term::<FloatTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Hashmap => {
                self.as_typed_term::<HashmapTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Hashset => {
                self.as_typed_term::<HashsetTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Int => self.as_typed_term::<IntTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Lambda => {
                self.as_typed_term::<LambdaTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Let => self.as_typed_term::<LetTerm>().as_inner().to_json(),
            TermTypeDiscriminants::List => self.as_typed_term::<ListTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Nil => self.as_typed_term::<NilTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Partial => {
                self.as_typed_term::<PartialTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Pointer => {
                self.as_typed_term::<PointerTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Record => {
                self.as_typed_term::<RecordTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Signal => {
                self.as_typed_term::<SignalTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::String => {
                self.as_typed_term::<StringTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Symbol => {
                self.as_typed_term::<SymbolTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::Tree => self.as_typed_term::<TreeTerm>().as_inner().to_json(),
            TermTypeDiscriminants::Variable => {
                self.as_typed_term::<VariableTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::MapIterator => {
                self.as_typed_term::<MapIteratorTerm>().as_inner().to_json()
            }
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .to_json(),
            TermTypeDiscriminants::ZipIterator => {
                self.as_typed_term::<ZipIteratorTerm>().as_inner().to_json()
            }
        }
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.id() == target.id() {
            return Ok(None);
        }
        match (
            &self.read_value(|term| term.type_id()),
            &target.read_value(|term| term.type_id()),
        ) {
            (TermTypeDiscriminants::Application, TermTypeDiscriminants::Application) => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<ApplicationTerm>().as_inner()),
            (TermTypeDiscriminants::Boolean, TermTypeDiscriminants::Boolean) => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<BooleanTerm>().as_inner()),
            (TermTypeDiscriminants::Builtin, TermTypeDiscriminants::Builtin) => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<BuiltinTerm>().as_inner()),
            (TermTypeDiscriminants::Cell, TermTypeDiscriminants::Cell) => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<CellTerm>().as_inner()),
            (TermTypeDiscriminants::Condition, TermTypeDiscriminants::Condition) => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<ConditionTerm>().as_inner()),
            (TermTypeDiscriminants::Constructor, TermTypeDiscriminants::Constructor) => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<ConstructorTerm>().as_inner()),
            (TermTypeDiscriminants::Date, TermTypeDiscriminants::Date) => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<DateTerm>().as_inner()),
            (TermTypeDiscriminants::Effect, TermTypeDiscriminants::Effect) => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<EffectTerm>().as_inner()),
            (TermTypeDiscriminants::Float, TermTypeDiscriminants::Float) => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<FloatTerm>().as_inner()),
            (TermTypeDiscriminants::Hashmap, TermTypeDiscriminants::Hashmap) => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<HashmapTerm>().as_inner()),
            (TermTypeDiscriminants::Hashset, TermTypeDiscriminants::Hashset) => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<HashsetTerm>().as_inner()),
            (TermTypeDiscriminants::Int, TermTypeDiscriminants::Int) => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<IntTerm>().as_inner()),
            (TermTypeDiscriminants::Lambda, TermTypeDiscriminants::Lambda) => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<LambdaTerm>().as_inner()),
            (TermTypeDiscriminants::Let, TermTypeDiscriminants::Let) => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<LetTerm>().as_inner()),
            (TermTypeDiscriminants::List, TermTypeDiscriminants::List) => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<ListTerm>().as_inner()),
            (TermTypeDiscriminants::Nil, TermTypeDiscriminants::Nil) => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<NilTerm>().as_inner()),
            (TermTypeDiscriminants::Partial, TermTypeDiscriminants::Partial) => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<PartialTerm>().as_inner()),
            (TermTypeDiscriminants::Pointer, TermTypeDiscriminants::Pointer) => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<PointerTerm>().as_inner()),
            (TermTypeDiscriminants::Record, TermTypeDiscriminants::Record) => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<RecordTerm>().as_inner()),
            (TermTypeDiscriminants::Signal, TermTypeDiscriminants::Signal) => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<SignalTerm>().as_inner()),
            (TermTypeDiscriminants::String, TermTypeDiscriminants::String) => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<StringTerm>().as_inner()),
            (TermTypeDiscriminants::Symbol, TermTypeDiscriminants::Symbol) => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<SymbolTerm>().as_inner()),
            (TermTypeDiscriminants::Tree, TermTypeDiscriminants::Tree) => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<TreeTerm>().as_inner()),
            (TermTypeDiscriminants::Variable, TermTypeDiscriminants::Variable) => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<VariableTerm>().as_inner()),
            (TermTypeDiscriminants::EmptyIterator, TermTypeDiscriminants::EmptyIterator) => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<EmptyIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::EvaluateIterator, TermTypeDiscriminants::EvaluateIterator) => {
                self.as_typed_term::<EvaluateIteratorTerm>()
                    .as_inner()
                    .patch(&target.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            (TermTypeDiscriminants::FilterIterator, TermTypeDiscriminants::FilterIterator) => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<FilterIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::FlattenIterator, TermTypeDiscriminants::FlattenIterator) => {
                self.as_typed_term::<FlattenIteratorTerm>()
                    .as_inner()
                    .patch(&target.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            (
                TermTypeDiscriminants::HashmapKeysIterator,
                TermTypeDiscriminants::HashmapKeysIterator,
            ) => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<HashmapKeysIteratorTerm>().as_inner()),
            (
                TermTypeDiscriminants::HashmapValuesIterator,
                TermTypeDiscriminants::HashmapValuesIterator,
            ) => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .patch(
                    &target
                        .as_typed_term::<HashmapValuesIteratorTerm>()
                        .as_inner(),
                ),
            (TermTypeDiscriminants::IntegersIterator, TermTypeDiscriminants::IntegersIterator) => {
                self.as_typed_term::<IntegersIteratorTerm>()
                    .as_inner()
                    .patch(&target.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            (
                TermTypeDiscriminants::IntersperseIterator,
                TermTypeDiscriminants::IntersperseIterator,
            ) => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<IntersperseIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::MapIterator, TermTypeDiscriminants::MapIterator) => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<MapIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::OnceIterator, TermTypeDiscriminants::OnceIterator) => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<OnceIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::RangeIterator, TermTypeDiscriminants::RangeIterator) => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<RangeIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::RepeatIterator, TermTypeDiscriminants::RepeatIterator) => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<RepeatIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::SkipIterator, TermTypeDiscriminants::SkipIterator) => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<SkipIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::TakeIterator, TermTypeDiscriminants::TakeIterator) => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<TakeIteratorTerm>().as_inner()),
            (TermTypeDiscriminants::ZipIterator, TermTypeDiscriminants::ZipIterator) => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .patch(&target.as_typed_term::<ZipIteratorTerm>().as_inner()),
            _ => target.to_json().map(Some),
        }
    }
}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<Term, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                std::fmt::Debug::fmt(&self.as_typed_term::<ApplicationTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Boolean => {
                std::fmt::Debug::fmt(&self.as_typed_term::<BooleanTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Builtin => {
                std::fmt::Debug::fmt(&self.as_typed_term::<BuiltinTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Cell => {
                std::fmt::Debug::fmt(&self.as_typed_term::<CellTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Condition => {
                std::fmt::Debug::fmt(&self.as_typed_term::<ConditionTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Constructor => {
                std::fmt::Debug::fmt(&self.as_typed_term::<ConstructorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Date => {
                std::fmt::Debug::fmt(&self.as_typed_term::<DateTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Effect => {
                std::fmt::Debug::fmt(&self.as_typed_term::<EffectTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Float => {
                std::fmt::Debug::fmt(&self.as_typed_term::<FloatTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Hashmap => {
                std::fmt::Debug::fmt(&self.as_typed_term::<HashmapTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Hashset => {
                std::fmt::Debug::fmt(&self.as_typed_term::<HashsetTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Int => {
                std::fmt::Debug::fmt(&self.as_typed_term::<IntTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Lambda => {
                std::fmt::Debug::fmt(&self.as_typed_term::<LambdaTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Let => {
                std::fmt::Debug::fmt(&self.as_typed_term::<LetTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::List => {
                std::fmt::Debug::fmt(&self.as_typed_term::<ListTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Nil => {
                std::fmt::Debug::fmt(&self.as_typed_term::<NilTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Partial => {
                std::fmt::Debug::fmt(&self.as_typed_term::<PartialTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Pointer => {
                std::fmt::Debug::fmt(&self.as_typed_term::<PointerTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Record => {
                std::fmt::Debug::fmt(&self.as_typed_term::<RecordTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Signal => {
                std::fmt::Debug::fmt(&self.as_typed_term::<SignalTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::String => {
                std::fmt::Debug::fmt(&self.as_typed_term::<StringTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Symbol => {
                std::fmt::Debug::fmt(&self.as_typed_term::<SymbolTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Tree => {
                std::fmt::Debug::fmt(&self.as_typed_term::<TreeTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Variable => {
                std::fmt::Debug::fmt(&self.as_typed_term::<VariableTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::EmptyIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<EmptyIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::EvaluateIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::FilterIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<FilterIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::FlattenIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<FlattenIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::HashmapKeysIterator => std::fmt::Debug::fmt(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
                f,
            ),
            TermTypeDiscriminants::HashmapValuesIterator => std::fmt::Debug::fmt(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
                f,
            ),
            TermTypeDiscriminants::IntegersIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<IntegersIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::IntersperseIterator => std::fmt::Debug::fmt(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
                f,
            ),
            TermTypeDiscriminants::MapIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<MapIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::OnceIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<OnceIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::RangeIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<RangeIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::RepeatIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<RepeatIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::SkipIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<SkipIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::TakeIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<TakeIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::ZipIterator => {
                std::fmt::Debug::fmt(&self.as_typed_term::<ZipIteratorTerm>().as_inner(), f)
            }
        }
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<Term, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                std::fmt::Display::fmt(&self.as_typed_term::<ApplicationTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Boolean => {
                std::fmt::Display::fmt(&self.as_typed_term::<BooleanTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Builtin => {
                std::fmt::Display::fmt(&self.as_typed_term::<BuiltinTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Cell => {
                std::fmt::Display::fmt(&self.as_typed_term::<CellTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Condition => {
                std::fmt::Display::fmt(&self.as_typed_term::<ConditionTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Constructor => {
                std::fmt::Display::fmt(&self.as_typed_term::<ConstructorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Date => {
                std::fmt::Display::fmt(&self.as_typed_term::<DateTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Effect => {
                std::fmt::Display::fmt(&self.as_typed_term::<EffectTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Float => {
                std::fmt::Display::fmt(&self.as_typed_term::<FloatTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Hashmap => {
                std::fmt::Display::fmt(&self.as_typed_term::<HashmapTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Hashset => {
                std::fmt::Display::fmt(&self.as_typed_term::<HashsetTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Int => {
                std::fmt::Display::fmt(&self.as_typed_term::<IntTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Lambda => {
                std::fmt::Display::fmt(&self.as_typed_term::<LambdaTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Let => {
                std::fmt::Display::fmt(&self.as_typed_term::<LetTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::List => {
                std::fmt::Display::fmt(&self.as_typed_term::<ListTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Nil => {
                std::fmt::Display::fmt(&self.as_typed_term::<NilTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Partial => {
                std::fmt::Display::fmt(&self.as_typed_term::<PartialTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Pointer => {
                std::fmt::Display::fmt(&self.as_typed_term::<PointerTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Record => {
                std::fmt::Display::fmt(&self.as_typed_term::<RecordTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Signal => {
                std::fmt::Display::fmt(&self.as_typed_term::<SignalTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::String => {
                std::fmt::Display::fmt(&self.as_typed_term::<StringTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Symbol => {
                std::fmt::Display::fmt(&self.as_typed_term::<SymbolTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Tree => {
                std::fmt::Display::fmt(&self.as_typed_term::<TreeTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::Variable => {
                std::fmt::Display::fmt(&self.as_typed_term::<VariableTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::EmptyIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<EmptyIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::EvaluateIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::FilterIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<FilterIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::FlattenIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<FlattenIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::HashmapKeysIterator => std::fmt::Display::fmt(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
                f,
            ),
            TermTypeDiscriminants::HashmapValuesIterator => std::fmt::Display::fmt(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
                f,
            ),
            TermTypeDiscriminants::IntegersIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<IntegersIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::IntersperseIterator => std::fmt::Display::fmt(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
                f,
            ),
            TermTypeDiscriminants::MapIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<MapIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::OnceIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<OnceIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::RangeIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<RangeIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::RepeatIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<RepeatIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::SkipIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<SkipIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::TakeIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<TakeIteratorTerm>().as_inner(), f)
            }
            TermTypeDiscriminants::ZipIterator => {
                std::fmt::Display::fmt(&self.as_typed_term::<ZipIteratorTerm>().as_inner(), f)
            }
        }
    }
}
impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<TermType, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.read_value(|term| TermTypeDiscriminants::from(term)) {
            TermTypeDiscriminants::Application => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Boolean => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Builtin => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Cell => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Condition => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Constructor => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Date => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Effect => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Float => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Hashmap => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Hashset => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Int => std::fmt::Debug::fmt(&self.read_value(|value| *value), f),
            TermTypeDiscriminants::Lambda => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Let => std::fmt::Debug::fmt(&self.read_value(|value| *value), f),
            TermTypeDiscriminants::List => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Nil => std::fmt::Debug::fmt(&self.read_value(|value| *value), f),
            TermTypeDiscriminants::Partial => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Pointer => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Record => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Signal => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::String => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Symbol => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Tree => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::Variable => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::EmptyIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::EvaluateIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::FilterIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::FlattenIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::HashmapValuesIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::IntegersIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::IntersperseIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::MapIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::OnceIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::RangeIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::RepeatIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::SkipIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::TakeIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
            TermTypeDiscriminants::ZipIterator => {
                std::fmt::Debug::fmt(&self.read_value(|value| *value), f)
            }
        }
    }
}

#[cfg(test)]
impl TermType {
    pub(crate) fn as_bytes(&self) -> &[u32] {
        let num_words = crate::pad_to_4_byte_offset(self.size_of() as usize) / 4;
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u32, num_words) }
    }
}

#[repr(transparent)]
pub struct TypedTerm<V> {
    term: Term,
    _type: PhantomData<V>,
}
impl<V> TypedTerm<V> {
    pub fn id(&self) -> HashId {
        self.term.id()
    }
    pub(crate) fn get_inner(&self) -> &V {
        unsafe {
            match &self.term.as_value() {
                TermType::Application(inner) => std::mem::transmute::<&ApplicationTerm, &V>(inner),
                TermType::Boolean(inner) => std::mem::transmute::<&BooleanTerm, &V>(inner),
                TermType::Builtin(inner) => std::mem::transmute::<&BuiltinTerm, &V>(inner),
                TermType::Cell(inner) => std::mem::transmute::<&CellTerm, &V>(inner),
                TermType::Condition(inner) => std::mem::transmute::<&ConditionTerm, &V>(inner),
                TermType::Constructor(inner) => std::mem::transmute::<&ConstructorTerm, &V>(inner),
                TermType::Date(inner) => std::mem::transmute::<&DateTerm, &V>(inner),
                TermType::Effect(inner) => std::mem::transmute::<&EffectTerm, &V>(inner),
                TermType::Float(inner) => std::mem::transmute::<&FloatTerm, &V>(inner),
                TermType::Hashmap(inner) => std::mem::transmute::<&HashmapTerm, &V>(inner),
                TermType::Hashset(inner) => std::mem::transmute::<&HashsetTerm, &V>(inner),
                TermType::Int(inner) => std::mem::transmute::<&IntTerm, &V>(inner),
                TermType::Lambda(inner) => std::mem::transmute::<&LambdaTerm, &V>(inner),
                TermType::Let(inner) => std::mem::transmute::<&LetTerm, &V>(inner),
                TermType::List(inner) => std::mem::transmute::<&ListTerm, &V>(inner),
                TermType::Nil(inner) => std::mem::transmute::<&NilTerm, &V>(inner),
                TermType::Partial(inner) => std::mem::transmute::<&PartialTerm, &V>(inner),
                TermType::Pointer(inner) => std::mem::transmute::<&PointerTerm, &V>(inner),
                TermType::Record(inner) => std::mem::transmute::<&RecordTerm, &V>(inner),
                TermType::Signal(inner) => std::mem::transmute::<&SignalTerm, &V>(inner),
                TermType::String(inner) => std::mem::transmute::<&StringTerm, &V>(inner),
                TermType::Symbol(inner) => std::mem::transmute::<&SymbolTerm, &V>(inner),
                TermType::Tree(inner) => std::mem::transmute::<&TreeTerm, &V>(inner),
                TermType::Variable(inner) => std::mem::transmute::<&VariableTerm, &V>(inner),
                TermType::EmptyIterator(inner) => {
                    std::mem::transmute::<&EmptyIteratorTerm, &V>(inner)
                }
                TermType::EvaluateIterator(inner) => {
                    std::mem::transmute::<&EvaluateIteratorTerm, &V>(inner)
                }
                TermType::FilterIterator(inner) => {
                    std::mem::transmute::<&FilterIteratorTerm, &V>(inner)
                }
                TermType::FlattenIterator(inner) => {
                    std::mem::transmute::<&FlattenIteratorTerm, &V>(inner)
                }
                TermType::HashmapKeysIterator(inner) => {
                    std::mem::transmute::<&HashmapKeysIteratorTerm, &V>(inner)
                }
                TermType::HashmapValuesIterator(inner) => {
                    std::mem::transmute::<&HashmapValuesIteratorTerm, &V>(inner)
                }
                TermType::IntegersIterator(inner) => {
                    std::mem::transmute::<&IntegersIteratorTerm, &V>(inner)
                }
                TermType::IntersperseIterator(inner) => {
                    std::mem::transmute::<&IntersperseIteratorTerm, &V>(inner)
                }
                TermType::MapIterator(inner) => std::mem::transmute::<&MapIteratorTerm, &V>(inner),
                TermType::OnceIterator(inner) => {
                    std::mem::transmute::<&OnceIteratorTerm, &V>(inner)
                }
                TermType::RangeIterator(inner) => {
                    std::mem::transmute::<&RangeIteratorTerm, &V>(inner)
                }
                TermType::RepeatIterator(inner) => {
                    std::mem::transmute::<&RepeatIteratorTerm, &V>(inner)
                }
                TermType::SkipIterator(inner) => {
                    std::mem::transmute::<&SkipIteratorTerm, &V>(inner)
                }
                TermType::TakeIterator(inner) => {
                    std::mem::transmute::<&TakeIteratorTerm, &V>(inner)
                }
                TermType::ZipIterator(inner) => std::mem::transmute::<&ZipIteratorTerm, &V>(inner),
            }
        }
    }
}

impl<A: Arena + Clone, V> ArenaRef<TypedTerm<V>, A> {
    pub fn as_inner(&self) -> ArenaRef<V, A> {
        ArenaRef::<V, _>::new(
            self.arena.clone(),
            self.get_value_pointer()
                .offset(TERM_TYPE_DISCRIMINANT_SIZE as u32),
        )
    }
    pub(crate) fn as_term(&self) -> &ArenaRef<Term, A> {
        unsafe { std::mem::transmute::<&ArenaRef<TypedTerm<V>, A>, &ArenaRef<Term, A>>(self) }
    }
    pub(crate) fn get_value_pointer(&self) -> ArenaPointer {
        self.as_term().get_value_pointer()
    }
}

impl<A: Arena + Clone> ArenaRef<Term, A> {
    pub(crate) fn as_typed_term<V>(&self) -> &ArenaRef<TypedTerm<V>, A> {
        unsafe { std::mem::transmute::<&ArenaRef<Term, A>, &ArenaRef<TypedTerm<V>, A>>(self) }
    }
}

impl<A: Arena + Clone, V> GraphNode for ArenaRef<TypedTerm<V>, A>
where
    ArenaRef<V, A>: GraphNode,
{
    fn size(&self) -> usize {
        <ArenaRef<V, A> as GraphNode>::size(&self.as_inner())
    }
    fn capture_depth(&self) -> StackOffset {
        <ArenaRef<V, A> as GraphNode>::capture_depth(&self.as_inner())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        <ArenaRef<V, A> as GraphNode>::free_variables(&self.as_inner())
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        <ArenaRef<V, A> as GraphNode>::count_variable_usages(&self.as_inner(), offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        <ArenaRef<V, A> as GraphNode>::dynamic_dependencies(&self.as_inner(), deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        <ArenaRef<V, A> as GraphNode>::has_dynamic_dependencies(&self.as_inner(), deep)
    }
    fn is_static(&self) -> bool {
        <ArenaRef<V, A> as GraphNode>::is_static(&self.as_inner())
    }
    fn is_atomic(&self) -> bool {
        <ArenaRef<V, A> as GraphNode>::is_atomic(&self.as_inner())
    }
    fn is_complex(&self) -> bool {
        <ArenaRef<V, A> as GraphNode>::is_complex(&self.as_inner())
    }
}

impl<A: Arena + Clone, V> PartialEq for ArenaRef<TypedTerm<V>, A>
where
    ArenaRef<V, A>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_term() == other.as_term()
    }
}
impl<A: Arena + Clone, V> Eq for ArenaRef<TypedTerm<V>, A> where ArenaRef<V, A>: Eq {}

impl<A: Arena + Clone, V> std::fmt::Debug for ArenaRef<TypedTerm<V>, A>
where
    ArenaRef<V, A>: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Term")
            .field("hash", &self.read_value(|term| term.id()))
            .field("value", &self.as_inner())
            .finish()
    }
}

impl<A: Arena + Clone, V> std::fmt::Display for ArenaRef<TypedTerm<V>, A>
where
    ArenaRef<V, A>: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <ArenaRef<V, A> as std::fmt::Display>::fmt(&self.as_inner(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_type() {
        assert_eq!(std::mem::size_of::<TermType>(), 28);
        assert_eq!(TermTypeDiscriminants::Application as u32, 0);
        assert_eq!(TermTypeDiscriminants::Boolean as u32, 1);
        assert_eq!(TermTypeDiscriminants::Builtin as u32, 2);
        assert_eq!(TermTypeDiscriminants::Cell as u32, 3);
        assert_eq!(TermTypeDiscriminants::Condition as u32, 4);
        assert_eq!(TermTypeDiscriminants::Constructor as u32, 5);
        assert_eq!(TermTypeDiscriminants::Date as u32, 6);
        assert_eq!(TermTypeDiscriminants::Effect as u32, 7);
        assert_eq!(TermTypeDiscriminants::Float as u32, 8);
        assert_eq!(TermTypeDiscriminants::Hashmap as u32, 9);
        assert_eq!(TermTypeDiscriminants::Hashset as u32, 10);
        assert_eq!(TermTypeDiscriminants::Int as u32, 11);
        assert_eq!(TermTypeDiscriminants::Lambda as u32, 12);
        assert_eq!(TermTypeDiscriminants::Let as u32, 13);
        assert_eq!(TermTypeDiscriminants::List as u32, 14);
        assert_eq!(TermTypeDiscriminants::Nil as u32, 15);
        assert_eq!(TermTypeDiscriminants::Partial as u32, 16);
        assert_eq!(TermTypeDiscriminants::Pointer as u32, 17);
        assert_eq!(TermTypeDiscriminants::Record as u32, 18);
        assert_eq!(TermTypeDiscriminants::Signal as u32, 19);
        assert_eq!(TermTypeDiscriminants::String as u32, 20);
        assert_eq!(TermTypeDiscriminants::Symbol as u32, 21);
        assert_eq!(TermTypeDiscriminants::Tree as u32, 22);
        assert_eq!(TermTypeDiscriminants::Variable as u32, 23);
        assert_eq!(TermTypeDiscriminants::EmptyIterator as u32, 24);
        assert_eq!(TermTypeDiscriminants::EvaluateIterator as u32, 25);
        assert_eq!(TermTypeDiscriminants::FilterIterator as u32, 26);
        assert_eq!(TermTypeDiscriminants::FlattenIterator as u32, 27);
        assert_eq!(TermTypeDiscriminants::HashmapKeysIterator as u32, 28);
        assert_eq!(TermTypeDiscriminants::HashmapValuesIterator as u32, 29);
        assert_eq!(TermTypeDiscriminants::IntegersIterator as u32, 30);
        assert_eq!(TermTypeDiscriminants::IntersperseIterator as u32, 31);
        assert_eq!(TermTypeDiscriminants::MapIterator as u32, 32);
        assert_eq!(TermTypeDiscriminants::OnceIterator as u32, 33);
        assert_eq!(TermTypeDiscriminants::RangeIterator as u32, 34);
        assert_eq!(TermTypeDiscriminants::RepeatIterator as u32, 35);
        assert_eq!(TermTypeDiscriminants::SkipIterator as u32, 36);
        assert_eq!(TermTypeDiscriminants::TakeIterator as u32, 37);
        assert_eq!(TermTypeDiscriminants::ZipIterator as u32, 38);
    }
}
