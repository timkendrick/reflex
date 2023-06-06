// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::iter::repeat;

use crate::{hash::TermSize, TermPointer};

pub trait ArenaAllocator: Sized {
    fn len(&self) -> usize;
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer;
    fn get<T>(&self, offset: TermPointer) -> &T;
    fn get_mut<T>(&mut self, offset: TermPointer) -> &mut T;
    fn get_offset<T>(&self, value: &T) -> TermPointer;
    fn slice<T: Sized>(&self, offset: TermPointer, count: usize) -> &[T];
    fn extend(&mut self, offset: TermPointer, size: usize);
    fn shrink(&mut self, offset: TermPointer, size: usize);
}

pub struct VecAllocator(Vec<u32>);

impl VecAllocator {
    pub fn from_vec_u32(data: Vec<u32>) -> Self {
        Self(data)
    }
}
impl Default for VecAllocator {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl ArenaAllocator for VecAllocator {
    fn len(&self) -> usize {
        let Self(data) = self;
        data.len() * 4
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        let offset = TermPointer(self.len() as u32);
        let static_size = pad_to_4_byte_offset(std::mem::size_of::<T>());
        let actual_size = pad_to_4_byte_offset(value.size_of());
        self.extend(offset, static_size);
        let target = self.get_mut(offset);
        *target = value;
        if actual_size < static_size {
            self.shrink(offset.offset(static_size as u32), static_size - actual_size);
        }
        TermPointer::from(offset)
    }
    fn get<T>(&self, offset: TermPointer) -> &T {
        let Self(data) = self;
        let offset = u32::from(offset) as usize;
        let item = &data[offset / 4];
        unsafe { std::mem::transmute::<&u32, &T>(item) }
    }
    fn get_mut<T>(&mut self, offset: TermPointer) -> &mut T {
        let Self(data) = self;
        let offset = u32::from(offset) as usize;
        let item = &mut data[offset / 4];
        unsafe { std::mem::transmute::<&mut u32, &mut T>(item) }
    }
    fn get_offset<T>(&self, value: &T) -> TermPointer {
        let Self(data) = self;
        let offset = (value as *const T as usize) - (&data[0] as *const u32 as usize);
        TermPointer::from(offset as u32)
    }
    fn slice<T: Sized>(&self, offset: TermPointer, count: usize) -> &[T] {
        let Self(data) = self;
        let offset = u32::from(offset) as usize;
        unsafe { std::slice::from_raw_parts((&data[offset / 4]) as *const u32 as *const T, count) }
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
}
impl VecAllocator {
    pub fn as_slice(&self) -> &[u32] {
        let Self(data) = self;
        data
    }
    pub fn as_bytes(&self) -> &[u8] {
        let Self(data) = self;
        unsafe { std::mem::transmute::<&[u32], &[u8]>(data) }
    }
}
impl Into<Vec<u32>> for VecAllocator {
    fn into(self) -> Vec<u32> {
        let Self(data) = self;
        data
    }
}
impl Into<Vec<u8>> for VecAllocator {
    fn into(self) -> Vec<u8> {
        let Self(data) = self;
        unsafe { std::mem::transmute::<Vec<u32>, Vec<u8>>(data) }
    }
}

pub struct SliceAllocator<'a>(&'a mut [u32]);
impl<'a> ArenaAllocator for SliceAllocator<'a> {
    fn len(&self) -> usize {
        self.data()[0] as usize
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        let offset = TermPointer(self.len() as u32);
        self.extend(offset, value.size_of());
        let target = self.get_mut(offset);
        *target = value;
        TermPointer::from(offset)
    }
    fn get<T>(&self, offset: TermPointer) -> &'a T {
        let pointer = self.get_pointer(offset.into());
        unsafe { std::mem::transmute::<*const T, &T>(pointer) }
    }
    fn get_mut<T>(&mut self, offset: TermPointer) -> &'a mut T {
        let pointer = self.get_pointer(offset.into());
        unsafe { std::mem::transmute::<*const T, &mut T>(pointer) }
    }
    fn get_offset<T>(&self, value: &T) -> TermPointer {
        let offset = (value as *const T as usize) - (self.data()[0] as *const u32 as usize);
        TermPointer::from(offset as u32)
    }
    fn slice<T: Sized>(&self, offset: TermPointer, count: usize) -> &[T] {
        let pointer = self.get_pointer(offset.into());
        unsafe { std::slice::from_raw_parts::<T>(pointer, count) }
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            self.set_len(offset as usize + size);
            // FIXME: Implement SliceAllocator::extend
            panic!("Unable to extend slice allocator");
        }
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        let offset = u32::from(offset);
        if offset != self.len() as u32 {
            panic!("Invalid allocator offset");
        } else {
            self.set_len(offset as usize - size);
        }
    }
}
impl<'a> SliceAllocator<'a> {
    fn get_pointer<T>(&self, offset: u32) -> *const T {
        let Self(allocator_offset) = self;
        let allocator_offset = *allocator_offset as *const [u32] as *const u32 as u32;
        (allocator_offset + offset) as *const T
    }
    fn set_len(&mut self, value: usize) {
        let Self(data) = self;
        data[0] = value as u32;
    }
    fn data(&'a self) -> &'a [u32] {
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
