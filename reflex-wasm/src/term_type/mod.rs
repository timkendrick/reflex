// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
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
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
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
            TermType::Application(term) => term.size(),
            TermType::Boolean(term) => term.size(),
            TermType::Builtin(term) => term.size(),
            TermType::Cell(term) => term.size(),
            TermType::Compiled(term) => term.size(),
            TermType::Condition(term) => term.size(),
            TermType::Constructor(term) => term.size(),
            TermType::Date(term) => term.size(),
            TermType::Effect(term) => term.size(),
            TermType::Float(term) => term.size(),
            TermType::Hashmap(term) => term.size(),
            TermType::Hashset(term) => term.size(),
            TermType::Int(term) => term.size(),
            TermType::Lambda(term) => term.size(),
            TermType::Let(term) => term.size(),
            TermType::List(term) => term.size(),
            TermType::Nil(term) => term.size(),
            TermType::Partial(term) => term.size(),
            TermType::Pointer(term) => term.size(),
            TermType::Record(term) => term.size(),
            TermType::Signal(term) => term.size(),
            TermType::String(term) => term.size(),
            TermType::Symbol(term) => term.size(),
            TermType::Tree(term) => term.size(),
            TermType::Variable(term) => term.size(),
            TermType::EmptyIterator(term) => term.size(),
            TermType::EvaluateIterator(term) => term.size(),
            TermType::FilterIterator(term) => term.size(),
            TermType::FlattenIterator(term) => term.size(),
            TermType::HashmapKeysIterator(term) => term.size(),
            TermType::HashmapValuesIterator(term) => term.size(),
            TermType::IntegersIterator(term) => term.size(),
            TermType::IntersperseIterator(term) => term.size(),
            TermType::MapIterator(term) => term.size(),
            TermType::OnceIterator(term) => term.size(),
            TermType::RangeIterator(term) => term.size(),
            TermType::RepeatIterator(term) => term.size(),
            TermType::SkipIterator(term) => term.size(),
            TermType::TakeIterator(term) => term.size(),
            TermType::ZipIterator(term) => term.size(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for TermType {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        match self {
            Self::Application(term) => hasher
                .write_byte(TermTypeDiscriminants::Application as u8)
                .hash(term, allocator),
            Self::Boolean(term) => hasher
                .write_byte(TermTypeDiscriminants::Boolean as u8)
                .hash(term, allocator),
            Self::Builtin(term) => hasher
                .write_byte(TermTypeDiscriminants::Builtin as u8)
                .hash(term, allocator),
            Self::Cell(term) => hasher
                .write_byte(TermTypeDiscriminants::Cell as u8)
                .hash(term, allocator),
            Self::Compiled(term) => hasher
                .write_byte(TermTypeDiscriminants::Compiled as u8)
                .hash(term, allocator),
            Self::Condition(term) => hasher
                .write_byte(TermTypeDiscriminants::Condition as u8)
                .hash(term, allocator),
            Self::Constructor(term) => hasher
                .write_byte(TermTypeDiscriminants::Constructor as u8)
                .hash(term, allocator),
            Self::Date(term) => hasher
                .write_byte(TermTypeDiscriminants::Date as u8)
                .hash(term, allocator),
            Self::Effect(term) => hasher
                .write_byte(TermTypeDiscriminants::Effect as u8)
                .hash(term, allocator),
            Self::Float(term) => hasher
                .write_byte(TermTypeDiscriminants::Float as u8)
                .hash(term, allocator),
            Self::Hashmap(term) => hasher
                .write_byte(TermTypeDiscriminants::Hashmap as u8)
                .hash(term, allocator),
            Self::Hashset(term) => hasher
                .write_byte(TermTypeDiscriminants::Hashset as u8)
                .hash(term, allocator),
            Self::Int(term) => hasher
                .write_byte(TermTypeDiscriminants::Int as u8)
                .hash(term, allocator),
            Self::Lambda(term) => hasher
                .write_byte(TermTypeDiscriminants::Lambda as u8)
                .hash(term, allocator),
            Self::Let(term) => hasher
                .write_byte(TermTypeDiscriminants::Let as u8)
                .hash(term, allocator),
            Self::List(term) => hasher
                .write_byte(TermTypeDiscriminants::List as u8)
                .hash(term, allocator),
            Self::Nil(term) => hasher
                .write_byte(TermTypeDiscriminants::Nil as u8)
                .hash(term, allocator),
            Self::Partial(term) => hasher
                .write_byte(TermTypeDiscriminants::Partial as u8)
                .hash(term, allocator),
            Self::Pointer(term) => hasher
                .write_byte(TermTypeDiscriminants::Pointer as u8)
                .hash(term, allocator),
            Self::Record(term) => hasher
                .write_byte(TermTypeDiscriminants::Record as u8)
                .hash(term, allocator),
            Self::Signal(term) => hasher
                .write_byte(TermTypeDiscriminants::Signal as u8)
                .hash(term, allocator),
            Self::String(term) => hasher
                .write_byte(TermTypeDiscriminants::String as u8)
                .hash(term, allocator),
            Self::Symbol(term) => hasher
                .write_byte(TermTypeDiscriminants::Symbol as u8)
                .hash(term, allocator),
            Self::Tree(term) => hasher
                .write_byte(TermTypeDiscriminants::Tree as u8)
                .hash(term, allocator),
            Self::Variable(term) => hasher
                .write_byte(TermTypeDiscriminants::Variable as u8)
                .hash(term, allocator),
            Self::EmptyIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::EmptyIterator as u8)
                .hash(term, allocator),
            Self::EvaluateIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::EvaluateIterator as u8)
                .hash(term, allocator),
            Self::FilterIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::FilterIterator as u8)
                .hash(term, allocator),
            Self::FlattenIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::FlattenIterator as u8)
                .hash(term, allocator),
            Self::HashmapKeysIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::HashmapKeysIterator as u8)
                .hash(term, allocator),
            Self::HashmapValuesIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::HashmapValuesIterator as u8)
                .hash(term, allocator),
            Self::IntegersIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::IntegersIterator as u8)
                .hash(term, allocator),
            Self::IntersperseIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::IntersperseIterator as u8)
                .hash(term, allocator),
            Self::MapIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::MapIterator as u8)
                .hash(term, allocator),
            Self::OnceIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::OnceIterator as u8)
                .hash(term, allocator),
            Self::RangeIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::RangeIterator as u8)
                .hash(term, allocator),
            Self::RepeatIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::RepeatIterator as u8)
                .hash(term, allocator),
            Self::SkipIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::SkipIterator as u8)
                .hash(term, allocator),
            Self::TakeIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::TakeIterator as u8)
                .hash(term, allocator),
            Self::ZipIterator(term) => hasher
                .write_byte(TermTypeDiscriminants::ZipIterator as u8)
                .hash(term, allocator),
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
