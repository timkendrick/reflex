// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::{
    core::{
        ConditionListType, DependencyList, Eagerness, Expression, GraphNode, Internable,
        SerializeJson, StackOffset,
    },
    hash::HashId,
};
use reflex_utils::{MapIntoIterator, WithExactSizeIterator};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermTypeDiscriminants, TypedTerm},
    ArenaPointer, ArenaRef, IntoArenaRefIter, Term,
};
use reflex_macros::PointerIter;

use super::{ConditionTerm, WasmExpression};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct TreeTerm {
    pub left: ArenaPointer,
    pub right: ArenaPointer,
    pub length: u32,
}
impl TermSize for TreeTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TreeTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let hasher = if self.left.is_null() {
            hasher.write_bool(false)
        } else {
            arena.read_value::<Term, _>(self.left, |term| hasher.hash(term, arena))
        };
        let hasher = if self.right.is_null() {
            hasher.write_bool(false)
        } else {
            arena.read_value::<Term, _>(self.right, |term| hasher.hash(term, arena))
        };
        hasher.hash(&self.length, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<TreeTerm, A> {
    pub fn left(&self) -> Option<ArenaRef<Term, A>> {
        let pointer = self.read_value(|term| term.left);
        if pointer == ArenaPointer::null() {
            None
        } else {
            Some(ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
        }
    }
    pub fn right(&self) -> Option<ArenaRef<Term, A>> {
        let pointer = self.read_value(|term| term.right);
        if pointer == ArenaPointer::null() {
            None
        } else {
            Some(ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
        }
    }
    pub fn iter(&self) -> TreeIterator<'_, A> {
        TreeIterator::new(&self.arena, self.pointer)
    }
    pub fn nodes(&self) -> IntoArenaRefIter<'_, Term, A, TreeIterator<'_, A>> {
        IntoArenaRefIter::new(&self.arena, self.iter())
    }
    pub fn len(&self) -> u32 {
        self.read_value(|term| term.length)
    }
}

impl<A: Arena + Clone> ConditionListType<WasmExpression<A>> for ArenaRef<TreeTerm, A> {
    type Iterator<'a> = WithExactSizeIterator<MapIntoIterator<
        MatchConditionTermsIterator<'a, TreeIterator<'a, A>, A>,
        ArenaRef<TypedTerm<ConditionTerm>, A>,
        <WasmExpression<A> as Expression>::SignalRef<'a>
    >>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
        Self: 'a;
    fn id(&self) -> HashId {
        self.read_value(|term| {
            // FIXME: convert to 64-bit term hashes
            u32::from(term.hash(TermHasher::default(), &self.arena).finish()) as HashId
        })
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        WithExactSizeIterator::new(
            // This assumes every node in the tree is a condition term
            self.len() as usize,
            MapIntoIterator::new(MatchConditionTermsIterator::new(&self.arena, self.iter())),
        )
    }
}

impl<A: Arena + Clone> ConditionListType<WasmExpression<A>> for ArenaRef<TypedTerm<TreeTerm>, A> {
    type Iterator<'a> = <ArenaRef<TreeTerm, A> as ConditionListType<WasmExpression<A>>>::Iterator<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
        Self: 'a;

    fn id(&self) -> HashId {
        <ArenaRef<TreeTerm, A> as ConditionListType<WasmExpression<A>>>::id(&self.as_inner())
    }
    fn len(&self) -> usize {
        <ArenaRef<TreeTerm, A> as ConditionListType<WasmExpression<A>>>::len(&self.as_inner())
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        let inner = self.as_inner();
        WithExactSizeIterator::new(
            // This assumes every node in the tree is a condition term
            inner.len() as usize,
            MapIntoIterator::new(MatchConditionTermsIterator::new(
                &self.arena,
                TreeIterator::new(&self.arena, inner.pointer),
            )),
        )
    }
}

pub struct MatchConditionTermsIterator<'a, T, A>
where
    T: Iterator<Item = ArenaPointer>,
    A: Arena,
{
    inner: T,
    arena: &'a A,
}
impl<'a, T, A> MatchConditionTermsIterator<'a, T, A>
where
    T: Iterator<Item = ArenaPointer>,
    A: Arena,
{
    pub fn new(arena: &'a A, inner: T) -> Self {
        Self { arena, inner }
    }
}

impl<'a, T, A> Iterator for MatchConditionTermsIterator<'a, T, A>
where
    T: Iterator<Item = ArenaPointer>,
    A: Arena + Clone,
{
    type Item = ArenaRef<TypedTerm<ConditionTerm>, A>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|item_pointer| {
            let item = ArenaRef::<Term, _>::new(self.arena.clone(), item_pointer);
            match item.read_value(|term| term.type_id()) {
                TermTypeDiscriminants::Condition => {
                    let condition_term = ArenaRef::<TypedTerm<ConditionTerm>, _>::new(
                        self.arena.clone(),
                        item_pointer,
                    );
                    Some(condition_term)
                }
                _ => self.next(),
            }
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T, A> ExactSizeIterator for MatchConditionTermsIterator<'a, T, A>
where
    T: Iterator<Item = ArenaPointer> + ExactSizeIterator,
    A: Arena + Clone,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<TreeTerm, A> {
    fn size(&self) -> usize {
        1 + self.left().map(|term| term.size()).unwrap_or(0)
            + self.right().map(|term| term.size()).unwrap_or(0)
    }
    fn capture_depth(&self) -> StackOffset {
        self.left()
            .map(|term| term.size())
            .unwrap_or(0)
            .max(self.right().map(|term| term.size()).unwrap_or(0))
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let left_free_variables = self
            .left()
            .map(|term| term.free_variables())
            .unwrap_or_default();
        let right_free_variables = self
            .right()
            .map(|term| term.free_variables())
            .unwrap_or_default();
        if left_free_variables.is_empty() {
            right_free_variables
        } else if right_free_variables.is_empty() {
            left_free_variables
        } else {
            let mut combined = left_free_variables;
            combined.extend(right_free_variables);
            combined
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.left()
            .map(|term| term.count_variable_usages(offset))
            .unwrap_or(0)
            + self
                .right()
                .map(|term| term.count_variable_usages(offset))
                .unwrap_or(0)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.left()
                .map(|term| term.dynamic_dependencies(deep))
                .unwrap_or_default()
                .union(
                    self.right()
                        .map(|term| term.dynamic_dependencies(deep))
                        .unwrap_or_default(),
                )
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.left()
                .map(|term| term.has_dynamic_dependencies(deep))
                .unwrap_or(false)
                || self
                    .right()
                    .map(|term| term.has_dynamic_dependencies(deep))
                    .unwrap_or(false)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.left().map(|term| term.is_atomic()).unwrap_or(true)
            || self.right().map(|term| term.is_atomic()).unwrap_or(true)
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<TreeTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<TreeTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Clarify PartialEq implementations for container terms
        // This assumes that trees with the same length and hash are almost certainly identical
        self.len() == other.len()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<TreeTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<TreeTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<TreeTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let left = self.read_value(|term| term.left);
        let right = self.read_value(|term| term.right);
        write!(f, "(")?;
        if ArenaPointer::is_null(left) {
            write!(f, "NULL")?;
        } else {
            write!(f, "{}", ArenaRef::<Term, _>::new(self.arena.clone(), left))?;
        }
        write!(f, " . ")?;
        if ArenaPointer::is_null(right) {
            write!(f, "NULL")?;
        } else {
            write!(f, "{}", ArenaRef::<Term, _>::new(self.arena.clone(), right))?;
        }
        write!(f, ")")
    }
}

pub struct TreeIterator<'a, A: Arena> {
    stack: TreeIteratorStack,
    arena: &'a A,
}
impl<'a, A: Arena> TreeIterator<'a, A> {
    fn new(arena: &'a A, root: ArenaPointer) -> Self {
        Self {
            arena,
            stack: TreeIteratorStack {
                cursor: TreeIteratorCursor::Left,
                items: vec![root],
            },
        }
    }
}
struct TreeIteratorStack {
    cursor: TreeIteratorCursor,
    items: Vec<ArenaPointer>,
}
impl TreeIteratorStack {
    fn push(&mut self, item: ArenaPointer) {
        match self.cursor {
            TreeIteratorCursor::Left => {
                // If we were processing the left branch, create a new stack entry so that we can return later to process the right branch
                self.items.push(item);
            }
            TreeIteratorCursor::Right => {
                // If we were processing the right branch, the cell is no longer needed so the current stack entry can be reused
                let num_items = self.items.len();
                self.items[num_items - 1] = item;
                self.cursor = TreeIteratorCursor::Left;
            }
        };
    }
    fn pop(&mut self) {
        // The stack is only ever popped when processing the right branch; its parent's left branch must already have been processed
        // so it's safe to assume that we always jump from processing a child's right branch to the parent's right branch.
        match self.cursor {
            TreeIteratorCursor::Left => unreachable!(),
            TreeIteratorCursor::Right => {
                self.items.pop();
            }
        }
    }
    fn peek(&self) -> Option<&ArenaPointer> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.items.len() - 1])
        }
    }
}
impl<'a, A: Arena + Clone> Iterator for TreeIterator<'a, A> {
    type Item = ArenaPointer;
    fn next(&mut self) -> Option<Self::Item> {
        let (cursor, child_pointer) = match self.stack.peek().copied() {
            None => None,
            Some(tree_term) => {
                let child_pointer = {
                    match self.stack.cursor {
                        TreeIteratorCursor::Left => self
                            .arena
                            .read_value::<TreeTerm, _>(tree_term, |tree| tree.left),
                        TreeIteratorCursor::Right => self
                            .arena
                            .read_value::<TreeTerm, _>(tree_term, |tree| tree.right),
                    }
                };
                Some((self.stack.cursor, child_pointer))
            }
        }?;
        match cursor {
            TreeIteratorCursor::Left => {
                if child_pointer.is_null() {
                    // If this is the null leaf marker, we need to shift from the left to the right branch
                    self.stack.cursor = TreeIteratorCursor::Right;
                    return self.next();
                } else {
                    // Determine whether the current item is itself a cell which needs to be traversed deeper
                    match self
                        .arena
                        .read_value::<Term, _>(child_pointer, |term| term.type_id())
                    {
                        // If so, push the cell to the stack and repeat the iteration with the updated stack
                        TermTypeDiscriminants::Tree => {
                            let tree_term = ArenaRef::<TypedTerm<TreeTerm>, _>::new(
                                self.arena.clone(),
                                child_pointer,
                            );
                            self.stack.push(tree_term.as_inner().as_pointer());
                            return self.next();
                        }
                        // Otherwise emit the value and shift from the left to the right branch
                        _ => {
                            self.stack.cursor = TreeIteratorCursor::Right;
                            Some(child_pointer)
                        }
                    }
                }
            }
            TreeIteratorCursor::Right => {
                if child_pointer.is_null() {
                    // If this is the null leaf marker, we are at the end of a list and need to pop the stack
                    // Pop the current entry from the stack and repeat the iteration with the updated stack
                    self.stack.pop();
                    return self.next();
                } else {
                    // Determine whether the current item is itself a cell which needs to be traversed deeper
                    match self
                        .arena
                        .read_value::<Term, _>(child_pointer, |term| term.type_id())
                    {
                        // If so, push the cell to the stack and repeat the iteration with the updated stack
                        TermTypeDiscriminants::Tree => {
                            let tree_term = ArenaRef::<TypedTerm<TreeTerm>, _>::new(
                                self.arena.clone(),
                                child_pointer,
                            );
                            self.stack.push(tree_term.as_inner().as_pointer());
                            return self.next();
                        }
                        _ => {
                            self.stack.pop();
                            Some(child_pointer)
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TreeIteratorCursor {
    Left,
    Right,
}

impl<A: Arena + Clone> Internable for ArenaRef<TreeTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn tree() {
        assert_eq!(
            TermType::Tree(TreeTerm {
                left: ArenaPointer(12345),
                right: ArenaPointer(67890),
                length: 54321
            })
            .as_bytes(),
            [TermTypeDiscriminants::Tree as u32, 12345, 67890, 54321],
        );
    }
}
