// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::marker::PhantomData;

pub trait EventSink {
    type Event;
    fn emit(&mut self, event: &Self::Event);
}

#[derive(Debug, Clone, Copy)]
pub struct NoopEventSink<T>(PhantomData<T>);

impl<T> Default for NoopEventSink<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> EventSink for NoopEventSink<T> {
    type Event = T;
    fn emit(&mut self, _event: &Self::Event) {}
}

impl<TInner, T> EventSink for Option<TInner>
where
    TInner: EventSink<Event = T>,
{
    type Event = T;
    fn emit(&mut self, event: &Self::Event) {
        if let Some(inner) = self {
            inner.emit(event);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EitherEventSink<T1, T2> {
    Left(T1),
    Right(T2),
}

impl<T1, T2, T> EventSink for EitherEventSink<T1, T2>
where
    T1: EventSink<Event = T>,
    T2: EventSink<Event = T>,
{
    type Event = T;
    fn emit(&mut self, event: &Self::Event) {
        match self {
            Self::Left(inner) => inner.emit(event),
            Self::Right(inner) => inner.emit(event),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChainEventSink<T1, T2> {
    left: T1,
    right: T2,
}

impl<T1, T2, T> EventSink for ChainEventSink<T1, T2>
where
    T1: EventSink<Event = T>,
    T2: EventSink<Event = T>,
{
    type Event = T;
    fn emit(&mut self, event: &Self::Event) {
        self.left.emit(event);
        self.right.emit(event);
    }
}
