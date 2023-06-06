// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use allocator::TermAllocator;
use hash::{TermHash, TermHashState, TermHasher, TermSize};

pub mod allocator;
pub mod hash;
pub mod stdlib;
pub mod term_type;

use term_type::TermType;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Term {
    header: TermHeader,
    pub value: TermType,
}
impl Term {
    pub fn new(value: TermType, allocator: &impl TermAllocator) -> Self {
        Self {
            header: TermHeader {
                hash: value.hash(TermHasher::default(), allocator).finish(),
            },
            value,
        }
    }
    pub fn get_hash(&mut self) -> TermHashState {
        self.header.hash
    }
    pub fn get_value_pointer(term: TermPointer) -> TermPointer {
        term.offset(std::mem::size_of::<TermHeader>() as u32)
    }
    pub fn as_bytes(&self) -> &[u32] {
        let num_words = pad_to_4_byte_offset(self.size() as usize) / 4;
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u32, num_words) }
    }
    pub(crate) fn set_hash(&mut self, value: TermHashState) {
        self.header.hash = value;
    }
}
impl TermSize for Term {
    fn size(&self) -> usize {
        std::mem::size_of::<TermHashState>() + self.value.size()
    }
}
impl TermHash for Term {
    fn hash(&self, hasher: TermHasher, _allocator: &impl TermAllocator) -> TermHasher {
        hasher.write_hash(self.header.hash)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TermHeader {
    hash: TermHashState,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[repr(transparent)]
pub struct TermPointer(u32);
impl TermPointer {
    pub fn null() -> Self {
        Self(0xFFFFFFFF)
    }
    pub fn uninitialized() -> Self {
        Self(0)
    }
    pub fn is_null(self) -> bool {
        let Self(offset) = self;
        offset == 0xFFFFFFFF
    }
    pub fn is_uninitialized(self) -> bool {
        let Self(offset) = self;
        offset == 0
    }
    pub(crate) fn offset(self, offset: u32) -> Self {
        let Self(existing_offset) = self;
        Self(existing_offset + offset)
    }
}
impl From<TermPointer> for u32 {
    fn from(value: TermPointer) -> Self {
        let TermPointer(value) = value;
        value
    }
}
impl From<u32> for TermPointer {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl TermHash for TermPointer {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        let target: &Term = allocator.get(*self);
        target.hash(hasher, allocator)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Array<T> {
    capacity: u32,
    length: u32,
    items: [T; 0],
}
impl<T> Default for Array<T> {
    fn default() -> Self {
        Self {
            capacity: 0,
            length: 0,
            items: [],
        }
    }
}
impl<T> Array<T>
where
    T: Sized,
{
    pub fn extend(
        list: TermPointer,
        items: impl IntoIterator<Item = T, IntoIter = impl ExactSizeIterator<Item = T>>,
        allocator: &mut impl TermAllocator,
    ) {
        let items = items.into_iter();
        let num_items = items.len();
        let capacity = num_items as u32;
        let length = num_items as u32;
        let items_offset = list.offset(std::mem::size_of::<Array<T>>() as u32);
        *allocator.get_mut::<u32>(list) = capacity;
        *allocator.get_mut::<u32>(list.offset(std::mem::size_of::<u32>() as u32)) = length;
        allocator.extend(items_offset, num_items * std::mem::size_of::<T>());
        let array_offset = u32::from(items_offset);
        for (index, item) in items.enumerate() {
            *allocator.get_mut(TermPointer::from(
                array_offset + ((index * std::mem::size_of::<T>()) as u32),
            )) = item;
        }
    }
    pub fn len(&self) -> usize {
        self.length as usize
    }
    pub fn iter(&self) -> ArrayIter<T> {
        ArrayIter::new(self)
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(unsafe { self.get_unchecked(index) })
        } else {
            None
        }
    }
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        let offset = &self.items as *const [T; 0] as usize;
        let pointer = (offset + (index * 4)) as *const T;
        std::mem::transmute::<*const T, &T>(pointer)
    }
    pub fn get_item_offset(list: TermPointer, index: usize) -> TermPointer {
        list.offset((std::mem::size_of::<Array<T>>() + (index * std::mem::size_of::<T>())) as u32)
    }
}
impl<T> TermSize for Array<T>
where
    T: Sized,
{
    fn size(&self) -> usize {
        std::mem::size_of::<Self>() + ((self.capacity as usize) * std::mem::size_of::<T>())
    }
}
impl<T> TermHash for Array<T>
where
    T: Copy + TermHash,
{
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        self.iter()
            .fold(hasher.write_u32(self.length), |hasher, item| {
                item.hash(hasher, allocator)
            })
    }
}

pub struct ArrayIter<'a, T: Sized> {
    array: &'a Array<T>,
    length: usize,
    index: usize,
}
impl<'a, T: Sized> ArrayIter<'a, T> {
    fn new(items: &'a Array<T>) -> Self {
        Self {
            length: items.len(),
            array: items,
            index: 0,
        }
    }
}
impl<'a, T> Iterator for ArrayIter<'a, T>
where
    T: Sized,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            None
        } else {
            let index = self.index;
            self.index += 1;
            Some(unsafe { self.array.get_unchecked(index) })
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.length
    }
}
impl<'a, T> ExactSizeIterator for ArrayIter<'a, T>
where
    T: Sized,
{
    fn len(&self) -> usize {
        self.length - self.index
    }
}

pub fn pad_to_4_byte_offset(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        (((value - 1) / 4) + 1) * 4
    }
}
