// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, marker::PhantomData};

use reflex::{
    core::{Arity, DependencyList, Expression, GraphNode, NodeId, SerializeJson, StackOffset},
    hash::HashId,
};
use serde_json::Value as JsonValue;
use strum_macros::EnumDiscriminants;

mod application;
mod boolean;
mod builtin;
mod cell;
mod compiled;
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
pub use compiled::*;
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
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[repr(C)]
pub enum TermType {
    Application(ApplicationTerm),
    Boolean(BooleanTerm),
    Builtin(BuiltinTerm),
    Cell(CellTerm),
    Compiled(CompiledTerm),
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
            value if value == Self::Compiled as u32 => Ok(Self::Compiled),
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
        let discriminant_size = std::mem::size_of::<u32>();
        let value_size = match self {
            TermType::Application(term) => term.size_of(),
            Self::Boolean(term) => term.size_of(),
            Self::Builtin(term) => term.size_of(),
            Self::Cell(term) => term.size_of(),
            Self::Compiled(term) => term.size_of(),
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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        match self {
            TermType::Application(term) => hasher
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
            Self::Compiled(term) => hasher
                .write_u8(TermTypeDiscriminants::Compiled as u8)
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
impl<'a> Into<Option<&'a CompiledTerm>> for &'a TermType {
    fn into(self) -> Option<&'a CompiledTerm> {
        match self {
            TermType::Compiled(term) => Some(term),
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

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, Term, A> {
    pub fn arity(&self) -> Option<Arity> {
        match &self.as_value().value {
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)).arity()
            }
            TermType::Compiled(term) => Some(
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)).arity(),
            ),
            TermType::Constructor(term) => Some(
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .arity(),
            ),
            TermType::Lambda(term) => Some(
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).arity(),
            ),
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)).arity()
            }
            _ => None,
        }
    }
}

impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, Term, A> {}
impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, Term, A> {
    fn eq(&self, other: &Self) -> bool {
        if self.as_value().header.hash != other.as_value().header.hash {
            return false;
        }
        match (&self.as_value().value, &other.as_value().value) {
            (TermType::Application(left), TermType::Application(right)) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<ApplicationTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::Boolean(left), TermType::Boolean(right)) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<BooleanTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Builtin(left), TermType::Builtin(right)) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<BuiltinTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Cell(left), TermType::Cell(right)) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<CellTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Compiled(left), TermType::Compiled(right)) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<CompiledTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Condition(left), TermType::Condition(right)) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<ConditionTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Constructor(left), TermType::Constructor(right)) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<ConstructorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::Date(left), TermType::Date(right)) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<DateTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Effect(left), TermType::Effect(right)) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<EffectTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Float(left), TermType::Float(right)) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<FloatTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Hashmap(left), TermType::Hashmap(right)) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<HashmapTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Hashset(left), TermType::Hashset(right)) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<HashsetTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Int(left), TermType::Int(right)) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<IntTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Lambda(left), TermType::Lambda(right)) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<LambdaTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Let(left), TermType::Let(right)) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<LetTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::List(left), TermType::List(right)) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<ListTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Nil(left), TermType::Nil(right)) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<NilTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Partial(left), TermType::Partial(right)) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<PartialTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Pointer(left), TermType::Pointer(right)) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<PointerTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Record(left), TermType::Record(right)) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<RecordTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Signal(left), TermType::Signal(right)) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<SignalTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::String(left), TermType::String(right)) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<StringTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Symbol(left), TermType::Symbol(right)) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<SymbolTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Tree(left), TermType::Tree(right)) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<TreeTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::Variable(left), TermType::Variable(right)) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<VariableTerm, _>::new(other.arena, self.arena.get_offset(right))
            }
            (TermType::EmptyIterator(left), TermType::EmptyIterator(right)) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<EmptyIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::EvaluateIterator(left), TermType::EvaluateIterator(right)) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<EvaluateIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::FilterIterator(left), TermType::FilterIterator(right)) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<FilterIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::FlattenIterator(left), TermType::FlattenIterator(right)) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<FlattenIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::HashmapKeysIterator(left), TermType::HashmapKeysIterator(right)) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<HashmapKeysIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::HashmapValuesIterator(left), TermType::HashmapValuesIterator(right)) => {
                ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(left),
                ) == ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    other.arena,
                    self.arena.get_offset(right),
                )
            }
            (TermType::IntegersIterator(left), TermType::IntegersIterator(right)) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<IntegersIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::IntersperseIterator(left), TermType::IntersperseIterator(right)) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<IntersperseIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::MapIterator(left), TermType::MapIterator(right)) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<MapIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::OnceIterator(left), TermType::OnceIterator(right)) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<OnceIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::RangeIterator(left), TermType::RangeIterator(right)) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<RangeIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::RepeatIterator(left), TermType::RepeatIterator(right)) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<RepeatIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::SkipIterator(left), TermType::SkipIterator(right)) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<SkipIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::TakeIterator(left), TermType::TakeIterator(right)) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<TakeIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            (TermType::ZipIterator(left), TermType::ZipIterator(right)) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(left))
                    == ArenaRef::<ZipIteratorTerm, _>::new(
                        other.arena,
                        self.arena.get_offset(right),
                    )
            }
            _ => false,
        }
    }
}

pub(crate) type WasmExpression<'heap, A> = ArenaRef<'heap, Term, A>;

impl<'heap, A: ArenaAllocator> Expression for ArenaRef<'heap, Term, A> {
    type String = ArenaRef<'heap, TypedTerm<StringTerm>, A>;
    type Builtin = Stdlib;
    type Signal = ArenaRef<'heap, TypedTerm<ConditionTerm>, A>;
    type SignalList = ArenaRef<'heap, TypedTerm<TreeTerm>, A>;
    type StructPrototype = ArenaRef<'heap, TypedTerm<ListTerm>, A>;
    type ExpressionList = ArenaRef<'heap, TypedTerm<ListTerm>, A>;
    type NilTerm = ArenaRef<'heap, TypedTerm<NilTerm>, A>;
    type BooleanTerm = ArenaRef<'heap, TypedTerm<BooleanTerm>, A>;
    type IntTerm = ArenaRef<'heap, TypedTerm<IntTerm>, A>;
    type FloatTerm = ArenaRef<'heap, TypedTerm<FloatTerm>, A>;
    type StringTerm = ArenaRef<'heap, TypedTerm<StringTerm>, A>;
    type SymbolTerm = ArenaRef<'heap, TypedTerm<SymbolTerm>, A>;
    type VariableTerm = ArenaRef<'heap, TypedTerm<VariableTerm>, A>;
    type EffectTerm = ArenaRef<'heap, TypedTerm<EffectTerm>, A>;
    type LetTerm = ArenaRef<'heap, TypedTerm<LetTerm>, A>;
    type LambdaTerm = ArenaRef<'heap, TypedTerm<LambdaTerm>, A>;
    type ApplicationTerm = ArenaRef<'heap, TypedTerm<ApplicationTerm>, A>;
    type PartialApplicationTerm = ArenaRef<'heap, TypedTerm<PartialTerm>, A>;
    // FIXME: implement recursive term type
    type RecursiveTerm = ArenaRef<'heap, TypedTerm<NilTerm>, A>;
    type BuiltinTerm = ArenaRef<'heap, TypedTerm<BuiltinTerm>, A>;
    type CompiledFunctionTerm = ArenaRef<'heap, TypedTerm<CompiledTerm>, A>;
    type RecordTerm = ArenaRef<'heap, TypedTerm<RecordTerm>, A>;
    type ConstructorTerm = ArenaRef<'heap, TypedTerm<ConstructorTerm>, A>;
    type ListTerm = ArenaRef<'heap, TypedTerm<ListTerm>, A>;
    type HashmapTerm = ArenaRef<'heap, TypedTerm<HashmapTerm>, A>;
    type HashsetTerm = ArenaRef<'heap, TypedTerm<HashsetTerm>, A>;
    type SignalTerm = ArenaRef<'heap, TypedTerm<SignalTerm>, A>;

    type StringRef<'a> = ArenaRef<'heap, TypedTerm<StringTerm>, A> where Self: 'a;
    type SignalRef<'a> = ArenaRef<'heap, TypedTerm<ConditionTerm>, A> where Self::Signal: 'a, Self: 'a;
    type StructPrototypeRef<'a> = ArenaRef<'heap, TypedTerm<ListTerm>, A> where Self::StructPrototype: 'a, Self: 'a;
    type SignalListRef<'a> = ArenaRef<'heap, TypedTerm<TreeTerm>, A> where Self::SignalList: 'a, Self: 'a;
    type ExpressionListRef<'a> = ArenaRef<'heap, TypedTerm<ListTerm>, A> where Self::ExpressionList: 'a, Self: 'a;
    type ExpressionRef<'a> = ArenaRef<'heap, Term, A> where Self: 'a;
}

impl<'heap, A: ArenaAllocator> NodeId for ArenaRef<'heap, Term, A> {
    fn id(&self) -> HashId {
        self.as_value().id()
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, Term, A> {
    fn size(&self) -> usize {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .size(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .size()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).size()
            }
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).capture_depth()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).capture_depth()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).capture_depth()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .capture_depth(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .capture_depth()
            }
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .free_variables(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .free_variables()
            }
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .count_variable_usages(offset),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .count_variable_usages(offset)
            }
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .dynamic_dependencies(deep),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .dynamic_dependencies(deep)
            }
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .has_dynamic_dependencies(deep),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .has_dynamic_dependencies(deep)
            }
        }
    }
    fn is_static(&self) -> bool {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)).is_static()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .is_static(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
        }
    }
    fn is_atomic(&self) -> bool {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)).is_atomic()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .is_atomic(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_atomic()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_static()
            }
        }
    }
    fn is_complex(&self) -> bool {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)).is_complex()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .is_complex(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .is_complex()
            }
        }
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, Term, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        match &self.as_value().value {
            TermType::Application(term) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::Boolean(term) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Builtin(term) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Cell(term) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Compiled(term) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Condition(term) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Constructor(term) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::Date(term) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Effect(term) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Float(term) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Hashmap(term) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Hashset(term) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Int(term) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Lambda(term) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Let(term) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::List(term) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Nil(term) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Partial(term) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Pointer(term) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Record(term) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Signal(term) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::String(term) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Symbol(term) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Tree(term) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::Variable(term) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term)).to_json()
            }
            TermType::EmptyIterator(term) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::EvaluateIterator(term) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::FilterIterator(term) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::FlattenIterator(term) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::HashmapKeysIterator(term) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::HashmapValuesIterator(term) => ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                self.arena,
                self.arena.get_offset(term),
            )
            .to_json(),
            TermType::IntegersIterator(term) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::IntersperseIterator(term) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::MapIterator(term) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::OnceIterator(term) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::RangeIterator(term) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::RepeatIterator(term) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::SkipIterator(term) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::TakeIterator(term) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
            TermType::ZipIterator(term) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .to_json()
            }
        }
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.id() == target.id() {
            return Ok(None);
        }
        match (&self.as_value().value, &target.as_value().value) {
            (TermType::Application(term), TermType::Application(target)) => {
                ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Boolean(term), TermType::Boolean(target)) => {
                ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Builtin(term), TermType::Builtin(target)) => {
                ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Cell(term), TermType::Cell(target)) => {
                ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Compiled(term), TermType::Compiled(target)) => {
                ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Condition(term), TermType::Condition(target)) => {
                ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Constructor(term), TermType::Constructor(target)) => {
                ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Date(term), TermType::Date(target)) => {
                ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Effect(term), TermType::Effect(target)) => {
                ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Float(term), TermType::Float(target)) => {
                ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Hashmap(term), TermType::Hashmap(target)) => {
                ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Hashset(term), TermType::Hashset(target)) => {
                ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Int(term), TermType::Int(target)) => {
                ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Lambda(term), TermType::Lambda(target)) => {
                ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Let(term), TermType::Let(target)) => {
                ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::List(term), TermType::List(target)) => {
                ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Nil(term), TermType::Nil(target)) => {
                ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Partial(term), TermType::Partial(target)) => {
                ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Pointer(term), TermType::Pointer(target)) => {
                ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Record(term), TermType::Record(target)) => {
                ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Signal(term), TermType::Signal(target)) => {
                ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::String(term), TermType::String(target)) => {
                ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Symbol(term), TermType::Symbol(target)) => {
                ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Tree(term), TermType::Tree(target)) => {
                ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::Variable(term), TermType::Variable(target)) => {
                ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::EmptyIterator(term), TermType::EmptyIterator(target)) => {
                ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<EmptyIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::EvaluateIterator(term), TermType::EvaluateIterator(target)) => {
                ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<EvaluateIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::FilterIterator(term), TermType::FilterIterator(target)) => {
                ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<FilterIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::FlattenIterator(term), TermType::FlattenIterator(target)) => {
                ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<FlattenIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::HashmapKeysIterator(term), TermType::HashmapKeysIterator(target)) => {
                ArenaRef::<HashmapKeysIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<HashmapKeysIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::HashmapValuesIterator(term), TermType::HashmapValuesIterator(target)) => {
                ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                )
                .patch(&ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(target),
                ))
            }
            (TermType::IntegersIterator(term), TermType::IntegersIterator(target)) => {
                ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<IntegersIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::IntersperseIterator(term), TermType::IntersperseIterator(target)) => {
                ArenaRef::<IntersperseIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<IntersperseIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::MapIterator(term), TermType::MapIterator(target)) => {
                ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            (TermType::OnceIterator(term), TermType::OnceIterator(target)) => {
                ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<OnceIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ),
                )
            }
            (TermType::RangeIterator(term), TermType::RangeIterator(target)) => {
                ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<RangeIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::RepeatIterator(term), TermType::RepeatIterator(target)) => {
                ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term))
                    .patch(&ArenaRef::<RepeatIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ))
            }
            (TermType::SkipIterator(term), TermType::SkipIterator(target)) => {
                ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<SkipIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ),
                )
            }
            (TermType::TakeIterator(term), TermType::TakeIterator(target)) => {
                ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<TakeIteratorTerm, _>::new(
                        self.arena,
                        self.arena.get_offset(target),
                    ),
                )
            }
            (TermType::ZipIterator(term), TermType::ZipIterator(target)) => {
                ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)).patch(
                    &ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(target)),
                )
            }
            _ => target.to_json().map(Some),
        }
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, Term, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.as_value().value {
            TermType::Application(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Boolean(term) => std::fmt::Debug::fmt(
                &ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Builtin(term) => std::fmt::Debug::fmt(
                &ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Cell(term) => std::fmt::Debug::fmt(
                &ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Compiled(term) => std::fmt::Debug::fmt(
                &ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Condition(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Constructor(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Date(term) => std::fmt::Debug::fmt(
                &ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Effect(term) => std::fmt::Debug::fmt(
                &ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Float(term) => std::fmt::Debug::fmt(
                &ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Hashmap(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Hashset(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Int(term) => std::fmt::Debug::fmt(
                &ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Lambda(term) => std::fmt::Debug::fmt(
                &ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Let(term) => std::fmt::Debug::fmt(
                &ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::List(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Nil(term) => std::fmt::Debug::fmt(
                &ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Partial(term) => std::fmt::Debug::fmt(
                &ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Pointer(term) => std::fmt::Debug::fmt(
                &ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Record(term) => std::fmt::Debug::fmt(
                &ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Signal(term) => std::fmt::Debug::fmt(
                &ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::String(term) => std::fmt::Debug::fmt(
                &ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Symbol(term) => std::fmt::Debug::fmt(
                &ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Tree(term) => std::fmt::Debug::fmt(
                &ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Variable(term) => std::fmt::Debug::fmt(
                &ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::EmptyIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::EvaluateIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::FilterIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::FlattenIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::HashmapKeysIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashmapKeysIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::HashmapValuesIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::IntegersIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::IntersperseIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<IntersperseIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::MapIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::OnceIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::RangeIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::RepeatIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::SkipIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::TakeIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::ZipIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
        }
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, Term, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.as_value().value {
            TermType::Application(term) => std::fmt::Display::fmt(
                &ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Boolean(term) => std::fmt::Display::fmt(
                &ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Builtin(term) => std::fmt::Display::fmt(
                &ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Cell(term) => std::fmt::Display::fmt(
                &ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Compiled(term) => std::fmt::Display::fmt(
                &ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Condition(term) => std::fmt::Display::fmt(
                &ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Constructor(term) => std::fmt::Display::fmt(
                &ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Date(term) => std::fmt::Display::fmt(
                &ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Effect(term) => std::fmt::Display::fmt(
                &ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Float(term) => std::fmt::Display::fmt(
                &ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Hashmap(term) => std::fmt::Display::fmt(
                &ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Hashset(term) => std::fmt::Display::fmt(
                &ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Int(term) => std::fmt::Display::fmt(
                &ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Lambda(term) => std::fmt::Display::fmt(
                &ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Let(term) => std::fmt::Display::fmt(
                &ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::List(term) => std::fmt::Display::fmt(
                &ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Nil(term) => std::fmt::Display::fmt(
                &ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Partial(term) => std::fmt::Display::fmt(
                &ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Pointer(term) => std::fmt::Display::fmt(
                &ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Record(term) => std::fmt::Display::fmt(
                &ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Signal(term) => std::fmt::Display::fmt(
                &ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::String(term) => std::fmt::Display::fmt(
                &ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Symbol(term) => std::fmt::Display::fmt(
                &ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Tree(term) => std::fmt::Display::fmt(
                &ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Variable(term) => std::fmt::Display::fmt(
                &ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::EmptyIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::EvaluateIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::FilterIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::FlattenIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::HashmapKeysIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<HashmapKeysIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::HashmapValuesIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::IntegersIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::IntersperseIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<IntersperseIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::MapIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::OnceIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::RangeIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::RepeatIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::SkipIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::TakeIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::ZipIterator(term) => std::fmt::Display::fmt(
                &ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
        }
    }
}
impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, TermType, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_value() {
            TermType::Application(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ApplicationTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Boolean(term) => std::fmt::Debug::fmt(
                &ArenaRef::<BooleanTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Builtin(term) => std::fmt::Debug::fmt(
                &ArenaRef::<BuiltinTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Cell(term) => std::fmt::Debug::fmt(
                &ArenaRef::<CellTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Compiled(term) => std::fmt::Debug::fmt(
                &ArenaRef::<CompiledTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Condition(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ConditionTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Constructor(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ConstructorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Date(term) => std::fmt::Debug::fmt(
                &ArenaRef::<DateTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Effect(term) => std::fmt::Debug::fmt(
                &ArenaRef::<EffectTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Float(term) => std::fmt::Debug::fmt(
                &ArenaRef::<FloatTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Hashmap(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashmapTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Hashset(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashsetTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Int(term) => std::fmt::Debug::fmt(
                &ArenaRef::<IntTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Lambda(term) => std::fmt::Debug::fmt(
                &ArenaRef::<LambdaTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Let(term) => std::fmt::Debug::fmt(
                &ArenaRef::<LetTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::List(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ListTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Nil(term) => std::fmt::Debug::fmt(
                &ArenaRef::<NilTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Partial(term) => std::fmt::Debug::fmt(
                &ArenaRef::<PartialTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Pointer(term) => std::fmt::Debug::fmt(
                &ArenaRef::<PointerTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Record(term) => std::fmt::Debug::fmt(
                &ArenaRef::<RecordTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Signal(term) => std::fmt::Debug::fmt(
                &ArenaRef::<SignalTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::String(term) => std::fmt::Debug::fmt(
                &ArenaRef::<StringTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Symbol(term) => std::fmt::Debug::fmt(
                &ArenaRef::<SymbolTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Tree(term) => std::fmt::Debug::fmt(
                &ArenaRef::<TreeTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::Variable(term) => std::fmt::Debug::fmt(
                &ArenaRef::<VariableTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::EmptyIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<EmptyIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::EvaluateIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<EvaluateIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::FilterIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<FilterIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::FlattenIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<FlattenIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::HashmapKeysIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashmapKeysIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::HashmapValuesIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<HashmapValuesIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::IntegersIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<IntegersIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::IntersperseIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<IntersperseIteratorTerm, _>::new(
                    self.arena,
                    self.arena.get_offset(term),
                ),
                f,
            ),
            TermType::MapIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<MapIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::OnceIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<OnceIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::RangeIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<RangeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::RepeatIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<RepeatIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::SkipIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<SkipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::TakeIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<TakeIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
            TermType::ZipIterator(term) => std::fmt::Debug::fmt(
                &ArenaRef::<ZipIteratorTerm, _>::new(self.arena, self.arena.get_offset(term)),
                f,
            ),
        }
    }
}

#[cfg(test)]
impl TermType {
    fn as_bytes(&self) -> &[u32] {
        let num_words = crate::pad_to_4_byte_offset(self.size_of() as usize) / 4;
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u32, num_words) }
    }
}

#[repr(C)]
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
                TermType::Compiled(inner) => std::mem::transmute::<&CompiledTerm, &V>(inner),
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

impl<'heap, A: ArenaAllocator, V> ArenaRef<'heap, TypedTerm<V>, A> {
    pub fn as_inner(&self) -> ArenaRef<'heap, V, A> {
        ArenaRef::<V, _>::new(
            self.arena,
            self.arena.get_offset(self.as_value().get_inner()),
        )
    }
    pub(crate) fn as_term(&self) -> &ArenaRef<'heap, Term, A> {
        unsafe {
            std::mem::transmute::<&ArenaRef<'heap, TypedTerm<V>, A>, &ArenaRef<'heap, Term, A>>(
                self,
            )
        }
    }
}

impl<'heap, A: ArenaAllocator, V> GraphNode for ArenaRef<'heap, TypedTerm<V>, A>
where
    ArenaRef<'heap, V, A>: GraphNode,
{
    fn size(&self) -> usize {
        <ArenaRef<'heap, V, A> as GraphNode>::size(&self.as_inner())
    }
    fn capture_depth(&self) -> StackOffset {
        <ArenaRef<'heap, V, A> as GraphNode>::capture_depth(&self.as_inner())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        <ArenaRef<'heap, V, A> as GraphNode>::free_variables(&self.as_inner())
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        <ArenaRef<'heap, V, A> as GraphNode>::count_variable_usages(&self.as_inner(), offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        <ArenaRef<'heap, V, A> as GraphNode>::dynamic_dependencies(&self.as_inner(), deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        <ArenaRef<'heap, V, A> as GraphNode>::has_dynamic_dependencies(&self.as_inner(), deep)
    }
    fn is_static(&self) -> bool {
        <ArenaRef<'heap, V, A> as GraphNode>::is_static(&self.as_inner())
    }
    fn is_atomic(&self) -> bool {
        <ArenaRef<'heap, V, A> as GraphNode>::is_atomic(&self.as_inner())
    }
    fn is_complex(&self) -> bool {
        <ArenaRef<'heap, V, A> as GraphNode>::is_complex(&self.as_inner())
    }
}

impl<'heap, A: ArenaAllocator, V> PartialEq for ArenaRef<'heap, TypedTerm<V>, A>
where
    ArenaRef<'heap, V, A>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_term() == other.as_term()
    }
}
impl<'heap, A: ArenaAllocator, V> Eq for ArenaRef<'heap, TypedTerm<V>, A> where
    ArenaRef<'heap, V, A>: Eq
{
}

impl<'heap, A: ArenaAllocator, V> std::fmt::Debug for ArenaRef<'heap, TypedTerm<V>, A>
where
    ArenaRef<'heap, V, A>: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Term")
            .field("hash", &self.as_value().id())
            .field("value", &self.as_inner())
            .finish()
    }
}

impl<'heap, A: ArenaAllocator, V> std::fmt::Display for ArenaRef<'heap, TypedTerm<V>, A>
where
    ArenaRef<'heap, V, A>: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <ArenaRef<'heap, V, A> as std::fmt::Display>::fmt(&self.as_inner(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_type() {
        assert_eq!(std::mem::size_of::<TermType>(), 20);
        assert_eq!(TermTypeDiscriminants::Application as u32, 0);
        assert_eq!(TermTypeDiscriminants::Boolean as u32, 1);
        assert_eq!(TermTypeDiscriminants::Builtin as u32, 2);
        assert_eq!(TermTypeDiscriminants::Cell as u32, 3);
        assert_eq!(TermTypeDiscriminants::Compiled as u32, 4);
        assert_eq!(TermTypeDiscriminants::Condition as u32, 5);
        assert_eq!(TermTypeDiscriminants::Constructor as u32, 6);
        assert_eq!(TermTypeDiscriminants::Date as u32, 7);
        assert_eq!(TermTypeDiscriminants::Effect as u32, 8);
        assert_eq!(TermTypeDiscriminants::Float as u32, 9);
        assert_eq!(TermTypeDiscriminants::Hashmap as u32, 10);
        assert_eq!(TermTypeDiscriminants::Hashset as u32, 11);
        assert_eq!(TermTypeDiscriminants::Int as u32, 12);
        assert_eq!(TermTypeDiscriminants::Lambda as u32, 13);
        assert_eq!(TermTypeDiscriminants::Let as u32, 14);
        assert_eq!(TermTypeDiscriminants::List as u32, 15);
        assert_eq!(TermTypeDiscriminants::Nil as u32, 16);
        assert_eq!(TermTypeDiscriminants::Partial as u32, 17);
        assert_eq!(TermTypeDiscriminants::Pointer as u32, 18);
        assert_eq!(TermTypeDiscriminants::Record as u32, 19);
        assert_eq!(TermTypeDiscriminants::Signal as u32, 20);
        assert_eq!(TermTypeDiscriminants::String as u32, 21);
        assert_eq!(TermTypeDiscriminants::Symbol as u32, 22);
        assert_eq!(TermTypeDiscriminants::Tree as u32, 23);
        assert_eq!(TermTypeDiscriminants::Variable as u32, 24);
        assert_eq!(TermTypeDiscriminants::EmptyIterator as u32, 25);
        assert_eq!(TermTypeDiscriminants::EvaluateIterator as u32, 26);
        assert_eq!(TermTypeDiscriminants::FilterIterator as u32, 27);
        assert_eq!(TermTypeDiscriminants::FlattenIterator as u32, 28);
        assert_eq!(TermTypeDiscriminants::HashmapKeysIterator as u32, 29);
        assert_eq!(TermTypeDiscriminants::HashmapValuesIterator as u32, 30);
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
