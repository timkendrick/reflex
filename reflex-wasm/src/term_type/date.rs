// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use reflex::core::{
    DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset,
    TimestampTermType, TimestampValue,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ConstValue,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    utils::{chunks_to_i64, i64_to_chunks},
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct DateTerm {
    pub timestamp: [u32; 2],
}
impl TermSize for DateTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for DateTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.timestamp, arena)
    }
}
impl From<i64> for DateTerm {
    fn from(value: i64) -> Self {
        Self {
            timestamp: i64_to_chunks(value),
        }
    }
}
impl From<DateTerm> for i64 {
    fn from(value: DateTerm) -> Self {
        let DateTerm { timestamp, .. } = value;
        chunks_to_i64(timestamp)
    }
}

impl<A: Arena + Clone> ArenaRef<DateTerm, A> {
    pub fn timestamp(&self) -> i64 {
        self.read_value(|term| i64::from(*term))
    }
}

impl<A: Arena + Clone> TimestampTermType for ArenaRef<DateTerm, A> {
    fn millis(&self) -> TimestampValue {
        self.timestamp() as TimestampValue
    }
}

impl<A: Arena + Clone> TimestampTermType for ArenaRef<TypedTerm<DateTerm>, A> {
    fn millis(&self) -> TimestampValue {
        <ArenaRef<DateTerm, A> as TimestampTermType>::millis(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<DateTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<DateTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::String(format!("{}", self)))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.timestamp() == target.timestamp() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<DateTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp() == other.timestamp()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<DateTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<DateTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<DateTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&DateTimestamp(self.timestamp()), f)
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<DateTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DateTimestamp(pub i64);

impl std::fmt::Display for DateTimestamp {
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

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<DateTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let timestamp = self.timestamp();
        let block = CompiledBlockBuilder::new(stack);
        // Push the value argument onto the stack
        // => [value]
        let block = block.push(instruction::core::Const {
            value: ConstValue::I64(timestamp),
        });
        // Invoke the term constructor
        // => [DateTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateDate,
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
            TermType::Date(DateTerm::from(timestamp)).as_bytes(),
            [
                TermTypeDiscriminants::Date as u32,
                i64_to_chunks(timestamp)[0],
                i64_to_chunks(timestamp)[1]
            ],
        );
    }
}
