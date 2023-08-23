// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use super::{DagReporter, DagVisitor};

#[derive(Default, Debug)]
pub struct NoopDagReporter;

impl<K, V> DagVisitor<K, V> for NoopDagReporter {
    fn visit_node(self, _key: &K, _value: &V) -> Self {
        self
    }
}

impl<K, V> DagReporter<K, V> for NoopDagReporter {}

#[derive(Default, Debug)]
pub struct CounterDagReporter {
    pub visited_nodes: usize,
    pub added_nodes: usize,
    pub removed_nodes: usize,
    pub added_edges: usize,
    pub removed_edges: usize,
    pub added_roots: usize,
    pub removed_roots: usize,
    pub added_leaves: usize,
    pub removed_leaves: usize,
}

impl<K, V> DagVisitor<K, V> for CounterDagReporter {
    fn visit_node(mut self, _key: &K, _value: &V) -> Self {
        self.visited_nodes += 1;
        self
    }
}

impl<K, V> DagReporter<K, V> for CounterDagReporter {
    fn add_node(mut self, _key: &K, _value: &V) -> Self {
        self.added_nodes += 1;
        self
    }
    fn remove_node(mut self, _key: K, _value: V) -> Self {
        self.removed_nodes += 1;
        self
    }
    fn add_edge(mut self, _from: &K, _to: &K) -> Self {
        self.added_edges += 1;
        self
    }
    fn remove_edge(mut self, _from: &K, _to: &K) -> Self {
        self.removed_edges += 1;
        self
    }
    fn add_root(mut self, _key: &K) -> Self {
        self.added_roots += 1;
        self
    }
    fn remove_root(mut self, _key: &K) -> Self {
        self.removed_roots += 1;
        self
    }
    fn add_leaf(mut self, _key: &K) -> Self {
        self.added_leaves += 1;
        self
    }
    fn remove_leaf(mut self, _key: &K) -> Self {
        self.removed_leaves += 1;
        self
    }
}

#[derive(Default, Debug)]
pub struct CollectorDagReporter<K, V> {
    pub visited_nodes: Vec<K>,
    pub added_nodes: Vec<K>,
    pub removed_nodes: Vec<(K, V)>,
    pub added_edges: Vec<(K, K)>,
    pub removed_edges: Vec<(K, K)>,
    pub added_roots: Vec<K>,
    pub removed_roots: Vec<K>,
    pub added_leaves: Vec<K>,
    pub removed_leaves: Vec<K>,
}

impl<K, V> DagVisitor<K, V> for CollectorDagReporter<K, V>
where
    K: Clone,
{
    fn visit_node(mut self, key: &K, _value: &V) -> Self {
        self.visited_nodes.push(key.clone());
        self
    }
}

impl<K, V> DagReporter<K, V> for CollectorDagReporter<K, V>
where
    K: Clone,
{
    fn add_node(mut self, key: &K, _value: &V) -> Self {
        self.added_nodes.push(key.clone());
        self
    }
    fn remove_node(mut self, key: K, value: V) -> Self {
        self.removed_nodes.push((key, value));
        self
    }
    fn add_edge(mut self, from: &K, to: &K) -> Self {
        self.added_edges.push((from.clone(), to.clone()));
        self
    }
    fn remove_edge(mut self, from: &K, to: &K) -> Self {
        self.removed_edges.push((from.clone(), to.clone()));
        self
    }
    fn add_root(mut self, key: &K) -> Self {
        self.added_roots.push(key.clone());
        self
    }
    fn remove_root(mut self, key: &K) -> Self {
        self.removed_roots.push(key.clone());
        self
    }
    fn add_leaf(mut self, key: &K) -> Self {
        self.added_leaves.push(key.clone());
        self
    }
    fn remove_leaf(mut self, key: &K) -> Self {
        self.removed_leaves.push(key.clone());
        self
    }
}
