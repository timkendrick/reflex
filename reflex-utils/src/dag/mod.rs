// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::{
        hash_map::{Entry, RandomState},
        HashMap, HashSet,
    },
    hash::{BuildHasher, Hash},
};

use nohash_hasher::BuildNoHashHasher;

pub mod reporter;

pub trait DagVisitor<K, V> {
    fn visit_node(self, key: &K, value: &V) -> Self;
}

pub trait DagReporter<K, V>: DagVisitor<K, V>
where
    Self: Sized,
{
    #[allow(unused_variables)]
    fn add_node(self, key: &K, value: &V) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn remove_node(self, key: K, value: V) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn add_edge(self, from: &K, to: &K) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn remove_edge(self, from: &K, to: &K) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn add_root(self, key: &K) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn remove_root(self, key: &K) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn add_leaf(self, key: &K) -> Self {
        self
    }
    #[allow(unused_variables)]
    fn remove_leaf(self, key: &K) -> Self {
        self
    }
}

pub type IntDag<K, V> = Dag<K, V, BuildNoHashHasher<K>>;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DagEdgeDirection {
    Outbound,
    Inbound,
}

#[derive(Debug)]
pub struct Dag<K, V, S: BuildHasher + Default = RandomState> {
    nodes: HashMap<K, DagNode<K, V, S>>,
}

impl<K, V, S: BuildHasher + Default> Default for Dag<K, V, S> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

impl<K: Hash + PartialEq + Eq + Clone, V, S: BuildHasher + Default> Dag<K, V, S> {
    pub fn add_node<R: DagReporter<K, V>>(&mut self, key: K, value: V, reporter: R) -> R {
        match self.nodes.entry(key.clone()) {
            Entry::Occupied(_) => reporter,
            Entry::Vacant(entry) => {
                let node = entry.insert(DagNode::<K, V, S>::new(value));
                reporter
                    .add_root(&key)
                    .add_leaf(&key)
                    .add_node(&key, &node.value)
            }
        }
    }
    pub fn add_edge<R: DagReporter<K, V>>(&mut self, from: K, to: K, reporter: R) -> R {
        match (self.nodes.get(&from), self.nodes.get(&to)) {
            (None, _) | (_, None) => reporter,
            (Some(from_node), Some(_to_node)) if from_node.edges.contains(&to) => reporter,
            _ => {
                let reporter = if let Some(from_node) = self.nodes.get_mut(&from) {
                    if from_node.edges.insert(to.clone()) && from_node.edges.len() == 1 {
                        reporter.remove_leaf(&from)
                    } else {
                        reporter
                    }
                } else {
                    reporter
                };
                let reporter = if let Some(to_node) = self.nodes.get_mut(&to) {
                    if to_node.inbound_edges.insert(from.clone())
                        && to_node.inbound_edges.len() == 1
                    {
                        reporter.remove_root(&to)
                    } else {
                        reporter
                    }
                } else {
                    reporter
                };
                reporter.add_edge(&from, &to)
            }
        }
    }
    pub fn remove_node<R: DagReporter<K, V>>(&mut self, key: &K, reporter: R) -> R {
        match self.nodes.remove_entry(key) {
            None => reporter,
            Some((key, node)) => {
                let DagNode {
                    value,
                    edges,
                    inbound_edges,
                    marked: _,
                } = node;
                // Remove all inbound edges from adjoining nodes to this node
                let reporter = if inbound_edges.is_empty() {
                    reporter.remove_root(&key)
                } else {
                    inbound_edges.into_iter().fold(reporter, {
                        |reporter, from| {
                            if let Some((to, remaining_edges)) =
                                self.nodes.get_mut(&from).and_then(|from_node| {
                                    from_node
                                        .edges
                                        .take(&key)
                                        .map(|key| (key, from_node.edges.len()))
                                })
                            {
                                let reporter = reporter.remove_edge(&from, &to);
                                let reporter = if remaining_edges == 0 {
                                    reporter.add_leaf(&from)
                                } else {
                                    reporter
                                };
                                reporter
                            } else {
                                reporter
                            }
                        }
                    })
                };
                // Remove all outbound edges from this node to adjoining nodes
                let reporter = if edges.is_empty() {
                    reporter.remove_leaf(&key)
                } else {
                    edges
                        .into_iter()
                        .fold(reporter, |reporter, to| reporter.remove_edge(&key, &to))
                };
                reporter.remove_node(key, value)
            }
        }
    }
    pub fn remove_edge<R: DagReporter<K, V>>(&mut self, from: &K, to: &K, reporter: R) -> R {
        match (self.nodes.get(&from), self.nodes.get(&to)) {
            (None, _) | (_, None) => reporter,
            _ => {
                let removed_edge = if let Some(from_node) = self.nodes.get_mut(from) {
                    from_node
                        .edges
                        .take(to)
                        .map(|key| (key, from_node.edges.len()))
                } else {
                    None
                };
                let removed_reversed_edge = if let Some(to_node) = self.nodes.get_mut(to) {
                    to_node
                        .inbound_edges
                        .take(from)
                        .map(|key| (key, to_node.inbound_edges.len()))
                } else {
                    None
                };
                match (removed_edge, removed_reversed_edge) {
                    (Some((to, remaining_edges)), Some((from, remaining_inbound_edges))) => {
                        let reporter = reporter.remove_edge(&from, &to);
                        let reporter = if remaining_inbound_edges == 0 {
                            reporter.add_root(&to)
                        } else {
                            reporter
                        };
                        let reporter = if remaining_edges == 0 {
                            reporter.add_leaf(&from)
                        } else {
                            reporter
                        };
                        reporter
                    }
                    _ => reporter,
                }
            }
        }
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.nodes.get(key).map(|node| &node.value)
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn walk(&self, root: &K, direction: DagEdgeDirection) -> DagIter<'_, K, V, S> {
        DagIter::new(
            self,
            direction,
            self.nodes.get_key_value(root).map(|(key, _)| key),
        )
    }
    pub fn visit<R: DagVisitor<K, V>>(
        &mut self,
        roots: impl IntoIterator<Item = K>,
        direction: DagEdgeDirection,
        reporter: R,
    ) -> R {
        // Iterate over all unique nodes that are reachable from the provided roots
        let mut reporter = reporter;
        let mut queue = Vec::<K>::from_iter(roots.into_iter());
        while let Some(key) = queue.pop() {
            if let Some(node) = self.nodes.get_mut(&key) {
                // If this node has already been visited, move onto the next one
                if node.is_marked() {
                    continue;
                }
                // Mark the node as visited
                node.mark();
                // Push the node's children onto the queue for proecssing
                let children = match direction {
                    DagEdgeDirection::Outbound => &node.edges,
                    DagEdgeDirection::Inbound => &node.inbound_edges,
                };
                for edge in children.iter() {
                    queue.push(edge.clone());
                }
                // Invoke the visitor method
                reporter = reporter.visit_node(&key, &node.value);
            }
        }
        for node in self.nodes.values_mut() {
            node.unmark();
        }
        reporter
    }
    pub fn clear<R: DagReporter<K, V>>(&mut self, reporter: R) -> R {
        let reporter =
            std::mem::take(&mut self.nodes)
                .into_iter()
                .fold(reporter, |reporter, (key, node)| {
                    let DagNode {
                        value,
                        edges,
                        inbound_edges,
                        marked: _,
                    } = node;
                    let reporter = reporter.visit_node(&key, &value);
                    let reporter = if edges.is_empty() {
                        reporter.remove_leaf(&key)
                    } else {
                        reporter
                    };
                    let reporter = if inbound_edges.is_empty() {
                        reporter.remove_root(&key)
                    } else {
                        reporter
                    };
                    let reporter = edges
                        .into_iter()
                        .fold(reporter, |reporter, to| reporter.remove_edge(&key, &to));
                    reporter.remove_node(key, value)
                });
        reporter
    }
    pub fn remove_deep<R: DagReporter<K, V>>(
        &mut self,
        roots: impl IntoIterator<Item = K>,
        direction: DagEdgeDirection,
        reporter: R,
    ) -> R {
        let mut queue = Vec::<K>::from_iter(roots);
        let mut reporter = reporter;
        while let Some(key) = queue.pop() {
            if let Some(node) = self.nodes.get(&key) {
                // Push the node's children onto the queue for proecssing
                let children = match direction {
                    DagEdgeDirection::Inbound => &node.inbound_edges,
                    DagEdgeDirection::Outbound => &node.edges,
                };
                for edge in children.iter().cloned() {
                    queue.push(edge);
                }
                // Remove the node (including all its inbound/outbound edges)
                reporter = reporter.visit_node(&key, &node.value);
                reporter = self.remove_node(&key, reporter)
            }
        }
        reporter
    }
    pub fn retain_deep<R: DagReporter<K, V>>(
        &mut self,
        roots: impl IntoIterator<Item = K>,
        direction: DagEdgeDirection,
        reporter: R,
    ) -> R {
        // Mark all nodes that are reachable from the provided roots
        let reporter = self.mark_deep(roots, direction, reporter);
        // Iterate over the full set of nodes to determine the subset of nodes that were not visited
        let removed_keys = self
            .nodes
            .iter_mut()
            .filter_map(|(key, node)| {
                let marked = node.unmark();
                if !marked {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        // Remove the subset of nodes (including all their inbound/outbound edges)
        removed_keys
            .iter()
            .fold(reporter, |reporter, key| self.remove_node(key, reporter))
    }
    fn mark_deep<R: DagReporter<K, V>>(
        &mut self,
        roots: impl IntoIterator<Item = K>,
        direction: DagEdgeDirection,
        reporter: R,
    ) -> R {
        // Mark all nodes that are reachable from the provided roots
        let mut queue = Vec::<K>::from_iter(roots);
        let mut reporter = reporter;
        while let Some(key) = queue.pop() {
            if let Some(node) = self.nodes.get_mut(&key) {
                // If this node has already been processed, move onto the next one
                if node.is_marked() {
                    continue;
                }
                reporter = reporter.visit_node(&key, &node.value);
                // Mark the node as reachable
                node.mark();
                // Push the node's children onto the queue for proecssing
                let children = match direction {
                    DagEdgeDirection::Inbound => &node.inbound_edges,
                    DagEdgeDirection::Outbound => &node.edges,
                };
                for edge in children.iter() {
                    queue.push(edge.clone());
                }
            }
        }
        reporter
    }
}

pub struct DagIter<'a, K, V, S: BuildHasher + Default> {
    graph: &'a Dag<K, V, S>,
    direction: DagEdgeDirection,
    queue: Vec<&'a K>,
}

impl<'a, K, V, S: BuildHasher + Default> DagIter<'a, K, V, S> {
    fn new(
        graph: &'a Dag<K, V, S>,
        direction: DagEdgeDirection,
        roots: impl IntoIterator<Item = &'a K>,
    ) -> Self {
        Self {
            graph,
            direction,
            queue: roots.into_iter().collect(),
        }
    }
}

impl<'a, K: Hash + PartialEq + Eq + Clone, V, S: BuildHasher + Default> Iterator
    for DagIter<'a, K, V, S>
{
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.queue.pop()?;
        match self.graph.nodes.get(key) {
            None => self.next(),
            Some(node) => {
                let children = match self.direction {
                    DagEdgeDirection::Outbound => &node.edges,
                    DagEdgeDirection::Inbound => &node.inbound_edges,
                };
                self.queue.extend(children.iter());
                Some((key, &node.value))
            }
        }
    }
}

#[derive(Debug)]
struct DagNode<K, V, S: BuildHasher + Default> {
    value: V,
    edges: HashSet<K, S>,
    inbound_edges: HashSet<K, S>,
    marked: bool,
}

impl<K: Hash + PartialEq + Eq + Clone, V, S: BuildHasher + Default> DagNode<K, V, S> {
    fn new(value: V) -> Self {
        Self {
            value,
            edges: Default::default(),
            inbound_edges: Default::default(),
            marked: Default::default(),
        }
    }
    fn is_marked(&self) -> bool {
        self.marked
    }
    fn mark(&mut self) {
        self.marked = true;
    }
    fn unmark(&mut self) -> bool {
        std::mem::replace(&mut self.marked, false)
    }
}
