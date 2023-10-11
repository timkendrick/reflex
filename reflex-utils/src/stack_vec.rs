// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

use std::iter::once;

/// List that reserves space on the stack to hold up to the first `N` items. Instances containing more items will be stored on the heap.
///
/// This can be useful when working with temporary lists of small but unknown size in scenarios where it is important to minimize heap allocations.
///
/// Once heap storage has been allocated for the list due to insufficient reserved space, the list contents will remain on the heap regardless of how many items are subsequently removed from the list.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StackVec<const N: usize, T> {
    items: Option<StackVecItems<N, T>>,
}

impl<const N: usize, T> Default for StackVec<N, T>
where
    StackVecItems<N, T>: Default,
{
    fn default() -> Self {
        Self {
            items: Some(StackVecItems::<N, T>::default()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum StackVecItems<const N: usize, T> {
    Stack {
        items: [Option<T>; N],
        length: usize,
    },
    Heap(Vec<T>),
}

impl<const N: usize, T> Default for StackVecItems<N, T>
where
    [Option<T>; N]: Default,
{
    fn default() -> Self {
        Self::Stack {
            items: Default::default(),
            length: 0,
        }
    }
}

impl<const N: usize, T> StackVec<N, T> {
    pub fn push(&mut self, value: T) {
        self.items = Some(match self.items.take() {
            None => unreachable!(),
            Some(items) => match items {
                StackVecItems::Stack {
                    mut items,
                    length: existing_length,
                } => {
                    if existing_length < items.len() {
                        items[existing_length] = Some(value);
                        StackVecItems::Stack {
                            items,
                            length: existing_length + 1,
                        }
                    } else {
                        StackVecItems::Heap(
                            items.into_iter().flatten().chain(once(value)).collect(),
                        )
                    }
                }
                StackVecItems::Heap(mut existing_items) => {
                    existing_items.push(value);
                    StackVecItems::Heap(existing_items)
                }
            },
        })
    }
    pub fn pop(&mut self) -> Option<T> {
        let (result, remaining) = match self.items.take() {
            None => unreachable!(),
            Some(items) => match items {
                StackVecItems::Stack {
                    mut items,
                    length: existing_length,
                } => match existing_length {
                    0 => (None, StackVecItems::Stack { items, length: 0 }),
                    existing_length => (
                        items[existing_length - 1].take(),
                        StackVecItems::Stack {
                            items,
                            length: existing_length - 1,
                        },
                    ),
                },
                StackVecItems::Heap(mut remaining_items) => {
                    let result = remaining_items.pop();
                    let remaining = StackVecItems::Heap(remaining_items);
                    (result, remaining)
                }
            },
        };
        self.items = Some(remaining);
        result
    }
    pub fn len(&self) -> usize {
        match &self.items {
            None => unreachable!(),
            Some(items) => match items {
                StackVecItems::Stack { items: _, length } => *length,
                StackVecItems::Heap(items) => items.len(),
            },
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const N: usize, T> Extend<T> for StackVec<N, T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        match &mut self.items {
            Some(StackVecItems::Heap(existing_items)) => {
                existing_items.extend(iter);
            }
            _ => {
                if let Some(value) = iter.next() {
                    self.push(value);
                    self.extend(iter);
                }
            }
        }
    }
}

impl<const N: usize, T> FromIterator<T> for StackVec<N, T>
where
    StackVecItems<N, T>: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut instance = Self::default();
        instance.extend(iter);
        instance
    }
}

impl<const N: usize, T> IntoIterator for StackVec<N, T> {
    type Item = T;

    type IntoIter = StackVecIntoIter<N, T>;

    fn into_iter(self) -> Self::IntoIter {
        StackVecIntoIter::from(self)
    }
}

pub enum StackVecIntoIter<const N: usize, T> {
    Stack(std::iter::Flatten<std::iter::Take<std::array::IntoIter<Option<T>, N>>>),
    Heap(std::vec::IntoIter<T>),
}

impl<const N: usize, T> From<StackVec<N, T>> for StackVecIntoIter<N, T> {
    fn from(value: StackVec<N, T>) -> Self {
        match value.items {
            None => unreachable!(),
            Some(items) => match items {
                StackVecItems::Stack { items, length } => {
                    StackVecIntoIter::Stack(items.into_iter().take(length).flatten())
                }
                StackVecItems::Heap(items) => StackVecIntoIter::Heap(items.into_iter()),
            },
        }
    }
}

impl<const N: usize, T> Iterator for StackVecIntoIter<N, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Stack(items) => items.next(),
            Self::Heap(items) => items.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let instance = StackVec::<3, usize>::default();
        let actual = instance.into_iter().collect::<Vec<_>>();
        let expected = Vec::<usize>::new();
        assert_eq!(expected, actual);
    }

    #[test]
    fn push() {
        {
            let mut instance = StackVec::<3, usize>::default();
            instance.push(3);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [Some(3), None, None],
                length: 1,
            });
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::default();
            instance.push(3);
            instance.push(4);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [Some(3), Some(4), None],
                length: 2,
            });
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::default();
            instance.push(3);
            instance.push(4);
            instance.push(5);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [Some(3), Some(4), Some(5)],
                length: 3,
            });
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::default();
            instance.push(3);
            instance.push(4);
            instance.push(5);
            instance.push(6);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Heap(vec![3, 4, 5, 6]));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn pop() {
        {
            let mut instance = StackVec::<3, usize>::from_iter([]);
            {
                let actual = instance.pop();
                let expected = None;
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [None, None, None],
                    length: 0,
                });
                assert_eq!(expected, actual);
            }
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3]);
            {
                let actual = instance.pop();
                let expected = Some(3);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [None, None, None],
                    length: 0,
                });
                assert_eq!(expected, actual);
            }
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4]);
            {
                let actual = instance.pop();
                let expected = Some(4);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [Some(3), None, None],
                    length: 1,
                });
                assert_eq!(expected, actual);
            }
            {
                let actual = instance.pop();
                let expected = Some(3);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [None, None, None],
                    length: 0,
                });
                assert_eq!(expected, actual);
            }
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5]);
            {
                let actual = instance.pop();
                let expected = Some(5);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [Some(3), Some(4), None],
                    length: 2,
                });
                assert_eq!(expected, actual);
            }
            {
                let actual = instance.pop();
                let expected = Some(4);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [Some(3), None, None],
                    length: 1,
                });
                assert_eq!(expected, actual);
            }
            {
                let actual = instance.pop();
                let expected = Some(3);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Stack {
                    items: [None, None, None],
                    length: 0,
                });
                assert_eq!(expected, actual);
            }
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            {
                let actual = instance.pop();
                let expected = Some(6);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Heap(vec![3, 4, 5]));
                assert_eq!(expected, actual);
            }
            {
                let actual = instance.pop();
                let expected = Some(5);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Heap(vec![3, 4]));
                assert_eq!(expected, actual);
            }
            {
                let actual = instance.pop();
                let expected = Some(4);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Heap(vec![3]));
                assert_eq!(expected, actual);
            }
            {
                let actual = instance.pop();
                let expected = Some(3);
                assert_eq!(expected, actual);
                let actual = &instance.items;
                let expected = &Some(StackVecItems::Heap(vec![]));
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn len() {
        {
            let instance = StackVec::<3, usize>::from_iter([]);
            let actual = instance.len();
            let expected = 0;
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3]);
            let actual = instance.len();
            let expected = 1;
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4]);
            let actual = instance.len();
            let expected = 2;
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4, 5]);
            let actual = instance.len();
            let expected = 3;
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let actual = instance.len();
            let expected = 4;
            assert_eq!(expected, actual);
            {
                let _ = instance.pop();
                let actual = instance.len();
                let expected = 3;
                assert_eq!(expected, actual);
            }
            {
                let _ = instance.pop();
                let actual = instance.len();
                let expected = 2;
                assert_eq!(expected, actual);
            }
            {
                let _ = instance.pop();
                let actual = instance.len();
                let expected = 1;
                assert_eq!(expected, actual);
            }
            {
                let _ = instance.pop();
                let actual = instance.len();
                let expected = 0;
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn is_empty() {
        {
            let instance = StackVec::<3, usize>::from_iter([]);
            let actual = instance.is_empty();
            let expected = true;
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3]);
            let actual = instance.is_empty();
            let expected = false;
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4]);
            let actual = instance.is_empty();
            let expected = false;
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4, 5]);
            let actual = instance.is_empty();
            let expected = false;
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let actual = instance.is_empty();
            let expected = false;
            assert_eq!(expected, actual);
            {
                let _ = instance.pop();
                let actual = instance.is_empty();
                let expected = false;
                assert_eq!(expected, actual);
            }
            {
                let _ = instance.pop();
                let actual = instance.is_empty();
                let expected = false;
                assert_eq!(expected, actual);
            }
            {
                let _ = instance.pop();
                let actual = instance.is_empty();
                let expected = false;
                assert_eq!(expected, actual);
            }
            {
                let _ = instance.pop();
                let actual = instance.is_empty();
                let expected = true;
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn from_iter() {
        {
            let instance = StackVec::<3, usize>::from_iter([]);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [None, None, None],
                length: 0,
            });
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3]);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [Some(3), None, None],
                length: 1,
            });
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4]);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [Some(3), Some(4), None],
                length: 2,
            });
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4, 5]);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Stack {
                items: [Some(3), Some(4), Some(5)],
                length: 3,
            });
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let actual = &instance.items;
            let expected = &Some(StackVecItems::Heap(vec![3, 4, 5, 6]));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn into_iter() {
        {
            let instance = StackVec::<3, usize>::from_iter([]);
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = Vec::<usize>::new();
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3]);
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3];
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4]);
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3, 4];
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4, 5]);
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3, 4, 5];
            assert_eq!(expected, actual);
        }
        {
            let instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3, 4, 5, 6];
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let _ = instance.pop();
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3, 4, 5];
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let _ = instance.pop();
            let _ = instance.pop();
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3, 4];
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let _ = instance.pop();
            let _ = instance.pop();
            let _ = instance.pop();
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = vec![3];
            assert_eq!(expected, actual);
        }
        {
            let mut instance = StackVec::<3, usize>::from_iter([3, 4, 5, 6]);
            let _ = instance.pop();
            let _ = instance.pop();
            let _ = instance.pop();
            let _ = instance.pop();
            let actual = instance.into_iter().collect::<Vec<_>>();
            let expected = Vec::<usize>::new();
            assert_eq!(expected, actual);
        }
    }
}
