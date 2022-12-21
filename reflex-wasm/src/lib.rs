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
pub mod hash;
pub mod parser;
pub mod stdlib;
pub mod term_type;

// impl<'heap, A: ArenaAllocator> ExpressionFactory<ArenaRef<'heap, Term, A>> for &'heap A {
//     fn create_let_term(
//         &self,
//         initializer: ArenaRef<'heap, Term, A>,
//         body: ArenaRef<'heap, Term, A>,
//     ) -> ArenaRef<'heap, Term, A> {
//         let arena = *self;
//         debug_assert!(initializer.allocator == arena && body.allocator == arena);
//         let instance = self.allocate(LetTerm {
//             initializer: initializer.into(),
//             body: body.into(),
//         });
//         ArenaRef::new(arena, instance)
//     }
//     fn match_let_term<'a>(
//         &self,
//         expression: &'a ArenaRef<'heap, Term, A>,
//     ) -> Option<&'a ArenaRef<'heap, TypedTerm<LetTerm>, A>> {
//         let arena = *self;
//         match &expression.as_deref().value {
//             TermType::Let(_) => Some(unsafe { expression.transmute::<TypedTerm<LetTerm>>() }),
//             _ => None,
//         }
//     }
// }

pub struct ArenaRef<'a, T, A: ArenaAllocator> {
    pub(crate) arena: &'a A,
    value: &'a T,
}
impl<'a, T, A: ArenaAllocator> std::hash::Hash for ArenaRef<'a, T, A>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_value().hash(state)
    }
}
impl<'a, T, A: ArenaAllocator> ArenaRef<'a, T, A> {
    fn new(arena: &'a A, value: &'a T) -> Self {
        Self { arena, value }
    }
    pub fn as_value(&self) -> &'a T {
        &self.value
    }
}
impl<'a, T, A: ArenaAllocator> Copy for ArenaRef<'a, T, A> {}
impl<'a, T, A: ArenaAllocator> Clone for ArenaRef<'a, T, A> {
    fn clone(&self) -> Self {
        Self {
            arena: self.arena,
            value: self.value,
        }
    }
}

impl<'a, 'heap: 'a, T, A: ArenaAllocator> From<&'a ArenaRef<'heap, T, A>>
    for ArenaRef<'heap, T, A>
{
    fn from(value: &'a ArenaRef<'heap, T, A>) -> Self {
        *value
    }
}

impl<'a, 'heap: 'a, T, A: ArenaAllocator> RefType<'a, Self> for ArenaRef<'heap, T, A> {
    fn as_deref(&self) -> &'a Self {
        // This is safe because we know 'heap lasts longer than 'a, which ensures the reference will be freed before 'heap
        unsafe { std::mem::transmute::<&Self, &'a Self>(self) }
    }
}

pub struct IntoArenaRefIterator<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>>
{
    arena: &'a A,
    inner: TInner,
    _item: PhantomData<T>,
}
impl<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>>
    IntoArenaRefIterator<'a, T, A, TInner>
{
    fn new(arena: &'a A, inner: TInner) -> Self {
        Self {
            arena,
            inner,
            _item: PhantomData,
        }
    }
}
impl<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>> Iterator
    for IntoArenaRefIterator<'a, T, A, TInner>
{
    type Item = ArenaRef<'a, T, A>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|pointer| ArenaRef::new(self.arena, self.arena.get(pointer)))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>> ExactSizeIterator
    for IntoArenaRefIterator<'a, T, A, TInner>
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
    pub(crate) fn as_value(&self) -> &TermType {
        &self.value
    }
    pub(crate) fn set_hash(&mut self, value: TermHashState) {
        self.header.hash = value;
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
        let target: &Term = arena.get(*self);
        target.hash(hasher, arena)
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
        arena: &mut impl ArenaAllocator,
    ) {
        let items = items.into_iter();
        let num_items = items.len();
        let capacity = num_items as u32;
        let length = num_items as u32;
        let items_offset = list.offset(std::mem::size_of::<Array<T>>() as u32);
        *arena.get_mut::<u32>(list) = capacity;
        *arena.get_mut::<u32>(list.offset(std::mem::size_of::<u32>() as u32)) = length;
        arena.extend(items_offset, num_items * std::mem::size_of::<T>());
        let array_offset = u32::from(items_offset);
        for (index, item) in items.enumerate() {
            *arena.get_mut(TermPointer::from(
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
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() + ((self.capacity as usize) * std::mem::size_of::<T>())
    }
}
impl<T> TermHash for Array<T>
where
    T: Copy + TermHash,
{
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        self.iter()
            .fold(hasher.write_u32(self.length), |hasher, item| {
                item.hash(hasher, arena)
            })
    }
}

impl<'heap, T, A: ArenaAllocator> ArenaRef<'heap, Array<T>, A> {
    pub fn len(&self) -> usize {
        self.as_value().len()
    }
    pub fn iter(&self) -> ArrayIter<'heap, T> {
        self.as_value().iter()
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_value().get(index)
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
