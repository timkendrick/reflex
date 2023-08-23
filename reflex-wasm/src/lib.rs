// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{hash::Hash, marker::PhantomData};

use reflex::{
    core::{NodeId, RefType},
    hash::HashId,
};
use reflex_macros::PointerIter;

use crate::{
    allocator::{Arena, ArenaAllocator},
    hash::{TermHash, TermHashState, TermHasher, TermSize},
    stdlib::Stdlib,
    term_type::{TermType, TermTypeDiscriminants},
};

pub use anyhow;
pub use wasi_common;
pub use wasmtime;
pub use wasmtime_wasi;

pub mod allocator;
pub mod builtins;
pub mod cache;
pub mod cli;
pub mod compiler;
pub mod exports;
pub mod factory;
pub mod hash;
pub mod interpreter;
pub mod serialize;
pub mod stdlib;
pub mod term_type;
pub mod utils;

// Memory is allocated in 64KiB pages according to WASM spec
pub const WASM_PAGE_SIZE: usize = 64 * 1024;

#[derive(PartialEq, Eq, Clone, Copy, PointerIter, Hash, Debug)]
#[repr(C)]
pub struct FunctionIndex(u32);
impl FunctionIndex {
    pub fn as_stdlib(&self) -> Option<Stdlib> {
        let Self(target) = self;
        Stdlib::try_from(*target).ok()
    }
}
impl TermSize for FunctionIndex {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FunctionIndex {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let Self(uid) = self;
        hasher.hash(uid, arena)
    }
}
impl From<u32> for FunctionIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<FunctionIndex> for u32 {
    fn from(value: FunctionIndex) -> Self {
        let FunctionIndex(value) = value;
        value
    }
}
impl From<Stdlib> for FunctionIndex {
    fn from(value: Stdlib) -> Self {
        Self(u32::from(value))
    }
}
impl TryFrom<FunctionIndex> for Stdlib {
    type Error = <Self as TryFrom<u32>>::Error;
    fn try_from(value: FunctionIndex) -> Result<Self, Self::Error> {
        Self::try_from(u32::from(value))
    }
}
impl std::fmt::Display for FunctionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Stdlib::try_from(*self) {
            Ok(stdlib) => write!(f, "<stdlib:{}>", stdlib),
            Err(_) => write!(f, "<fn:{}>", self.0),
        }
    }
}

pub struct ArenaRef<T, A: Arena> {
    arena: A,
    pointer: ArenaPointer,
    _type: PhantomData<T>,
}
impl<T, A: Arena> std::hash::Hash for ArenaRef<T, A>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.read_value(|value| value.hash(state))
    }
}
impl<T, A: Arena> ArenaRef<T, A> {
    pub fn new(arena: A, pointer: ArenaPointer) -> Self {
        Self {
            arena,
            pointer,
            _type: PhantomData,
        }
    }
    pub fn read_value<V>(&self, selector: impl FnOnce(&T) -> V) -> V {
        self.arena.read_value::<T, V>(self.pointer, selector)
    }
    pub fn inner_pointer<V>(&self, selector: impl FnOnce(&T) -> &V) -> ArenaPointer {
        self.arena.inner_pointer::<T, V>(self.pointer, selector)
    }
    pub fn arena(&self) -> &A {
        &self.arena
    }
    pub fn as_pointer(&self) -> ArenaPointer {
        self.pointer
    }
    pub fn inner_ref<V>(&self, selector: impl FnOnce(&T) -> &V) -> ArenaRef<V, A>
    where
        A: Clone,
    {
        ArenaRef::new(
            self.arena.clone(),
            self.arena.inner_pointer::<T, V>(self.pointer, selector),
        )
    }
}
impl<T, A: Arena> Copy for ArenaRef<T, A> where A: Copy {}
impl<T, A: Arena> Clone for ArenaRef<T, A>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        Self {
            arena: self.arena.clone(),
            pointer: self.pointer,
            _type: PhantomData,
        }
    }
}

impl<'a, T, A: Arena> From<&'a ArenaRef<T, A>> for ArenaRef<T, A>
where
    A: Clone,
{
    fn from(value: &'a ArenaRef<T, A>) -> Self {
        value.clone()
    }
}

impl<T, A: Arena> RefType<Self> for ArenaRef<T, A> {
    fn as_deref(&self) -> &Self {
        self
    }
}

pub trait ArenaPointerIterator
where
    Self: Iterator<Item = ArenaPointer> + Sized,
{
    fn skip_null_pointers(self) -> SkipNullPointersIter<Self>;
    fn skip_uninitialized_pointers<'a, A: Arena>(
        self,
        arena: &'a A,
    ) -> SkipUninitializedPointersIter<'a, Self, A>;
    fn into_arena_refs<'a, T: 'a, A: Arena + Clone>(
        self,
        arena: &'a A,
    ) -> IntoArenaRefIter<'a, T, A, Self>;
}

impl<_Self> ArenaPointerIterator for _Self
where
    Self: Iterator<Item = ArenaPointer> + Sized,
{
    fn skip_null_pointers(self) -> SkipNullPointersIter<Self> {
        SkipNullPointersIter::new(self)
    }
    fn skip_uninitialized_pointers<'a, A: Arena>(
        self,
        arena: &'a A,
    ) -> SkipUninitializedPointersIter<'a, Self, A> {
        SkipUninitializedPointersIter::new(self, arena)
    }
    fn into_arena_refs<'a, T: 'a, A: Arena + Clone>(
        self,
        arena: &'a A,
    ) -> IntoArenaRefIter<'a, T, A, Self> {
        IntoArenaRefIter::new(arena, self)
    }
}

impl<'a, T: 'a, A: Arena + Clone, TInner: Iterator<Item = ArenaPointer>> Iterator
    for IntoArenaRefIter<'a, T, A, TInner>
{
    type Item = ArenaRef<T, A>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|pointer| ArenaRef::<T, _>::new(self.arena.clone(), pointer))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T: 'a, A: Arena + Clone, TInner: Iterator<Item = ArenaPointer>> ExactSizeIterator
    for IntoArenaRefIter<'a, T, A, TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct IntoArenaRefIter<'a, T: 'a, A: Arena, TInner: Iterator<Item = ArenaPointer>> {
    arena: &'a A,
    inner: TInner,
    _item: PhantomData<T>,
}

impl<'a, T: 'a, A: Arena, TInner: Iterator<Item = ArenaPointer>>
    IntoArenaRefIter<'a, T, A, TInner>
{
    fn new(arena: &'a A, inner: TInner) -> Self {
        Self {
            arena,
            inner,
            _item: PhantomData,
        }
    }
}

impl<'a, T: 'a, A: Arena, TInner: Iterator<Item = ArenaPointer>> Clone
    for IntoArenaRefIter<'a, T, A, TInner>
where
    TInner: Clone,
{
    fn clone(&self) -> Self {
        Self {
            arena: self.arena,
            inner: self.inner.clone(),
            _item: PhantomData,
        }
    }
}

pub struct SkipUninitializedPointersIter<'a, T: Iterator<Item = ArenaPointer>, A: Arena> {
    inner: T,
    arena: &'a A,
}

impl<'a, T: Iterator<Item = ArenaPointer>, A: Arena> SkipUninitializedPointersIter<'a, T, A> {
    pub fn new(inner: T, arena: &'a A) -> Self {
        Self { inner, arena }
    }
}

impl<'a, T: Iterator<Item = ArenaPointer>, A: Arena> Clone
    for SkipUninitializedPointersIter<'a, T, A>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            arena: self.arena,
        }
    }
}

impl<'a, T: Iterator<Item = ArenaPointer>, A: Arena> Iterator
    for SkipUninitializedPointersIter<'a, T, A>
{
    type Item = ArenaPointer;
    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.inner.next()?;
        while self
            .arena
            .read_value::<u32, bool>(item, |value| ArenaPointer::from(*value).is_uninitialized())
        {
            item = self.inner.next()?
        }
        Some(item)
    }
}

pub struct SkipNullPointersIter<T: Iterator<Item = ArenaPointer>> {
    inner: T,
}

impl<T: Iterator<Item = ArenaPointer>> SkipNullPointersIter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Iterator<Item = ArenaPointer>> Clone for SkipNullPointersIter<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Iterator<Item = ArenaPointer>> Iterator for SkipNullPointersIter<T> {
    type Item = ArenaPointer;
    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.inner.next()?;
        while item.is_null() {
            item = self.inner.next()?
        }
        Some(item)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Term {
    header: TermHeader,
    value: TermType,
}
impl Term {
    pub fn new(value: TermType, arena: &impl Arena) -> Self {
        Self {
            header: TermHeader {
                hash: value.hash(TermHasher::default(), arena).finish(),
            },
            value,
        }
    }
    pub fn id(self) -> HashId {
        HashId::from(self.header.hash)
    }
    pub fn get_hash_pointer(term: ArenaPointer) -> ArenaPointer {
        term.offset(0)
    }
    pub fn get_value_pointer(term: ArenaPointer) -> ArenaPointer {
        term.offset(std::mem::size_of::<TermHeader>() as u32)
    }
    pub fn as_bytes(&self) -> &[u32] {
        let num_words = pad_to_4_byte_offset(self.size_of() as usize) / 4;
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u32, num_words) }
    }
    pub(crate) fn type_id(&self) -> TermTypeDiscriminants {
        TermTypeDiscriminants::from(&self.value)
    }
    pub fn as_value(&self) -> &TermType {
        &self.value
    }
}
impl TermSize for Term {
    fn size_of(&self) -> usize {
        std::mem::size_of::<TermHashState>() + self.value.size_of()
    }
}
impl TermHash for Term {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        self.value.hash(hasher, arena)
    }
}

impl<A: Arena> ArenaRef<Term, A> {
    pub(crate) fn get_value_pointer(&self) -> ArenaPointer {
        Term::get_value_pointer(self.pointer)
    }
}

impl<A: Arena> NodeId for ArenaRef<Term, A> {
    fn id(&self) -> HashId {
        self.read_value(|term| term.id())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TermHeader {
    hash: TermHashState,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[repr(transparent)]
pub struct ArenaPointer(u32);

impl reflex::hash::IsEnabled for ArenaPointer {}

impl ArenaPointer {
    pub fn null() -> Self {
        Self(0xFFFFFFFF)
    }
    pub fn uninitialized() -> Self {
        Self(0x00000000)
    }
    pub fn is_null(self) -> bool {
        let Self(offset) = self;
        offset == 0xFFFFFFFF
    }
    pub fn is_uninitialized(self) -> bool {
        let Self(offset) = self;
        offset == 0
    }
    pub fn as_non_null(self) -> Option<Self> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }
    pub(crate) fn offset(self, offset: u32) -> Self {
        let Self(existing_offset) = self;
        Self(existing_offset + offset)
    }
}
impl From<ArenaPointer> for u32 {
    fn from(value: ArenaPointer) -> Self {
        let ArenaPointer(value) = value;
        value
    }
}
impl From<u32> for ArenaPointer {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Array<T> {
    pub capacity: u32,
    pub length: u32,
    pub items: [T; 0],
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
impl<T> TermSize for Array<T>
where
    T: Sized,
{
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() + ((self.capacity as usize) * std::mem::size_of::<T>())
    }
}
impl<T> TermHash for Array<T>
where
    T: TermHash,
{
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let hasher = hasher.write_u32(self.length);
        self.items()
            .fold(hasher, |hasher, item| item.hash(hasher, arena))
    }
}

impl<T: Sized> Array<T> {
    pub fn len(&self) -> usize {
        self.length as usize
    }
    pub fn extend(
        list: ArenaPointer,
        items: impl IntoIterator<Item = T, IntoIter = impl ExactSizeIterator<Item = T>>,
        arena: &mut impl ArenaAllocator,
    ) {
        let items = items.into_iter();
        let num_items = items.len();
        let capacity = num_items as u32;
        let length = num_items as u32;
        let capacity_offset = list;
        let length_offset = list.offset(std::mem::size_of::<u32>() as u32);
        let items_offset =
            list.offset((std::mem::size_of::<Self>() - std::mem::size_of::<[T; 0]>()) as u32);
        arena.write::<u32>(capacity_offset, capacity);
        arena.write::<u32>(length_offset, length);
        arena.extend(items_offset, num_items * std::mem::size_of::<T>());
        let array_offset = u32::from(items_offset);
        for (index, item) in items.enumerate() {
            arena.write(
                ArenaPointer::from(array_offset + ((index * std::mem::size_of::<T>()) as u32)),
                item,
            );
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(unsafe { self.get_unchecked(index) })
        } else {
            None
        }
    }
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        let offset = &self.items as *const T as usize;
        let pointer = (offset + (index * std::mem::size_of::<T>())) as *const T;
        std::mem::transmute::<*const T, &T>(pointer)
    }
    pub fn items(&self) -> ArrayRefIter<'_, T> {
        ArrayRefIter::new(self)
    }
    pub(crate) fn iter<'a, A: Arena>(list: ArenaPointer, arena: &'a A) -> ArrayValueIter<'a, T, A>
    where
        T: Copy,
    {
        ArrayValueIter {
            length: arena.read_value::<Array<T>, _>(list, |value| value.length as usize),
            offset: Self::get_item_offset(list, 0),
            arena,
            _item: PhantomData,
        }
    }
    pub(crate) fn get_item_offset(list: ArenaPointer, index: usize) -> ArenaPointer {
        list.offset((std::mem::size_of::<Array<T>>() + (index * std::mem::size_of::<T>())) as u32)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Array<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Array")
            .field("capacity", &self.capacity)
            .field("length", &self.length)
            .field("items", &self.items().collect::<Vec<_>>())
            .finish()
    }
}

impl<T, A: Arena> ArenaRef<Array<T>, A> {
    pub fn len(&self) -> usize {
        self.arena
            .read_value::<Array<T>, u32>(self.pointer, |value| value.length) as usize
    }
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Copy,
    {
        self.read_value(|items| items.get(index).copied())
    }
    pub fn iter<'a>(&'a self) -> ArrayValueIter<'a, T, A>
    where
        T: Copy,
    {
        Array::<T>::iter(self.pointer, &self.arena)
    }
    pub fn item_offset(&self, index: usize) -> ArenaPointer {
        if index < self.len() {
            Array::<T>::get_item_offset(self.as_pointer(), index)
        } else {
            ArenaPointer::null()
        }
    }
    pub fn item_offsets(&self) -> ArrayPointerIter<T> {
        ArrayPointerIter::new(self)
    }
}

#[derive(Debug)]
pub struct ArrayPointerIter<T: Sized> {
    offset: ArenaPointer,
    length: usize,
    _item: PhantomData<T>,
}

impl<T: Sized> ArrayPointerIter<T> {
    fn new<A: Arena>(list: &ArenaRef<Array<T>, A>) -> ArrayPointerIter<T> {
        Self {
            offset: Array::<T>::get_item_offset(list.as_pointer(), 0),
            length: list.len(),
            _item: PhantomData,
        }
    }
}

impl<'a, T: Sized> Clone for ArrayPointerIter<T> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            length: self.length,
            _item: PhantomData,
        }
    }
}

impl<'a, T: Sized> Copy for ArrayPointerIter<T> {}

impl<'a, T: Sized> Iterator for ArrayPointerIter<T> {
    type Item = ArenaPointer;
    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            let current_pointer = self.offset;
            self.offset = current_pointer.offset(std::mem::size_of::<T>() as u32);
            self.length -= 1;
            Some(current_pointer)
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

#[derive(Debug)]
pub struct ArrayRefIter<'a, T: Sized> {
    array: &'a Array<T>,
    offset: usize,
}

impl<'a, T: Sized> ArrayRefIter<'a, T> {
    fn new(array: &'a Array<T>) -> Self {
        Self { array, offset: 0 }
    }
}

impl<'a, T: Sized> Clone for ArrayRefIter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            array: self.array,
            offset: self.offset,
        }
    }
}

impl<'a, T: Sized> Copy for ArrayRefIter<'a, T> {}

impl<'a, T: Sized> Iterator for ArrayRefIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.array.get(self.offset) {
            None => None,
            Some(value) => {
                self.offset += 1;
                Some(value)
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.array.len() - self.offset;
        (length, Some(length))
    }
}

impl<'a, T: Sized> ExactSizeIterator for ArrayRefIter<'a, T> {}

pub struct ArrayValueIter<'a, T: Sized + Copy, A: Arena> {
    length: usize,
    offset: ArenaPointer,
    arena: &'a A,
    _item: PhantomData<T>,
}

impl<'a, T: Sized + Copy, A: Arena> Clone for ArrayValueIter<'a, T, A> {
    fn clone(&self) -> Self {
        Self {
            length: self.length.clone(),
            offset: self.offset.clone(),
            arena: self.arena,
            _item: PhantomData,
        }
    }
}

impl<'a, T: Sized + Copy, A: Arena> Copy for ArrayValueIter<'a, T, A> {}

impl<'a, T: Sized + Copy, A: Arena> Iterator for ArrayValueIter<'a, T, A> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            let item = self.arena.read_value::<T, T>(self.offset, |term| *term);
            self.offset = self.offset.offset(std::mem::size_of::<T>() as u32);
            self.length -= 1;
            Some(item)
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

impl<'a, T: Sized + Copy, A: Arena> ExactSizeIterator for ArrayValueIter<'a, T, A> {}

pub fn pad_to_4_byte_offset(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        (((value - 1) / 4) + 1) * 4
    }
}

pub trait PointerIter {
    type Iter<'a>: Iterator<Item = ArenaPointer>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a;
}

#[cfg(test)]
mod tests {
    use reflex::core::NodeId;
    use reflex_macros::PointerIter;

    use crate::{
        allocator::{Arena, ArenaAllocator, VecAllocator},
        hash::TermSize,
        ArenaPointer, ArenaRef,
    };

    use super::*;

    #[derive(PointerIter, PartialEq, Eq, Debug, Clone)]
    #[repr(C)]
    pub struct TreeNode {
        pub id: u64,
        pub first: ArenaPointer,
        pub after: u32,
        pub second: ArenaPointer,
    }
    impl TermSize for TreeNode {
        fn size_of(&self) -> usize {
            std::mem::size_of::<Self>()
        }
    }

    impl NodeId for TreeNode {
        fn id(&self) -> HashId {
            self.id
        }
    }

    impl<A: Arena + Clone> NodeId for ArenaRef<TreeNode, A> {
        fn id(&self) -> HashId {
            self.read_value(|term| term.id())
        }
    }

    impl<A: Arena + Clone> PartialEq for ArenaRef<TreeNode, A> {
        fn eq(&self, other: &Self) -> bool {
            self.read_value(|value| value.id) == other.read_value(|value| value.id)
                && self.read_value(|value| value.after) == other.read_value(|value| value.after)
                && self.first() == other.first()
                && self.second() == other.second()
        }
    }

    impl<A: Arena + Clone> ArenaRef<TreeNode, A> {
        pub fn first(&self) -> Option<ArenaRef<TreeNode, A>> {
            self.read_value(|value| value.first)
                .as_non_null()
                .map(|pointer| ArenaRef::<TreeNode, _>::new(self.arena.clone(), pointer))
        }
        pub fn second(&self) -> Option<ArenaRef<TreeNode, A>> {
            self.read_value(|value| value.second)
                .as_non_null()
                .map(|pointer| ArenaRef::<TreeNode, _>::new(self.arena.clone(), pointer))
        }
    }

    impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<TreeNode, A> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.read_value(|term| std::fmt::Debug::fmt(term, f))
        }
    }

    #[test]
    fn pointer_iter_trait() {
        let mut allocator = VecAllocator::default();

        let term = TreeNode {
            id: 3,
            first: ArenaPointer::from(20),
            after: 123,
            second: ArenaPointer::from(50),
        };
        let instance = allocator.allocate(term);
        let expression = ArenaRef::<TreeNode, _>::new(&allocator, instance);
        let (before_pointer, before_size) = (instance.offset(0), std::mem::size_of::<u64>() as u32);
        let (first_pointer, first_size) = (
            before_pointer.offset(before_size),
            std::mem::size_of::<ArenaPointer>() as u32,
        );
        let (after_pointer, after_size) = (
            first_pointer.offset(first_size),
            std::mem::size_of::<u32>() as u32,
        );
        let (second_pointer, _second_size) = (
            after_pointer.offset(after_size),
            std::mem::size_of::<ArenaPointer>() as u32,
        );

        assert_eq!(
            expression.iter().as_slice(),
            &[first_pointer, second_pointer]
        );
    }
}
