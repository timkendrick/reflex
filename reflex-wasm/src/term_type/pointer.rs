// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, GraphNode, RefType, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PointerTerm {
    pub target: TermPointer,
}
impl TermSize for PointerTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PointerTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.target, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, PointerTerm, A> {
    pub fn target(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().target))
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, PointerTerm, A> {
    fn size(&self) -> usize {
        self.target().size()
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
        self.target().dynamic_dependencies(deep)
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.target().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        self.target().is_static()
    }
    fn is_atomic(&self) -> bool {
        self.target().is_atomic()
    }
    fn is_complex(&self) -> bool {
        self.target().is_complex()
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, PointerTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, PointerTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, PointerTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, PointerTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, PointerTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pointer({:#x})", u32::from(self.as_deref().target))
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn pointer() {
        assert_eq!(
            TermType::Pointer(PointerTerm {
                target: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Pointer as u32, 12345],
        );
    }
}
