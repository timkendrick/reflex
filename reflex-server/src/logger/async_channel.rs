// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use futures::Future;
use reflex_utils::event::EventSink;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct AsyncChannelEventSink<T> {
    events_tx: mpsc::UnboundedSender<T>,
}

impl<T> Clone for AsyncChannelEventSink<T> {
    fn clone(&self) -> Self {
        Self {
            events_tx: self.events_tx.clone(),
        }
    }
}

impl<T> AsyncChannelEventSink<T> {
    pub fn create(inner: impl EventSink<Event = T> + Send) -> (Self, impl Future<Output = ()>) {
        let (events_tx, mut events_rx) = mpsc::unbounded_channel::<T>();
        let task = {
            let mut inner = inner;
            async move {
                while let Some(event) = events_rx.recv().await {
                    inner.emit(&event);
                }
            }
        };
        (Self { events_tx }, task)
    }
}

impl<T> EventSink for AsyncChannelEventSink<T>
where
    T: Clone + Send + 'static,
{
    type Event = T;
    fn emit(&mut self, event: &Self::Event) {
        let _ = self.events_tx.send(event.clone());
    }
}
