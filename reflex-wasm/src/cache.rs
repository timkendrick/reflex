// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex_macros::PointerIter;

use crate::{
    allocator::Arena,
    term_type::{TreeTerm, TypedTerm},
    utils::{chunks_to_u64, u64_to_chunks},
    ArenaPointer, ArenaRef, Array, Term,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EvaluationCache {
    pub num_entries: u32,
    pub buckets: Array<EvaluationCacheBucket>,
}

impl<A: Arena + Clone> ArenaRef<EvaluationCache, A> {
    pub fn num_entries(&self) -> usize {
        self.read_value(|cache| cache.num_entries) as usize
    }
    pub fn num_buckets(&self) -> usize {
        self.read_value(|cache| cache.buckets.len())
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct EvaluationCacheBucket {
    pub key: [u32; 2],
    pub value: ArenaPointer,
    pub dependencies: ArenaPointer,
}

impl EvaluationCacheBucket {
    pub fn new(key: u64, value: ArenaPointer, dependencies: ArenaPointer) -> Self {
        Self {
            key: u64_to_chunks(key),
            value,
            dependencies,
        }
    }
    pub fn uninitialized() -> Self {
        Self {
            key: u64_to_chunks(0x0000000000000000),
            value: ArenaPointer::uninitialized(),
            dependencies: ArenaPointer::uninitialized(),
        }
    }
}

impl<A: Arena + Clone> ArenaRef<EvaluationCacheBucket, A> {
    pub fn key(&self) -> u64 {
        self.read_value(|bucket| chunks_to_u64(bucket.key))
    }
    pub fn value(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|bucket| bucket.value))
    }
    pub fn dependencies(&self) -> Option<ArenaRef<TypedTerm<TreeTerm>, A>> {
        let dependencies_pointer = self.read_value(|bucket| bucket.dependencies.as_non_null());
        dependencies_pointer
            .map(|pointer| ArenaRef::<TypedTerm<TreeTerm>, _>::new(self.arena.clone(), pointer))
    }
}
