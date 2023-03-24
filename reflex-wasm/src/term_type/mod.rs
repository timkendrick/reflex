// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{cell::RefCell, collections::HashSet, marker::PhantomData, rc::Rc};

use reflex::{
    core::{
        Arity, DependencyList, Eagerness, Expression, GraphNode, Internable, NodeId, SerializeJson,
        StackOffset,
    },
    hash::HashId,
};
use serde_json::Value as JsonValue;
use strum_macros::EnumDiscriminants;

use crate::{
    allocator::{Arena, ArenaAllocator},
    factory::WasmTermFactory,
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    ArenaPointer, ArenaRef, PointerIter, Term,
};

pub mod application;
pub mod boolean;
pub mod builtin;
pub mod cell;
pub mod condition;
pub mod constructor;
pub mod date;
pub mod effect;
pub mod float;
pub mod hashmap;
pub mod hashset;
pub mod int;
pub mod iterator;
pub mod lambda;
pub mod r#let;
pub mod list;
pub mod nil;
pub mod partial;
pub mod pointer;
pub mod record;
pub mod signal;
pub mod string;
pub mod symbol;
pub mod tree;
pub mod variable;

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
    IndexedAccessorIterator(IndexedAccessorIteratorTerm),
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
            value if value == Self::IndexedAccessorIterator as u32 => {
                Ok(Self::IndexedAccessorIterator)
            }
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
            Self::IndexedAccessorIterator(term) => term.size_of(),
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
            Self::IndexedAccessorIterator(term) => hasher
                .write_u8(TermTypeDiscriminants::IndexedAccessorIterator as u8)
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

#[derive(Clone, Debug)]
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
    IndexedAccessorIterator(IndexedAccessorIteratorTermPointerIter),
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
            TermPointerIterator::IndexedAccessorIterator(inner) => inner.next(),
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
            TermTypeDiscriminants::IndexedAccessorIterator => {
                TermPointerIterator::IndexedAccessorIterator(
                    self.as_typed_term::<IndexedAccessorIteratorTerm>()
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
            TermTypeDiscriminants::IndexedAccessorIterator => self
                .as_typed_term::<IndexedAccessorIteratorTerm>()
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

pub trait IteratorExpression: Expression {
    type IteratorTerm<'a>: IteratorTermType<Self>;
}

pub trait IteratorTermType<T: Expression> {}

pub trait IteratorExpressionFactory<T: Expression + IteratorExpression> {
    fn match_iterator_term<'a>(&self, expression: &'a T) -> Option<T::IteratorTerm<'a>>;
}

impl<A: Arena + Clone> IteratorExpression for WasmExpression<A> {
    type IteratorTerm<'a> = WasmIteratorTerm<A>;
}

#[derive(Clone, Copy, Debug)]
pub enum WasmIteratorTerm<A: Arena + Clone> {
    EmptyIterator(ArenaRef<EmptyIteratorTerm, A>),
    EvaluateIterator(ArenaRef<EvaluateIteratorTerm, A>),
    FilterIterator(ArenaRef<FilterIteratorTerm, A>),
    FlattenIterator(ArenaRef<FlattenIteratorTerm, A>),
    HashmapKeysIterator(ArenaRef<HashmapKeysIteratorTerm, A>),
    HashmapValuesIterator(ArenaRef<HashmapValuesIteratorTerm, A>),
    IndexedAccessorIterator(ArenaRef<IndexedAccessorIteratorTerm, A>),
    IntegersIterator(ArenaRef<IntegersIteratorTerm, A>),
    IntersperseIterator(ArenaRef<IntersperseIteratorTerm, A>),
    MapIterator(ArenaRef<MapIteratorTerm, A>),
    OnceIterator(ArenaRef<OnceIteratorTerm, A>),
    RangeIterator(ArenaRef<RangeIteratorTerm, A>),
    RepeatIterator(ArenaRef<RepeatIteratorTerm, A>),
    SkipIterator(ArenaRef<SkipIteratorTerm, A>),
    TakeIterator(ArenaRef<TakeIteratorTerm, A>),
    ZipIterator(ArenaRef<ZipIteratorTerm, A>),
}

impl<A: Arena + Clone> IteratorTermType<WasmExpression<A>> for WasmIteratorTerm<A> {}

impl<A: Arena> IteratorExpressionFactory<ArenaRef<Term, Self>> for WasmTermFactory<A>
where
    A: ArenaAllocator,
    Rc<RefCell<A>>: Arena,
{
    fn match_iterator_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<<ArenaRef<Term, Self> as IteratorExpression>::IteratorTerm<'a>> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::EmptyIterator => expression
                .as_empty_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::EmptyIterator),
            TermTypeDiscriminants::EvaluateIterator => expression
                .as_evaluate_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::EvaluateIterator),
            TermTypeDiscriminants::FilterIterator => expression
                .as_filter_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::FilterIterator),
            TermTypeDiscriminants::FlattenIterator => expression
                .as_flatten_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::FlattenIterator),
            TermTypeDiscriminants::HashmapKeysIterator => expression
                .as_hashmap_keys_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::HashmapKeysIterator),
            TermTypeDiscriminants::HashmapValuesIterator => expression
                .as_hashmap_values_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::HashmapValuesIterator),
            TermTypeDiscriminants::IndexedAccessorIterator => expression
                .as_indexed_accessor_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::IndexedAccessorIterator),
            TermTypeDiscriminants::IntegersIterator => expression
                .as_integers_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::IntegersIterator),
            TermTypeDiscriminants::IntersperseIterator => expression
                .as_intersperse_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::IntersperseIterator),
            TermTypeDiscriminants::MapIterator => expression
                .as_map_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::MapIterator),
            TermTypeDiscriminants::OnceIterator => expression
                .as_once_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::OnceIterator),
            TermTypeDiscriminants::RangeIterator => expression
                .as_range_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::RangeIterator),
            TermTypeDiscriminants::RepeatIterator => expression
                .as_repeat_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::RepeatIterator),
            TermTypeDiscriminants::SkipIterator => expression
                .as_skip_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::SkipIterator),
            TermTypeDiscriminants::TakeIterator => expression
                .as_take_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::TakeIterator),
            TermTypeDiscriminants::ZipIterator => expression
                .as_zip_iterator_term()
                .map(|term| term.as_inner())
                .map(WasmIteratorTerm::ZipIterator),
            _ => None,
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
impl<'a> Into<Option<&'a IndexedAccessorIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IndexedAccessorIteratorTerm> {
        match self {
            TermType::IndexedAccessorIterator(term) => Some(term),
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
            (
                TermTypeDiscriminants::IndexedAccessorIterator,
                TermTypeDiscriminants::IndexedAccessorIterator,
            ) => {
                self.as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner()
                    == other
                        .as_typed_term::<IndexedAccessorIteratorTerm>()
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

impl<A: Arena + Clone> Eq for ArenaRef<Term, A> {}

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
                GraphNode::size(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                GraphNode::size(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                GraphNode::size(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                GraphNode::size(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                GraphNode::size(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                GraphNode::size(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                GraphNode::size(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                GraphNode::size(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                GraphNode::size(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                GraphNode::size(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                GraphNode::size(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                GraphNode::size(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                GraphNode::size(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                GraphNode::size(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                GraphNode::size(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::size(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                GraphNode::size(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                GraphNode::size(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                GraphNode::size(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                GraphNode::size(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                GraphNode::size(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                GraphNode::size(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                GraphNode::size(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                GraphNode::size(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                GraphNode::size(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                GraphNode::size(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                GraphNode::size(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                GraphNode::size(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                GraphNode::size(&self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapValuesIterator => {
                GraphNode::size(&self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::size(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                GraphNode::size(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => {
                GraphNode::size(&self.as_typed_term::<IntersperseIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::MapIterator => {
                GraphNode::size(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                GraphNode::size(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                GraphNode::size(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                GraphNode::size(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                GraphNode::size(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                GraphNode::size(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                GraphNode::size(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
            }
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                GraphNode::capture_depth(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                GraphNode::capture_depth(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                GraphNode::capture_depth(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                GraphNode::capture_depth(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                GraphNode::capture_depth(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                GraphNode::capture_depth(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                GraphNode::capture_depth(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                GraphNode::capture_depth(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                GraphNode::capture_depth(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                GraphNode::capture_depth(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                GraphNode::capture_depth(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                GraphNode::capture_depth(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                GraphNode::capture_depth(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                GraphNode::capture_depth(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                GraphNode::capture_depth(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::capture_depth(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                GraphNode::capture_depth(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                GraphNode::capture_depth(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                GraphNode::capture_depth(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                GraphNode::capture_depth(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                GraphNode::capture_depth(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                GraphNode::capture_depth(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                GraphNode::capture_depth(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                GraphNode::capture_depth(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => GraphNode::capture_depth(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::HashmapValuesIterator => GraphNode::capture_depth(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::capture_depth(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => GraphNode::capture_depth(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::MapIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                GraphNode::capture_depth(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
            }
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                GraphNode::free_variables(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                GraphNode::free_variables(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                GraphNode::free_variables(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                GraphNode::free_variables(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                GraphNode::free_variables(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                GraphNode::free_variables(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                GraphNode::free_variables(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                GraphNode::free_variables(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                GraphNode::free_variables(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                GraphNode::free_variables(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                GraphNode::free_variables(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                GraphNode::free_variables(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                GraphNode::free_variables(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                GraphNode::free_variables(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                GraphNode::free_variables(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::free_variables(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                GraphNode::free_variables(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                GraphNode::free_variables(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                GraphNode::free_variables(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                GraphNode::free_variables(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                GraphNode::free_variables(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                GraphNode::free_variables(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                GraphNode::free_variables(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                GraphNode::free_variables(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                GraphNode::free_variables(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                GraphNode::free_variables(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                GraphNode::free_variables(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                GraphNode::free_variables(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => GraphNode::free_variables(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::HashmapValuesIterator => GraphNode::free_variables(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::free_variables(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                GraphNode::free_variables(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => GraphNode::free_variables(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::MapIterator => {
                GraphNode::free_variables(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                GraphNode::free_variables(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                GraphNode::free_variables(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                GraphNode::free_variables(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                GraphNode::free_variables(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                GraphNode::free_variables(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                GraphNode::free_variables(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
            }
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => GraphNode::count_variable_usages(
                &self.as_typed_term::<ApplicationTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Boolean => GraphNode::count_variable_usages(
                &self.as_typed_term::<BooleanTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Builtin => GraphNode::count_variable_usages(
                &self.as_typed_term::<BuiltinTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Cell => GraphNode::count_variable_usages(
                &self.as_typed_term::<CellTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Condition => GraphNode::count_variable_usages(
                &self.as_typed_term::<ConditionTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Constructor => GraphNode::count_variable_usages(
                &self.as_typed_term::<ConstructorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Date => GraphNode::count_variable_usages(
                &self.as_typed_term::<DateTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Effect => GraphNode::count_variable_usages(
                &self.as_typed_term::<EffectTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Float => GraphNode::count_variable_usages(
                &self.as_typed_term::<FloatTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Hashmap => GraphNode::count_variable_usages(
                &self.as_typed_term::<HashmapTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Hashset => GraphNode::count_variable_usages(
                &self.as_typed_term::<HashsetTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Int => GraphNode::count_variable_usages(
                &self.as_typed_term::<IntTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Lambda => GraphNode::count_variable_usages(
                &self.as_typed_term::<LambdaTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Let => GraphNode::count_variable_usages(
                &self.as_typed_term::<LetTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::List => GraphNode::count_variable_usages(
                &self.as_typed_term::<ListTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Nil => GraphNode::count_variable_usages(
                &self.as_typed_term::<NilTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Partial => GraphNode::count_variable_usages(
                &self.as_typed_term::<PartialTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Pointer => GraphNode::count_variable_usages(
                &self.as_typed_term::<PointerTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Record => GraphNode::count_variable_usages(
                &self.as_typed_term::<RecordTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Signal => GraphNode::count_variable_usages(
                &self.as_typed_term::<SignalTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::String => GraphNode::count_variable_usages(
                &self.as_typed_term::<StringTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Symbol => GraphNode::count_variable_usages(
                &self.as_typed_term::<SymbolTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Tree => GraphNode::count_variable_usages(
                &self.as_typed_term::<TreeTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::Variable => GraphNode::count_variable_usages(
                &self.as_typed_term::<VariableTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::EmptyIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<EmptyIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::EvaluateIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<EvaluateIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::FilterIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<FilterIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::FlattenIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<FlattenIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::HashmapKeysIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::HashmapValuesIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::count_variable_usages(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
                offset,
            ),
            TermTypeDiscriminants::IntegersIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<IntegersIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::IntersperseIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::MapIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<MapIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::OnceIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<OnceIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::RangeIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<RangeIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::RepeatIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<RepeatIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::SkipIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<SkipIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::TakeIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<TakeIteratorTerm>().as_inner(),
                offset,
            ),
            TermTypeDiscriminants::ZipIterator => GraphNode::count_variable_usages(
                &self.as_typed_term::<ZipIteratorTerm>().as_inner(),
                offset,
            ),
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<ApplicationTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Boolean => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<BooleanTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Builtin => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<BuiltinTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Cell => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<CellTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Condition => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<ConditionTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Constructor => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<ConstructorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Date => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<DateTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Effect => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<EffectTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Float => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<FloatTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Hashmap => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<HashmapTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Hashset => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<HashsetTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Int => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<IntTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Lambda => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<LambdaTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Let => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<LetTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::List => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<ListTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<NilTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Partial => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<PartialTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Pointer => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<PointerTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Record => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<RecordTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Signal => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<SignalTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::String => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<StringTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Symbol => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<SymbolTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Tree => {
                GraphNode::dynamic_dependencies(&self.as_typed_term::<TreeTerm>().as_inner(), deep)
            }
            TermTypeDiscriminants::Variable => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<VariableTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::EmptyIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<EmptyIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::EvaluateIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<EvaluateIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::FilterIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<FilterIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::FlattenIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<FlattenIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::HashmapKeysIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::HashmapValuesIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::dynamic_dependencies(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
                deep,
            ),
            TermTypeDiscriminants::IntegersIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<IntegersIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::IntersperseIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::MapIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<MapIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::OnceIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<OnceIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::RangeIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<RangeIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::RepeatIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<RepeatIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::SkipIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<SkipIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::TakeIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<TakeIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::ZipIterator => GraphNode::dynamic_dependencies(
                &self.as_typed_term::<ZipIteratorTerm>().as_inner(),
                deep,
            ),
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<ApplicationTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Boolean => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<BooleanTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Builtin => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<BuiltinTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Cell => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<CellTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Condition => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<ConditionTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Constructor => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<ConstructorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Date => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<DateTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Effect => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<EffectTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Float => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<FloatTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Hashmap => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<HashmapTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Hashset => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<HashsetTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Int => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<IntTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Lambda => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<LambdaTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Let => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<LetTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::List => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<ListTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Nil => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<NilTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Partial => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<PartialTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Pointer => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<PointerTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Record => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<RecordTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Signal => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<SignalTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::String => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<StringTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Symbol => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<SymbolTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Tree => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<TreeTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::Variable => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<VariableTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::EmptyIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<EmptyIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::EvaluateIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<EvaluateIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::FilterIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<FilterIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::FlattenIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<FlattenIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::HashmapKeysIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::HashmapValuesIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::has_dynamic_dependencies(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
                deep,
            ),
            TermTypeDiscriminants::IntegersIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<IntegersIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::IntersperseIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::MapIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<MapIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::OnceIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<OnceIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::RangeIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<RangeIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::RepeatIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<RepeatIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::SkipIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<SkipIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::TakeIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<TakeIteratorTerm>().as_inner(),
                deep,
            ),
            TermTypeDiscriminants::ZipIterator => GraphNode::has_dynamic_dependencies(
                &self.as_typed_term::<ZipIteratorTerm>().as_inner(),
                deep,
            ),
        }
    }
    fn is_static(&self) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                GraphNode::is_static(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                GraphNode::is_static(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                GraphNode::is_static(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                GraphNode::is_static(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                GraphNode::is_static(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                GraphNode::is_static(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                GraphNode::is_static(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                GraphNode::is_static(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                GraphNode::is_static(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                GraphNode::is_static(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                GraphNode::is_static(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                GraphNode::is_static(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                GraphNode::is_static(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                GraphNode::is_static(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                GraphNode::is_static(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::is_static(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                GraphNode::is_static(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                GraphNode::is_static(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                GraphNode::is_static(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                GraphNode::is_static(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                GraphNode::is_static(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                GraphNode::is_static(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                GraphNode::is_static(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                GraphNode::is_static(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                GraphNode::is_static(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                GraphNode::is_static(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                GraphNode::is_static(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                GraphNode::is_static(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                GraphNode::is_static(&self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapValuesIterator => {
                GraphNode::is_static(&self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::is_static(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                GraphNode::is_static(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => {
                GraphNode::is_static(&self.as_typed_term::<IntersperseIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::MapIterator => {
                GraphNode::is_static(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                GraphNode::is_static(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                GraphNode::is_static(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                GraphNode::is_static(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                GraphNode::is_static(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                GraphNode::is_static(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                GraphNode::is_static(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
            }
        }
    }
    fn is_atomic(&self) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                GraphNode::is_atomic(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                GraphNode::is_atomic(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                GraphNode::is_atomic(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                GraphNode::is_atomic(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                GraphNode::is_atomic(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                GraphNode::is_atomic(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                GraphNode::is_atomic(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                GraphNode::is_atomic(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                GraphNode::is_atomic(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                GraphNode::is_atomic(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                GraphNode::is_atomic(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                GraphNode::is_atomic(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                GraphNode::is_atomic(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                GraphNode::is_atomic(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                GraphNode::is_atomic(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::is_atomic(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                GraphNode::is_atomic(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                GraphNode::is_atomic(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                GraphNode::is_atomic(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                GraphNode::is_atomic(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                GraphNode::is_atomic(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                GraphNode::is_atomic(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                GraphNode::is_atomic(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                GraphNode::is_atomic(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapValuesIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::is_atomic(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<IntersperseIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::MapIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                GraphNode::is_atomic(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
            }
        }
    }
    fn is_complex(&self) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                GraphNode::is_complex(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                GraphNode::is_complex(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                GraphNode::is_complex(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                GraphNode::is_complex(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                GraphNode::is_complex(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                GraphNode::is_complex(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                GraphNode::is_complex(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                GraphNode::is_complex(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                GraphNode::is_complex(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                GraphNode::is_complex(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                GraphNode::is_complex(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                GraphNode::is_complex(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                GraphNode::is_complex(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                GraphNode::is_complex(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                GraphNode::is_complex(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                GraphNode::is_complex(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                GraphNode::is_complex(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                GraphNode::is_complex(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                GraphNode::is_complex(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                GraphNode::is_complex(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                GraphNode::is_complex(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                GraphNode::is_complex(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                GraphNode::is_complex(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                GraphNode::is_complex(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                GraphNode::is_complex(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                GraphNode::is_complex(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                GraphNode::is_complex(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                GraphNode::is_complex(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                GraphNode::is_complex(&self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapValuesIterator => {
                GraphNode::is_complex(&self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IndexedAccessorIterator => GraphNode::is_complex(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                GraphNode::is_complex(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => {
                GraphNode::is_complex(&self.as_typed_term::<IntersperseIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::MapIterator => {
                GraphNode::is_complex(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                GraphNode::is_complex(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                GraphNode::is_complex(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                GraphNode::is_complex(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                GraphNode::is_complex(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                GraphNode::is_complex(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                GraphNode::is_complex(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
            }
        }
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<Term, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                SerializeJson::to_json(&self.as_typed_term::<ApplicationTerm>().as_inner())
            }
            TermTypeDiscriminants::Boolean => {
                SerializeJson::to_json(&self.as_typed_term::<BooleanTerm>().as_inner())
            }
            TermTypeDiscriminants::Builtin => {
                SerializeJson::to_json(&self.as_typed_term::<BuiltinTerm>().as_inner())
            }
            TermTypeDiscriminants::Cell => {
                SerializeJson::to_json(&self.as_typed_term::<CellTerm>().as_inner())
            }
            TermTypeDiscriminants::Condition => {
                SerializeJson::to_json(&self.as_typed_term::<ConditionTerm>().as_inner())
            }
            TermTypeDiscriminants::Constructor => {
                SerializeJson::to_json(&self.as_typed_term::<ConstructorTerm>().as_inner())
            }
            TermTypeDiscriminants::Date => {
                SerializeJson::to_json(&self.as_typed_term::<DateTerm>().as_inner())
            }
            TermTypeDiscriminants::Effect => {
                SerializeJson::to_json(&self.as_typed_term::<EffectTerm>().as_inner())
            }
            TermTypeDiscriminants::Float => {
                SerializeJson::to_json(&self.as_typed_term::<FloatTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashmap => {
                SerializeJson::to_json(&self.as_typed_term::<HashmapTerm>().as_inner())
            }
            TermTypeDiscriminants::Hashset => {
                SerializeJson::to_json(&self.as_typed_term::<HashsetTerm>().as_inner())
            }
            TermTypeDiscriminants::Int => {
                SerializeJson::to_json(&self.as_typed_term::<IntTerm>().as_inner())
            }
            TermTypeDiscriminants::Lambda => {
                SerializeJson::to_json(&self.as_typed_term::<LambdaTerm>().as_inner())
            }
            TermTypeDiscriminants::Let => {
                SerializeJson::to_json(&self.as_typed_term::<LetTerm>().as_inner())
            }
            TermTypeDiscriminants::List => {
                SerializeJson::to_json(&self.as_typed_term::<ListTerm>().as_inner())
            }
            TermTypeDiscriminants::Nil => {
                SerializeJson::to_json(&self.as_typed_term::<NilTerm>().as_inner())
            }
            TermTypeDiscriminants::Partial => {
                SerializeJson::to_json(&self.as_typed_term::<PartialTerm>().as_inner())
            }
            TermTypeDiscriminants::Pointer => {
                SerializeJson::to_json(&self.as_typed_term::<PointerTerm>().as_inner())
            }
            TermTypeDiscriminants::Record => {
                SerializeJson::to_json(&self.as_typed_term::<RecordTerm>().as_inner())
            }
            TermTypeDiscriminants::Signal => {
                SerializeJson::to_json(&self.as_typed_term::<SignalTerm>().as_inner())
            }
            TermTypeDiscriminants::String => {
                SerializeJson::to_json(&self.as_typed_term::<StringTerm>().as_inner())
            }
            TermTypeDiscriminants::Symbol => {
                SerializeJson::to_json(&self.as_typed_term::<SymbolTerm>().as_inner())
            }
            TermTypeDiscriminants::Tree => {
                SerializeJson::to_json(&self.as_typed_term::<TreeTerm>().as_inner())
            }
            TermTypeDiscriminants::Variable => {
                SerializeJson::to_json(&self.as_typed_term::<VariableTerm>().as_inner())
            }
            TermTypeDiscriminants::EmptyIterator => {
                SerializeJson::to_json(&self.as_typed_term::<EmptyIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::EvaluateIterator => {
                SerializeJson::to_json(&self.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FilterIterator => {
                SerializeJson::to_json(&self.as_typed_term::<FilterIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::FlattenIterator => {
                SerializeJson::to_json(&self.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                SerializeJson::to_json(&self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::HashmapValuesIterator => SerializeJson::to_json(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
            ),
            TermTypeDiscriminants::IndexedAccessorIterator => SerializeJson::to_json(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
            ),
            TermTypeDiscriminants::IntegersIterator => {
                SerializeJson::to_json(&self.as_typed_term::<IntegersIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::IntersperseIterator => {
                SerializeJson::to_json(&self.as_typed_term::<IntersperseIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::MapIterator => {
                SerializeJson::to_json(&self.as_typed_term::<MapIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::OnceIterator => {
                SerializeJson::to_json(&self.as_typed_term::<OnceIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RangeIterator => {
                SerializeJson::to_json(&self.as_typed_term::<RangeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::RepeatIterator => {
                SerializeJson::to_json(&self.as_typed_term::<RepeatIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::SkipIterator => {
                SerializeJson::to_json(&self.as_typed_term::<SkipIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::TakeIterator => {
                SerializeJson::to_json(&self.as_typed_term::<TakeIteratorTerm>().as_inner())
            }
            TermTypeDiscriminants::ZipIterator => {
                SerializeJson::to_json(&self.as_typed_term::<ZipIteratorTerm>().as_inner())
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
            (TermTypeDiscriminants::Application, TermTypeDiscriminants::Application) => {
                SerializeJson::patch(
                    &self.as_typed_term::<ApplicationTerm>().as_inner(),
                    &target.as_typed_term::<ApplicationTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Boolean, TermTypeDiscriminants::Boolean) => {
                SerializeJson::patch(
                    &self.as_typed_term::<BooleanTerm>().as_inner(),
                    &target.as_typed_term::<BooleanTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Builtin, TermTypeDiscriminants::Builtin) => {
                SerializeJson::patch(
                    &self.as_typed_term::<BuiltinTerm>().as_inner(),
                    &target.as_typed_term::<BuiltinTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Cell, TermTypeDiscriminants::Cell) => SerializeJson::patch(
                &self.as_typed_term::<CellTerm>().as_inner(),
                &target.as_typed_term::<CellTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Condition, TermTypeDiscriminants::Condition) => {
                SerializeJson::patch(
                    &self.as_typed_term::<ConditionTerm>().as_inner(),
                    &target.as_typed_term::<ConditionTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Constructor, TermTypeDiscriminants::Constructor) => {
                SerializeJson::patch(
                    &self.as_typed_term::<ConstructorTerm>().as_inner(),
                    &target.as_typed_term::<ConstructorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Date, TermTypeDiscriminants::Date) => SerializeJson::patch(
                &self.as_typed_term::<DateTerm>().as_inner(),
                &target.as_typed_term::<DateTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Effect, TermTypeDiscriminants::Effect) => SerializeJson::patch(
                &self.as_typed_term::<EffectTerm>().as_inner(),
                &target.as_typed_term::<EffectTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Float, TermTypeDiscriminants::Float) => SerializeJson::patch(
                &self.as_typed_term::<FloatTerm>().as_inner(),
                &target.as_typed_term::<FloatTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Hashmap, TermTypeDiscriminants::Hashmap) => {
                SerializeJson::patch(
                    &self.as_typed_term::<HashmapTerm>().as_inner(),
                    &target.as_typed_term::<HashmapTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Hashset, TermTypeDiscriminants::Hashset) => {
                SerializeJson::patch(
                    &self.as_typed_term::<HashsetTerm>().as_inner(),
                    &target.as_typed_term::<HashsetTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Int, TermTypeDiscriminants::Int) => SerializeJson::patch(
                &self.as_typed_term::<IntTerm>().as_inner(),
                &target.as_typed_term::<IntTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Lambda, TermTypeDiscriminants::Lambda) => SerializeJson::patch(
                &self.as_typed_term::<LambdaTerm>().as_inner(),
                &target.as_typed_term::<LambdaTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Let, TermTypeDiscriminants::Let) => SerializeJson::patch(
                &self.as_typed_term::<LetTerm>().as_inner(),
                &target.as_typed_term::<LetTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::List, TermTypeDiscriminants::List) => SerializeJson::patch(
                &self.as_typed_term::<ListTerm>().as_inner(),
                &target.as_typed_term::<ListTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Nil, TermTypeDiscriminants::Nil) => SerializeJson::patch(
                &self.as_typed_term::<NilTerm>().as_inner(),
                &target.as_typed_term::<NilTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Partial, TermTypeDiscriminants::Partial) => {
                SerializeJson::patch(
                    &self.as_typed_term::<PartialTerm>().as_inner(),
                    &target.as_typed_term::<PartialTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Pointer, TermTypeDiscriminants::Pointer) => {
                SerializeJson::patch(
                    &self.as_typed_term::<PointerTerm>().as_inner(),
                    &target.as_typed_term::<PointerTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::Record, TermTypeDiscriminants::Record) => SerializeJson::patch(
                &self.as_typed_term::<RecordTerm>().as_inner(),
                &target.as_typed_term::<RecordTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Signal, TermTypeDiscriminants::Signal) => SerializeJson::patch(
                &self.as_typed_term::<SignalTerm>().as_inner(),
                &target.as_typed_term::<SignalTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::String, TermTypeDiscriminants::String) => SerializeJson::patch(
                &self.as_typed_term::<StringTerm>().as_inner(),
                &target.as_typed_term::<StringTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Symbol, TermTypeDiscriminants::Symbol) => SerializeJson::patch(
                &self.as_typed_term::<SymbolTerm>().as_inner(),
                &target.as_typed_term::<SymbolTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Tree, TermTypeDiscriminants::Tree) => SerializeJson::patch(
                &self.as_typed_term::<TreeTerm>().as_inner(),
                &target.as_typed_term::<TreeTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::Variable, TermTypeDiscriminants::Variable) => {
                SerializeJson::patch(
                    &self.as_typed_term::<VariableTerm>().as_inner(),
                    &target.as_typed_term::<VariableTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::EmptyIterator, TermTypeDiscriminants::EmptyIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<EmptyIteratorTerm>().as_inner(),
                    &target.as_typed_term::<EmptyIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::EvaluateIterator, TermTypeDiscriminants::EvaluateIterator) => {
                self.as_typed_term::<EvaluateIteratorTerm>()
                    .as_inner()
                    .patch(&target.as_typed_term::<EvaluateIteratorTerm>().as_inner())
            }
            (TermTypeDiscriminants::FilterIterator, TermTypeDiscriminants::FilterIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<FilterIteratorTerm>().as_inner(),
                    &target.as_typed_term::<FilterIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::FlattenIterator, TermTypeDiscriminants::FlattenIterator) => {
                self.as_typed_term::<FlattenIteratorTerm>()
                    .as_inner()
                    .patch(&target.as_typed_term::<FlattenIteratorTerm>().as_inner())
            }
            (
                TermTypeDiscriminants::HashmapKeysIterator,
                TermTypeDiscriminants::HashmapKeysIterator,
            ) => SerializeJson::patch(
                &self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
                &target.as_typed_term::<HashmapKeysIteratorTerm>().as_inner(),
            ),
            (
                TermTypeDiscriminants::HashmapValuesIterator,
                TermTypeDiscriminants::HashmapValuesIterator,
            ) => SerializeJson::patch(
                &self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner(),
                &target
                    .as_typed_term::<HashmapValuesIteratorTerm>()
                    .as_inner(),
            ),
            (
                TermTypeDiscriminants::IndexedAccessorIterator,
                TermTypeDiscriminants::IndexedAccessorIterator,
            ) => SerializeJson::patch(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
                &target
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
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
            ) => SerializeJson::patch(
                &self.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
                &target.as_typed_term::<IntersperseIteratorTerm>().as_inner(),
            ),
            (TermTypeDiscriminants::MapIterator, TermTypeDiscriminants::MapIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<MapIteratorTerm>().as_inner(),
                    &target.as_typed_term::<MapIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::OnceIterator, TermTypeDiscriminants::OnceIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<OnceIteratorTerm>().as_inner(),
                    &target.as_typed_term::<OnceIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::RangeIterator, TermTypeDiscriminants::RangeIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<RangeIteratorTerm>().as_inner(),
                    &target.as_typed_term::<RangeIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::RepeatIterator, TermTypeDiscriminants::RepeatIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<RepeatIteratorTerm>().as_inner(),
                    &target.as_typed_term::<RepeatIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::SkipIterator, TermTypeDiscriminants::SkipIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<SkipIteratorTerm>().as_inner(),
                    &target.as_typed_term::<SkipIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::TakeIterator, TermTypeDiscriminants::TakeIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<TakeIteratorTerm>().as_inner(),
                    &target.as_typed_term::<TakeIteratorTerm>().as_inner(),
                )
            }
            (TermTypeDiscriminants::ZipIterator, TermTypeDiscriminants::ZipIterator) => {
                SerializeJson::patch(
                    &self.as_typed_term::<ZipIteratorTerm>().as_inner(),
                    &target.as_typed_term::<ZipIteratorTerm>().as_inner(),
                )
            }
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
            TermTypeDiscriminants::IndexedAccessorIterator => std::fmt::Debug::fmt(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
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
            TermTypeDiscriminants::IndexedAccessorIterator => std::fmt::Display::fmt(
                &self
                    .as_typed_term::<IndexedAccessorIteratorTerm>()
                    .as_inner(),
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
            TermTypeDiscriminants::IndexedAccessorIterator => {
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
                TermType::IndexedAccessorIterator(inner) => {
                    std::mem::transmute::<&IndexedAccessorIteratorTerm, &V>(inner)
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

impl<A: Arena + Clone, V> NodeId for ArenaRef<TypedTerm<V>, A> {
    fn id(&self) -> HashId {
        self.read_value(|term| term.id())
    }
}

impl<A: Arena + Clone> ArenaRef<Term, A> {
    pub(crate) fn as_typed_term<V>(&self) -> &ArenaRef<TypedTerm<V>, A> {
        unsafe { std::mem::transmute::<&ArenaRef<Term, A>, &ArenaRef<TypedTerm<V>, A>>(self) }
    }
    pub(crate) fn into_typed_term<V>(self) -> ArenaRef<TypedTerm<V>, A> {
        let term = unsafe {
            std::mem::transmute_copy::<ArenaRef<Term, A>, ArenaRef<TypedTerm<V>, A>>(&self)
        };
        std::mem::forget(self);
        term
    }
    pub fn as_application_term(&self) -> Option<&ArenaRef<TypedTerm<ApplicationTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => Some(self.as_typed_term::<ApplicationTerm>()),
            _ => None,
        }
    }
    pub fn into_application_term(self) -> Option<ArenaRef<TypedTerm<ApplicationTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => Some(self.into_typed_term::<ApplicationTerm>()),
            _ => None,
        }
    }
    pub fn as_boolean_term(&self) -> Option<&ArenaRef<TypedTerm<BooleanTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Boolean => Some(self.as_typed_term::<BooleanTerm>()),
            _ => None,
        }
    }
    pub fn into_boolean_term(self) -> Option<ArenaRef<TypedTerm<BooleanTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Boolean => Some(self.into_typed_term::<BooleanTerm>()),
            _ => None,
        }
    }
    pub fn as_builtin_term(&self) -> Option<&ArenaRef<TypedTerm<BuiltinTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Builtin => Some(self.as_typed_term::<BuiltinTerm>()),
            _ => None,
        }
    }
    pub fn into_builtin_term(self) -> Option<ArenaRef<TypedTerm<BuiltinTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Builtin => Some(self.into_typed_term::<BuiltinTerm>()),
            _ => None,
        }
    }
    pub fn as_cell_term(&self) -> Option<&ArenaRef<TypedTerm<CellTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Cell => Some(self.as_typed_term::<CellTerm>()),
            _ => None,
        }
    }
    pub fn into_cell_term(self) -> Option<ArenaRef<TypedTerm<CellTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Cell => Some(self.into_typed_term::<CellTerm>()),
            _ => None,
        }
    }
    pub fn as_condition_term(&self) -> Option<&ArenaRef<TypedTerm<ConditionTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Condition => Some(self.as_typed_term::<ConditionTerm>()),
            _ => None,
        }
    }
    pub fn into_condition_term(self) -> Option<ArenaRef<TypedTerm<ConditionTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Condition => Some(self.into_typed_term::<ConditionTerm>()),
            _ => None,
        }
    }
    pub fn as_constructor_term(&self) -> Option<&ArenaRef<TypedTerm<ConstructorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Constructor => Some(self.as_typed_term::<ConstructorTerm>()),
            _ => None,
        }
    }
    pub fn into_constructor_term(self) -> Option<ArenaRef<TypedTerm<ConstructorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Constructor => Some(self.into_typed_term::<ConstructorTerm>()),
            _ => None,
        }
    }
    pub fn as_date_term(&self) -> Option<&ArenaRef<TypedTerm<DateTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Date => Some(self.as_typed_term::<DateTerm>()),
            _ => None,
        }
    }
    pub fn into_date_term(self) -> Option<ArenaRef<TypedTerm<DateTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Date => Some(self.into_typed_term::<DateTerm>()),
            _ => None,
        }
    }
    pub fn as_effect_term(&self) -> Option<&ArenaRef<TypedTerm<EffectTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Effect => Some(self.as_typed_term::<EffectTerm>()),
            _ => None,
        }
    }
    pub fn into_effect_term(self) -> Option<ArenaRef<TypedTerm<EffectTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Effect => Some(self.into_typed_term::<EffectTerm>()),
            _ => None,
        }
    }
    pub fn as_float_term(&self) -> Option<&ArenaRef<TypedTerm<FloatTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Float => Some(self.as_typed_term::<FloatTerm>()),
            _ => None,
        }
    }
    pub fn into_float_term(self) -> Option<ArenaRef<TypedTerm<FloatTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Float => Some(self.into_typed_term::<FloatTerm>()),
            _ => None,
        }
    }
    pub fn as_hashmap_term(&self) -> Option<&ArenaRef<TypedTerm<HashmapTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Hashmap => Some(self.as_typed_term::<HashmapTerm>()),
            _ => None,
        }
    }
    pub fn into_hashmap_term(self) -> Option<ArenaRef<TypedTerm<HashmapTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Hashmap => Some(self.into_typed_term::<HashmapTerm>()),
            _ => None,
        }
    }
    pub fn as_hashset_term(&self) -> Option<&ArenaRef<TypedTerm<HashsetTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Hashset => Some(self.as_typed_term::<HashsetTerm>()),
            _ => None,
        }
    }
    pub fn into_hashset_term(self) -> Option<ArenaRef<TypedTerm<HashsetTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Hashset => Some(self.into_typed_term::<HashsetTerm>()),
            _ => None,
        }
    }
    pub fn as_int_term(&self) -> Option<&ArenaRef<TypedTerm<IntTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Int => Some(self.as_typed_term::<IntTerm>()),
            _ => None,
        }
    }
    pub fn into_int_term(self) -> Option<ArenaRef<TypedTerm<IntTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Int => Some(self.into_typed_term::<IntTerm>()),
            _ => None,
        }
    }
    pub fn as_lambda_term(&self) -> Option<&ArenaRef<TypedTerm<LambdaTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Lambda => Some(self.as_typed_term::<LambdaTerm>()),
            _ => None,
        }
    }
    pub fn into_lambda_term(self) -> Option<ArenaRef<TypedTerm<LambdaTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Lambda => Some(self.into_typed_term::<LambdaTerm>()),
            _ => None,
        }
    }
    pub fn as_let_term(&self) -> Option<&ArenaRef<TypedTerm<LetTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Let => Some(self.as_typed_term::<LetTerm>()),
            _ => None,
        }
    }
    pub fn into_let_term(self) -> Option<ArenaRef<TypedTerm<LetTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Let => Some(self.into_typed_term::<LetTerm>()),
            _ => None,
        }
    }
    pub fn as_list_term(&self) -> Option<&ArenaRef<TypedTerm<ListTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::List => Some(self.as_typed_term::<ListTerm>()),
            _ => None,
        }
    }
    pub fn into_list_term(self) -> Option<ArenaRef<TypedTerm<ListTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::List => Some(self.into_typed_term::<ListTerm>()),
            _ => None,
        }
    }
    pub fn as_nil_term(&self) -> Option<&ArenaRef<TypedTerm<NilTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Nil => Some(self.as_typed_term::<NilTerm>()),
            _ => None,
        }
    }
    pub fn into_nil_term(self) -> Option<ArenaRef<TypedTerm<NilTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Nil => Some(self.into_typed_term::<NilTerm>()),
            _ => None,
        }
    }
    pub fn as_partial_term(&self) -> Option<&ArenaRef<TypedTerm<PartialTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Partial => Some(self.as_typed_term::<PartialTerm>()),
            _ => None,
        }
    }
    pub fn into_partial_term(self) -> Option<ArenaRef<TypedTerm<PartialTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Partial => Some(self.into_typed_term::<PartialTerm>()),
            _ => None,
        }
    }
    pub fn as_pointer_term(&self) -> Option<&ArenaRef<TypedTerm<PointerTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Pointer => Some(self.as_typed_term::<PointerTerm>()),
            _ => None,
        }
    }
    pub fn into_pointer_term(self) -> Option<ArenaRef<TypedTerm<PointerTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Pointer => Some(self.into_typed_term::<PointerTerm>()),
            _ => None,
        }
    }
    pub fn as_record_term(&self) -> Option<&ArenaRef<TypedTerm<RecordTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Record => Some(self.as_typed_term::<RecordTerm>()),
            _ => None,
        }
    }
    pub fn into_record_term(self) -> Option<ArenaRef<TypedTerm<RecordTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Record => Some(self.into_typed_term::<RecordTerm>()),
            _ => None,
        }
    }
    pub fn as_signal_term(&self) -> Option<&ArenaRef<TypedTerm<SignalTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Signal => Some(self.as_typed_term::<SignalTerm>()),
            _ => None,
        }
    }
    pub fn into_signal_term(self) -> Option<ArenaRef<TypedTerm<SignalTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Signal => Some(self.into_typed_term::<SignalTerm>()),
            _ => None,
        }
    }
    pub fn as_string_term(&self) -> Option<&ArenaRef<TypedTerm<StringTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::String => Some(self.as_typed_term::<StringTerm>()),
            _ => None,
        }
    }
    pub fn into_string_term(self) -> Option<ArenaRef<TypedTerm<StringTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::String => Some(self.into_typed_term::<StringTerm>()),
            _ => None,
        }
    }
    pub fn as_symbol_term(&self) -> Option<&ArenaRef<TypedTerm<SymbolTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Symbol => Some(self.as_typed_term::<SymbolTerm>()),
            _ => None,
        }
    }
    pub fn into_symbol_term(self) -> Option<ArenaRef<TypedTerm<SymbolTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Symbol => Some(self.into_typed_term::<SymbolTerm>()),
            _ => None,
        }
    }
    pub fn as_tree_term(&self) -> Option<&ArenaRef<TypedTerm<TreeTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Tree => Some(self.as_typed_term::<TreeTerm>()),
            _ => None,
        }
    }
    pub fn into_tree_term(self) -> Option<ArenaRef<TypedTerm<TreeTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Tree => Some(self.into_typed_term::<TreeTerm>()),
            _ => None,
        }
    }
    pub fn as_variable_term(&self) -> Option<&ArenaRef<TypedTerm<VariableTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Variable => Some(self.as_typed_term::<VariableTerm>()),
            _ => None,
        }
    }
    pub fn into_variable_term(self) -> Option<ArenaRef<TypedTerm<VariableTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Variable => Some(self.into_typed_term::<VariableTerm>()),
            _ => None,
        }
    }
    pub fn as_empty_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<EmptyIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::EmptyIterator => Some(self.as_typed_term::<EmptyIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_empty_iterator_term(self) -> Option<ArenaRef<TypedTerm<EmptyIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::EmptyIterator => {
                Some(self.into_typed_term::<EmptyIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_evaluate_iterator_term(
        &self,
    ) -> Option<&ArenaRef<TypedTerm<EvaluateIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::EvaluateIterator => {
                Some(self.as_typed_term::<EvaluateIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_evaluate_iterator_term(
        self,
    ) -> Option<ArenaRef<TypedTerm<EvaluateIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::EvaluateIterator => {
                Some(self.into_typed_term::<EvaluateIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_filter_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<FilterIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::FilterIterator => {
                Some(self.as_typed_term::<FilterIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_filter_iterator_term(self) -> Option<ArenaRef<TypedTerm<FilterIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::FilterIterator => {
                Some(self.into_typed_term::<FilterIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_flatten_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<FlattenIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::FlattenIterator => {
                Some(self.as_typed_term::<FlattenIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_flatten_iterator_term(self) -> Option<ArenaRef<TypedTerm<FlattenIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::FlattenIterator => {
                Some(self.into_typed_term::<FlattenIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_hashmap_keys_iterator_term(
        &self,
    ) -> Option<&ArenaRef<TypedTerm<HashmapKeysIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::HashmapKeysIterator => {
                Some(self.as_typed_term::<HashmapKeysIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_hashmap_keys_iterator_term(
        self,
    ) -> Option<ArenaRef<TypedTerm<HashmapKeysIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::HashmapKeysIterator => {
                Some(self.into_typed_term::<HashmapKeysIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_hashmap_values_iterator_term(
        &self,
    ) -> Option<&ArenaRef<TypedTerm<HashmapValuesIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::HashmapValuesIterator => {
                Some(self.as_typed_term::<HashmapValuesIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_hashmap_values_iterator_term(
        self,
    ) -> Option<ArenaRef<TypedTerm<HashmapValuesIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::HashmapValuesIterator => {
                Some(self.into_typed_term::<HashmapValuesIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_indexed_accessor_iterator_term(
        &self,
    ) -> Option<&ArenaRef<TypedTerm<IndexedAccessorIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::IndexedAccessorIterator => {
                Some(self.as_typed_term::<IndexedAccessorIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_indexed_accessor_iterator_term(
        self,
    ) -> Option<ArenaRef<TypedTerm<IndexedAccessorIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::IndexedAccessorIterator => {
                Some(self.into_typed_term::<IndexedAccessorIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_integers_iterator_term(
        &self,
    ) -> Option<&ArenaRef<TypedTerm<IntegersIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::IntegersIterator => {
                Some(self.as_typed_term::<IntegersIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_integers_iterator_term(
        self,
    ) -> Option<ArenaRef<TypedTerm<IntegersIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::IntegersIterator => {
                Some(self.into_typed_term::<IntegersIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_intersperse_iterator_term(
        &self,
    ) -> Option<&ArenaRef<TypedTerm<IntersperseIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::IntersperseIterator => {
                Some(self.as_typed_term::<IntersperseIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_intersperse_iterator_term(
        self,
    ) -> Option<ArenaRef<TypedTerm<IntersperseIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::IntersperseIterator => {
                Some(self.into_typed_term::<IntersperseIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_map_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<MapIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::MapIterator => Some(self.as_typed_term::<MapIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_map_iterator_term(self) -> Option<ArenaRef<TypedTerm<MapIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::MapIterator => Some(self.into_typed_term::<MapIteratorTerm>()),
            _ => None,
        }
    }
    pub fn as_once_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<OnceIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::OnceIterator => Some(self.as_typed_term::<OnceIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_once_iterator_term(self) -> Option<ArenaRef<TypedTerm<OnceIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::OnceIterator => Some(self.into_typed_term::<OnceIteratorTerm>()),
            _ => None,
        }
    }
    pub fn as_range_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<RangeIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::RangeIterator => Some(self.as_typed_term::<RangeIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_range_iterator_term(self) -> Option<ArenaRef<TypedTerm<RangeIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::RangeIterator => {
                Some(self.into_typed_term::<RangeIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_repeat_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<RepeatIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::RepeatIterator => {
                Some(self.as_typed_term::<RepeatIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn into_repeat_iterator_term(self) -> Option<ArenaRef<TypedTerm<RepeatIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::RepeatIterator => {
                Some(self.into_typed_term::<RepeatIteratorTerm>())
            }
            _ => None,
        }
    }
    pub fn as_skip_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<SkipIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::SkipIterator => Some(self.as_typed_term::<SkipIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_skip_iterator_term(self) -> Option<ArenaRef<TypedTerm<SkipIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::SkipIterator => Some(self.into_typed_term::<SkipIteratorTerm>()),
            _ => None,
        }
    }
    pub fn as_take_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<TakeIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::TakeIterator => Some(self.as_typed_term::<TakeIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_take_iterator_term(self) -> Option<ArenaRef<TypedTerm<TakeIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::TakeIterator => Some(self.into_typed_term::<TakeIteratorTerm>()),
            _ => None,
        }
    }
    pub fn as_zip_iterator_term(&self) -> Option<&ArenaRef<TypedTerm<ZipIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::ZipIterator => Some(self.as_typed_term::<ZipIteratorTerm>()),
            _ => None,
        }
    }
    pub fn into_zip_iterator_term(self) -> Option<ArenaRef<TypedTerm<ZipIteratorTerm>, A>> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::ZipIterator => Some(self.into_typed_term::<ZipIteratorTerm>()),
            _ => None,
        }
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
        assert_eq!(std::mem::size_of::<TermType>(), 36);
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
        assert_eq!(TermTypeDiscriminants::IndexedAccessorIterator as u32, 30);
        assert_eq!(TermTypeDiscriminants::IntegersIterator as u32, 31);
        assert_eq!(TermTypeDiscriminants::IntersperseIterator as u32, 32);
        assert_eq!(TermTypeDiscriminants::MapIterator as u32, 33);
        assert_eq!(TermTypeDiscriminants::OnceIterator as u32, 34);
        assert_eq!(TermTypeDiscriminants::RangeIterator as u32, 35);
        assert_eq!(TermTypeDiscriminants::RepeatIterator as u32, 36);
        assert_eq!(TermTypeDiscriminants::SkipIterator as u32, 37);
        assert_eq!(TermTypeDiscriminants::TakeIterator as u32, 38);
        assert_eq!(TermTypeDiscriminants::ZipIterator as u32, 39);
    }
}
