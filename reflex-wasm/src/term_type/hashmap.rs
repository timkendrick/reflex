// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TermType,
    Array, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HashmapTerm {
    pub num_entries: u32,
    pub buckets: Array<HashmapBucket>,
}
impl TermSize for HashmapTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<HashmapBucket>>()
            + self.buckets.size()
    }
}
impl TermHash for HashmapTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        self.buckets
            .iter()
            .filter(|bucket| !bucket.key.is_uninitialized())
            .fold(
                hasher.hash(&self.num_entries, allocator),
                |hasher, bucket| hasher.hash(bucket, allocator),
            )
    }
}
impl HashmapTerm {
    pub fn allocate(
        entries: impl IntoIterator<
            Item = (TermPointer, TermPointer),
            IntoIter = impl ExactSizeIterator<Item = (TermPointer, TermPointer)>,
        >,
        allocator: &mut impl TermAllocator,
    ) -> TermPointer {
        let entries = entries.into_iter();
        let num_entries = entries.len();
        let capacity = HashmapTerm::default_capacity(num_entries);
        let term = Term::new(
            TermType::Hashmap(Self {
                num_entries: num_entries as u32,
                buckets: Default::default(),
            }),
            allocator,
        );
        let term_size = term.size();
        let instance = allocator.allocate(term);
        let list =
            instance.offset((term_size - std::mem::size_of::<Array<HashmapBucket>>()) as u32);
        let empty_buckets = (0..capacity).map(|_| HashmapBucket {
            key: TermPointer::uninitialized(),
            value: TermPointer::uninitialized(),
        });
        Array::<HashmapBucket>::extend(list, empty_buckets, allocator);
        for (key, value) in entries {
            let hash = TermHasher::default()
                .hash::<Term, _>(allocator.get(key), allocator)
                .finish();
            let mut bucket_index = (u32::from(hash) as usize) % capacity;
            while !TermPointer::is_uninitialized(*allocator.get::<TermPointer>(Array::<
                HashmapBucket,
            >::get_item_offset(
                list,
                bucket_index,
            ))) {
                bucket_index = (bucket_index + 1) % capacity;
            }
            let bucket_offset = Array::<HashmapBucket>::get_item_offset(list, bucket_index);
            *allocator.get_mut::<HashmapBucket>(bucket_offset) = HashmapBucket { key, value };
        }
        let hash = {
            allocator
                .get::<Term>(instance)
                .hash(Default::default(), allocator)
                .finish()
        };
        allocator.get_mut::<Term>(instance).set_hash(hash);
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
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for HashmapBucket {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        if self.key.is_uninitialized() {
            return hasher;
        } else {
            hasher
                .hash(&self.key, allocator)
                .hash(&self.value, allocator)
        }
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
