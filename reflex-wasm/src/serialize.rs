use std::marker::PhantomData;

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::{
    core::NodeId,
    hash::{HashId, IntMap},
};
use reflex_utils::{
    PostOrderQueueVisitor, StatefulPostOrderVisitor, StatefulPostOrderVisitorAlgorithm, Visitable,
};

use crate::{
    allocator::{Arena, ArenaAllocator},
    hash::TermSize,
    ArenaPointer, ArenaRef,
};

pub struct SerializerState {
    pub(crate) allocated_terms: IntMap<HashId, ArenaPointer>,
    pub(crate) next_offset: ArenaPointer,
}
impl SerializerState {
    pub fn new(
        allocated_terms: impl IntoIterator<Item = (HashId, ArenaPointer)>,
        next_offset: ArenaPointer,
    ) -> Self {
        Self {
            allocated_terms: allocated_terms.into_iter().collect(),
            next_offset,
        }
    }
    pub fn end_offset(&self) -> ArenaPointer {
        self.next_offset
    }
}

pub trait Serialize {
    fn serialize<A: ArenaAllocator>(
        &self,
        destination: &mut A,
        state: &mut SerializerState,
    ) -> ArenaPointer;
}

impl<ASource: Arena + Clone, T: Clone + TermSize> Serialize for ArenaRef<T, ASource>
where
    ArenaRef<T, ASource>: NodeId + Visitable<ArenaPointer> + Visitable<ArenaRef<T, ASource>>,
{
    fn serialize<ADest: ArenaAllocator>(
        &self,
        destination: &mut ADest,
        state: &mut SerializerState,
    ) -> ArenaPointer {
        let (algorithm, mut state) = SerializeArenaTerms::new_with_state(destination, state);
        PostOrderQueueVisitor.apply_mut(self.clone(), &algorithm, &mut state)
    }
}

struct SerializeArenaTerms<'a, ADest: ArenaAllocator> {
    state: PhantomData<SerializeArenaTermsState<'a, ADest>>,
}

impl<'a, ADest: ArenaAllocator> SerializeArenaTerms<'a, ADest> {
    fn new_with_state(
        destination: &'a mut ADest,
        serializer_state: &'a mut SerializerState,
    ) -> (Self, SerializeArenaTermsState<'a, ADest>) {
        (
            Self { state: PhantomData },
            SerializeArenaTermsState {
                destination,
                serializer_state,
            },
        )
    }
}

struct SerializeArenaTermsState<'a, ADest: ArenaAllocator> {
    destination: &'a mut ADest,
    serializer_state: &'a mut SerializerState,
}

impl<'a, T, ASource, ADest> StatefulPostOrderVisitorAlgorithm<ArenaRef<T, ASource>>
    for SerializeArenaTerms<'a, ADest>
where
    T: Clone + TermSize,
    ArenaRef<T, ASource>: NodeId + Visitable<ArenaPointer> + Visitable<ArenaRef<T, ASource>>,
    ASource: Arena + Clone,
    ADest: ArenaAllocator,
{
    type Result = ArenaPointer;

    type State = SerializeArenaTermsState<'a, ADest>;

    fn visit_mut(
        &self,
        term: ArenaRef<T, ASource>,
        child_results: impl IntoIterator<Item = Self::Result>,
        state: &mut SerializeArenaTermsState<'a, ADest>,
    ) -> Self::Result {
        let SerializeArenaTermsState {
            destination,
            serializer_state,
        } = state;

        // Check if we have already serialized this before
        let cached_result = serializer_state.allocated_terms.get(&term.id()).copied();
        if let Some(existing) = cached_result {
            return existing;
        }

        // Get a list of internal field offsets within the current struct for all the pointer fields
        // that hold non-null references (this will be used to overwrite the values with the
        // serialized child target addresses)
        let internal_pointer_struct_offsets = Visitable::<ArenaPointer>::children(&term)
            // Filter out null pointers
            .filter(|struct_field_address| {
                let target_pointer = term
                    .arena
                    .read_value(*struct_field_address, |target_pointer: &ArenaPointer| {
                        *target_pointer
                    });
                !target_pointer.is_null()
            })
            // Get the offset of the field within the struct
            .map(|struct_field_address| u32::from(struct_field_address) - u32::from(term.pointer));

        // Pair up the field offsets with the corresponding serialized child terms
        let children = internal_pointer_struct_offsets
            .zip(child_results)
            .collect::<Vec<_>>();

        // Allocate space for the term in the target arena
        let allocator_offset = serializer_state.next_offset;
        let term_pointer = allocator_offset;
        let term_size = term.read_value(|term| term.size_of());
        destination.extend(allocator_offset, term_size);

        // Copy the term contents from the source arena to the target arena
        for index in 0..(term_size / 4) {
            let delta = (index * 4) as u32;
            let value = term
                .arena
                .read_value::<u32, _>(term.pointer.offset(delta), |value| *value);
            destination.write::<u32>(term_pointer.offset(delta), value);
        }

        // Update the serializer state to reflect the current allocator size
        serializer_state.next_offset = term_pointer.offset(term_size as u32);

        // Overwrite child pointers with updated addresses
        for (field_offset, child_pointer) in children {
            destination.write(term_pointer.offset(field_offset), child_pointer)
        }

        // Cache the term for future usages
        serializer_state
            .allocated_terms
            .insert(term.id(), term_pointer);

        // Return the target term arena pointer
        term_pointer
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::{
        allocator::{ArenaAllocator, ArenaIterator, VecAllocator},
        term_type::{IntTerm, TermType, TreeTerm},
        ArenaPointerIterator, ArenaRef, Term,
    };

    use super::*;

    #[test]
    fn serialize_trait() {
        let mut source_arena = VecAllocator::default();

        let _filler =
            source_arena.allocate(Term::new(TermType::Int(IntTerm::from(1)), &source_arena));

        let leaf = source_arena.allocate(Term::new(TermType::Int(IntTerm::from(2)), &source_arena));

        let root = source_arena.allocate(Term::new(
            TermType::Tree(TreeTerm {
                left: leaf,
                right: leaf,
                length: 2,
                depth: 1,
            }),
            &source_arena,
        ));

        let source_arena = Rc::new(RefCell::new(&mut source_arena));
        let root_ref = ArenaRef::<Term, _>::new(source_arena.clone(), root);

        let leaf_ref = ArenaRef::<Term, _>::new(source_arena.clone(), leaf);

        let mut target_arena = VecAllocator::default();

        let _filler =
            target_arena.allocate(Term::new(TermType::Int(IntTerm::from(3)), &target_arena));

        let _filler =
            target_arena.allocate(Term::new(TermType::Int(IntTerm::from(4)), &target_arena));

        let mut serializer_state = {
            let arena = &target_arena;
            let start_offset = arena.start_offset();
            let end_offset = arena.end_offset();
            let heap_values = ArenaIterator::<Term, _>::new(arena, start_offset, end_offset)
                .into_arena_refs::<Term, _>(&arena)
                .map(|term| {
                    let term_id = term.id();
                    (term_id, term.pointer)
                });
            let next_offset = arena.end_offset();
            SerializerState::new(heap_values, next_offset)
        };

        let mut target_arena = Rc::new(RefCell::new(&mut target_arena));

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
