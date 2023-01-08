// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::{
    core::NodeId,
    hash::{HashId, IntMap},
};

use crate::{
    allocator::ArenaAllocator, hash::TermSize, ArenaRef, IntoArenaRefIterator, PointerIter,
    TermPointer,
};

pub struct SerializerState {
    allocated_terms: IntMap<HashId, TermPointer>,
    next_offset: TermPointer,
}
impl SerializerState {
    pub(crate) fn new(
        allocated_terms: impl IntoIterator<Item = (HashId, TermPointer)>,
        next_offset: TermPointer,
    ) -> Self {
        Self {
            allocated_terms: allocated_terms.into_iter().collect(),
            next_offset,
        }
    }
    pub fn from_existing_arena<T, A: ArenaAllocator + PointerIter + Clone>(arena: &A) -> Self
    where
        ArenaRef<T, A>: NodeId,
    {
        let next_offset = arena.len() as u32;
        Self::new(
            arena.iter().as_arena_refs::<T>(arena).map(|term| {
                let term_id = term.id();
                (term_id, term.pointer)
            }),
            TermPointer::from(next_offset),
        )
    }
}

pub trait Serialize {
    fn serialize<A: ArenaAllocator>(
        &self,
        destination: &mut A,
        state: &mut SerializerState,
    ) -> TermPointer;
}

impl<ASource: ArenaAllocator + Clone, T: Clone + TermSize> Serialize for ArenaRef<T, ASource>
where
    ArenaRef<T, ASource>: PointerIter + NodeId,
{
    fn serialize<ADest: ArenaAllocator>(
        &self,
        destination: &mut ADest,
        state: &mut SerializerState,
    ) -> TermPointer {
        // Check if we have already serialized this before
        let cached_result = state.allocated_terms.get(&self.id()).copied();
        if let Some(existing) = cached_result {
            return existing;
        }

        // Iterate over the source term children and serialize them into the target arena
        let children = PointerIter::iter(self)
            .filter_map(|inner_pointer| {
                let value_pointer = self
                    .arena
                    .read_value(inner_pointer, |target_pointer: &TermPointer| {
                        *target_pointer
                    })
                    .as_non_null()?;
                Some((inner_pointer, value_pointer))
            })
            .map(|(inner_pointer, value_pointer)| {
                (
                    // The offset of the field of the term within the struct
                    u32::from(inner_pointer) - u32::from(self.pointer),
                    ArenaRef::<T, ASource>::new(self.arena.clone(), value_pointer)
                        .serialize(destination, state),
                )
            })
            .collect::<Vec<_>>();

        // Allocate space for the term in the target arena
        let allocator_offset = state.next_offset;
        let term_pointer = allocator_offset;
        let term_size = self.read_value(|t| t.size_of());
        destination.extend(allocator_offset, term_size);

        // Copy the term contents from the source arena to the target arena
        for index in 0..(term_size / 4) {
            let delta = (index * 4) as u32;
            let value = self
                .arena
                .read_value::<u32, _>(self.pointer.offset(delta), |value| *value);
            destination.write::<u32>(term_pointer.offset(delta), value);
        }

        // Update the serializer state to reflect the current allocator size
        state.next_offset = term_pointer.offset(term_size as u32);

        // Overwrite child pointers with updated addresses
        for (delta, child_pointer) in children {
            destination.write(term_pointer.offset(delta), child_pointer)
        }

        // Cache the term for future usages
        state.allocated_terms.insert(self.id(), term_pointer);

        // Return the target term arena pointer
        term_pointer
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::{
        allocator::{ArenaAllocator, VecAllocator},
        term_type::{IntTerm, TermType, TreeTerm},
        ArenaRef, Term,
    };

    use super::*;

    #[test]
    fn serialize_trait() {
        let mut source_arena = VecAllocator::default();

        let _filler = source_arena.allocate(Term::new(
            TermType::Int(IntTerm { value: 1 }),
            &source_arena,
        ));

        let leaf = source_arena.allocate(Term::new(
            TermType::Int(IntTerm { value: 2 }),
            &source_arena,
        ));

        let root = source_arena.allocate(Term::new(
            TermType::Tree(TreeTerm {
                left: leaf,
                right: leaf,
                length: 2,
            }),
            &source_arena,
        ));

        let source_arena = Rc::new(RefCell::new(source_arena));
        let root_ref = ArenaRef::<Term, _>::new(source_arena.clone(), root);

        let leaf_ref = ArenaRef::<Term, _>::new(source_arena.clone(), leaf);

        let mut target_arena = VecAllocator::default();

        let _filler = target_arena.allocate(Term::new(
            TermType::Int(IntTerm { value: 3 }),
            &target_arena,
        ));

        let _filler = target_arena.allocate(Term::new(
            TermType::Int(IntTerm { value: 4 }),
            &target_arena,
        ));

        let mut target_arena = Rc::new(RefCell::new(target_arena));

        let mut serializer_state = SerializerState::from_existing_arena::<Term, _>(&target_arena);

        let serialized_expression = root_ref.serialize(&mut target_arena, &mut serializer_state);

        let serialized_root_ref =
            ArenaRef::<Term, _>::new(target_arena.clone(), serialized_expression);

        assert_eq!(root_ref, serialized_root_ref);

        let target_left_pointer = serialized_root_ref
            .as_typed_term::<TreeTerm>()
            .as_inner()
            .read_value(|term| term.left);
        let serialized_leaf_ref =
            ArenaRef::<Term, _>::new(target_arena.clone(), target_left_pointer);
        assert_eq!(leaf_ref, serialized_leaf_ref);

        let target_right_pointer = serialized_root_ref
            .as_typed_term::<TreeTerm>()
            .as_inner()
            .read_value(|term| term.right);
        assert_eq!(target_left_pointer, target_right_pointer);
    }
}
