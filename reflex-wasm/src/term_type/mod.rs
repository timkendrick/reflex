// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::RefType;
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
    ArenaRef,
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
impl TermSize for TermType {
    fn size(&self) -> usize {
        let discriminant_size = std::mem::size_of::<u32>();
        let value_size = match self {
            Self::Application(term) => term.size(),
            Self::Boolean(term) => term.size(),
            Self::Builtin(term) => term.size(),
            Self::Cell(term) => term.size(),
            Self::Compiled(term) => term.size(),
            Self::Condition(term) => term.size(),
            Self::Constructor(term) => term.size(),
            Self::Date(term) => term.size(),
            Self::Effect(term) => term.size(),
            Self::Float(term) => term.size(),
            Self::Hashmap(term) => term.size(),
            Self::Hashset(term) => term.size(),
            Self::Int(term) => term.size(),
            Self::Lambda(term) => term.size(),
            Self::Let(term) => term.size(),
            Self::List(term) => term.size(),
            Self::Nil(term) => term.size(),
            Self::Partial(term) => term.size(),
            Self::Pointer(term) => term.size(),
            Self::Record(term) => term.size(),
            Self::Signal(term) => term.size(),
            Self::String(term) => term.size(),
            Self::Symbol(term) => term.size(),
            Self::Tree(term) => term.size(),
            Self::Variable(term) => term.size(),
            Self::EmptyIterator(term) => term.size(),
            Self::EvaluateIterator(term) => term.size(),
            Self::FilterIterator(term) => term.size(),
            Self::FlattenIterator(term) => term.size(),
            Self::HashmapKeysIterator(term) => term.size(),
            Self::HashmapValuesIterator(term) => term.size(),
            Self::IntegersIterator(term) => term.size(),
            Self::IntersperseIterator(term) => term.size(),
            Self::MapIterator(term) => term.size(),
            Self::OnceIterator(term) => term.size(),
            Self::RangeIterator(term) => term.size(),
            Self::RepeatIterator(term) => term.size(),
            Self::SkipIterator(term) => term.size(),
            Self::TakeIterator(term) => term.size(),
            Self::ZipIterator(term) => term.size(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for TermType {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
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
            Self::Application(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a BooleanTerm>> for &'a TermType {
    fn into(self) -> Option<&'a BooleanTerm> {
        match self {
            Self::Boolean(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a BuiltinTerm>> for &'a TermType {
    fn into(self) -> Option<&'a BuiltinTerm> {
        match self {
            Self::Builtin(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a CellTerm>> for &'a TermType {
    fn into(self) -> Option<&'a CellTerm> {
        match self {
            Self::Cell(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a CompiledTerm>> for &'a TermType {
    fn into(self) -> Option<&'a CompiledTerm> {
        match self {
            Self::Compiled(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ConditionTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ConditionTerm> {
        match self {
            Self::Condition(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ConstructorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ConstructorTerm> {
        match self {
            Self::Constructor(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a DateTerm>> for &'a TermType {
    fn into(self) -> Option<&'a DateTerm> {
        match self {
            Self::Date(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a EffectTerm>> for &'a TermType {
    fn into(self) -> Option<&'a EffectTerm> {
        match self {
            Self::Effect(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a FloatTerm>> for &'a TermType {
    fn into(self) -> Option<&'a FloatTerm> {
        match self {
            Self::Float(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashmapTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashmapTerm> {
        match self {
            Self::Hashmap(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashsetTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashsetTerm> {
        match self {
            Self::Hashset(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a IntTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IntTerm> {
        match self {
            Self::Int(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a LambdaTerm>> for &'a TermType {
    fn into(self) -> Option<&'a LambdaTerm> {
        match self {
            Self::Lambda(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a LetTerm>> for &'a TermType {
    fn into(self) -> Option<&'a LetTerm> {
        match self {
            Self::Let(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ListTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ListTerm> {
        match self {
            Self::List(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a NilTerm>> for &'a TermType {
    fn into(self) -> Option<&'a NilTerm> {
        match self {
            Self::Nil(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a PartialTerm>> for &'a TermType {
    fn into(self) -> Option<&'a PartialTerm> {
        match self {
            Self::Partial(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a PointerTerm>> for &'a TermType {
    fn into(self) -> Option<&'a PointerTerm> {
        match self {
            Self::Pointer(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a RecordTerm>> for &'a TermType {
    fn into(self) -> Option<&'a RecordTerm> {
        match self {
            Self::Record(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a SignalTerm>> for &'a TermType {
    fn into(self) -> Option<&'a SignalTerm> {
        match self {
            Self::Signal(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a StringTerm>> for &'a TermType {
    fn into(self) -> Option<&'a StringTerm> {
        match self {
            Self::String(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a SymbolTerm>> for &'a TermType {
    fn into(self) -> Option<&'a SymbolTerm> {
        match self {
            Self::Symbol(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a TreeTerm>> for &'a TermType {
    fn into(self) -> Option<&'a TreeTerm> {
        match self {
            Self::Tree(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a VariableTerm>> for &'a TermType {
    fn into(self) -> Option<&'a VariableTerm> {
        match self {
            Self::Variable(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a EmptyIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a EmptyIteratorTerm> {
        match self {
            Self::EmptyIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a EvaluateIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a EvaluateIteratorTerm> {
        match self {
            Self::EvaluateIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a FilterIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a FilterIteratorTerm> {
        match self {
            Self::FilterIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a FlattenIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a FlattenIteratorTerm> {
        match self {
            Self::FlattenIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashmapKeysIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashmapKeysIteratorTerm> {
        match self {
            Self::HashmapKeysIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a HashmapValuesIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a HashmapValuesIteratorTerm> {
        match self {
            Self::HashmapValuesIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a IntegersIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IntegersIteratorTerm> {
        match self {
            Self::IntegersIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a IntersperseIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a IntersperseIteratorTerm> {
        match self {
            Self::IntersperseIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a MapIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a MapIteratorTerm> {
        match self {
            Self::MapIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a OnceIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a OnceIteratorTerm> {
        match self {
            Self::OnceIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a RangeIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a RangeIteratorTerm> {
        match self {
            Self::RangeIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a RepeatIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a RepeatIteratorTerm> {
        match self {
            Self::RepeatIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a SkipIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a SkipIteratorTerm> {
        match self {
            Self::SkipIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a TakeIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a TakeIteratorTerm> {
        match self {
            Self::TakeIterator(term) => Some(term),
            _ => None,
        }
    }
}
impl<'a> Into<Option<&'a ZipIteratorTerm>> for &'a TermType {
    fn into(self) -> Option<&'a ZipIteratorTerm> {
        match self {
            Self::ZipIterator(term) => Some(term),
            _ => None,
        }
    }
}

impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, TermType, A> {}
impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, TermType, A> {
    fn eq(&self, other: &Self) -> bool {
        match (self.as_deref(), other.as_deref()) {
            (TermType::Application(left), TermType::Application(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Boolean(left), TermType::Boolean(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Builtin(left), TermType::Builtin(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Cell(left), TermType::Cell(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Compiled(left), TermType::Compiled(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Condition(left), TermType::Condition(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Constructor(left), TermType::Constructor(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Date(left), TermType::Date(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Effect(left), TermType::Effect(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Float(left), TermType::Float(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Hashmap(left), TermType::Hashmap(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Hashset(left), TermType::Hashset(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Int(left), TermType::Int(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Lambda(left), TermType::Lambda(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Let(left), TermType::Let(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::List(left), TermType::List(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Nil(left), TermType::Nil(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Partial(left), TermType::Partial(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Pointer(left), TermType::Pointer(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Record(left), TermType::Record(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Signal(left), TermType::Signal(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::String(left), TermType::String(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Symbol(left), TermType::Symbol(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Tree(left), TermType::Tree(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::Variable(left), TermType::Variable(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::EmptyIterator(left), TermType::EmptyIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::EvaluateIterator(left), TermType::EvaluateIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::FilterIterator(left), TermType::FilterIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::FlattenIterator(left), TermType::FlattenIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::HashmapKeysIterator(left), TermType::HashmapKeysIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::HashmapValuesIterator(left), TermType::HashmapValuesIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::IntegersIterator(left), TermType::IntegersIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::IntersperseIterator(left), TermType::IntersperseIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::MapIterator(left), TermType::MapIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::OnceIterator(left), TermType::OnceIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::RangeIterator(left), TermType::RangeIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::RepeatIterator(left), TermType::RepeatIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::SkipIterator(left), TermType::SkipIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::TakeIterator(left), TermType::TakeIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
            (TermType::ZipIterator(left), TermType::ZipIterator(right)) => {
                ArenaRef::new(self.arena, left) == ArenaRef::new(other.arena, right)
            }
        }
    }
}
impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, TermType, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_deref() {
            TermType::Application(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::Boolean(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Builtin(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Cell(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Compiled(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Condition(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::Constructor(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::Date(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Effect(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Float(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Hashmap(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Hashset(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Int(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Lambda(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Let(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::List(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Nil(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Partial(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Pointer(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Record(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Signal(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::String(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Symbol(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Tree(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Variable(term) => std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::EmptyIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::EvaluateIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::FilterIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::FlattenIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::HashmapKeysIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::HashmapValuesIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::IntegersIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::IntersperseIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::MapIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::OnceIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::RangeIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::RepeatIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::SkipIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::TakeIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::ZipIterator(term) => {
                std::fmt::Display::fmt(&ArenaRef::new(self.arena, term), f)
            }
        }
    }
}
impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, TermType, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_deref() {
            TermType::Application(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::Boolean(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Builtin(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Cell(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Compiled(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Condition(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Constructor(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::Date(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Effect(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Float(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Hashmap(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Hashset(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Int(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Lambda(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Let(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::List(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Nil(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Partial(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Pointer(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Record(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Signal(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::String(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Symbol(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Tree(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::Variable(term) => std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f),
            TermType::EmptyIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::EvaluateIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::FilterIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::FlattenIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::HashmapKeysIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::HashmapValuesIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::IntegersIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::IntersperseIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::MapIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::OnceIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::RangeIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::RepeatIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::SkipIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::TakeIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
            TermType::ZipIterator(term) => {
                std::fmt::Debug::fmt(&ArenaRef::new(self.arena, term), f)
            }
        }
    }
}

#[cfg(test)]
impl TermType {
    fn as_bytes(&self) -> &[u32] {
        let num_words = crate::pad_to_4_byte_offset(self.size() as usize) / 4;
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u32, num_words) }
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
