// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, marker::PhantomData};

use reflex::core::{
    ConditionType, DependencyList, Eagerness, Expression, GraphNode, Internable, SerializeJson,
    SignalType, StackOffset, StateToken, StringValue,
};
use serde_json::Value as JsonValue;
use strum_macros::EnumDiscriminants;

use super::{ListTerm, StringTerm, WasmExpression};
use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermTypeDiscriminants, TypedTerm},
    ArenaPointer, ArenaRef, PointerIter, Term,
};
use reflex_macros::PointerIter;

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

#[derive(Clone, Debug)]
pub enum ConditionTermPointerIter {
    Custom(CustomConditionPointerIter),
    Pending(PendingConditionPointerIter),
    Error(ErrorConditionPointerIter),
    TypeError(TypeErrorConditionPointerIter),
    InvalidFunctionTarget(InvalidFunctionTargetConditionPointerIter),
    InvalidFunctionArgs(InvalidFunctionArgsConditionPointerIter),
    InvalidPointer(InvalidPointerConditionPointerIter),
}

impl Iterator for ConditionTermPointerIter {
    type Item = ArenaPointer;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Custom(inner) => inner.next(),
            Self::Pending(inner) => inner.next(),
            Self::Error(inner) => inner.next(),
            Self::TypeError(inner) => inner.next(),
            Self::InvalidFunctionTarget(inner) => inner.next(),
            Self::InvalidFunctionArgs(inner) => inner.next(),
            Self::InvalidPointer(inner) => inner.next(),
        }
    }
}

impl ConditionTerm {
    fn condition_type(&self) -> ConditionTermDiscriminants {
        ConditionTermDiscriminants::from(self)
    }
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
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let hasher = hasher.write_u8(self.condition_type() as u8);
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

impl<A: Arena + Clone> ArenaRef<ConditionTerm, A> {
    pub fn signal_type(&self) -> SignalType {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .signal_type(),
            ConditionTermDiscriminants::Pending => SignalType::Pending,
            ConditionTermDiscriminants::Error => SignalType::Error,
            ConditionTermDiscriminants::TypeError => SignalType::Error,
            ConditionTermDiscriminants::InvalidFunctionTarget => SignalType::Error,
            ConditionTermDiscriminants::InvalidFunctionArgs => SignalType::Error,
            ConditionTermDiscriminants::InvalidPointer => SignalType::Error,
        }
    }
    pub fn payload(&self) -> Option<ArenaRef<Term, A>> {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => Some(
                self.as_typed_condition::<CustomCondition>()
                    .as_inner()
                    .payload(),
            ),
            ConditionTermDiscriminants::Error => Some(
                self.as_typed_condition::<ErrorCondition>()
                    .as_inner()
                    .payload(),
            ),
            _ => None,
        }
    }
    pub fn token(&self) -> Option<ArenaRef<Term, A>> {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => Some(
                self.as_typed_condition::<CustomCondition>()
                    .as_inner()
                    .token(),
            ),
            _ => None,
        }
    }
    pub(crate) fn condition_type(&self) -> ConditionTermDiscriminants {
        self.read_value(|term| term.condition_type())
    }
    pub(crate) fn as_typed_condition<V>(&self) -> &ArenaRef<TypedCondition<V>, A> {
        unsafe {
            std::mem::transmute::<&ArenaRef<ConditionTerm, A>, &ArenaRef<TypedCondition<V>, A>>(
                self,
            )
        }
    }
}

impl<A: Arena + Clone> PointerIter for ArenaRef<ConditionTerm, A> {
    type Iter<'a> = ConditionTermPointerIter
    where
        Self: 'a;
    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => ConditionTermPointerIter::Custom(
                self.as_typed_condition::<CustomCondition>()
                    .as_inner()
                    .iter(),
            ),
            ConditionTermDiscriminants::Error => ConditionTermPointerIter::Error(
                self.as_typed_condition::<ErrorCondition>()
                    .as_inner()
                    .iter(),
            ),
            ConditionTermDiscriminants::Pending => ConditionTermPointerIter::Pending(
                self.as_typed_condition::<PendingCondition>()
                    .as_inner()
                    .iter(),
            ),
            ConditionTermDiscriminants::TypeError => ConditionTermPointerIter::TypeError(
                self.as_typed_condition::<TypeErrorCondition>()
                    .as_inner()
                    .iter(),
            ),
            ConditionTermDiscriminants::InvalidFunctionTarget => {
                ConditionTermPointerIter::InvalidFunctionTarget(
                    self.as_typed_condition::<InvalidFunctionTargetCondition>()
                        .as_inner()
                        .iter(),
                )
            }
            ConditionTermDiscriminants::InvalidFunctionArgs => {
                ConditionTermPointerIter::InvalidFunctionArgs(
                    self.as_typed_condition::<InvalidFunctionArgsCondition>()
                        .as_inner()
                        .iter(),
                )
            }
            ConditionTermDiscriminants::InvalidPointer => ConditionTermPointerIter::InvalidPointer(
                self.as_typed_condition::<InvalidPointerCondition>()
                    .as_inner()
                    .iter(),
            ),
        }
    }
}

#[repr(transparent)]
pub(crate) struct TypedCondition<V> {
    condition: ConditionTerm,
    _type: PhantomData<V>,
}
impl<V> TypedCondition<V> {
    pub(crate) fn get_inner(&self) -> &V {
        unsafe {
            match &self.condition {
                ConditionTerm::Custom(inner) => std::mem::transmute::<&CustomCondition, &V>(inner),
                ConditionTerm::Pending(inner) => {
                    std::mem::transmute::<&PendingCondition, &V>(inner)
                }
                ConditionTerm::Error(inner) => std::mem::transmute::<&ErrorCondition, &V>(inner),
                ConditionTerm::TypeError(inner) => {
                    std::mem::transmute::<&TypeErrorCondition, &V>(inner)
                }
                ConditionTerm::InvalidFunctionTarget(inner) => {
                    std::mem::transmute::<&InvalidFunctionTargetCondition, &V>(inner)
                }
                ConditionTerm::InvalidFunctionArgs(inner) => {
                    std::mem::transmute::<&InvalidFunctionArgsCondition, &V>(inner)
                }
                ConditionTerm::InvalidPointer(inner) => {
                    std::mem::transmute::<&InvalidPointerCondition, &V>(inner)
                }
            }
        }
    }
}

impl<A: Arena + Clone, V> ArenaRef<TypedCondition<V>, A> {
    pub fn as_inner(&self) -> ArenaRef<V, A> {
        self.inner_ref(|condition| condition.get_inner())
    }
}

impl<A: Arena + Clone> ConditionType<WasmExpression<A>> for ArenaRef<TypedTerm<ConditionTerm>, A> {
    fn id(&self) -> StateToken {
        self.read_value(|term| term.id())
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

impl<A: Arena + Clone> GraphNode for ArenaRef<ConditionTerm, A> {
    fn size(&self) -> usize {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .size(),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .size(),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .size(),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .size(),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .size(),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .size(),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .size(),
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .capture_depth(),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .capture_depth(),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .capture_depth(),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .capture_depth(),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .capture_depth(),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .capture_depth(),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .capture_depth(),
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .free_variables(),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .free_variables(),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .free_variables(),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .free_variables(),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .free_variables(),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .free_variables(),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .free_variables(),
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .count_variable_usages(offset),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .count_variable_usages(offset),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .count_variable_usages(offset),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .count_variable_usages(offset),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .count_variable_usages(offset),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .count_variable_usages(offset),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .count_variable_usages(offset),
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .dynamic_dependencies(deep),
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .has_dynamic_dependencies(deep),
        }
    }
    fn is_static(&self) -> bool {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .is_static(),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .is_static(),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .is_static(),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .is_static(),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .is_static(),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .is_static(),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .is_static(),
        }
    }
    fn is_atomic(&self) -> bool {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .is_atomic(),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .is_atomic(),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .is_atomic(),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .is_atomic(),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .is_atomic(),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .is_atomic(),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .is_atomic(),
        }
    }
    fn is_complex(&self) -> bool {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .is_complex(),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .is_complex(),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .is_complex(),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .is_complex(),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .is_complex(),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .is_complex(),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .is_complex(),
        }
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<ConditionTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<ConditionTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.condition_type(),
            other.read_value(|term| term.condition_type()),
        ) {
            (ConditionTermDiscriminants::Custom, ConditionTermDiscriminants::Custom) => {
                self.as_typed_condition::<CustomCondition>().as_inner()
                    == other.as_typed_condition::<CustomCondition>().as_inner()
            }
            (ConditionTermDiscriminants::Pending, ConditionTermDiscriminants::Pending) => {
                self.as_typed_condition::<PendingCondition>().as_inner()
                    == other.as_typed_condition::<PendingCondition>().as_inner()
            }
            (ConditionTermDiscriminants::Error, ConditionTermDiscriminants::Error) => {
                self.as_typed_condition::<ErrorCondition>().as_inner()
                    == other.as_typed_condition::<ErrorCondition>().as_inner()
            }
            (ConditionTermDiscriminants::TypeError, ConditionTermDiscriminants::TypeError) => {
                self.as_typed_condition::<TypeErrorCondition>().as_inner()
                    == other.as_typed_condition::<TypeErrorCondition>().as_inner()
            }
            (
                ConditionTermDiscriminants::InvalidFunctionTarget,
                ConditionTermDiscriminants::InvalidFunctionTarget,
            ) => {
                self.as_typed_condition::<InvalidFunctionTargetCondition>()
                    .as_inner()
                    == other
                        .as_typed_condition::<InvalidFunctionTargetCondition>()
                        .as_inner()
            }
            (
                ConditionTermDiscriminants::InvalidFunctionArgs,
                ConditionTermDiscriminants::InvalidFunctionArgs,
            ) => {
                self.as_typed_condition::<InvalidFunctionArgsCondition>()
                    .as_inner()
                    == other
                        .as_typed_condition::<InvalidFunctionArgsCondition>()
                        .as_inner()
            }
            (
                ConditionTermDiscriminants::InvalidPointer,
                ConditionTermDiscriminants::InvalidPointer,
            ) => {
                self.as_typed_condition::<InvalidPointerCondition>()
                    .as_inner()
                    == other
                        .as_typed_condition::<InvalidPointerCondition>()
                        .as_inner()
            }
            _ => false,
        }
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<ConditionTerm, A> {}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<ConditionTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => {
                std::fmt::Display::fmt(&self.as_typed_condition::<CustomCondition>().as_inner(), f)
            }
            ConditionTermDiscriminants::Pending => {
                std::fmt::Display::fmt(&self.as_typed_condition::<PendingCondition>().as_inner(), f)
            }
            ConditionTermDiscriminants::Error => {
                std::fmt::Display::fmt(&self.as_typed_condition::<ErrorCondition>().as_inner(), f)
            }
            ConditionTermDiscriminants::TypeError => std::fmt::Display::fmt(
                &self.as_typed_condition::<TypeErrorCondition>().as_inner(),
                f,
            ),
            ConditionTermDiscriminants::InvalidFunctionTarget => std::fmt::Display::fmt(
                &self
                    .as_typed_condition::<InvalidFunctionTargetCondition>()
                    .as_inner(),
                f,
            ),
            ConditionTermDiscriminants::InvalidFunctionArgs => std::fmt::Display::fmt(
                &self
                    .as_typed_condition::<InvalidFunctionArgsCondition>()
                    .as_inner(),
                f,
            ),
            ConditionTermDiscriminants::InvalidPointer => std::fmt::Display::fmt(
                &self
                    .as_typed_condition::<InvalidPointerCondition>()
                    .as_inner(),
                f,
            ),
        }
    }
}
impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<ConditionTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct CustomCondition {
    pub effect_type: ArenaPointer,
    pub payload: ArenaPointer,
    pub token: ArenaPointer,
}
impl TermSize for CustomCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CustomCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher
            .hash(&self.effect_type, arena)
            .hash(&self.payload, arena)
            .hash(&self.token, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<CustomCondition, A> {
    pub fn signal_type(&self) -> SignalType {
        let effect_type = self.effect_type();
        // FIXME: Allow arbitrary expressions as condition types
        let custom_effect_type = match effect_type.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::String => String::from(
                effect_type
                    .as_typed_term::<StringTerm>()
                    .as_inner()
                    .as_str(),
            ),
            _ => format!("{}", effect_type),
        };
        SignalType::Custom(custom_effect_type)
    }
}

impl<A: Arena + Clone> ArenaRef<CustomCondition, A> {
    pub fn effect_type(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.effect_type))
    }
    pub fn payload(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.payload))
    }
    pub fn token(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.token))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<CustomCondition, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<CustomCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.effect_type() == other.effect_type()
            && self.payload() == other.payload()
            && self.token() == other.token()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<CustomCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<CustomCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<CustomCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Custom({}):{}:{}>",
            self.effect_type(),
            self.payload(),
            self.token()
        )
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct PendingCondition;
impl TermSize for PendingCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PendingCondition {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<PendingCondition, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<PendingCondition, A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<PendingCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<PendingCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<PendingCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Pending>")
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct ErrorCondition {
    pub payload: ArenaPointer,
}
impl TermSize for ErrorCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ErrorCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.payload, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<ErrorCondition, A> {
    pub fn payload(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.payload))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<ErrorCondition, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<ErrorCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.payload() == other.payload()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<ErrorCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<ErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<ErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Error:{}>", self.payload())
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct TypeErrorCondition {
    pub expected: u32,
    pub payload: ArenaPointer,
}
impl TermSize for TypeErrorCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TypeErrorCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher
            .hash(&self.expected, arena)
            .hash(&self.payload, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<TypeErrorCondition, A> {
    pub fn expected(&self) -> u32 {
        self.read_value(|term| term.expected)
    }
    pub fn payload(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.payload))
    }
    pub fn type_id(&self) -> Option<TermTypeDiscriminants> {
        TermTypeDiscriminants::try_from(self.expected()).ok()
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<TypeErrorCondition, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<TypeErrorCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.read_value(|term| term.expected) == other.read_value(|term| term.expected)
            && self.payload() == other.payload()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<TypeErrorCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<TypeErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<TypeErrorCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(expected) = self.type_id() {
            write!(f, "<TypeError:{:?}:{}>", expected, self.payload())
        } else {
            write!(f, "<TypeError:<unknown>:{}>", self.payload())
        }
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct InvalidFunctionTargetCondition {
    pub target: ArenaPointer,
}
impl TermSize for InvalidFunctionTargetCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidFunctionTargetCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.target, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<InvalidFunctionTargetCondition, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.target))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<InvalidFunctionTargetCondition, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<InvalidFunctionTargetCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<InvalidFunctionTarget:{}>", self.target())
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct InvalidFunctionArgsCondition {
    pub target: ArenaPointer,
    pub args: ArenaPointer,
}
impl TermSize for InvalidFunctionArgsCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidFunctionArgsCondition {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.target, arena).hash(&self.args, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<InvalidFunctionArgsCondition, A> {
    pub fn target(&self) -> Option<ArenaRef<Term, A>> {
        self.read_value(|term| term.target)
            .as_non_null()
            .map(|pointer| ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
    }
    pub fn args(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.args),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        self.target()
            .map(|target| target.capture_depth())
            .unwrap_or(0)
            .max(self.args().capture_depth())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.target()
            .map(|target| target.free_variables())
            .unwrap_or_default()
            .into_iter()
            .chain(self.args().free_variables())
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.target()
            .map(|target| target.count_variable_usages(offset))
            .unwrap_or(0)
            + self.args().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.target()
                .map(|target| target.dynamic_dependencies(deep))
                .unwrap_or_default()
                .union(self.args().dynamic_dependencies(deep))
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.target()
                .map(|target| target.has_dynamic_dependencies(deep))
                .unwrap_or(false)
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

impl<A: Arena + Clone> PartialEq for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<InvalidFunctionArgsCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<InvalidFunctionArgs:")?;
        match self.target() {
            Some(target) => write!(f, "{}", target)?,
            None => write!(f, "NULL")?,
        }
        write!(f, ":{}>", self.args())
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct InvalidPointerCondition;
impl TermSize for InvalidPointerCondition {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for InvalidPointerCondition {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<InvalidPointerCondition, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<InvalidPointerCondition, A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<InvalidPointerCondition, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<InvalidPointerCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<InvalidPointerCondition, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<InvalidPointer>")
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<ConditionTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
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
                effect_type: ArenaPointer(0x54321),
                payload: ArenaPointer(0x98765),
                token: ArenaPointer(0x12345),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::Custom as u32,
                0x54321,
                0x98765,
                0x12345,
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
                payload: ArenaPointer(0x54321),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::Error as u32,
                0x54321,
            ],
        );
    }

    #[test]
    fn condition_type_error() {
        assert_eq!(
            TermType::Condition(ConditionTerm::TypeError(TypeErrorCondition {
                expected: 0x54321,
                payload: ArenaPointer(0x98765),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::TypeError as u32,
                0x54321,
                0x98765,
            ],
        );
    }

    #[test]
    fn condition_invalid_function_target() {
        assert_eq!(
            TermType::Condition(ConditionTerm::InvalidFunctionTarget(
                InvalidFunctionTargetCondition {
                    target: ArenaPointer(0x54321),
                },
            ))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::InvalidFunctionTarget as u32,
                0x54321,
            ],
        );
    }

    #[test]
    fn condition_invalid_function_args() {
        assert_eq!(
            TermType::Condition(ConditionTerm::InvalidFunctionArgs(
                InvalidFunctionArgsCondition {
                    target: ArenaPointer(0x54321),
                    args: ArenaPointer(0x98765),
                },
            ))
            .as_bytes(),
            [
                TermTypeDiscriminants::Condition as u32,
                ConditionTermDiscriminants::InvalidFunctionArgs as u32,
                0x54321,
                0x98765,
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
