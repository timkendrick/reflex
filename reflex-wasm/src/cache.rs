// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::Arena,
    hash::{TermHashState, TermSize},
    term_type::{TreeTerm, TypedTerm},
    utils::{chunks_to_u64, u64_to_chunks},
    ArenaPointer, ArenaRef, PointerIter, Term,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ApplicationCache {
    pub value: ArenaPointer,
    pub dependencies: ArenaPointer,
    pub overall_state_hash: [u32; 2],
    pub minimal_state_hash: [u32; 2],
}

pub type ApplicationCachePointerIter =
    std::iter::Chain<std::option::IntoIter<ArenaPointer>, std::option::IntoIter<ArenaPointer>>;

impl<A: Arena + Clone> PointerIter for ArenaRef<ApplicationCache, A> {
    type Iter<'a> = ApplicationCachePointerIter
    where
        Self: 'a;
    fn iter<'a>(&self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        let value = self.read_value(|term| {
            term.value
                .as_non_null()
                .map(|_| self.inner_pointer(|term| &term.value))
        });
        let dependencies = self.read_value(|term| {
            term.dependencies
                .as_non_null()
                .map(|_| self.inner_pointer(|term| &term.dependencies))
        });
        value.into_iter().chain(dependencies)
    }
}

impl TermSize for ApplicationCache {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Default for ApplicationCache {
    fn default() -> Self {
        Self {
            value: ArenaPointer::null(),
            dependencies: ArenaPointer::null(),
            overall_state_hash: u64_to_chunks(0xFFFFFFFFFFFFFFFF),
            minimal_state_hash: u64_to_chunks(0xFFFFFFFFFFFFFFFF),
        }
    }
}

impl<A: Arena + Clone> ArenaRef<ApplicationCache, A> {
    pub fn value(&self) -> Option<ArenaRef<Term, A>> {
        let pointer = self.read_value(|value| value.value).as_non_null()?;
        Some(ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
    }
    pub fn dependencies(&self) -> Option<ArenaRef<TypedTerm<TreeTerm>, A>> {
        let pointer = self.read_value(|value| value.dependencies).as_non_null()?;
        Some(ArenaRef::<TypedTerm<TreeTerm>, _>::new(
            self.arena.clone(),
            pointer,
        ))
    }
    pub fn overall_state_hash(&self) -> Option<TermHashState> {
        let value = self.read_value(|value| chunks_to_u64(value.overall_state_hash));
        if value == 0xFFFFFFFFFFFFFFFF {
            None
        } else {
            Some(TermHashState::from(value))
        }
    }
    pub fn minimal_state_hash(&self) -> Option<TermHashState> {
        let value = self.read_value(|value| chunks_to_u64(value.minimal_state_hash));
        if value == 0xFFFFFFFFFFFFFFFF {
            None
        } else {
            Some(TermHashState::from(value))
        }
    }
}
