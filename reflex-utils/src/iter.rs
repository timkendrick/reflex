// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::marker::PhantomData;

#[derive(Debug)]
pub struct WithExactSizeIterator<T: Iterator> {
    remaining: usize,
    iter: T,
}
impl<T: Iterator> WithExactSizeIterator<T> {
    pub fn new(len: usize, iter: T) -> Self {
        Self {
            remaining: len,
            iter,
        }
    }
}
impl<T: Iterator> Clone for WithExactSizeIterator<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            remaining: self.remaining,
            iter: self.iter.clone(),
        }
    }
}
impl<T: Iterator> Iterator for WithExactSizeIterator<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            self.iter.next()
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = ExactSizeIterator::len(self);
        (len, Some(len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let len = ExactSizeIterator::len(&self);
        len
    }
}
impl<T: Iterator> ExactSizeIterator for WithExactSizeIterator<T> {
    fn len(&self) -> usize {
        self.remaining
    }
}

pub struct MapIntoIterator<T: Iterator<Item = T1>, T1, T2: From<T1>> {
    inner: T,
    _from: PhantomData<T1>,
    _to: PhantomData<T2>,
}

impl<T: Iterator<Item = T1>, T1, T2: From<T1>> MapIntoIterator<T, T1, T2> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _from: PhantomData,
            _to: PhantomData,
        }
    }
}

impl<T: Iterator<Item = T1>, T1, T2: From<T1>> Clone for MapIntoIterator<T, T1, T2>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _from: PhantomData,
            _to: PhantomData,
        }
    }
}

impl<T, T1, T2> Iterator for MapIntoIterator<T, T1, T2>
where
    T: Iterator<Item = T1>,
    T2: From<T1>,
{
    type Item = T2;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Into::into)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T, T1, T2> ExactSizeIterator for MapIntoIterator<T, T1, T2>
where
    T: Iterator<Item = T1> + ExactSizeIterator,
    T2: From<T1>,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}
