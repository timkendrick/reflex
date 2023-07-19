// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use crate::{
    utils::{u32_get_byte, u64_get_byte},
    Arena,
};

pub trait TermSize {
    fn size_of(&self) -> usize;
}
impl TermSize for bool {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for u8 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for u32 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for [u32; 2] {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for u64 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for i32 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for i64 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for f32 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermSize for f64 {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

pub trait TermHash {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher;
}
impl TermHash for bool {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_bool(*self)
    }
}
impl TermHash for u8 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_u8(*self)
    }
}
impl TermHash for u32 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_u32(*self)
    }
}
impl TermHash for [u32; 2] {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_u64(unsafe { std::mem::transmute::<[u32; 2], u64>(*self) })
    }
}
impl TermHash for u64 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_u64(*self)
    }
}
impl TermHash for i32 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_i32(*self)
    }
}
impl TermHash for i64 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_i64(*self)
    }
}
impl TermHash for f32 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_f32(*self)
    }
}
impl TermHash for f64 {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher.write_f64(*self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[repr(transparent)]
pub struct TermHashState(u64);
impl From<TermHashState> for u64 {
    fn from(value: TermHashState) -> Self {
        let TermHashState(value) = value;
        value
    }
}
impl From<u64> for TermHashState {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy)]
pub struct TermHasher {
    state: TermHashState,
}
impl Default for TermHasher {
    fn default() -> Self {
        Self {
            state: TermHashState(0xcbf29ce484222325),
        }
    }
}
impl TermHasher {
    pub fn hash<T: TermHash, A: Arena>(self, value: &T, arena: &A) -> Self {
        value.hash(self, arena)
    }
    pub fn finish(self) -> TermHashState {
        let Self { state } = self;
        state
    }
    pub fn write_bool(self, value: bool) -> Self {
        self.write_u8(value as u8)
    }
    pub fn write_u8(self, value: u8) -> Self {
        let Self { state } = self;
        let TermHashState(state) = state;
        Self {
            state: TermHashState((state ^ (value as u64)).wrapping_mul(0x100000001b3)),
        }
    }
    pub fn write_u32(self, value: u32) -> Self {
        self.write_u8(u32_get_byte(value, 0))
            .write_u8(u32_get_byte(value, 1))
            .write_u8(u32_get_byte(value, 2))
            .write_u8(u32_get_byte(value, 3))
    }
    pub fn write_u64(self, value: u64) -> Self {
        self.write_u8(u64_get_byte(value, 0))
            .write_u8(u64_get_byte(value, 1))
            .write_u8(u64_get_byte(value, 2))
            .write_u8(u64_get_byte(value, 3))
            .write_u8(u64_get_byte(value, 4))
            .write_u8(u64_get_byte(value, 5))
            .write_u8(u64_get_byte(value, 6))
            .write_u8(u64_get_byte(value, 7))
    }
    pub fn write_i32(self, value: i32) -> Self {
        self.write_u32(unsafe { std::mem::transmute::<i32, u32>(value) })
    }
    pub fn write_i64(self, value: i64) -> Self {
        self.write_u64(unsafe { std::mem::transmute::<i64, u64>(value) })
    }
    pub fn write_f32(self, value: f32) -> Self {
        self.write_u32(unsafe { std::mem::transmute::<f32, u32>(value) })
    }
    pub fn write_f64(self, value: f64) -> Self {
        self.write_u64(unsafe { std::mem::transmute::<f64, u64>(value) })
    }
    pub fn write_hash(self, value: TermHashState) -> Self {
        self.write_u64(value.into())
    }
}
