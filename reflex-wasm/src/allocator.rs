// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    cell::{Ref, RefCell},
    iter::repeat,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{hash::TermSize, PointerIter, Term, TermPointer};

pub trait ArenaAllocator: Sized {
    type Slice<'a>: Deref<Target = [u8]>
    where
        Self: 'a;

    fn len(&self) -> usize;
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer;
    fn extend(&mut self, offset: TermPointer, size: usize);
    fn shrink(&mut self, offset: TermPointer, size: usize);
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T);
    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V;
    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer;
    fn as_slice<'a>(&'a self, offset: TermPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a;
}

impl<'heap, A: ArenaAllocator> ArenaAllocator for &'heap mut A {
    type Slice<'a> = A::Slice<'a>
    where
        Self: 'a;
    fn len(&self) -> usize {
        self.deref().len()
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        self.deref_mut().allocate(value)
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        self.deref_mut().extend(offset, size)
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        self.deref_mut().shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        self.deref_mut().write(offset, value)
    }
    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer {
        self.deref().inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: TermPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        self.deref().as_slice(offset, length)
    }
}

impl<A: for<'a> ArenaAllocator<Slice<'a> = &'a [u8]> + 'static> ArenaAllocator for Rc<RefCell<A>> {
    type Slice<'a> = Ref<'a, [u8]>
        where
            Self: 'a;
    fn len(&self) -> usize {
        self.deref().borrow().len()
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        self.deref().borrow_mut().allocate(value)
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        self.deref().borrow_mut().extend(offset, size)
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        self.deref().borrow_mut().shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        self.deref().borrow_mut().write(offset, value)
    }
    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().borrow().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer {
        self.deref()
            .borrow()
            .inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: TermPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        Ref::map(self.deref().borrow(), |arena| {
            arena.as_slice(offset, length)
        })
    }
}

pub struct VecAllocator(Vec<u32>);
impl VecAllocator {
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() % 4 != 0 {
            panic!("Invalid VecAllocator data alignment");
        }
        Self::from_vec_u32(
            data.chunks_exact(4)
                .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
                .collect(),
        )
    }
    pub fn from_vec_u32(data: Vec<u32>) -> Self {
        Self(data)
    }
    pub(crate) fn get_ref<T>(&self, offset: TermPointer) -> &T {
        let offset = u32::from(offset) as usize;
        if (offset % 4 != 0) || (offset >= <Self as ArenaAllocator>::len(self)) {
            panic!(
                "Invalid allocator offset: {} (length: {})",
                offset,
                <Self as ArenaAllocator>::len(self)
            );
        }
        let Self(data) = self;
        let item = &data[offset / 4];
        unsafe { std::mem::transmute::<&u32, &T>(item) }
    }
    pub(crate) fn get_mut<T>(&mut self, offset: TermPointer) -> &mut T {
        let offset = u32::from(offset) as usize;
        if (offset % 4 != 0) || (offset >= <Self as ArenaAllocator>::len(self)) {
            panic!(
                "Invalid allocator offset: {} (length: {})",
                offset,
                <Self as ArenaAllocator>::len(self)
            );
        }
        let Self(data) = self;
        let item = &mut data[offset / 4];
        unsafe { std::mem::transmute::<&mut u32, &mut T>(item) }
    }
    pub fn into_inner(self) -> Vec<u32> {
        let Self(data) = self;
        data
    }
    fn start_offset(&self) -> TermPointer {
        // Skip over the initial 4-byte length marker
        TermPointer::from(std::mem::size_of::<u32>() as u32)
    }
}
impl Default for VecAllocator {
    fn default() -> Self {
        // Start with an initial 4-byte length marker to match the WASM allocator representation
        Self(vec![0x00000004u32])
    }
}
impl ArenaAllocator for VecAllocator {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;
    fn len(&self) -> usize {
        let Self(data) = self;
        data.len() * 4
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        let offset = TermPointer(self.len() as u32);
        let static_size = pad_to_4_byte_offset(std::mem::size_of::<T>());
        let actual_size = pad_to_4_byte_offset(value.size_of());
        self.extend(offset, static_size.max(actual_size));
        self.write(offset, value);
        if actual_size < static_size {
            self.shrink(offset.offset(static_size as u32), static_size - actual_size);
        }
        TermPointer::from(offset)
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!(
                "Invalid allocator extend offset: {} (length: {})",
                u32::from(offset),
                self.len()
            );
        } else {
            let Self(data) = self;
            // Ensure all allocations are 32-bit aligned
            let padded_size = pad_to_4_byte_offset(size) as u32;
            // Extend the allocation with zero-filled bytes
            data.extend(repeat(0).take((padded_size as usize) / 4));
            // Update the length marker
            data[0] += padded_size;
        }
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!(
                "Invalid allocator shrink offset: {} (length: {})",
                u32::from(offset),
                self.len()
            );
        } else {
            let Self(data) = self;
            // Ensure all allocations are 32-bit aligned
            let padded_size = pad_to_4_byte_offset(size) as u32;
            // Truncate the allocation
            data.truncate((offset - padded_size) as usize / 4);
            // Update the length marker
            data[0] -= padded_size;
        }
    }
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        *self.get_mut::<T>(offset) = value
    }
    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V {
        selector(self.get_ref::<T>(offset))
    }
    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer {
        let target = self.get_ref::<T>(offset);
        let outer_pointer = target as *const T as usize;
        let inner_pointer = selector(target) as *const V as usize;
        offset.offset((inner_pointer - outer_pointer) as u32)
    }
    fn as_slice<'a>(&'a self, offset: TermPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        let offset = u32::from(offset) as usize;
        &self.as_bytes()[offset..(offset + length)]
    }
}
impl VecAllocator {
    pub fn as_words(&self) -> &[u32] {
        let Self(data) = self;
        data
    }
    pub fn as_bytes(&self) -> &[u8] {
        let Self(data) = self;
        unsafe {
            std::slice::from_raw_parts::<u8>(&data[0] as *const u32 as *const u8, data.len() * 4)
        }
    }
}
impl Into<Vec<u32>> for VecAllocator {
    fn into(self) -> Vec<u32> {
        let Self(data) = self;
        data
    }
}

// FIXME: Remove hard-coded assumption that VecAllocator always contains Term values
// e.g. impl<T: NodeId + TermSize> PointerIter for TypedVecAllocator<T>
impl PointerIter for VecAllocator {
    type Iter<'a> = ArenaIterator<'a, Self, Term>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        ArenaIterator::new(self, self.start_offset())
    }
}
impl PointerIter for Rc<RefCell<VecAllocator>> {
    type Iter<'a> = ArenaIterator<'a, Self, Term>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        ArenaIterator::new(self, self.deref().borrow().start_offset())
    }
}

fn pad_to_4_byte_offset(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        (((value - 1) / 4) + 1) * 4
    }
}

pub struct ArenaIterator<'a, A: ArenaAllocator, T: TermSize> {
    arena: &'a A,
    next_offset: TermPointer,
    _value: PhantomData<T>,
}

impl<'a, A: ArenaAllocator, T: TermSize> ArenaIterator<'a, A, T> {
    pub fn new(arena: &'a A, start_point: TermPointer) -> Self {
        ArenaIterator {
            arena,
            next_offset: start_point,
            _value: Default::default(),
        }
    }
}

impl<'a, A: ArenaAllocator, T: TermSize> Iterator for ArenaIterator<'a, A, T> {
    type Item = TermPointer;

    fn next(&mut self) -> Option<Self::Item> {
        if u32::from(self.next_offset) >= (self.arena.len() as u32) {
            None
        } else {
            let current_pointer = self.next_offset;

            // Determine the size of the current struct
            let value_size = self
                .arena
                .read_value::<T, _>(current_pointer, |term| term.size_of());

            // Increment the next offset by the size of the struct
            self.next_offset = current_pointer.offset(value_size as u32);

            // Return a pointer to the current struct
            Some(current_pointer)
        }
    }
}
