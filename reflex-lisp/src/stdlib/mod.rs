// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{
    Applicable, Arity, EvaluationCache, Expression, ExpressionFactory, HeapAllocator, Uid, Uuid,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub mod car;
pub mod cdr;
pub mod cons;

pub use car::*;
pub use cdr::*;
pub use cons::*;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize, EnumIter)]
pub enum Stdlib {
    Car,
    Cdr,
    Cons,
}
impl Stdlib {
    pub fn entries() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}
impl Uid for Stdlib {
    fn uid(&self) -> Uuid {
        match self {
            Self::Car => Uid::uid(&Car {}),
            Self::Cdr => Uid::uid(&Cdr {}),
            Self::Cons => Uid::uid(&Cons {}),
        }
    }
}
impl TryFrom<Uuid> for Stdlib {
    type Error = ();
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        match value {
            Car::UUID => Ok(Self::Car),
            Cdr::UUID => Ok(Self::Cdr),
            Cons::UUID => Ok(Self::Cons),
            _ => Err(()),
        }
    }
}
impl Stdlib {
    pub fn arity(&self) -> Arity {
        match self {
            Self::Car => Car::arity(),
            Self::Cdr => Cdr::arity(),
            Self::Cons => Cons::arity(),
        }
    }
    pub fn should_parallelize<T: Expression>(&self, args: &[T]) -> bool {
        match self {
            Self::Car => Applicable::<T>::should_parallelize(&Car {}, args),
            Self::Cdr => Applicable::<T>::should_parallelize(&Cdr {}, args),
            Self::Cons => Applicable::<T>::should_parallelize(&Cons {}, args),
        }
    }
    pub fn apply<T: Expression>(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        cache: &mut impl EvaluationCache<T>,
    ) -> Result<T, String> {
        match self {
            Self::Car => Applicable::<T>::apply(&Car {}, args, factory, allocator, cache),
            Self::Cdr => Applicable::<T>::apply(&Cdr {}, args, factory, allocator, cache),
            Self::Cons => Applicable::<T>::apply(&Cons {}, args, factory, allocator, cache),
        }
    }
}
impl std::fmt::Display for Stdlib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lisp:{:?}>", self)
    }
}

impl From<Car> for Stdlib {
    fn from(_value: Car) -> Self {
        Self::Car
    }
}
impl From<Cdr> for Stdlib {
    fn from(_value: Cdr) -> Self {
        Self::Cdr
    }
}
impl From<Cons> for Stdlib {
    fn from(_value: Cons) -> Self {
        Self::Cons
    }
}
