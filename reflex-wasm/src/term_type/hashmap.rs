// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once, marker::PhantomData};

use reflex::{
    core::{
        ArgType, DependencyList, Expression, GraphNode, HashmapTermType, HashsetTermType, NodeId,
        RefType, SerializeJson, StackOffset,
    },
    hash::HashId,
};
use reflex_utils::MapIntoIterator;
use serde_json::Value as JsonValue;

use crate::{
    allocator::{Arena, ArenaAllocator},
    compiler::{
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompilerOptions, CompilerResult, CompilerStack, CompilerState,
        ConstValue, Internable, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermType, TypedTerm, WasmExpression},
    ArenaArrayIter, ArenaPointer, ArenaRef, Array, IntoArenaRefIter, PointerIter, Term,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashmapTerm {
    pub num_entries: u32,
    pub buckets: Array<HashmapBucket>,
}

pub type HashmapTermPointerIter = std::vec::IntoIter<ArenaPointer>;

impl<A: Arena> PointerIter for ArenaRef<HashmapTerm, A> {
    type Iter<'a> = HashmapTermPointerIter
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        let items_pointer = self.inner_pointer(|term| &term.buckets.items);
        self.read_value(|term| {
            term.buckets
                .items()
                .enumerate()
                .filter(|(_, bucket)| !bucket.key.is_uninitialized())
                .map(|(index, _)| {
                    let item_offset = index * std::mem::size_of::<HashmapBucket>();
                    items_pointer.offset(item_offset as u32)
                })
                .flat_map(|bucket_pointer| {
                    let key_pointer = self
                        .arena
                        .inner_pointer::<HashmapBucket, _>(bucket_pointer, |bucket| &bucket.key);
                    let value_pointer = self
                        .arena
                        .inner_pointer::<HashmapBucket, _>(bucket_pointer, |bucket| &bucket.value);
                    [key_pointer, value_pointer]
                })
                .collect::<Vec<_>>()
        })
        .into_iter()
    }
}

impl TermSize for HashmapTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<HashmapBucket>>()
            + self.buckets.size_of()
    }
}
impl TermHash for HashmapTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let hasher = hasher.hash(&self.num_entries, arena);
        self.buckets
            .items()
            .filter(|bucket| !bucket.key.is_uninitialized())
            .take(self.num_entries as usize)
            .fold(hasher, |hasher, item| item.hash(hasher, arena))
    }
}
impl HashmapTerm {
    pub fn allocate(
        entries: impl IntoIterator<
            Item = (ArenaPointer, ArenaPointer),
            IntoIter = impl ExactSizeIterator<Item = (ArenaPointer, ArenaPointer)>,
        >,
        arena: &mut impl ArenaAllocator,
    ) -> ArenaPointer {
        let entries = entries.into_iter();
        let num_entries = entries.len();
        let capacity = HashmapTerm::default_capacity(num_entries);
        let term = Term::new(
            TermType::Hashmap(Self {
                num_entries: num_entries as u32,
                buckets: Default::default(),
            }),
            arena,
        );
        let term_size = term.size_of();
        let instance = arena.allocate(term);
        let list =
            instance.offset((term_size - std::mem::size_of::<Array<HashmapBucket>>()) as u32);
        let empty_buckets = (0..capacity).map(|_| HashmapBucket {
            key: ArenaPointer::uninitialized(),
            value: ArenaPointer::uninitialized(),
        });
        Array::<HashmapBucket>::extend(list, empty_buckets, arena);
        for (key, value) in entries {
            let key_hash = arena.read_value::<Term, HashId>(key, |term| u64::from(term.id()));
            let mut bucket_index = (u64::from(key_hash) % capacity as u64) as usize;
            while !ArenaPointer::is_uninitialized(arena.read_value::<HashmapBucket, ArenaPointer>(
                Array::<HashmapBucket>::get_item_offset(list, bucket_index),
                |bucket| bucket.key,
            )) {
                bucket_index = (bucket_index + 1) % capacity;
            }
            let bucket_offset = Array::<HashmapBucket>::get_item_offset(list, bucket_index);
            arena.write::<HashmapBucket>(bucket_offset, HashmapBucket { key, value });
        }
        let hash = arena.read_value::<Term, _>(instance, |term| {
            TermHasher::default().hash(term, arena).finish()
        });
        arena.write::<u64>(Term::get_hash_pointer(instance), u64::from(hash));
        instance
    }
    fn default_capacity(num_entries: usize) -> usize {
        (num_entries * 4) / 3
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashmapBucket {
    pub key: ArenaPointer,
    pub value: ArenaPointer,
}
impl TermSize for HashmapBucket {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashmapBucket {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        if self.key.is_uninitialized() {
            return hasher;
        } else {
            let key_hash = arena.read_value::<Term, _>(self.key, |term| term.id());
            let value_hash = arena.read_value::<Term, _>(self.value, |term| term.id());
            hasher.hash(&key_hash, arena).hash(&value_hash, arena)
        }
    }
}

impl<A: Arena + Clone> ArenaRef<HashmapTerm, A> {
    pub(crate) fn buckets_pointer(&self) -> ArenaPointer {
        self.inner_pointer(|value| &value.buckets)
    }
    pub fn capacity(&self) -> usize {
        self.read_value(|term| term.buckets.capacity as usize)
    }
    pub fn num_entries(&self) -> usize {
        self.read_value(|term| term.num_entries as usize)
    }
    pub fn buckets(&self) -> ArenaRef<Array<HashmapBucket>, A> {
        ArenaRef::<Array<HashmapBucket>, _>::new(self.arena.clone(), self.buckets_pointer())
    }
    pub fn entries(&self) -> HashmapBucketsIterator<ArenaArrayIter<'_, HashmapBucket, A>> {
        HashmapBucketsIterator::new(
            self.num_entries(),
            Array::<HashmapBucket>::iter(self.buckets_pointer(), &self.arena),
        )
    }
    pub fn keys(
        &self,
    ) -> IntoArenaRefIter<
        '_,
        Term,
        A,
        HashmapBucketKeysIterator<ArenaArrayIter<'_, HashmapBucket, A>>,
    > {
        IntoArenaRefIter::new(&self.arena, HashmapBucketKeysIterator::new(self.entries()))
    }
    pub fn values(
        &self,
    ) -> IntoArenaRefIter<
        '_,
        Term,
        A,
        HashmapBucketValuesIterator<ArenaArrayIter<'_, HashmapBucket, A>>,
    > {
        IntoArenaRefIter::new(
            &self.arena,
            HashmapBucketValuesIterator::new(self.entries()),
        )
    }
}

impl<A: Arena + Clone> ArenaRef<TypedTerm<HashmapTerm>, A> {
    fn buckets_pointer(&self) -> ArenaPointer {
        self.inner_pointer(|term| &term.get_inner().buckets)
    }
}

impl<A: Arena + Clone> HashmapTermType<WasmExpression<A>> for ArenaRef<HashmapTerm, A> {
    type KeysIterator<'a> = MapIntoIterator<
        IntoArenaRefIter<'a, Term, A, HashmapBucketKeysIterator<ArenaArrayIter<'a, HashmapBucket, A>>>,
        ArenaRef<Term, A>,
        <WasmExpression<A> as Expression>::ExpressionRef<'a>
    >
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    type ValuesIterator<'a> = MapIntoIterator<
        IntoArenaRefIter<'a, Term, A, HashmapBucketValuesIterator<ArenaArrayIter<'a, HashmapBucket, A>>>,
        ArenaRef<Term, A>,
        <WasmExpression<A> as Expression>::ExpressionRef<'a>
    >
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    fn get<'a>(
        &'a self,
        key: &WasmExpression<A>,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        // TODO: implement `HashMapTermType::get()` using hashmap lookup
        self.entries()
            .map(|bucket| {
                (
                    ArenaRef::<Term, _>::new(self.arena.clone(), bucket.key),
                    ArenaRef::<Term, _>::new(self.arena.clone(), bucket.value),
                )
            })
            .filter_map({
                let key_id = key.id();
                move |(bucket_key, bucket_value)| {
                    if bucket_key.as_deref().id() == key_id {
                        Some(bucket_value.into())
                    } else {
                        None
                    }
                }
            })
            .next()
    }
    fn keys<'a>(&'a self) -> Self::KeysIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        MapIntoIterator::new(self.keys())
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        MapIntoIterator::new(self.values())
    }
}

impl<A: Arena + Clone> HashmapTermType<WasmExpression<A>> for ArenaRef<TypedTerm<HashmapTerm>, A> {
    type KeysIterator<'a> = <ArenaRef<HashmapTerm, A> as HashmapTermType<WasmExpression<A>>>::KeysIterator<'a>
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    type ValuesIterator<'a> = <ArenaRef<HashmapTerm, A> as HashmapTermType<WasmExpression<A>>>::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    fn get<'a>(
        &'a self,
        key: &WasmExpression<A>,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<HashmapTerm, A> as HashmapTermType<WasmExpression<A>>>::get(&self.as_inner(), key)
    }
    fn keys<'a>(&'a self) -> Self::KeysIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        let inner = self.as_inner();
        let buckets = HashmapBucketsIterator::new(
            inner.num_entries(),
            Array::<HashmapBucket>::iter(self.buckets_pointer(), &self.arena),
        );
        MapIntoIterator::new(IntoArenaRefIter::new(
            &self.arena,
            HashmapBucketKeysIterator::new(buckets),
        ))
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        let inner = self.as_inner();
        let buckets = HashmapBucketsIterator::new(
            inner.num_entries(),
            Array::<HashmapBucket>::iter(self.buckets_pointer(), &self.arena),
        );
        MapIntoIterator::new(IntoArenaRefIter::new(
            &self.arena,
            HashmapBucketValuesIterator::new(buckets),
        ))
    }
}

impl<A: Arena + Clone> HashsetTermType<WasmExpression<A>> for ArenaRef<TypedTerm<HashmapTerm>, A> {
    type ValuesIterator<'a> = <ArenaRef<HashmapTerm, A> as HashmapTermType<WasmExpression<A>>>::KeysIterator<'a>
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    fn contains(&self, value: &WasmExpression<A>) -> bool {
        <Self as HashsetTermType<WasmExpression<A>>>::values(self).any({
            let value_id = value.read_value(|term| term.id());
            move |item| {
                if item.read_value(|term| term.id()) == value_id {
                    true
                } else {
                    false
                }
            }
        })
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        let inner = self.as_inner();
        let buckets = HashmapBucketsIterator::new(
            inner.num_entries(),
            Array::<HashmapBucket>::iter(self.buckets_pointer(), &self.arena),
        );
        MapIntoIterator::new(IntoArenaRefIter::new(
            &self.arena,
            // Hashset values are stored as keys, with values set to null
            HashmapBucketKeysIterator::new(buckets),
        ))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<HashmapTerm, A> {
    fn size(&self) -> usize {
        1 + self.keys().map(|item| item.size()).sum::<usize>()
            + self.values().map(|item| item.size()).sum::<usize>()
    }
    fn capture_depth(&self) -> StackOffset {
        self.keys()
            .map(|item| item.capture_depth())
            .max()
            .unwrap_or(0)
            .max(
                self.values()
                    .map(|item| item.capture_depth())
                    .max()
                    .unwrap_or(0),
            )
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.keys()
            .flat_map(|item| item.free_variables())
            .chain(self.values().flat_map(|item| item.free_variables()))
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.keys()
            .map(|item| item.count_variable_usages(offset))
            .sum::<usize>()
            + self
                .values()
                .map(|item| item.count_variable_usages(offset))
                .sum::<usize>()
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.keys()
                .flat_map(|item| item.dynamic_dependencies(deep))
                .chain(
                    self.values()
                        .flat_map(|item| item.dynamic_dependencies(deep)),
                )
                .collect()
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.keys().any(|item| item.has_dynamic_dependencies(deep))
                || self
                    .values()
                    .any(|item| item.has_dynamic_dependencies(deep))
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.keys().all(|item| item.is_atomic()) && self.values().all(|item| item.is_atomic())
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<HashmapTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<HashmapTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        // This assumes that hashmaps with the same size and hash are almost certainly identical
        // TODO: Clarify PartialEq implementations for container terms
        self.num_entries() == other.num_entries()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<HashmapTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<HashmapTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<HashmapTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_displayed_entries = 10;
        let keys = self.keys();
        let values = self.values();
        let entries = keys.zip(values);
        let num_entries = entries.len();
        write!(
            f,
            "HashMap({})",
            if num_entries <= max_displayed_entries {
                entries
                    .map(|(key, value)| format!("{} => {}", key, value))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                entries
                    .take(max_displayed_entries - 1)
                    .map(|(key, value)| format!("{} => {}", key, value))
                    .chain(once(format!(
                        "...{} more entries",
                        num_entries - (max_displayed_entries - 1)
                    )))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}

pub struct HashmapBucketKeysIterator<TInner: Iterator<Item = HashmapBucket>> {
    buckets: HashmapBucketsIterator<TInner>,
}
impl<TInner: Iterator<Item = HashmapBucket>> HashmapBucketKeysIterator<TInner> {
    pub fn new(buckets: HashmapBucketsIterator<TInner>) -> Self {
        Self { buckets }
    }
}
impl<TInner: Iterator<Item = HashmapBucket>> Iterator for HashmapBucketKeysIterator<TInner> {
    type Item = ArenaPointer;
    fn next(&mut self) -> Option<Self::Item> {
        self.buckets.next().map(|bucket| bucket.key)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.buckets.size_hint()
    }
}
impl<TInner: Iterator<Item = HashmapBucket>> ExactSizeIterator for HashmapBucketKeysIterator<TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.buckets.len()
    }
}

pub struct HashmapBucketValuesIterator<TInner: Iterator<Item = HashmapBucket>> {
    buckets: HashmapBucketsIterator<TInner>,
}
impl<TInner: Iterator<Item = HashmapBucket>> HashmapBucketValuesIterator<TInner> {
    pub fn new(buckets: HashmapBucketsIterator<TInner>) -> Self {
        Self { buckets }
    }
}
impl<TInner: Iterator<Item = HashmapBucket>> Iterator for HashmapBucketValuesIterator<TInner> {
    type Item = ArenaPointer;
    fn next(&mut self) -> Option<Self::Item> {
        self.buckets.next().map(|bucket| bucket.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.buckets.size_hint()
    }
}
impl<TInner: Iterator<Item = HashmapBucket>> ExactSizeIterator
    for HashmapBucketValuesIterator<TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.buckets.len()
    }
}

pub struct HashmapBucketsIterator<TInner: Iterator<Item = HashmapBucket>> {
    /// Iterator containing both empty and non-empty source buckets
    buckets: TInner,
    /// Number of non-empty buckets (i.e. actual length of the iterator)
    num_entries: usize,
    index: usize,
    _item: PhantomData<HashmapBucket>,
}
impl<TInner: Iterator<Item = HashmapBucket>> HashmapBucketsIterator<TInner> {
    pub(crate) fn new(num_entries: usize, buckets: TInner) -> Self {
        Self {
            buckets,
            num_entries,
            index: 0,
            _item: PhantomData,
        }
    }
}
impl<TInner: Iterator<Item = HashmapBucket>> Iterator for HashmapBucketsIterator<TInner> {
    type Item = HashmapBucket;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.num_entries {
            None
        } else {
            self.index += 1;
            let mut item = self.buckets.next();
            loop {
                match item {
                    // If this is an empty bucket, skip to the next bucket
                    Some(HashmapBucket { key, value: _ })
                        if key == ArenaPointer::uninitialized() =>
                    {
                        item = self.buckets.next();
                    }
                    _ => break,
                }
            }
            item
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.num_entries.saturating_sub(self.index);
        (len, Some(len))
    }
}
impl<TInner: Iterator<Item = HashmapBucket>> ExactSizeIterator for HashmapBucketsIterator<TInner> where
    TInner: ExactSizeIterator
{
}

impl<A: Arena + Clone> Internable for ArenaRef<HashmapTerm, A> {
    fn should_intern(&self, eager: ArgType) -> bool {
        self.keys().all(|term| term.should_intern(eager))
            && self.values().all(|term| term.should_intern(eager))
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<HashmapTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let capacity = self.capacity();
        let keys = self.keys();
        let values = self.values();
        let block = CompiledBlockBuilder::new(stack);
        // Push the capacity onto the stack
        // => [capacity]
        let block = block.push(instruction::core::Const {
            value: ConstValue::U32(capacity as u32),
        });
        // Allocate the hashmap term
        // => [HashmapTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::AllocateHashmap,
        });
        // Assign the hashmap entries
        let block = keys.zip(values).fold(
            Result::<_, CompilerError<_>>::Ok(block),
            |block, (key, value)| {
                let block = block?;
                // Duplicate the hashmap term pointer onto the stack
                // => [HashmapTerm, HashmapTerm]
                let block = block.push(instruction::core::Duplicate {
                    value_type: ValueType::HeapPointer,
                });
                // Yield the entry's key onto the stack
                // => [HashmapTerm, HashmapTerm, Term]
                let block = block.append_inner(|stack| key.compile(stack, state, options))?;
                // Yield the entry's value onto the stack
                // => [HashmapTerm, HashmapTerm, Term, Term]
                let block = block.append_inner(|stack| value.compile(stack, state, options))?;
                // Insert the entry into the hashmap
                // => [HashmapTerm]
                let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                    target: RuntimeBuiltin::InsertHashmapEntry,
                });
                Ok(block)
            },
        )?;
        // Initialize the hashmap term
        // => [HashmapTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::InitHashmap,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use crate::{
        allocator::VecAllocator,
        term_type::{IntTerm, TermType, TermTypeDiscriminants},
        utils::chunks_to_u64,
    };

    use super::*;

    #[test]
    fn hashmap() {
        assert_eq!(
            TermType::Hashmap(HashmapTerm {
                num_entries: 0x54321,
                buckets: Default::default(),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Hashmap as u32, 0x54321, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let entries = (3..6)
                .map(|index| {
                    (
                        allocator
                            .allocate(Term::new(TermType::Int(IntTerm::from(index)), &allocator)),
                        allocator
                            .allocate(Term::new(TermType::Int(IntTerm::from(-index)), &allocator)),
                    )
                })
                .collect::<Vec<_>>();
            let instance = HashmapTerm::allocate(entries.clone(), &mut allocator);
            let result = allocator.get_ref::<Term>(instance).as_bytes();
            // TODO: Test term hashing
            let _hash = chunks_to_u64([result[0], result[1]]);
            let discriminant = result[2];
            let num_entries = result[3];
            let buckets_length = result[4];
            let buckets_capacity = result[5];
            let buckets_data = &result[6..];
            let expected_capacity = HashmapTerm::default_capacity(entries.len());
            assert_eq!(discriminant, TermTypeDiscriminants::Hashmap as u32);
            assert_eq!(num_entries, entries.len() as u32);
            assert_eq!(buckets_length, expected_capacity as u32);
            assert_eq!(buckets_capacity, expected_capacity as u32);
            assert_eq!(buckets_data.len(), expected_capacity * 2);
            assert_eq!(
                {
                    let mut buckets = buckets_data
                        .chunks(2)
                        .map(|entry| (entry[0], entry[1]))
                        .collect::<Vec<_>>();
                    buckets.sort();
                    buckets
                },
                {
                    let mut buckets = entries
                        .iter()
                        .map(|(key, value)| (u32::from(*key), u32::from(*value)))
                        .chain(repeat((0, 0)).take(expected_capacity - entries.len()))
                        .collect::<Vec<_>>();
                    buckets.sort();
                    buckets
                }
            );
        }
    }
}
