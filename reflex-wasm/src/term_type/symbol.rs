// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{SymbolId, SymbolTermType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SymbolTerm {
    pub id: u32,
}
impl TermSize for SymbolTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for SymbolTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.id, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, SymbolTerm, A> {
    fn id(&self) -> u32 {
        self.as_deref().id
    }
}

impl<'heap, A: ArenaAllocator> SymbolTermType for ArenaRef<'heap, SymbolTerm, A> {
    fn id(&self) -> SymbolId {
        self.id() as SymbolId
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn symbol() {
        assert_eq!(
            TermType::Symbol(SymbolTerm { id: 12345 }).as_bytes(),
            [TermTypeDiscriminants::Symbol as u32, 12345],
        );
    }
}
