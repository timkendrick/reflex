// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    cell::{Ref, RefCell},
    iter::repeat,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{hash::TermSize, TermPointer};

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
    pub fn from_vec_u32(data: Vec<u32>) -> Self {
        Self(data)
    }
    pub(crate) fn get_ref<T>(&self, offset: TermPointer) -> &T {
        let Self(data) = self;
        let offset = u32::from(offset) as usize;
        if (offset % 4 != 0) || (offset + std::mem::size_of::<T>() > data.len()) {
            panic!("Invalid allocator offset");
        }
        let item = &data[offset / 4];
        unsafe { std::mem::transmute::<&u32, &T>(item) }
    }
    pub(crate) fn get_mut<T>(&mut self, offset: TermPointer) -> &mut T {
        let Self(data) = self;
        let offset = u32::from(offset) as usize;
        if (offset % 4 != 0) || (offset + std::mem::size_of::<T>() > data.len()) {
            panic!("Invalid allocator offset");
        }
        let item = &mut data[offset / 4];
        unsafe { std::mem::transmute::<&mut u32, &mut T>(item) }
    }
}
impl Default for VecAllocator {
    fn default() -> Self {
        Self(Default::default())
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
        self.extend(offset, static_size);
        self.write(offset, value);
        if actual_size < static_size {
            self.shrink(offset.offset(static_size as u32), static_size - actual_size);
        }
        TermPointer::from(offset)
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            let Self(data) = self;
            data.extend(repeat(0).take(pad_to_4_byte_offset(size) / 4));
        }
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            let Self(data) = self;
            data.truncate((offset as u32 as usize - pad_to_4_byte_offset(size)) / 4);
        }
    }
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        *self.get_mut(offset) = value
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

fn pad_to_4_byte_offset(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        (((value - 1) / 4) + 1) * 4
    }
}
