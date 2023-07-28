// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use reflex::core::{
    DependencyList, GraphNode, SerializeJson, StackOffset, TimestampTermType, TimestampValue,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ConstValue, Eagerness,
        Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    utils::{chunks_to_i64, i64_to_chunks},
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct TimestampTerm {
    pub millis: [u32; 2],
}
impl TermSize for TimestampTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TimestampTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.millis, arena)
    }
}
impl From<i64> for TimestampTerm {
    fn from(value: i64) -> Self {
        Self {
            millis: i64_to_chunks(value),
        }
    }
}
impl From<TimestampTerm> for i64 {
    fn from(value: TimestampTerm) -> Self {
        let TimestampTerm { millis, .. } = value;
        chunks_to_i64(millis)
    }
}

impl<A: Arena + Clone> ArenaRef<TimestampTerm, A> {
    pub fn millis(&self) -> i64 {
        self.read_value(|term| i64::from(*term))
    }
}

impl<A: Arena + Clone> TimestampTermType for ArenaRef<TimestampTerm, A> {
    fn millis(&self) -> TimestampValue {
        self.millis() as TimestampValue
    }
}

impl<A: Arena + Clone> TimestampTermType for ArenaRef<TypedTerm<TimestampTerm>, A> {
    fn millis(&self) -> TimestampValue {
        <ArenaRef<TimestampTerm, A> as TimestampTermType>::millis(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<TimestampTerm, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::new()
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<TimestampTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::String(format!("{}", self)))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.millis() == target.millis() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<TimestampTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.millis() == other.millis()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<TimestampTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<TimestampTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<TimestampTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&UtcTimestamp(self.millis()), f)
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<TimestampTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct UtcTimestamp(pub i64);

impl std::fmt::Display for UtcTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(timestamp) = *self;
        let seconds = timestamp / 1000;
        let millis = timestamp % 10;
        let nanos = millis * 1000;
        write!(
            f,
            "{}",
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(seconds, nanos as u32).unwrap_or_default(),
                Utc,
            )
            .to_rfc3339_opts(SecondsFormat::AutoSi, true)
        )
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<TimestampTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let millis = self.millis();
        let block = CompiledBlockBuilder::new(stack);
        // Push the value argument onto the stack
        // => [value]
        let block = block.push(instruction::core::Const {
            value: ConstValue::I64(millis),
        });
        // Invoke the term constructor
        // => [TimestampTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateTimestamp,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        term_type::{TermType, TermTypeDiscriminants},
        utils::i64_to_chunks,
    };

    use super::*;

    #[test]
    fn date() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert_eq!(
            TermType::Timestamp(TimestampTerm::from(timestamp)).as_bytes(),
            [
                TermTypeDiscriminants::Timestamp as u32,
                i64_to_chunks(timestamp)[0],
                i64_to_chunks(timestamp)[1]
            ],
        );
    }
}
