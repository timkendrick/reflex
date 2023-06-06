// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::{
    core::{ConditionListType, DependencyList, Expression, GraphNode, SerializeJson, StackOffset},
    hash::HashId,
};
use reflex_utils::{MapIntoIterator, WithExactSizeIterator};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermTypeDiscriminants, TypedTerm},
    ArenaRef, IntoArenaRefIterator, Term, TermPointer,
};

use super::{ConditionTerm, WasmExpression};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TreeTerm {
    pub left: TermPointer,
    pub right: TermPointer,
    pub length: u32,
}
impl TermSize for TreeTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TreeTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let hasher = if self.left.is_null() {
            hasher.write_bool(false)
        } else {
            hasher.hash(arena.get::<Term>(self.left), arena)
        };
        let hasher = if self.right.is_null() {
            hasher.write_bool(false)
        } else {
            hasher.hash(arena.get::<Term>(self.right), arena)
        };
        hasher.hash(&self.length, arena)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, TreeTerm, A> {
    pub fn left(&self) -> Option<ArenaRef<'heap, Term, A>> {
        let pointer = self.as_value().left;
        if pointer == TermPointer::null() {
            None
        } else {
            Some(ArenaRef::<Term, _>::new(self.arena, pointer))
        }
    }
    pub fn right(&self) -> Option<ArenaRef<'heap, Term, A>> {
        let pointer = self.as_value().right;
        if pointer == TermPointer::null() {
            None
        } else {
            Some(ArenaRef::<Term, _>::new(self.arena, pointer))
        }
    }
    pub fn iter<'a>(&'a self) -> TreeIterator<'heap, A> {
        TreeIterator::new(*self)
    }
    pub fn nodes<'a>(&'a self) -> IntoArenaRefIterator<'heap, Term, A, TreeIterator<'a, A>> {
        IntoArenaRefIterator::new(self.arena, self.iter())
    }
    pub fn len(&self) -> u32 {
        self.as_value().length
    }
}

impl<'heap, A: ArenaAllocator> ConditionListType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TreeTerm, A>
{
    type Iterator<'a> = WithExactSizeIterator<MapIntoIterator<
        MatchConditionTermsIterator<'heap, TreeIterator<'heap, A>, A>,
        ArenaRef<'heap, TypedTerm<ConditionTerm>, A>,
        <WasmExpression<'heap, A> as Expression>::SignalRef<'a>
    >>
    where
        <WasmExpression<'heap, A> as Expression>::Signal: 'a,
        WasmExpression<'heap, A>: 'a,
        Self: 'a;
    fn id(&self) -> HashId {
        // FIXME: convert to 64-bit term hashes
        u32::from(
            self.as_value()
                .hash(TermHasher::default(), self.arena)
                .finish(),
        ) as HashId
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        <WasmExpression<'heap, A> as Expression>::Signal: 'a,
        WasmExpression<'heap, A>: 'a,
    {
        WithExactSizeIterator::new(
            // This assumes every node in the tree is a condition term
            self.len() as usize,
            MapIntoIterator::new(MatchConditionTermsIterator::new(self.arena, self.iter())),
        )
    }
}

impl<'heap, A: ArenaAllocator> ConditionListType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<TreeTerm>, A>
{
    type Iterator<'a> = <ArenaRef<'heap, TreeTerm, A> as ConditionListType<WasmExpression<'heap, A>>>::Iterator<'a>
    where
        <WasmExpression<'heap, A> as Expression>::Signal: 'a,
        WasmExpression<'heap, A>: 'a,
        Self: 'a;

    fn id(&self) -> HashId {
        <ArenaRef<'heap, TreeTerm, A> as ConditionListType<WasmExpression<'heap, A>>>::id(
            &self.as_inner(),
        )
    }
    fn len(&self) -> usize {
        <ArenaRef<'heap, TreeTerm, A> as ConditionListType<WasmExpression<'heap, A>>>::len(
            &self.as_inner(),
        )
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        <WasmExpression<'heap, A> as Expression>::Signal: 'a,
        WasmExpression<'heap, A>: 'a,
    {
        <ArenaRef<'heap, TreeTerm, A> as ConditionListType<WasmExpression<'heap, A>>>::iter(
            &self.as_inner(),
        )
    }
}

pub struct MatchConditionTermsIterator<'heap, T, A>
where
    T: Iterator<Item = TermPointer>,
    A: ArenaAllocator,
{
    inner: T,
    arena: &'heap A,
}
impl<'heap, T, A> MatchConditionTermsIterator<'heap, T, A>
where
    T: Iterator<Item = TermPointer>,
    A: ArenaAllocator,
{
    pub fn new(arena: &'heap A, inner: T) -> Self {
        Self { arena, inner }
    }
}

impl<'heap, T, A> Iterator for MatchConditionTermsIterator<'heap, T, A>
where
    T: Iterator<Item = TermPointer>,
    A: ArenaAllocator,
{
    type Item = ArenaRef<'heap, TypedTerm<ConditionTerm>, A>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|item_pointer| {
            let item = ArenaRef::<Term, _>::new(self.arena, item_pointer);
            match item.as_value().type_id() {
                TermTypeDiscriminants::Condition => {
                    let condition_term =
                        ArenaRef::<TypedTerm<ConditionTerm>, _>::new(self.arena, item_pointer);
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

impl<'heap, T, A> ExactSizeIterator for MatchConditionTermsIterator<'heap, T, A>
where
    T: Iterator<Item = TermPointer> + ExactSizeIterator,
    A: ArenaAllocator,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, TreeTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, TreeTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, TreeTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Clarify PartialEq implementations for container terms
        // This assumes that trees with the same length and hash are almost certainly identical
        self.len() == other.len()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, TreeTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, TreeTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, TreeTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let left = self.as_value().left;
        let right = self.as_value().right;
        write!(f, "(")?;
        if TermPointer::is_null(left) {
            write!(f, "NULL")?;
        } else {
            write!(f, "{}", ArenaRef::<Term, _>::new(self.arena, left))?;
        }
        write!(f, " . ")?;
        if TermPointer::is_null(right) {
            write!(f, "NULL")?;
        } else {
            write!(f, "{}", ArenaRef::<Term, _>::new(self.arena, right))?;
        }
        write!(f, ")")
    }
}

pub struct TreeIterator<'heap, A: ArenaAllocator> {
    stack: TreeIteratorStack<'heap, A>,
    arena: &'heap A,
}
impl<'a, A: ArenaAllocator> TreeIterator<'a, A> {
    fn new(root: ArenaRef<'a, TreeTerm, A>) -> Self {
        Self {
            arena: root.arena,
            stack: TreeIteratorStack {
                cursor: TreeIteratorCursor::Left,
                items: vec![root],
            },
        }
    }
}
struct TreeIteratorStack<'a, A: ArenaAllocator> {
    cursor: TreeIteratorCursor,
    items: Vec<ArenaRef<'a, TreeTerm, A>>,
}
impl<'a, A: ArenaAllocator> TreeIteratorStack<'a, A> {
    fn push(&mut self, item: ArenaRef<'a, TreeTerm, A>) {
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
    fn peek(&self) -> Option<&ArenaRef<'a, TreeTerm, A>> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.items.len() - 1])
        }
    }
}
impl<'heap, A: ArenaAllocator> Iterator for TreeIterator<'heap, A> {
    type Item = TermPointer;
    fn next(&mut self) -> Option<Self::Item> {
        let (cursor, child_pointer, child) = match self.stack.peek() {
            None => None,
            Some(tree_term) => {
                let (child_pointer, child) = match self.stack.cursor {
                    TreeIteratorCursor::Left => (tree_term.as_value().left, tree_term.left()),
                    TreeIteratorCursor::Right => (tree_term.as_value().right, tree_term.right()),
                };
                Some((self.stack.cursor, child_pointer, child))
            }
        }?;
        match cursor {
            TreeIteratorCursor::Left => match child {
                // If this is the null leaf marker, we need to shift from the left to the right branch
                None => {
                    self.stack.cursor = TreeIteratorCursor::Right;
                    return self.next();
                }
                Some(child) => {
                    // Determine whether the current item is itself a cell which needs to be traversed deeper
                    match child.as_value().type_id() {
                        // If so, push the cell to the stack and repeat the iteration with the updated stack
                        TermTypeDiscriminants::Tree => {
                            let tree_term =
                                ArenaRef::<TypedTerm<TreeTerm>, _>::new(self.arena, child_pointer);
                            self.stack.push(tree_term.as_inner());
                            return self.next();
                        }
                        // Otherwise emit the value and shift from the left to the right branch
                        _ => {
                            self.stack.cursor = TreeIteratorCursor::Right;
                            Some(child_pointer)
                        }
                    }
                }
            },
            TreeIteratorCursor::Right => match child {
                // If this is the null leaf marker, we are at the end of a list and need to pop the stack
                None => {
                    // Pop the current entry from the stack and repeat the iteration with the updated stack
                    self.stack.pop();
                    return self.next();
                }
                Some(child) => {
                    // Determine whether the current item is itself a cell which needs to be traversed deeper
                    match child.as_value().type_id() {
                        // If so, push the cell to the stack and repeat the iteration with the updated stack
                        TermTypeDiscriminants::Tree => {
                            let tree_term =
                                ArenaRef::<TypedTerm<TreeTerm>, _>::new(self.arena, child_pointer);
                            self.stack.push(tree_term.as_inner());
                            return self.next();
                        }
                        _ => {
                            self.stack.pop();
                            Some(child_pointer)
                        }
                    }
                }
            },
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TreeIteratorCursor {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn tree() {
        assert_eq!(
            TermType::Tree(TreeTerm {
                left: TermPointer(12345),
                right: TermPointer(67890),
                length: 54321
            })
            .as_bytes(),
            [TermTypeDiscriminants::Tree as u32, 12345, 67890, 54321],
        );
    }
}
