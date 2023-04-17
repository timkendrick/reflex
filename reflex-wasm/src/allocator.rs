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

use crate::{hash::TermSize, ArenaPointer};

pub trait Arena {
    type Slice<'a>: Deref<Target = [u8]>
    where
        Self: 'a;

    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V;
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer;
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a;
}

impl<'slice> Arena for &'slice [u8] {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        selector(get_slice_ref::<T>(self, offset))
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        let target = get_slice_ref::<T>(self, offset);
        let outer_pointer = target as *const T as usize;
        let inner_pointer = selector(target) as *const V as usize;
        offset.offset((inner_pointer - outer_pointer) as u32)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        let offset = u32::from(offset) as usize;
        &self[offset..(offset + length)]
    }
}

fn get_slice_ref<T: Sized>(slice: &[u8], offset: ArenaPointer) -> &T {
    let index = u32::from(offset) as usize;
    let is_zero_sized_value = std::mem::size_of::<T>() == 0;
    let is_invalid_offset = match is_zero_sized_value {
        true => index > slice.len(),
        false => index >= slice.len(),
    };
    if is_invalid_offset {
        panic!(
            "Invalid allocator offset: {} (length: {})",
            index,
            slice.len()
        );
    }
    unsafe { std::mem::transmute::<&u8, &T>(slice.get_unchecked(index)) }
}

pub trait ArenaAllocator: Arena {
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer;
    fn extend(&mut self, offset: ArenaPointer, size: usize);
    fn shrink(&mut self, offset: ArenaPointer, size: usize);
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T);
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
    pub(crate) fn get_ref<T>(&self, offset: ArenaPointer) -> &T {
        let is_zero_sized_value = std::mem::size_of::<T>() == 0;
        let is_invalid_offset = match is_zero_sized_value {
            true => offset > self.end_offset(),
            false => offset >= self.end_offset(),
        };
        if (u32::from(offset) % 4 != 0) || is_invalid_offset {
            panic!(
                "Invalid allocator offset: {} (length: {})",
                u32::from(offset),
                u32::from(self.end_offset())
            );
        }
        let Self(data) = self;
        let words = data.as_slice();
        let index = (u32::from(offset) / 4) as usize;
        unsafe { std::mem::transmute::<&u32, &T>(words.get_unchecked(index)) }
    }
    pub(crate) fn get_mut<T>(&mut self, offset: ArenaPointer) -> &mut T {
        let is_zero_sized_value = std::mem::size_of::<T>() == 0;
        let is_invalid_offset = match is_zero_sized_value {
            true => offset > self.end_offset(),
            false => offset >= self.end_offset(),
        };
        if (u32::from(offset) % 4 != 0) || is_invalid_offset {
            panic!(
                "Invalid allocator offset: {} (length: {})",
                u32::from(offset),
                u32::from(self.end_offset())
            );
        }
        let Self(data) = self;
        let words = data.as_mut_slice();
        let index = (u32::from(offset) / 4) as usize;
        unsafe { std::mem::transmute::<&mut u32, &mut T>(words.get_unchecked_mut(index)) }
    }
    pub fn into_inner(self) -> Vec<u32> {
        let Self(data) = self;
        data
    }
    pub fn into_bytes(self) -> Vec<u8> {
        let Self(data) = self;
        data.into_iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }
    pub(crate) fn start_offset(&self) -> ArenaPointer {
        // Skip over the initial 4-byte length marker
        ArenaPointer::from(std::mem::size_of::<u32>() as u32)
    }
    pub(crate) fn end_offset(&self) -> ArenaPointer {
        let Self(data) = self;
        ArenaPointer::from(data[0])
    }
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

impl Default for VecAllocator {
    fn default() -> Self {
        // Start with an initial 4-byte length marker to match the WASM allocator representation
        Self(vec![0x00000004u32])
    }
}

impl Into<Vec<u32>> for VecAllocator {
    fn into(self) -> Vec<u32> {
        let Self(data) = self;
        data
    }
}

impl Arena for VecAllocator {
    type Slice<'a> = &'a [u8]
        where
            Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        selector(self.get_ref::<T>(offset))
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        let target = self.get_ref::<T>(offset);
        let outer_pointer = target as *const T as usize;
        let inner_pointer = selector(target) as *const V as usize;
        offset.offset((inner_pointer - outer_pointer) as u32)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
    {
        let offset = u32::from(offset) as usize;
        &self.as_bytes()[offset..(offset + length)]
    }
}

impl ArenaAllocator for VecAllocator {
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        let pointer = self.end_offset();
        let static_size = pad_to_4_byte_offset(std::mem::size_of::<T>());
        let actual_size = pad_to_4_byte_offset(value.size_of());
        self.extend(pointer, static_size.max(actual_size));
        self.write(pointer, value);
        if actual_size < static_size {
            self.shrink(
                pointer.offset(static_size as u32),
                static_size - actual_size,
            );
        }
        pointer
    }
    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        if offset != self.end_offset() {
            panic!(
                "Invalid allocator extend offset: {} (length: {})",
                u32::from(offset),
                u32::from(self.end_offset())
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
    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        if offset != self.end_offset() {
            panic!(
                "Invalid allocator shrink offset: {} (length: {})",
                u32::from(offset),
                u32::from(self.end_offset())
            );
        } else {
            let Self(data) = self;
            // Ensure all allocations are 32-bit aligned
            let padded_size = pad_to_4_byte_offset(size) as u32;
            // Truncate the allocation
            data.truncate((u32::from(offset) - padded_size) as usize / 4);
            // Update the length marker
            data[0] -= padded_size;
        }
    }
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        *self.get_mut::<T>(offset) = value
    }
}

// TODO: Abstract reference-wrapped arena types into blanket trait implementation

impl<'heap> Arena for &'heap VecAllocator {
    type Slice<'a> = &'a [u8]
    where
        Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        self.deref().inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a,
    {
        self.deref().as_slice(offset, length)
    }
}

impl<'heap> Arena for &'heap mut VecAllocator {
    type Slice<'a> = &'a [u8]
    where
        Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        self.deref().inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a,
    {
        self.deref().as_slice(offset, length)
    }
}

impl<'heap> ArenaAllocator for &'heap mut VecAllocator {
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        self.deref_mut().allocate(value)
    }
    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        self.deref_mut().extend(offset, size)
    }
    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        self.deref_mut().shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        self.deref_mut().write(offset, value)
    }
}

impl<'heap> Arena for Rc<VecAllocator> {
    type Slice<'a> = &'a [u8]
    where
        Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        self.deref().inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a,
    {
        self.deref().as_slice(offset, length)
    }
}

impl<'heap> Arena for Rc<RefCell<VecAllocator>> {
    type Slice<'a> = Ref<'a, [u8]>
        where
            Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().borrow().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        self.deref()
            .borrow()
            .inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a,
    {
        Ref::map(self.deref().borrow(), |arena| {
            arena.as_slice(offset, length)
        })
    }
}

impl<'heap> Arena for Rc<RefCell<&'heap mut VecAllocator>> {
    type Slice<'a> = Ref<'a, [u8]>
        where
            Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.deref().borrow().read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        self.deref()
            .borrow()
            .inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a,
    {
        Ref::map(self.deref().borrow(), |arena| {
            arena.as_slice(offset, length)
        })
    }
}

impl<'heap> ArenaAllocator for Rc<RefCell<&'heap mut VecAllocator>> {
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        self.deref().borrow_mut().allocate(value)
    }
    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        self.deref().borrow_mut().extend(offset, size)
    }
    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        self.deref().borrow_mut().shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        self.deref().borrow_mut().write(offset, value)
    }
}

fn pad_to_4_byte_offset(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        (((value - 1) / 4) + 1) * 4
    }
}

pub struct ArenaIterator<'a, T: TermSize, A: Arena> {
    arena: &'a A,
    next_offset: ArenaPointer,
    end_offset: ArenaPointer,
    _value: PhantomData<T>,
}

impl<'a, T: TermSize, A: Arena> ArenaIterator<'a, T, A> {
    pub fn new(arena: &'a A, start_offset: ArenaPointer, end_offset: ArenaPointer) -> Self {
        ArenaIterator {
            arena,
            next_offset: start_offset,
            end_offset,
            _value: Default::default(),
        }
    }
}

impl<'a, T: TermSize, A: Arena> Iterator for ArenaIterator<'a, T, A> {
    type Item = ArenaPointer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_offset >= self.end_offset {
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
