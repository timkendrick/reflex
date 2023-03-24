// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::{Arena, ArenaAllocator},
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, CompilerVariableBindings, ValueType,
    },
    hash::{TermHash, TermHashState, TermHasher, TermSize},
    term_type::TermType,
    ArenaPointer, ArenaRef, Array, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct CellTerm {
    pub fields: Array<u32>,
}
impl TermSize for CellTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<u32>>() + self.fields.size_of()
    }
}
impl TermHash for CellTerm {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher
    }
}
impl CellTerm {
    pub fn allocate(
        values: impl IntoIterator<Item = u32, IntoIter = impl ExactSizeIterator<Item = u32>>,
        arena: &mut impl ArenaAllocator,
    ) -> ArenaPointer {
        let values = values.into_iter();
        let term = Term::new(
            TermType::Cell(Self {
                fields: Default::default(),
            }),
            arena,
        );
        let term_size = term.size_of();
        let instance = arena.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<u32>>()) as u32);
        Array::<u32>::extend(list, values, arena);
        let hash = TermHashState::from(u32::from(instance) as u64);
        arena.write::<u64>(Term::get_hash_pointer(instance), u64::from(hash));
        instance
    }
}

impl<A: Arena + Clone> ArenaRef<CellTerm, A> {
    pub fn capacity(&self) -> u32 {
        self.read_value(|term| term.fields.capacity)
    }
    pub fn fields(&self) -> impl Iterator<Item = u32> + '_ {
        Array::<u32>::iter(self.inner_pointer(|value| &value.fields), &self.arena)
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<CellTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<CellTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<CellTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.arena, &other.arena) && self.pointer == other.pointer
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<CellTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<CellTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<CellTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{{{}}}",
            self.fields()
                .map(|word| format!("{:#x}", word))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<CellTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        true
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<CellTerm, A> {
    fn compile(
        &self,
        _state: &mut CompilerState,
        _bindings: &CompilerVariableBindings,
        _options: &CompilerOptions,
        _stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let capacity = self.capacity();
        let fields = self.fields();
        let mut instructions = CompiledBlock::default();
        // Push the capacity argument onto the stack
        // => [capacity]
        instructions.push(CompiledInstruction::u32_const(capacity));
        // Allocate the cell term
        // => [CellTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::AllocateCell,
        ));
        // Write the cell contents into the newly-allocated cell term
        for (index, value) in fields.enumerate() {
            instructions.extend([
                // Duplicate the cell term pointer onto the stack
                // => [CellTerm, CellTerm]
                CompiledInstruction::Duplicate(ValueType::HeapPointer),
                // Push the field index onto the stack
                // => [CellTerm, CellTerm, index]
                CompiledInstruction::u32_const(index as u32),
                // Push the cell value onto the stack
                // => [CellTerm, CellTerm, index, value]
                CompiledInstruction::u32_const(value),
                // Update the cell term's field at the given index
                // => [CellTerm]
                CompiledInstruction::CallRuntimeBuiltin(RuntimeBuiltin::SetCellField),
            ]);
        }
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        term_type::{TermType, TermTypeDiscriminants},
        utils::chunks_to_u64,
    };

    use super::*;

    #[test]
    fn cell() {
        assert_eq!(
            TermType::Cell(CellTerm {
                fields: Default::default()
            })
            .as_bytes(),
            [TermTypeDiscriminants::Cell as u32, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let entries = [0x54321, 0x98765];
            let instance = CellTerm::allocate(entries, &mut allocator);
            let result = allocator.get_ref::<Term>(instance).as_bytes();
            let hash = chunks_to_u64([result[0], result[1]]);
            let discriminant = result[2];
            let data_length = result[3];
            let data_capacity = result[4];
            let data = &result[5..];
            assert_eq!(hash, u32::from(instance) as u64);
            assert_eq!(discriminant, TermTypeDiscriminants::Cell as u32);
            assert_eq!(data_length, entries.len() as u32);
            assert_eq!(data_capacity, entries.len() as u32);
            assert_eq!(data, [0x54321, 0x98765]);
        }
    }
}
