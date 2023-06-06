// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use strum_macros::EnumDiscriminants;

use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    TermPointer,
};

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
            ConditionTerm::Custom(condition) => condition.size(),
            ConditionTerm::Pending(condition) => condition.size(),
            ConditionTerm::Error(condition) => condition.size(),
            ConditionTerm::TypeError(condition) => condition.size(),
            ConditionTerm::InvalidFunctionTarget(condition) => condition.size(),
            ConditionTerm::InvalidFunctionArgs(condition) => condition.size(),
            ConditionTerm::InvalidPointer(condition) => condition.size(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for ConditionTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        match self {
            ConditionTerm::Custom(condition) => condition.hash(hasher.write_byte(0), allocator),
            ConditionTerm::Pending(condition) => condition.hash(hasher.write_byte(0), allocator),
            ConditionTerm::Error(condition) => condition.hash(hasher.write_byte(0), allocator),
            ConditionTerm::TypeError(condition) => condition.hash(hasher.write_byte(0), allocator),
            ConditionTerm::InvalidFunctionTarget(condition) => {
                condition.hash(hasher.write_byte(0), allocator)
            }
            ConditionTerm::InvalidFunctionArgs(condition) => {
                condition.hash(hasher.write_byte(0), allocator)
            }
            ConditionTerm::InvalidPointer(condition) => {
                condition.hash(hasher.write_byte(0), allocator)
            }
        }
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
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.effect_type, allocator)
            .hash(&self.payload, allocator)
            .hash(&self.token, allocator)
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
    fn hash(&self, hasher: TermHasher, _allocator: &impl TermAllocator) -> TermHasher {
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
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.payload, allocator)
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
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.expected, allocator)
            .hash(&self.payload, allocator)
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
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.target, allocator)
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
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.target, allocator)
            .hash(&self.args, allocator)
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
    fn hash(&self, hasher: TermHasher, _allocator: &impl TermAllocator) -> TermHasher {
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
