// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{hash::Hash, marker::PhantomData};

use allocator::ArenaAllocator;
use hash::{TermHash, TermHashState, TermHasher, TermSize};

use reflex::{core::RefType, hash::HashId};
use term_type::*;

pub mod allocator;
pub mod compiler;
pub mod factory;
pub mod hash;
pub mod interpreter;
pub mod parser;
pub mod stdlib;
pub mod term_type;
pub mod utils;

pub struct ArenaRef<T, A: ArenaAllocator> {
    pub(crate) arena: A,
    pub pointer: TermPointer,
    _type: PhantomData<T>,
}
impl<T, A: ArenaAllocator> std::hash::Hash for ArenaRef<T, A>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.read_value(|value| value.hash(state))
    }
}
impl<T, A: ArenaAllocator> ArenaRef<T, A> {
    pub fn new(arena: A, pointer: TermPointer) -> Self {
        Self {
            arena,
            pointer,
            _type: PhantomData,
        }
    }
    pub fn read_value<V>(&self, selector: impl FnOnce(&T) -> V) -> V {
        self.arena.read_value::<T, V>(self.pointer, selector)
    }
    pub fn inner_pointer<V>(&self, selector: impl FnOnce(&T) -> &V) -> TermPointer {
        self.arena.inner_pointer::<T, V>(self.pointer, selector)
    }
    pub(crate) fn as_pointer(&self) -> TermPointer {
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
impl<T, A: ArenaAllocator> Copy for ArenaRef<T, A> where A: Copy {}
impl<T, A: ArenaAllocator> Clone for ArenaRef<T, A>
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

impl<'a, T, A: ArenaAllocator> From<&'a ArenaRef<T, A>> for ArenaRef<T, A>
where
    A: Clone,
{
    fn from(value: &'a ArenaRef<T, A>) -> Self {
        value.clone()
    }
}

impl<T, A: ArenaAllocator> RefType<Self> for ArenaRef<T, A> {
    fn as_deref(&self) -> &Self {
        self
    }
}

pub struct IntoArenaRefIter<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>> {
    arena: &'a A,
    inner: TInner,
    _item: PhantomData<T>,
}
impl<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>>
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

pub trait IntoArenaRefIterator<'a, T: 'a, A: ArenaAllocator>
where
    Self: Iterator<Item = TermPointer> + Sized,
{
    fn as_arena_ref(self, arena: &'a A) -> IntoArenaRefIter<'a, T, A, Self>;
}

impl<'a, _Self, T: 'a, A: ArenaAllocator> IntoArenaRefIterator<'a, T, A> for _Self
where
    Self: Iterator<Item = TermPointer> + Sized,
{
    fn as_arena_ref(self, arena: &'a A) -> IntoArenaRefIter<'a, T, A, Self> {
        IntoArenaRefIter::new(arena, self)
    }
}

impl<'a, T: 'a, A: ArenaAllocator + Clone, TInner: Iterator<Item = TermPointer>> Iterator
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
impl<'a, T: 'a, A: ArenaAllocator + Clone, TInner: Iterator<Item = TermPointer>> ExactSizeIterator
    for IntoArenaRefIter<'a, T, A, TInner>
where
    TInner: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Term {
    header: TermHeader,
    value: TermType,
}
impl Term {
    pub fn new(value: TermType, arena: &impl ArenaAllocator) -> Self {
        Self {
            header: TermHeader {
                hash: value.hash(TermHasher::default(), arena).finish(),
            },
            value,
        }
    }
    pub fn id(self) -> HashId {
        // FIXME: 64-bit term hash
        u32::from(self.header.hash) as HashId
    }
    pub fn get_hash_pointer(term: TermPointer) -> TermPointer {
        term.offset(0)
    }
    pub fn get_value_pointer(term: TermPointer) -> TermPointer {
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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        // TODO: Investigate shallow hashing for compound terms
        // hasher.write_hash(self.header.hash)
        self.value.hash(hasher, arena)
    }
}

impl<A: ArenaAllocator> ArenaRef<Term, A> {
    pub(crate) fn get_value_pointer(&self) -> TermPointer {
        Term::get_value_pointer(self.pointer)
    }
}

impl<A: ArenaAllocator> TermHash for ArenaRef<Term, A> {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        self.read_value(move |value| TermHash::hash(value, hasher, arena))
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TermHeader {
    hash: TermHashState,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
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
    pub(crate) fn as_non_null(self) -> Option<Self> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        arena.read_value::<Term, _>(*self, |term| term.hash(hasher, arena))
    }
}
impl std::fmt::Debug for TermPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#016x}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let hasher = hasher.write_u32(self.length);
        self.items()
            .fold(hasher, |hasher, item| item.hash(hasher, arena))
    }
}

impl<T> Array<T>
where
    T: Sized,
{
    pub fn len(&self) -> usize {
        self.length as usize
    }
    pub fn extend(
        list: TermPointer,
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
                TermPointer::from(array_offset + ((index * std::mem::size_of::<T>()) as u32)),
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
        let pointer = (offset + (index * 4)) as *const T;
        std::mem::transmute::<*const T, &T>(pointer)
    }
    pub fn items(&self) -> ArrayIter<'_, T> {
        ArrayIter::new(self)
    }
    pub fn get_item_offset(list: TermPointer, index: usize) -> TermPointer {
        list.offset((std::mem::size_of::<Array<T>>() + (index * std::mem::size_of::<T>())) as u32)
    }
    pub fn iter<'a, A: ArenaAllocator>(list: TermPointer, arena: &'a A) -> ArenaArrayIter<'a, T, A>
    where
        T: Copy,
    {
        ArenaArrayIter {
            length: arena.read_value::<Array<T>, _>(list, |value| value.length as usize),
            offset: Self::get_item_offset(list, 0),
            arena,
            _item: PhantomData,
        }
    }
}

impl<T, A: ArenaAllocator> ArenaRef<Array<T>, A> {
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
    pub fn iter<'a>(&'a self) -> ArenaArrayIter<'a, T, A>
    where
        T: Copy,
    {
        Array::<T>::iter(self.pointer, &self.arena)
    }
}

pub struct ArrayIter<'a, T> {
    array: &'a Array<T>,
    offset: usize,
}
impl<'a, T> ArrayIter<'a, T> {
    fn new(array: &'a Array<T>) -> Self {
        Self { array, offset: 0 }
    }
}
impl<'a, T: Sized> Iterator for ArrayIter<'a, T> {
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

pub struct ArenaArrayIter<'a, T: Sized + Copy, A: ArenaAllocator> {
    length: usize,
    offset: TermPointer,
    arena: &'a A,
    _item: PhantomData<T>,
}

impl<'a, T: Sized + Copy, A: ArenaAllocator> Iterator for ArenaArrayIter<'a, T, A> {
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
impl<'a, T: Sized + Copy, A: ArenaAllocator> ExactSizeIterator for ArenaArrayIter<'a, T, A> {}

pub fn pad_to_4_byte_offset(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        (((value - 1) / 4) + 1) * 4
    }
}

pub trait PointerIter {
    type Iter<'a>: Iterator<Item = TermPointer>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::allocator::{ArenaAllocator, VecAllocator};
    use crate::compiler::Serialize;
    use crate::hash::TermSize;
    use crate::{ArenaRef, PointerIter, TermPointer};
    use reflex::core::NodeId;
    use reflex_macros::PointerIter;

    impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<TreeNode, A> {
        fn eq(&self, other: &Self) -> bool {
            self.read_value(|value| value.id) == other.read_value(|value| value.id)
                && self.read_value(|value| value.after) == other.read_value(|value| value.after)
                && self.first() == other.first()
                && self.second() == other.second()
        }
    }

    impl<A: ArenaAllocator + Clone> ArenaRef<TreeNode, A> {
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

    impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<TreeNode, A> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.read_value(|term| std::fmt::Debug::fmt(term, f))
        }
    }

    #[derive(PointerIter, PartialEq, Eq, Debug, Clone)]
    #[repr(C)]
    pub struct TreeNode {
        pub id: u64,
        pub first: TermPointer,
        pub after: u32,
        pub second: TermPointer,
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

    impl<A: ArenaAllocator + Clone> NodeId for ArenaRef<TreeNode, A> {
        fn id(&self) -> HashId {
            self.read_value(|term| term.id)
        }
    }

    #[test]
    fn basic_usage() {
        let mut source_allocator = VecAllocator::default();

        let _filler = source_allocator.allocate(TreeNode {
            id: 22341u64,
            first: TermPointer::null(),
            after: 2232u32,
            second: TermPointer::null(),
        });

        let leaf = TreeNode {
            id: 15u64,
            first: TermPointer::null(),
            after: 16u32,
            second: TermPointer::null(),
        };
        let leaf_instance = source_allocator.allocate(leaf);

        let root = TreeNode {
            id: 15u64,
            first: leaf_instance,
            after: 16u32,
            second: leaf_instance,
        };
        let root_instance = source_allocator.allocate(root);
        let source_allocator = Rc::new(RefCell::new(source_allocator));
        let root_ref = ArenaRef::<TreeNode, _>::new(source_allocator.clone(), root_instance);

        let leaf_ref = ArenaRef::<TreeNode, _>::new(source_allocator.clone(), leaf_instance);

        let mut target_allocator = VecAllocator::default();

        let _filler = target_allocator.allocate(TreeNode {
            id: 234u64,
            first: TermPointer::null(),
            after: 2222u32,
            second: TermPointer::null(),
        });

        let _filler = target_allocator.allocate(TreeNode {
            id: 223444u64,
            first: TermPointer::null(),
            after: 224442u32,
            second: TermPointer::null(),
        });

        let mut target_allocator = Rc::new(RefCell::new(target_allocator));

        let serialized_expression =
            root_ref.serialize(&mut target_allocator, &mut Default::default());

        let serialized_expression_ref =
            ArenaRef::<TreeNode, _>::new(target_allocator.clone(), serialized_expression);

        assert_eq!(root_ref, serialized_expression_ref);

        let serialized_leaf_ref = serialized_expression_ref.read_value(|root| root.first);
        let serialized_leaf_ref =
            ArenaRef::<TreeNode, _>::new(target_allocator.clone(), serialized_leaf_ref);

        assert_eq!(leaf_ref, serialized_leaf_ref);
    }

    #[test]
    fn serialize_trait() {
        let mut allocator = VecAllocator::default();

        let term = TreeNode {
            id: 0u64,
            first: TermPointer::from(20),
            after: 0u32,
            second: TermPointer::from(50),
        };
        let instance = allocator.allocate(term);
        let expression = ArenaRef::<TreeNode, _>::new(&mut allocator, instance);
        let (before_pointer, before_size) = (instance.offset(0), std::mem::size_of::<u64>() as u32);
        let (first_pointer, first_size) = (
            before_pointer.offset(before_size),
            std::mem::size_of::<TermPointer>() as u32,
        );
        let (after_pointer, after_size) = (
            first_pointer.offset(first_size),
            std::mem::size_of::<u32>() as u32,
        );
        let (second_pointer, _second_size) = (
            after_pointer.offset(after_size),
            std::mem::size_of::<TermPointer>() as u32,
        );

        assert_eq!(
            expression.iter().as_slice(),
            &[first_pointer, second_pointer]
        );
    }
}
