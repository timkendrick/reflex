// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

use std::marker::PhantomData;

use crate::StackVec;

/// Trait representing a data structure whose internal nodes can be traversed
pub trait Visitable<T> {
    /// Type of internal node
    type Children: Iterator<Item = T>;

    /// Retrieve a list of internal nodes
    fn children(&self) -> Self::Children;
}

/// Trait representing a visitor strategy that can be used to apply a stateless post-order algorithm to each node within a data structure
pub trait StatelessPostOrderVisitor {
    /// Return the result of applying the provided `algorithm` to the provided `node` and all its descendants
    fn apply<A, T>(&self, node: T, algorithm: &A) -> A::Result
    where
        A: StatelessPostOrderVisitorAlgorithm<T>,
        T: Visitable<T>;
}

/// Trait representing a visitor strategy that can be used to apply a stateful post-order algorithm to each node within a data structure
pub trait StatefulPostOrderVisitor {
    /// Return the result of applying the provided `algorithm` to the provided `node` and all its descendants
    fn apply_mut<A, T>(&self, node: T, algorithm: &A, state: &mut A::State) -> A::Result
    where
        A: StatefulPostOrderVisitorAlgorithm<T>,
        T: Visitable<T>;
}

/// Trait representing a post-order traversal algorithm that does not require shared state
pub trait StatelessPostOrderVisitorAlgorithm<T> {
    /// Algorithm return value type
    type Result;

    /// Algorithm implementation for a given node, where `child_results` contains a list of results
    /// of the algorithm having been applied to each of the node's children
    fn visit(&self, node: T, child_results: impl IntoIterator<Item = Self::Result>)
        -> Self::Result;
}

/// Trait representing a post-order traversal algorithm that requires shared state
pub trait StatefulPostOrderVisitorAlgorithm<T> {
    /// Algorithm return value type
    type Result;

    /// Algorithm state
    type State;

    /// Algorithm implementation for a given node, where `child_results` contains a list of results
    /// of the algorithm having been applied to each of the node's children
    fn visit_mut(
        &self,
        node: T,
        child_results: impl IntoIterator<Item = Self::Result>,
        state: &mut Self::State,
    ) -> Self::Result;
}

/// Structure that implements a depth-first post-order traversal algorithm that uses the call stack to store traversal state
///
/// This minimizes unnecessary heap allocations, however has the potential to trigger stack overflows when traversing large data structures
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct PostOrderStackVisitor;

impl StatelessPostOrderVisitor for PostOrderStackVisitor {
    fn apply<A, T>(&self, node: T, algorithm: &A) -> A::Result
    where
        A: StatelessPostOrderVisitorAlgorithm<T>,
        T: Visitable<T>,
    {
        let children = node.children();
        let child_results = children.map(|child| self.apply(child, algorithm));
        algorithm.visit(node, child_results)
    }
}

impl StatefulPostOrderVisitor for PostOrderStackVisitor {
    fn apply_mut<A, T>(&self, node: T, algorithm: &A, state: &mut A::State) -> A::Result
    where
        A: StatefulPostOrderVisitorAlgorithm<T>,
        T: Visitable<T>,
    {
        // Define the maximum number of child results that can be stored on the stack for each node
        // (any nodes with more than this number of children will store their child results on the heap)
        const NUM_STACK_ALLOCATED_CHILD_RESULTS: usize = 3;
        type ChildResultsList<T> = StackVec<NUM_STACK_ALLOCATED_CHILD_RESULTS, T>;
        let children = node.children();
        let child_results = children
            .map(|child| self.apply_mut(child, algorithm, state))
            // We need to collect the child results before processing the parent to avoid multiple simultaneous mutable references
            // (note that the StackVec is used to prevent unnecessary heap allocations for branches with small numbers of children)
            .collect::<ChildResultsList<_>>();
        algorithm.visit_mut(node, child_results, state)
    }
}

/// Structure that implements a depth-first post-order traversal algorithm that uses the heap to store traversal state
///
/// This implies unnecessary heap allocations when traversing small data structures, however avoids triggering stack overflows when traversing large data structures
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct PostOrderQueueVisitor;

impl StatelessPostOrderVisitor for PostOrderQueueVisitor {
    fn apply<A, T>(&self, node: T, algorithm: &A) -> A::Result
    where
        A: StatelessPostOrderVisitorAlgorithm<T>,
        T: Visitable<T>,
    {
        let (algorithm, mut state) =
            StatefulPostOrderVisitorAlgorithmWrapper::new_with_state(algorithm);
        process_post_order_queue(&algorithm, node, &mut state)
    }
}

impl StatefulPostOrderVisitor for PostOrderQueueVisitor {
    fn apply_mut<A, T>(&self, node: T, algorithm: &A, state: &mut A::State) -> A::Result
    where
        A: StatefulPostOrderVisitorAlgorithm<T>,
        T: Visitable<T>,
    {
        process_post_order_queue(algorithm, node, state)
    }
}

fn process_post_order_queue<A, T>(algorithm: &A, node: T, state: &mut A::State) -> A::Result
where
    A: StatefulPostOrderVisitorAlgorithm<T>,
    T: Visitable<T>,
{
    enum PostOrderQueueVisitorEntry<T> {
        /// Pre-processing phase: this node has been queued up for processing but has not yet been visited
        Pre(T),
        /// Post-processing phase: this node's children have been processed but the node itself needs processing
        Post(T, usize),
        /// Pending phase: this is used internally within the algorithm while calculating the number of children for a given node
        Pending,
    }
    let mut queue = Vec::<PostOrderQueueVisitorEntry<T>>::new();
    let mut stack = Vec::<A::Result>::new();
    queue.push(PostOrderQueueVisitorEntry::Pre(node));
    while let Some(item) = queue.pop() {
        match item {
            PostOrderQueueVisitorEntry::Pre(node) => {
                let children = node.children();
                // Insert a placeholder for post-processing the current node once its children have been processed
                let placeholder_index = queue.len();
                queue.push(PostOrderQueueVisitorEntry::Pending);
                // Push queue entries for all the children of the current node
                let children_index = placeholder_index + 1;
                queue.extend(children.map(PostOrderQueueVisitorEntry::Pre));
                let child_count = queue.len() - children_index;
                // Items are popped from the back of the queue one-by-one for processing, so we need to reverse the
                // child queue entries to ensure the child nodes are processed in the correct order
                {
                    let child_entries = &mut queue[children_index..(children_index + child_count)];
                    child_entries.reverse();
                }
                // Now that we know the number of children, we can overwrite the placeholder
                queue[placeholder_index] = PostOrderQueueVisitorEntry::Post(node, child_count);
            }
            PostOrderQueueVisitorEntry::Post(node, child_count) => {
                let child_results = stack.drain((stack.len() - child_count)..);
                let result = algorithm.visit_mut(node, child_results, state);
                stack.push(result);
            }
            PostOrderQueueVisitorEntry::Pending => unreachable!(),
        }
    }
    match stack.pop() {
        Some(result) => result,
        None => unreachable!(),
    }
}

/// Stateful wrapper for a stateless post-order traversal algorithm
struct StatefulPostOrderVisitorAlgorithmWrapper<'a, A: StatelessPostOrderVisitorAlgorithm<T>, T> {
    inner: &'a A,
    _node: PhantomData<T>,
}

impl<'a, A: StatelessPostOrderVisitorAlgorithm<T>, T>
    StatefulPostOrderVisitorAlgorithmWrapper<'a, A, T>
{
    /// Create a stateful version of the provided `inner` algorithm alongside a valid state object
    pub fn new_with_state(
        inner: &'a A,
    ) -> (Self, <Self as StatefulPostOrderVisitorAlgorithm<T>>::State) {
        (
            Self {
                inner,
                _node: PhantomData,
            },
            <<Self as StatefulPostOrderVisitorAlgorithm<T>>::State as Default>::default(),
        )
    }
}

impl<'a, A: StatelessPostOrderVisitorAlgorithm<T>, T> StatefulPostOrderVisitorAlgorithm<T>
    for StatefulPostOrderVisitorAlgorithmWrapper<'a, A, T>
{
    type Result = A::Result;

    type State = ();

    fn visit_mut(
        &self,
        node: T,
        child_results: impl IntoIterator<Item = Self::Result>,
        _state: &mut Self::State,
    ) -> Self::Result {
        self.inner.visit(node, child_results)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use super::*;

    struct Tree<T> {
        value: T,
        left: Option<Box<Self>>,
        right: Option<Box<Self>>,
    }

    impl<'a, T> Visitable<Self> for &'a Tree<T> {
        type Children = std::iter::Chain<std::option::IntoIter<Self>, std::option::IntoIter<Self>>;

        fn children(&self) -> Self::Children {
            let left = self.left.as_ref().map(std::ops::Deref::deref);
            let right = self.right.as_ref().map(std::ops::Deref::deref);
            left.into_iter().chain(right)
        }
    }

    struct SumBranchWeight;

    impl<'a> StatelessPostOrderVisitorAlgorithm<&'a Tree<usize>> for SumBranchWeight {
        type Result = usize;

        fn visit(
            &self,
            node: &'a Tree<usize>,
            child_results: impl IntoIterator<Item = Self::Result>,
        ) -> Self::Result {
            node.value + child_results.into_iter().sum::<usize>()
        }
    }

    struct FlattenDeep;

    impl<'a, T> StatelessPostOrderVisitorAlgorithm<&'a Tree<T>> for FlattenDeep {
        type Result = Vec<&'a T>;

        fn visit(
            &self,
            node: &'a Tree<T>,
            child_results: impl IntoIterator<Item = Self::Result>,
        ) -> Self::Result {
            once(&node.value)
                .chain(child_results.into_iter().flatten())
                .collect()
        }
    }

    #[derive(Default)]
    struct AccumulatedSumBranchWeight;

    impl<'a> StatefulPostOrderVisitorAlgorithm<&'a Tree<usize>> for AccumulatedSumBranchWeight {
        type Result = usize;

        type State = usize;

        fn visit_mut(
            &self,
            node: &'a Tree<usize>,
            child_results: impl IntoIterator<Item = Self::Result>,
            previous_total: &mut Self::State,
        ) -> Self::Result {
            let updated_total =
                *previous_total + node.value + child_results.into_iter().sum::<usize>();
            *previous_total = updated_total;
            updated_total
        }
    }

    #[test]
    fn post_order_stack_visitor() {
        let tree = Tree {
            value: 3,
            left: Some(Box::new(Tree {
                value: 4,
                left: Some(Box::new(Tree {
                    value: 5,
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Tree {
                    value: 6,
                    left: None,
                    right: None,
                })),
            })),
            right: Some(Box::new(Tree {
                value: 7,
                left: Some(Box::new(Tree {
                    value: 8,
                    left: None,
                    right: None,
                })),
                right: None,
            })),
        };
        let actual = PostOrderStackVisitor.apply(&tree, &SumBranchWeight);
        let expected = 3 + 4 + 5 + 6 + 7 + 8;
        assert_eq!(expected, actual);
        let actual = PostOrderStackVisitor.apply(&tree, &FlattenDeep);
        let expected = vec![&3, &4, &5, &6, &7, &8];
        assert_eq!(expected, actual);
        let actual =
            PostOrderStackVisitor.apply_mut(&tree, &AccumulatedSumBranchWeight::default(), &mut 0);
        let expected = {
            let mut acc = 0;
            let result_5 = acc + 5;
            acc = result_5;
            let result_6 = acc + 6;
            acc = result_6;
            let result_4 = acc + 4 + result_5 + result_6;
            acc = result_4;
            let result_8 = acc + 8;
            acc = result_8;
            let result_7 = acc + 7 + result_8;
            acc = result_7;
            let result_3 = acc + 3 + result_4 + result_7;
            acc = result_3;
            acc
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_order_queue_visitor() {
        let tree = Tree {
            value: 3,
            left: Some(Box::new(Tree {
                value: 4,
                left: Some(Box::new(Tree {
                    value: 5,
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Tree {
                    value: 6,
                    left: None,
                    right: None,
                })),
            })),
            right: Some(Box::new(Tree {
                value: 7,
                left: Some(Box::new(Tree {
                    value: 8,
                    left: None,
                    right: None,
                })),
                right: None,
            })),
        };
        let actual = PostOrderQueueVisitor.apply(&tree, &SumBranchWeight);
        let expected = 3 + 4 + 5 + 6 + 7 + 8;
        assert_eq!(expected, actual);
        let actual = PostOrderQueueVisitor.apply(&tree, &FlattenDeep);
        let expected = vec![&3, &4, &5, &6, &7, &8];
        assert_eq!(expected, actual);
        let actual = PostOrderQueueVisitor.apply_mut(
            &tree,
            &mut AccumulatedSumBranchWeight::default(),
            &mut 0,
        );
        let expected = {
            let mut acc = 0;
            let result_5 = acc + 5;
            acc = result_5;
            let result_6 = acc + 6;
            acc = result_6;
            let result_4 = acc + 4 + result_5 + result_6;
            acc = result_4;
            let result_8 = acc + 8;
            acc = result_8;
            let result_7 = acc + 7 + result_8;
            acc = result_7;
            let result_3 = acc + 3 + result_4 + result_7;
            acc = result_3;
            acc
        };
        assert_eq!(expected, actual);
    }
}
