// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{ConditionType, Expression, RefType, SignalType, StateToken};
use strum_macros::EnumDiscriminants;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, TermPointer, TypedTerm,
};

use super::ListTerm;

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[repr(C)]
pub enum ConditionTerm {
    Custom(CustomCondition),
    Pending(PendingCondition),
    Error(ErrorCondition),
    TypeError(TypeErrorCondition),
    InvalidFunctionTarget(InvalidFunctionTargetCondition),
    InvalidFunctionArgs(InvalidFunctionArgsCondition),
    InvalidPointer(InvalidPointerCondition),
}
impl TermSize for ConditionTerm {
    fn size(&self) -> usize {
        let discriminant_size = std::mem::size_of::<u32>();
        let value_size = match self {
            Self::Custom(condition) => condition.size(),
            Self::Pending(condition) => condition.size(),
            Self::Error(condition) => condition.size(),
            Self::TypeError(condition) => condition.size(),
            Self::InvalidFunctionTarget(condition) => condition.size(),
            Self::InvalidFunctionArgs(condition) => condition.size(),
            Self::InvalidPointer(condition) => condition.size(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for ConditionTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        match self {
            Self::Custom(condition) => condition.hash(hasher.write_u8(0), arena),
            Self::Pending(condition) => condition.hash(hasher.write_u8(0), arena),
            Self::Error(condition) => condition.hash(hasher.write_u8(0), arena),
            Self::TypeError(condition) => condition.hash(hasher.write_u8(0), arena),
            Self::InvalidFunctionTarget(condition) => condition.hash(hasher.write_u8(0), arena),
            Self::InvalidFunctionArgs(condition) => condition.hash(hasher.write_u8(0), arena),
            Self::InvalidPointer(condition) => condition.hash(hasher.write_u8(0), arena),
        }
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ConditionType<T> for ArenaRef<'heap, ConditionTerm, A>
where
    for<'a> T::Ref<'a, T::ExpressionList<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn id(&self) -> StateToken {
        self.hash(TermHasher::default(), self.arena)
    }
    fn signal_type(&self) -> SignalType {
        match self.as_deref() {
            ConditionTerm::Custom(condition) => {
                SignalType::Custom(ArenaRef::new(self.arena, condition).signal_type())
            }
            ConditionTerm::Pending(_) => SignalType::Pending,
            ConditionTerm::Error(_) => SignalType::Error,
            ConditionTerm::TypeError(_) => SignalType::Error,
            ConditionTerm::InvalidFunctionTarget(_) => SignalType::Error,
            ConditionTerm::InvalidFunctionArgs(_) => SignalType::Error,
            ConditionTerm::InvalidPointer(_) => SignalType::Error,
        }
    }
    fn args<'a>(&'a self) -> T::Ref<'a, T::ExpressionList<T>>
    where
        T::ExpressionList<T>: 'a,
        T: 'a,
    {
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CustomCondition {
    pub effect_type: TermPointer,
    pub payload: TermPointer,
    pub token: TermPointer,
}
impl TermSize for CustomCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CustomCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(&self.effect_type, arena)
            .hash(&self.payload, arena)
            .hash(&self.token, arena)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PendingCondition;
impl TermSize for PendingCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PendingCondition {
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ErrorCondition {
    pub payload: TermPointer,
}
impl TermSize for ErrorCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ErrorCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.payload, arena)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TypeErrorCondition {
    pub expected: u32,
    pub payload: TermPointer,
}
impl TermSize for TypeErrorCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TypeErrorCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(&self.expected, arena)
            .hash(&self.payload, arena)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InvalidFunctionTargetCondition {
    pub target: TermPointer,
}
impl TermSize for InvalidFunctionTargetCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidFunctionTargetCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InvalidFunctionArgsCondition {
    pub target: TermPointer,
    pub args: TermPointer,
}
impl TermSize for InvalidFunctionArgsCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidFunctionArgsCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InvalidPointerCondition;
impl TermSize for InvalidPointerCondition {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidPointerCondition {
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn condition() {
        assert_eq!(std::mem::size_of::<ConditionTerm>(), 16);
        assert_eq!(ConditionTermDiscriminants::Custom as u32, 0);
        assert_eq!(ConditionTermDiscriminants::Pending as u32, 1);
        assert_eq!(ConditionTermDiscriminants::Error as u32, 2);
        assert_eq!(ConditionTermDiscriminants::TypeError as u32, 3);
        assert_eq!(ConditionTermDiscriminants::InvalidFunctionTarget as u32, 4);
        assert_eq!(ConditionTermDiscriminants::InvalidFunctionArgs as u32, 5);
        assert_eq!(ConditionTermDiscriminants::InvalidPointer as u32, 6);
    }

    #[test]
    fn condition_custom() {
        assert_eq!(
            TermType::Condition(ConditionTerm::Custom(CustomCondition {
                effect_type: TermPointer(12345),
                payload: TermPointer(45678),
                token: TermPointer(67890),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::Custom as u32,
                12345,
                45678,
                67890,
            ],
        );
    }

    #[test]
    fn condition_pending() {
        assert_eq!(
            TermType::Condition(ConditionTerm::Pending(PendingCondition)).as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::Pending as u32,
            ],
        );
    }

    #[test]
    fn condition_error() {
        assert_eq!(
            TermType::Condition(ConditionTerm::Error(ErrorCondition {
                payload: TermPointer(12345),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::Error as u32,
                12345,
            ],
        );
    }

    #[test]
    fn condition_type_error() {
        assert_eq!(
            TermType::Condition(ConditionTerm::TypeError(TypeErrorCondition {
                expected: 12345,
                payload: TermPointer(67890),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::TypeError as u32,
                12345,
                67890,
            ],
        );
    }

    #[test]
    fn condition_invalid_function_target() {
        assert_eq!(
            TermType::Condition(ConditionTerm::InvalidFunctionTarget(
                InvalidFunctionTargetCondition {
                    target: TermPointer(12345),
                },
            ))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::InvalidFunctionTarget as u32,
                12345,
            ],
        );
    }

    #[test]
    fn condition_invalid_function_args() {
        assert_eq!(
            TermType::Condition(ConditionTerm::InvalidFunctionArgs(
                InvalidFunctionArgsCondition {
                    target: TermPointer(12345),
                    args: TermPointer(67890),
                },
            ))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::InvalidFunctionArgs as u32,
                12345,
                67890,
            ],
        );
    }

    #[test]
    fn condition_invalid_pointer() {
        assert_eq!(
            TermType::Condition(ConditionTerm::InvalidPointer(InvalidPointerCondition)).as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::InvalidPointer as u32,
            ],
        );
    }
}
