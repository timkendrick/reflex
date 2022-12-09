// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::marker::PhantomData;

use reflex::core::{Expression, HashmapTermType, RefType};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermType,
    ArenaRef, Array, ArrayIter, IntoArenaRefIterator, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashmapTerm {
    pub num_entries: u32,
    pub buckets: Array<HashmapBucket>,
}
impl TermSize for HashmapTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<HashmapBucket>>()
            + self.buckets.size_of()
    }
}
impl TermHash for HashmapTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        self.buckets
            .iter()
            .filter(|bucket| !bucket.key.is_uninitialized())
            .fold(hasher.hash(&self.num_entries, arena), |hasher, bucket| {
                hasher.hash(bucket, arena)
            })
    }
}
impl HashmapTerm {
    pub fn allocate(
        entries: impl IntoIterator<
            Item = (TermPointer, TermPointer),
            IntoIter = impl ExactSizeIterator<Item = (TermPointer, TermPointer)>,
        >,
        arena: &mut impl ArenaAllocator,
    ) -> TermPointer {
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
            key: TermPointer::uninitialized(),
            value: TermPointer::uninitialized(),
        });
        Array::<HashmapBucket>::extend(list, empty_buckets, arena);
        for (key, value) in entries {
            let hash = TermHasher::default()
                .hash::<Term, _>(arena.get(key), arena)
                .finish();
            let mut bucket_index = (u32::from(hash) as usize) % capacity;
            while !TermPointer::is_uninitialized(
                *arena.get::<TermPointer>(Array::<HashmapBucket>::get_item_offset(
                    list,
                    bucket_index,
                )),
            ) {
                bucket_index = (bucket_index + 1) % capacity;
            }
            let bucket_offset = Array::<HashmapBucket>::get_item_offset(list, bucket_index);
            *arena.get_mut::<HashmapBucket>(bucket_offset) = HashmapBucket { key, value };
        }
        let hash = {
            arena
                .get::<Term>(instance)
                .hash(Default::default(), arena)
                .finish()
        };
        arena.get_mut::<Term>(instance).set_hash(hash);
        instance
    }
    fn default_capacity(num_entries: usize) -> usize {
        (num_entries * 4) / 3
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashmapBucket {
    pub key: TermPointer,
    pub value: TermPointer,
}
impl TermSize for HashmapBucket {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashmapBucket {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        if self.key.is_uninitialized() {
            return hasher;
        } else {
            hasher.hash(&self.key, arena).hash(&self.value, arena)
        }
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, HashmapTerm, A> {
    fn num_entries(&self) -> u32 {
        self.as_deref().num_entries
    }
    fn buckets(&self) -> ArenaRef<'heap, Array<HashmapBucket>, A> {
        ArenaRef::new(self.arena, &self.as_deref().buckets)
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> HashmapTermType<T> for ArenaRef<'heap, HashmapTerm, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
{
    type KeysIterator<'a> = IntoArenaRefIterator<'a, Term, A, HashmapBucketKeysIterator<'a, ArrayIter<'a, HashmapBucket>>>
    where
        T: 'a,
        Self: 'a;
    type ValuesIterator<'a> = IntoArenaRefIterator<'a, Term, A, HashmapBucketValuesIterator<'a, ArrayIter<'a, HashmapBucket>>>
    where
        T: 'a,
        Self: 'a;
    fn get<'a>(&'a self, key: &T) -> Option<T::Ref<'a, T>>
    where
        T: 'a,
    {
        self.keys().zip(self.values()).filter_map({
            let key_id = key.id();
            move |(bucket_key, bucket_value)| {
                if bucket_key.as_deref().id() == key_id {
                    Some(bucket_value.into())
                } else {
                    None
                }
            }
        })
    }
    fn keys<'a>(&'a self) -> Self::KeysIterator<'a>
    where
        T: 'a,
    {
        IntoArenaRefIterator::new(
            self.arena,
            HashmapBucketKeysIterator::new(self.num_entries(), self.buckets().as_deref().iter()),
        )
    }
    fn values<'a>(&'a self) -> Self::ValuesIterator<'a>
    where
        T: 'a,
    {
        IntoArenaRefIterator::new(
            self.arena,
            HashmapBucketValuesIterator::new(self.num_entries(), self.buckets().as_deref().iter()),
        )
    }
}

pub struct HashmapBucketKeysIterator<'a, TInner: Iterator<Item = &'a HashmapBucket>> {
    buckets: HashmapBucketsIterator<'a, TInner>,
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> HashmapBucketKeysIterator<'a, TInner> {
    fn new(num_entries: usize, buckets: TInner) -> Self {
        Self {
            buckets: HashmapBucketsIterator::new(num_entries, buckets),
        }
    }
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> Iterator
    for HashmapBucketKeysIterator<'a, TInner>
{
    type Item = TermPointer;
    fn next(&mut self) -> Option<Self::Item> {
        self.buckets.next().map(|bucket| bucket.key)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.buckets.size_hint()
    }
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> ExactSizeIterator
    for HashmapBucketKeysIterator<'a, TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.buckets.len()
    }
}

pub struct HashmapBucketValuesIterator<'a, TInner: Iterator<Item = &'a HashmapBucket>> {
    buckets: HashmapBucketsIterator<'a, TInner>,
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> HashmapBucketValuesIterator<'a, TInner> {
    fn new(num_entries: usize, buckets: TInner) -> Self {
        Self {
            buckets: HashmapBucketsIterator::new(num_entries, buckets),
        }
    }
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> Iterator
    for HashmapBucketValuesIterator<'a, TInner>
{
    type Item = TermPointer;
    fn next(&mut self) -> Option<Self::Item> {
        self.buckets.next().map(|bucket| bucket.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.buckets.size_hint()
    }
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> ExactSizeIterator
    for HashmapBucketValuesIterator<'a, TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.buckets.len()
    }
}

struct HashmapBucketsIterator<'a, TInner: Iterator<Item = &'a HashmapBucket>> {
    buckets: TInner,
    num_entries: usize,
    index: usize,
    _item: PhantomData<&'a HashmapBucket>,
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> HashmapBucketsIterator<'a, TInner> {
    fn new(num_entries: usize, buckets: TInner) -> Self {
        Self {
            buckets,
            num_entries,
            index: 0,
            _item: Default::default(),
        }
    }
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> Iterator
    for HashmapBucketsIterator<'a, TInner>
{
    type Item = &'a HashmapBucket;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.num_entries {
            None
        } else {
            let index = self.index;
            self.index += 1;
            let mut item = self.buckets.next();
            while let Some(HashmapBucket {
                key: TermPointer::uninitialized(),
                value: TermPointer::uninitialized(),
            }) = item
            {
                item = self.buckets.next()
            }
            item
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl<'a, TInner: Iterator<Item = &'a HashmapBucket>> ExactSizeIterator
    for HashmapBucketsIterator<'a, TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.num_entries - self.index
    }
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use crate::{
        allocator::VecAllocator,
        term_type::{IntTerm, TermType, TermTypeDiscriminants},
    };

    use super::*;

    #[test]
    fn hashmap() {
        assert_eq!(
            TermType::Hashmap(HashmapTerm {
                num_entries: 12345,
                buckets: Default::default(),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Hashmap as u32, 12345, 0, 0],
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
            let result = allocator.get::<Term>(instance).as_bytes();
            // TODO: Test term hashing
            let _hash = result[0];
            let discriminant = result[1];
            let num_entries = result[2];
            let buckets_length = result[3];
            let buckets_capacity = result[4];
            let buckets_data = &result[5..];
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
