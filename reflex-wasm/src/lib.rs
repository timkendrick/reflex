// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{convert::TryFrom, hash::Hash, marker::PhantomData, ops::Deref};

use allocator::{ArenaAllocator, VecAllocator};
use hash::{TermHash, TermHashState, TermHasher, TermSize};

pub mod allocator;
pub mod hash;
pub mod parser;
pub mod stdlib;
pub mod term_type;

use reflex::{
    core::{
        ApplicationTermType, Arity, Builtin, Expression, ExpressionFactory, RefType, Uid, Uuid,
    },
    hash::HashId,
};
use term_type::*;

impl<'heap, A: ArenaAllocator> ExpressionFactory<ArenaRef<'heap, Term, A>> for &'heap A {
    fn create_let_term(
        &self,
        initializer: ArenaRef<'heap, Term, A>,
        body: ArenaRef<'heap, Term, A>,
    ) -> ArenaRef<'heap, Term, A> {
        let arena = *self;
        debug_assert!(initializer.allocator == arena && body.allocator == arena);
        let instance = self.allocate(LetTerm {
            initializer: initializer.into(),
            body: body.into(),
        });
        ArenaRef::new(arena, instance)
    }
    fn match_let_term<'a>(
        &self,
        expression: &'a ArenaRef<'heap, Term, A>,
    ) -> Option<&'a ArenaRef<'heap, TypedTerm<LetTerm>, A>> {
        let arena = *self;
        match &expression.as_deref().value {
            TermType::Let(_) => Some(unsafe { expression.transmute::<TypedTerm<LetTerm>>() }),
            _ => None,
        }
    }
}

struct ArenaRef<'a, T, A: ArenaAllocator> {
    pub(crate) arena: &'a A,
    value: &'a T,
}
impl<'a, T, A: ArenaAllocator> std::hash::Hash for ArenaRef<'a, T, A>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_deref().hash(state)
    }
}
impl<'a, T, A: ArenaAllocator> ArenaRef<'a, T, A> {
    fn new(arena: &'a VecAllocator, value: &'a T) -> Self {
        Self { arena, value }
    }
    unsafe fn transmute<T2>(&self) -> &ArenaRef<'a, T2, A> {
        self
    }
}
impl<'a, T, A: ArenaAllocator> RefType<'a, T> for ArenaRef<'a, T, A> {
    fn as_deref(&self) -> &'a T {
        self.value
    }
}
impl<'a, T, A: ArenaAllocator> Copy for ArenaRef<'a, T, A> {}
impl<'a, T, A: ArenaAllocator> Clone for ArenaRef<'a, T, A> {
    fn clone(&self) -> Self {
        Self {
            arena: self.arena,
            value: self.pointer,
        }
    }
}

struct IntoArenaRefIterator<'a, T: 'a, A: ArenaAllocator, TInner: Iterator<Item = TermPointer>> {
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
            _item: Default::default(),
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

impl<'heap, T: Expression, A: ArenaAllocator, TInner: ApplicationTermType<T>> ApplicationTermType<T>
    for ArenaRef<'heap, TypedTerm<TInner>, A>
where
    for<'a> T::Ref<'a, T>: From<ArenaRef<'a, Term, A>>,
    for<'a> T::Ref<'a, T::ExpressionList<T>>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn target<'a>(&'a self) -> T::Ref<'a, T>
    where
        T: 'a,
    {
        ArenaRef::new(self.arena, self.as_deref().get_inner()).target()
    }
    fn args<'a>(&'a self) -> T::Ref<'a, T::ExpressionList<T>>
    where
        T: 'a,
        T::ExpressionList<T>: 'a,
    {
        ArenaRef::new(self.arena, self.as_deref().get_inner()).args()
    }
}

struct ArenaPointer<'heap, T, A: ArenaAllocator> {
    arena: &'heap A,
    pointer: TermPointer,
    _type: PhantomData<T>,
}
impl<'heap, T, A: ArenaAllocator> ArenaPointer<'heap, T, A> {
    fn new(arena: &'heap A, pointer: TermPointer) -> Self {
        Self {
            arena,
            pointer,
            _type: Default::default(),
        }
    }
}
impl<'heap, T, A: ArenaAllocator> RefType<'heap, T> for ArenaPointer<'heap, T, A> {
    fn as_deref(&self) -> &'heap T {
        self.arena.get(self.pointer)
    }
}
impl<'heap, T, A: ArenaAllocator> std::hash::Hash for ArenaPointer<'heap, T, A>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_deref().hash(state)
    }
}
impl<'heap, T, A: ArenaAllocator> Copy for ArenaPointer<'heap, T, A> {}
impl<'heap, T, A: ArenaAllocator> Clone for ArenaPointer<'heap, T, A> {
    fn clone(&self) -> Self {
        Self {
            arena: self.arena,
            pointer: self.pointer,
            _type: Default::default(),
        }
    }
}
impl<'heap, T, A: ArenaAllocator> From<ArenaPointer<'heap, T, A>> for TermPointer {
    fn from(value: ArenaPointer<'heap, T, A>) -> Self {
        let ArenaPointer { pointer, .. } = value;
        pointer
    }
}

struct MyPointer(u32);
impl Deref for MyPointer {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'heap, T, A: ArenaAllocator> Deref for ArenaPointer<'heap, T, A> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.arena.get(self.pointer)
    }
}

#[repr(C)]
struct TypedTerm<V> {
    header: TermHeader,
    value: TermType,
    _type: PhantomData<V>,
}
impl<V> TypedTerm<V> {
    fn get_inner(&self) -> &V {
        unsafe { &self.value }
    }
    fn id(&self) -> HashId {
        self.header.hash
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
enum WasmStdlib {}
impl std::fmt::Display for WasmStdlib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
impl TryFrom<Uuid> for WasmStdlib {
    type Error = ();
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        Err(())
    }
}
impl Uid for WasmStdlib {
    fn uid(&self) -> reflex::core::Uuid {
        Uuid::new_v4()
    }
}
impl Builtin for WasmStdlib {
    fn arity(&self) -> reflex::core::Arity {
        Arity::lazy(0, 0, None)
    }
    fn apply<T: Expression<Builtin = Self> + reflex::core::Applicable<T>>(
        &self,
        args: impl ExactSizeIterator<Item = T>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl reflex::core::HeapAllocator<T>,
        cache: &mut impl reflex::core::EvaluationCache<T>,
    ) -> Result<T, String> {
        Err(Default::default())
    }
    fn should_parallelize<T: Expression<Builtin = Self> + reflex::core::Applicable<T>>(
        &self,
        args: &[T],
    ) -> bool {
        false
    }
}

impl<'heap, A: ArenaAllocator> Expression for ArenaRef<'heap, Term, A> {
    type String<T: Expression> = Self::StringTerm<T>;
    type Builtin = WasmStdlib;
    type Signal<T: Expression> = ArenaRef<'heap, TypedTerm<ConditionTerm>, A>;
    type SignalList<T: Expression> = ArenaRef<'heap, TypedTerm<TreeTerm>, A>;
    type StructPrototype<T: Expression> = Self::ListTerm<T>;
    type ExpressionList<T: Expression> = Self::ListTerm<T>;
    type NilTerm = ArenaRef<'heap, TypedTerm<NilTerm>, A>;
    type BooleanTerm = ArenaRef<'heap, TypedTerm<BooleanTerm>, A>;
    type IntTerm = ArenaRef<'heap, TypedTerm<IntTerm>, A>;
    type FloatTerm = ArenaRef<'heap, TypedTerm<FloatTerm>, A>;
    type StringTerm<T: Expression> = ArenaRef<'heap, TypedTerm<StringTerm>, A>;
    type SymbolTerm = ArenaRef<'heap, TypedTerm<SymbolTerm>, A>;
    type VariableTerm = ArenaRef<'heap, TypedTerm<VariableTerm>, A>;
    type EffectTerm<T: Expression> = ArenaRef<'heap, TypedTerm<EffectTerm>, A>;
    type LetTerm<T: Expression> = ArenaRef<'heap, TypedTerm<LetTerm>, A>;
    type LambdaTerm<T: Expression> = ArenaRef<'heap, TypedTerm<LambdaTerm>, A>;
    type ApplicationTerm<T: Expression> = ArenaRef<'heap, TypedTerm<ApplicationTerm>, A>;
    type PartialApplicationTerm<T: Expression> = ArenaRef<'heap, TypedTerm<PartialTerm>, A>;
    // FIXME: implement recursive term type
    type RecursiveTerm<T: Expression> = ArenaRef<'heap, TypedTerm<NilTerm>, A>;
    type BuiltinTerm<T: Expression> = ArenaRef<'heap, TypedTerm<BuiltinTerm>, A>;
    type CompiledFunctionTerm<T: Expression> = ArenaRef<'heap, TypedTerm<CompiledTerm>, A>;
    type RecordTerm<T: Expression> = ArenaRef<'heap, TypedTerm<RecordTerm>, A>;
    type ConstructorTerm<T: Expression> = ArenaRef<'heap, TypedTerm<ConstructorTerm>, A>;
    type ListTerm<T: Expression> = ArenaRef<'heap, TypedTerm<ListTerm>, A>;
    type HashmapTerm<T: Expression> = ArenaRef<'heap, TypedTerm<HashmapTerm>, A>;
    type HashsetTerm<T: Expression> = ArenaRef<'heap, TypedTerm<HashsetTerm>, A>;
    type SignalTerm<T: Expression> = ArenaRef<'heap, TypedTerm<SignalTerm>, A>;

    type Ref<'a, TTarget> = ArenaRef<'a, TTarget, A> where TTarget: 'a, Self: 'a;

    fn id(&self) -> HashId {
        self.as_deref().id()
    }
}

impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, Term, A> {}
impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, Term, A> {
    fn eq(&self, other: &Self) -> bool {
        ArenaRef::new(self.arena, &self.as_deref().value)
            == ArenaRef::new(other.arena, &other.as_deref().value)
    }
}
impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, Term, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&ArenaRef::new(self.arena, &self.as_deref().value), f)
    }
}
impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, Term, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&ArenaRef::new(self.arena, &self.as_deref().value), f)
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
    pub fn id(&mut self) -> HashId {
        self.header.hash
    }
    pub fn get_value_pointer(term: TermPointer) -> TermPointer {
        term.offset(std::mem::size_of::<TermHeader>() as u32)
    }
    pub fn as_bytes(&self) -> &[u32] {
        let num_words = pad_to_4_byte_offset(self.size() as usize) / 4;
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u32, num_words) }
    }
    pub(crate) fn type_id(&self) -> TermTypeDiscriminants {
        TermTypeDiscriminants::from(&self.value)
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
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
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
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let target: &Term = arena.get(*self);
        target.hash(hasher, arena)
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
    fn size(&self) -> usize {
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
        self.as_deref().len()
    }
    pub fn iter(&self) -> ArrayIter<T> {
        self.as_deref().iter()
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_deref().get(index)
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
