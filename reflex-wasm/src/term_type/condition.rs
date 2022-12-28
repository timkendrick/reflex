// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ConditionType, DependencyList, Expression, GraphNode, SerializeJson, SignalType, StackOffset,
    StateToken,
};
use serde_json::Value as JsonValue;
use strum_macros::EnumDiscriminants;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermTypeDiscriminants, TypedTerm},
    ArenaRef, Term, TermPointer,
};

use super::{ListTerm, StringTerm, WasmExpression};

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
    fn size_of(&self) -> usize {
        let discriminant_size = std::mem::size_of::<u32>();
        let value_size = match self {
            Self::Custom(condition) => condition.size_of(),
            Self::Pending(condition) => condition.size_of(),
            Self::Error(condition) => condition.size_of(),
            Self::TypeError(condition) => condition.size_of(),
            Self::InvalidFunctionTarget(condition) => condition.size_of(),
            Self::InvalidFunctionArgs(condition) => condition.size_of(),
            Self::InvalidPointer(condition) => condition.size_of(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for ConditionTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let hasher = hasher.write_u8(ConditionTermDiscriminants::from(self) as u8);
        match self {
            Self::Custom(condition) => condition.hash(hasher, arena),
            Self::Pending(condition) => condition.hash(hasher, arena),
            Self::Error(condition) => condition.hash(hasher, arena),
            Self::TypeError(condition) => condition.hash(hasher, arena),
            Self::InvalidFunctionTarget(condition) => condition.hash(hasher, arena),
            Self::InvalidFunctionArgs(condition) => condition.hash(hasher, arena),
            Self::InvalidPointer(condition) => condition.hash(hasher, arena),
        }
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<ConditionTerm, A> {
    pub fn signal_type(&self) -> SignalType {
        match self.as_value() {
            ConditionTerm::Custom(condition) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(condition),
            )
            .signal_type(),
            ConditionTerm::Pending(_) => SignalType::Pending,
            ConditionTerm::Error(_) => SignalType::Error,
            ConditionTerm::TypeError(_) => SignalType::Error,
            ConditionTerm::InvalidFunctionTarget(_) => SignalType::Error,
            ConditionTerm::InvalidFunctionArgs(_) => SignalType::Error,
            ConditionTerm::InvalidPointer(_) => SignalType::Error,
        }
    }
    pub fn payload(&self) -> Option<ArenaRef<Term, A>> {
        match self.as_value() {
            ConditionTerm::Custom(inner) => Some(
                ArenaRef::<CustomCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .payload(),
            ),
            ConditionTerm::Error(inner) => Some(
                ArenaRef::<ErrorCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .payload(),
            ),
            _ => None,
        }
    }
    pub fn token(&self) -> Option<ArenaRef<Term, A>> {
        match self.as_value() {
            ConditionTerm::Custom(inner) => Some(
                ArenaRef::<CustomCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .token(),
            ),
            _ => None,
        }
    }
}

impl<A: ArenaAllocator + Clone> ConditionType<WasmExpression<A>>
    for ArenaRef<TypedTerm<ConditionTerm>, A>
{
    fn id(&self) -> StateToken {
        self.as_value().id()
    }
    fn signal_type(&self) -> SignalType {
        self.as_inner().signal_type()
    }
    fn payload<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a> {
        // FIXME: Improve condition API
        self.as_inner()
            .payload()
            .unwrap_or_else(|| self.as_term().clone())
    }
    fn token<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a> {
        // FIXME: Improve condition API
        self.as_inner()
            .token()
            .unwrap_or_else(|| self.as_term().clone())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<ConditionTerm, A> {
    fn size(&self) -> usize {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .size(),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .size(),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .size()
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .size(),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .size()
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .size()
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .size(),
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .capture_depth(),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .capture_depth(),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .capture_depth()
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .capture_depth(),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .capture_depth()
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .capture_depth()
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .capture_depth(),
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .free_variables(),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .free_variables(),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .free_variables()
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .free_variables(),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .free_variables()
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .free_variables()
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .free_variables(),
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .count_variable_usages(offset),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .count_variable_usages(offset),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .count_variable_usages(offset)
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .count_variable_usages(offset),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .count_variable_usages(offset)
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .count_variable_usages(offset)
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .count_variable_usages(offset),
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .dynamic_dependencies(deep),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .dynamic_dependencies(deep),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .dynamic_dependencies(deep)
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .dynamic_dependencies(deep),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .dynamic_dependencies(deep)
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .dynamic_dependencies(deep)
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .dynamic_dependencies(deep),
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .has_dynamic_dependencies(deep),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .has_dynamic_dependencies(deep),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .has_dynamic_dependencies(deep)
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .has_dynamic_dependencies(deep),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .has_dynamic_dependencies(deep)
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .has_dynamic_dependencies(deep)
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .has_dynamic_dependencies(deep),
        }
    }
    fn is_static(&self) -> bool {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_static(),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_static(),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .is_static()
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_static(),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .is_static()
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .is_static()
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_static(),
        }
    }
    fn is_atomic(&self) -> bool {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_atomic(),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_atomic(),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .is_atomic()
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_atomic(),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .is_atomic()
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .is_atomic()
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_atomic(),
        }
    }
    fn is_complex(&self) -> bool {
        match self.as_value() {
            ConditionTerm::Custom(inner) => ArenaRef::<CustomCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_complex(),
            ConditionTerm::Pending(inner) => ArenaRef::<PendingCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_complex(),
            ConditionTerm::Error(inner) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    .is_complex()
            }
            ConditionTerm::TypeError(inner) => ArenaRef::<TypeErrorCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_complex(),
            ConditionTerm::InvalidFunctionTarget(inner) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .is_complex()
            }
            ConditionTerm::InvalidFunctionArgs(inner) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                )
                .is_complex()
            }
            ConditionTerm::InvalidPointer(inner) => ArenaRef::<InvalidPointerCondition, _>::new(
                self.arena.clone(),
                self.arena.get_offset(inner),
            )
            .is_complex(),
        }
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<ConditionTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<ConditionTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        match (self.as_value(), other.as_value()) {
            (ConditionTerm::Custom(inner), ConditionTerm::Custom(other)) => {
                ArenaRef::<CustomCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ) == ArenaRef::<CustomCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(other),
                )
            }
            (ConditionTerm::Pending(inner), ConditionTerm::Pending(other)) => {
                ArenaRef::<PendingCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ) == ArenaRef::<PendingCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(other),
                )
            }
            (ConditionTerm::Error(inner), ConditionTerm::Error(other)) => {
                ArenaRef::<ErrorCondition, _>::new(self.arena.clone(), self.arena.get_offset(inner))
                    == ArenaRef::<ErrorCondition, _>::new(
                        self.arena.clone(),
                        self.arena.get_offset(other),
                    )
            }
            (ConditionTerm::TypeError(inner), ConditionTerm::TypeError(other)) => {
                ArenaRef::<TypeErrorCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ) == ArenaRef::<TypeErrorCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(other),
                )
            }
            (
                ConditionTerm::InvalidFunctionTarget(inner),
                ConditionTerm::InvalidFunctionTarget(other),
            ) => {
                ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ) == ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(other),
                )
            }
            (
                ConditionTerm::InvalidFunctionArgs(inner),
                ConditionTerm::InvalidFunctionArgs(other),
            ) => {
                ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ) == ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(other),
                )
            }
            (ConditionTerm::InvalidPointer(inner), ConditionTerm::InvalidPointer(other)) => {
                ArenaRef::<InvalidPointerCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ) == ArenaRef::<InvalidPointerCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(other),
                )
            }
            _ => false,
        }
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<ConditionTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<ConditionTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_value() {
            ConditionTerm::Custom(inner) => std::fmt::Display::fmt(
                &ArenaRef::<CustomCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
            ConditionTerm::Pending(inner) => std::fmt::Display::fmt(
                &ArenaRef::<PendingCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
            ConditionTerm::Error(inner) => std::fmt::Display::fmt(
                &ArenaRef::<ErrorCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
            ConditionTerm::TypeError(inner) => std::fmt::Display::fmt(
                &ArenaRef::<TypeErrorCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
            ConditionTerm::InvalidFunctionTarget(inner) => std::fmt::Display::fmt(
                &ArenaRef::<InvalidFunctionTargetCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
            ConditionTerm::InvalidFunctionArgs(inner) => std::fmt::Display::fmt(
                &ArenaRef::<InvalidFunctionArgsCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
            ConditionTerm::InvalidPointer(inner) => std::fmt::Display::fmt(
                &ArenaRef::<InvalidPointerCondition, _>::new(
                    self.arena.clone(),
                    self.arena.get_offset(inner),
                ),
                f,
            ),
        }
    }
}
impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<ConditionTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
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
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CustomCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(arena.get::<Term>(self.effect_type), arena)
            .hash(arena.get::<Term>(self.payload), arena)
            .hash(arena.get::<Term>(self.token), arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<CustomCondition, A> {
    pub fn signal_type(&self) -> SignalType {
        let effect_type = self.effect_type();
        // FIXME: Allow arbitrary expressions as condition types
        let custom_effect_type = match effect_type.as_value().type_id() {
            TermTypeDiscriminants::String => {
                let string_term = ArenaRef::<TypedTerm<StringTerm>, _>::new(
                    self.arena.clone(),
                    self.as_value().effect_type,
                );
                String::from(string_term.as_inner().as_value().as_str())
            }
            _ => format!("{}", effect_type),
        };
        SignalType::Custom(custom_effect_type)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<CustomCondition, A> {
    pub fn effect_type(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().effect_type)
    }
    pub fn payload(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().payload)
    }
    pub fn token(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().token)
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<CustomCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        self.payload()
            .capture_depth()
            .max(self.token().capture_depth())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.payload()
            .free_variables()
            .into_iter()
            .chain(self.token().free_variables())
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.payload().count_variable_usages(offset) + self.token().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.payload()
                .dynamic_dependencies(deep)
                .union(self.token().dynamic_dependencies(deep))
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.payload().has_dynamic_dependencies(deep)
                || self.token().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<CustomCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.effect_type() == other.effect_type()
            && self.payload() == other.payload()
            && self.token() == other.token()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<CustomCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<CustomCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<CustomCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Custom({}):{}:{}",
            self.effect_type(),
            self.payload(),
            self.token()
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PendingCondition;
impl TermSize for PendingCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PendingCondition {
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<PendingCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        Default::default()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<PendingCondition, A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<PendingCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<PendingCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<PendingCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pending")
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ErrorCondition {
    pub payload: TermPointer,
}
impl TermSize for ErrorCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ErrorCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.payload, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<ErrorCondition, A> {
    pub fn payload(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().payload)
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<ErrorCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        self.payload().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.payload().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.payload().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.payload().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.payload().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<ErrorCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.payload() == other.payload()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<ErrorCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<ErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<ErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error:{}", self.payload())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TypeErrorCondition {
    pub expected: u32,
    pub payload: TermPointer,
}
impl TermSize for TypeErrorCondition {
    fn size_of(&self) -> usize {
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

impl<A: ArenaAllocator + Clone> ArenaRef<TypeErrorCondition, A> {
    pub fn expected(&self) -> Option<TermTypeDiscriminants> {
        TermTypeDiscriminants::try_from(self.as_value().expected).ok()
    }
    pub fn payload(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().payload)
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<TypeErrorCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        self.payload().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.payload().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.payload().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.payload().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.payload().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<TypeErrorCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.as_value().expected == other.as_value().expected && self.payload() == other.payload()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<TypeErrorCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<TypeErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<TypeErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(expected) = self.expected() {
            write!(f, "TypeError:{:?}:{}", expected, self.payload())
        } else {
            write!(f, "TypeError:<unknown>:{}", self.payload())
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InvalidFunctionTargetCondition {
    pub target: TermPointer,
}
impl TermSize for InvalidFunctionTargetCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidFunctionTargetCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<InvalidFunctionTargetCondition, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().target)
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        self.target().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.target().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.target().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.target().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.target().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<InvalidFunctionTargetCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidFunctionTarget:{}", self.target())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InvalidFunctionArgsCondition {
    pub target: TermPointer,
    pub args: TermPointer,
}
impl TermSize for InvalidFunctionArgsCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidFunctionArgsCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<InvalidFunctionArgsCondition, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().target)
    }
    pub fn args(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(self.arena.clone(), self.as_value().args)
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        self.target()
            .capture_depth()
            .max(self.args().capture_depth())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.target()
            .free_variables()
            .into_iter()
            .chain(self.args().free_variables())
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.target().count_variable_usages(offset) + self.args().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.target()
                .dynamic_dependencies(deep)
                .union(self.args().dynamic_dependencies(deep))
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.target().has_dynamic_dependencies(deep)
                || self.args().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<InvalidFunctionArgsCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidFunctionArgs:{}:{}", self.target(), self.args())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InvalidPointerCondition;
impl TermSize for InvalidPointerCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidPointerCondition {
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<InvalidPointerCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        Default::default()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<InvalidPointerCondition, A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<InvalidPointerCondition, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<InvalidPointerCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<InvalidPointerCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidPointer")
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
