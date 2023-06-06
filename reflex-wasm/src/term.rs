// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::hash::TermHash;

#[derive(Clone, Copy, Debug)]
pub struct TermType(u32);
impl TermType {
    pub(crate) fn write_hash(&self, state: TermHash) -> TermHash {
        state.write_byte(self.0 as u8)
    }
}

#[repr(C)]
pub struct Term<const NUM_FIELDS: usize> {
    hash: TermHash,
    term_type: TermType,
    fields: [u32; NUM_FIELDS],
}
impl<const NUM_FIELDS: usize> Term<NUM_FIELDS> {
    pub fn new(term_type: TermType, hash: TermHash, fields: [u32; NUM_FIELDS]) -> Self {
        Self {
            hash,
            term_type,
            fields,
        }
    }
    pub fn is_null(&self) -> bool {
        let pointer = self as *const Self as u32;
        pointer == 0xFFFFFFFF
    }
    pub fn get_type(&self) -> TermType {
        self.term_type
    }
    pub fn get_hash(&self) -> TermHash {
        self.hash
    }
    pub fn get_field_pointer(&self, field_index: u32) -> u32 {
        let pointer = &self.fields as *const [u32; NUM_FIELDS] as u32;
        pointer + field_index
    }
    pub fn get_field_value(&self, field_index: u32) -> *const Term<0> {
        self.read_typed_field::<*const Term<0>>(field_index)
    }
    pub fn read_u32_field_value(&self, field_index: u32) -> u32 {
        self.read_typed_field::<u32>(field_index)
    }
    pub fn read_i32_field_value(&self, field_index: u32) -> i32 {
        self.read_typed_field::<i32>(field_index)
    }
    pub fn read_u64_field_value(&self, field_index: u32) -> u64 {
        self.read_typed_field::<u64>(field_index)
    }
    pub fn read_i64_field_value(&self, field_index: u32) -> i64 {
        self.read_typed_field::<i64>(field_index)
    }
    pub fn read_f32_field_value(&self, field_index: u32) -> f32 {
        self.read_typed_field::<f32>(field_index)
    }
    pub fn read_f64_field_value(&self, field_index: u32) -> f64 {
        self.read_typed_field::<f64>(field_index)
    }
    pub(crate) fn from_pointer(pointer: *const Self) -> &'static Self {
        unsafe { std::mem::transmute::<*const Self, &'static Self>(pointer) }
    }
    pub(crate) fn as_type<T>(&self) -> &'static T {
        unsafe { std::mem::transmute::<&Self, &'static T>(self) }
    }
    fn read_typed_field<T: Copy>(&self, field_index: u32) -> T {
        let pointer = self.get_field_pointer(field_index);
        unsafe { *(pointer as *const T) }
    }
}

pub(crate) fn i32_to_field(value: i32) -> u32 {
    unsafe { std::mem::transmute::<i32, u32>(value) }
}

pub(crate) fn u64_to_fields(value: u64) -> [u32; 2] {
    unsafe { std::mem::transmute::<u64, [u32; 2]>(value) }
}

pub(crate) fn i64_to_fields(value: i64) -> [u32; 2] {
    unsafe { std::mem::transmute::<i64, [u32; 2]>(value) }
}

pub(crate) fn f64_to_fields(value: f64) -> [u32; 2] {
    unsafe { std::mem::transmute::<f64, [u32; 2]>(value) }
}

pub(crate) trait TermTraits {
    fn is_static(&self) -> bool;
    fn is_atomic(&self) -> bool;
    fn is_truthy(&self) -> bool;
    fn equals(&self, other: &Self) -> bool;
    fn from_pointer<const NUM_FIELDS: usize>(pointer: *const Term<NUM_FIELDS>) -> &'static Self;
}

pub(crate) mod boolean;
pub(crate) mod builtin;
pub(crate) mod float;
pub(crate) mod int;
pub(crate) mod nil;
pub(crate) mod symbol;

pub const TERM_TYPE_APPLICATION: TermType = TermType(0);
pub const TERM_TYPE_BOOLEAN: TermType = TermType(1);
pub const TERM_TYPE_BUILTIN: TermType = TermType(2);
pub const TERM_TYPE_CELL: TermType = TermType(3);
pub const TERM_TYPE_HASHMAP: TermType = TermType(4);
pub const TERM_TYPE_LIST: TermType = TermType(5);
pub const TERM_TYPE_RECORD: TermType = TermType(6);
pub const TERM_TYPE_TREE: TermType = TermType(7);
pub const TERM_TYPE_CONDITION: TermType = TermType(8);
pub const TERM_TYPE_EFFECT: TermType = TermType(9);
pub const TERM_TYPE_FLOAT: TermType = TermType(10);
pub const TERM_TYPE_INT: TermType = TermType(11);
pub const TERM_TYPE_NIL: TermType = TermType(12);
pub const TERM_TYPE_PARTIAL: TermType = TermType(13);
pub const TERM_TYPE_POINTER: TermType = TermType(14);
pub const TERM_TYPE_SIGNAL: TermType = TermType(15);
pub const TERM_TYPE_STRING: TermType = TermType(16);
pub const TERM_TYPE_SYMBOL: TermType = TermType(17);
pub const TERM_TYPE_EMPTY_ITERATOR: TermType = TermType(18);
pub const TERM_TYPE_EVALUATE_ITERATOR: TermType = TermType(19);
pub const TERM_TYPE_FILTER_ITERATOR: TermType = TermType(20);
pub const TERM_TYPE_FLATTEN_ITERATOR: TermType = TermType(21);
pub const TERM_TYPE_HASHMAPKEYS_ITERATOR: TermType = TermType(22);
pub const TERM_TYPE_HASHMAPVALUES_ITERATOR: TermType = TermType(23);
pub const TERM_TYPE_INTEGERS_ITERATOR: TermType = TermType(24);
pub const TERM_TYPE_MAP_ITERATOR: TermType = TermType(25);
pub const TERM_TYPE_ONCE_ITERATOR: TermType = TermType(26);
pub const TERM_TYPE_RANGE_ITERATOR: TermType = TermType(27);
pub const TERM_TYPE_REPEAT_ITERATOR: TermType = TermType(28);
pub const TERM_TYPE_SKIP_ITERATOR: TermType = TermType(29);
pub const TERM_TYPE_TAKE_ITERATOR: TermType = TermType(30);
pub const TERM_TYPE_ZIP_ITERATOR: TermType = TermType(31);
