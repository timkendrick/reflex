// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::marker::PhantomData;

use reflex::{
    core::{ConditionListType, Expression, RefType},
    hash::HashId,
};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermTypeDiscriminants,
    ArenaRef, Term, TermPointer, TypedTerm,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TreeTerm {
    pub left: TermPointer,
    pub right: TermPointer,
}
impl TermSize for TreeTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TreeTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.left, arena).hash(&self.right, arena)
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ConditionListType<T>
    for ArenaRef<'heap, TypedTerm<TreeTerm>, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    type Iterator<'a> = TreeIterator<'a, T, A>
    where
        T::Signal<T>: 'a,
        T: 'a,
        Self: 'a;
    fn id(&self) -> HashId {
        self.as_deref().id()
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        T::Signal<T>: 'a,
        T: 'a,
    {
        TreeIterator::new(self)
    }
}

struct TreeIterator<'heap, T: Expression + 'heap, A: ArenaAllocator> {
    stack: TreeIteratorStack<'heap, A>,
    arena: &'heap A,
    _expression: PhantomData<T>,
}
impl<'a, T: Expression, A: ArenaAllocator> TreeIterator<'a, T, A> {
    fn new(root: ArenaRef<'a, TypedTerm<TreeTerm>, A>) -> Self {
        Self {
            arena: root.arena,
            stack: TreeIteratorStack {
                cursor: TreeIteratorCursor::Left,
                items: vec![root],
            },
            _expression: PhantomData,
        }
    }
}
struct TreeIteratorStack<'a, A: ArenaAllocator> {
    cursor: TreeIteratorCursor,
    items: Vec<ArenaRef<'a, TypedTerm<TreeTerm>, A>>,
}
impl<'a, A: ArenaAllocator> TreeIteratorStack<'a, A> {
    fn push(&mut self, item: ArenaRef<'a, TypedTerm<TreeTerm>, A>) {
        match self.cursor {
            TreeIteratorCursor::Left => {
                // If we were processing the left branch, create a new stack entry so that we can return later to process the right branch
                self.items.push(item);
            }
            TreeIteratorCursor::Right => {
                // If we were processing the right branch, the cell is no longer needed so the current stack entry can be reused
                self.items[self.items.len() - 1] = item;
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
                self.stack.pop();
            }
        }
    }
    fn peek(&self) -> Option<&TreeTerm> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items[self.items.len() - 1].as_deref().get_inner())
        }
    }
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
impl<'heap, T: Expression + 'heap, A: ArenaAllocator> Iterator for TreeIterator<'heap, T, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    type Item = T::Ref<'heap, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let (cursor, child_pointer) = match self.stack.peek() {
            None => None,
            Some(tree_term) => {
                let child_pointer = match self.stack.cursor {
                    TreeIteratorCursor::Left => tree_term.left,
                    TreeIteratorCursor::Right => tree_term.right,
                };
                Some((self.stack.cursor, child_pointer))
            }
        }?;
        match cursor {
            TreeIteratorCursor::Left => {
                let left_pointer = child_pointer;
                // If this is the null leaf marker, we need to shift from the left to the right branch
                if left_pointer == TermPointer::null() {
                    self.stack.cursor = TreeIteratorCursor::Right;
                    return self.next();
                }
                // Determine whether the current item is itself a cell which needs to be traversed deeper
                let left_term = ArenaRef::<Term, A>::new(self.arena, self.arena.get(left_pointer));
                match left_term.as_deref().type_id() {
                    // If so, push the cell to the stack and repeat the iteration with the updated stack
                    TermTypeDiscriminants::Tree => {
                        let tree_term = ArenaRef::<TypedTerm<TreeTerm>, A>::new(
                            self.arena,
                            self.arena.get(left_pointer),
                        );
                        self.stack.push(tree_term);
                        return self.next();
                    }
                    // Otherwise emit the value and shift from the left to the right branch
                    _ => {
                        let item = <T::Ref<T> as From<ArenaRef<'heap, Term, A>>>::from(left_term);
                        self.stack.cursor = TreeIteratorCursor::Right;
                        Some(item)
                    }
                }
            }
            TreeIteratorCursor::Right => {
                let right_pointer = child_pointer;
                // If this is the null leaf marker, we are at the end of a list and need to pop the stack
                if right_pointer == TermPointer::null() {
                    // Pop the current entry from the stack and repeat the iteration with the updated stack
                    self.stack.pop();
                    return self.next();
                }
                // Determine whether the current item is itself a cell which needs to be traversed deeper
                let right_term =
                    ArenaRef::<Term, A>::new(self.arena, self.arena.get(right_pointer));
                match right_term.as_deref().type_id() {
                    // If so, push the cell to the stack and repeat the iteration with the updated stack
                    TermTypeDiscriminants::Tree => {
                        let tree_term = ArenaRef::<TypedTerm<TreeTerm>, A>::new(
                            self.arena,
                            self.arena.get(right_pointer),
                        );
                        self.stack.push(tree_term);
                        return self.next();
                    }
                    _ => {
                        let item = <T::Ref<T> as From<ArenaRef<'heap, Term, A>>>::from(right_term);
                        self.stack.pop();
                        Some(item)
                    }
                }
            }
        }
    }
}
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
            })
            .as_bytes(),
            [TermTypeDiscriminants::Tree as u32, 12345, 67890],
        );
    }
}
