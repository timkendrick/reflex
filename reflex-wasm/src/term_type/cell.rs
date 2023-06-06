// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, GraphNode, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHashState, TermHasher, TermSize},
    term_type::TermType,
    ArenaRef, Array, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
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
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}
impl CellTerm {
    pub fn allocate(
        values: impl IntoIterator<Item = u32, IntoIter = impl ExactSizeIterator<Item = u32>>,
        arena: &mut impl ArenaAllocator,
    ) -> TermPointer {
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
        let hash = TermHashState::from(u32::from(instance));
        arena.get_mut::<Term>(instance).set_hash(hash);
        instance
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, CellTerm, A> {
    pub fn fields(&self) -> impl Iterator<Item = u32> + '_ {
        self.as_value().fields.iter().copied()
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, CellTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, CellTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, CellTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.as_value().fields, &other.as_value().fields)
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, CellTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, CellTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, CellTerm, A> {
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

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        term_type::{TermType, TermTypeDiscriminants},
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
            let entries = [12345, 67890];
            let instance = CellTerm::allocate(entries, &mut allocator);
            let result = allocator.get::<Term>(instance).as_bytes();
            let hash = result[0];
            let discriminant = result[1];
            let data_length = result[2];
            let data_capacity = result[3];
            let data = &result[4..];
            assert_eq!(hash, u32::from(instance));
            assert_eq!(discriminant, TermTypeDiscriminants::Cell as u32);
            assert_eq!(data_length, entries.len() as u32);
            assert_eq!(data_capacity, entries.len() as u32);
            assert_eq!(data, [12345, 67890]);
        }
    }
}
