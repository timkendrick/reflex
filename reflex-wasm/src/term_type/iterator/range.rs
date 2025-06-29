// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{ArgType, DependencyList, GraphNode, SerializeJson, StackOffset};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ConstValue, Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    utils::{chunks_to_i64, i64_to_chunks},
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct RangeIteratorTerm {
    pub offset: [u32; 2],
    pub length: u32,
}
impl RangeIteratorTerm {
    pub fn new(offset: i64, length: u32) -> Self {
        Self {
            offset: i64_to_chunks(offset),
            length,
        }
    }
}
impl TermSize for RangeIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RangeIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.offset, arena).write_u32(self.length)
    }
}

impl<A: Arena + Clone> ArenaRef<RangeIteratorTerm, A> {
    pub fn offset(&self) -> i64 {
        self.read_value(|term| chunks_to_i64(term.offset))
    }
    pub fn length(&self) -> u32 {
        self.read_value(|term| term.length)
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<RangeIteratorTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<RangeIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.offset() == other.offset() && self.length() == other.length()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<RangeIteratorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<RangeIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<RangeIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RangeIterator")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<RangeIteratorTerm, A> {
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

impl<A: Arena + Clone> Internable for ArenaRef<RangeIteratorTerm, A> {
    fn should_intern(&self, _eager: ArgType) -> bool {
        true
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<RangeIteratorTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let offset = self.offset();
        let length = self.length();
        let block = CompiledBlockBuilder::new(stack);
        // Push the offset argument onto the stack
        // => [offset]
        let block = block.push(instruction::core::Const {
            value: ConstValue::I64(offset),
        });
        // Push the length argument onto the stack
        // => [offset, length]
        let block = block.push(instruction::core::Const {
            value: ConstValue::U32(length),
        });
        // Invoke the term constructor
        // => [RangeIteratorTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateRangeIterator,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn range_iterator() {
        assert_eq!(
            TermType::RangeIterator(RangeIteratorTerm::new(0x987654321, 0x54321)).as_bytes(),
            [
                TermTypeDiscriminants::RangeIterator as u32,
                0x87654321,
                0x00000009,
                0x54321
            ],
        );
        assert_eq!(
            TermType::RangeIterator(RangeIteratorTerm::new(-0x987654321, 0x54321)).as_bytes(),
            {
                let [low, high] = i64_to_chunks(-0x987654321);
                [
                    TermTypeDiscriminants::RangeIterator as u32,
                    low,
                    high,
                    0x54321,
                ]
            },
        );
    }
}
