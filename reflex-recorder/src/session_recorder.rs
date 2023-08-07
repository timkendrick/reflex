// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{marker::PhantomData, ops::Deref};

use reflex_dispatcher::{Action, ProcessId, TaskFactory};
use reflex_scheduler::tokio::{
    AsyncMessage, AsyncMessageTimestamp, TokioCommand, TokioSchedulerLogger,
};
use reflex_utils::event::EventSink;

pub struct SessionRecorder<TRecorder, TAction, TTask>
where
    TRecorder: EventSink<Event = TAction>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    recorder: TRecorder,
    _action: PhantomData<TAction>,
    _task: PhantomData<TTask>,
}

impl<TRecorder, TAction, TTask> Clone for SessionRecorder<TRecorder, TAction, TTask>
where
    TRecorder: EventSink<Event = TAction> + Clone,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn clone(&self) -> Self {
        Self {
            recorder: self.recorder.clone(),
            _action: PhantomData,
            _task: PhantomData,
        }
    }
}

impl<TRecorder, TAction, TTask> SessionRecorder<TRecorder, TAction, TTask>
where
    TRecorder: EventSink<Event = TAction>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    pub fn new(recorder: TRecorder) -> Self {
        Self {
            recorder,
            _action: PhantomData,
            _task: PhantomData,
        }
    }
}

impl<TRecorder, TAction, TTask> TokioSchedulerLogger for SessionRecorder<TRecorder, TAction, TTask>
where
    TRecorder: EventSink<Event = TAction>,
    TAction: Action + Clone + 'static,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        _enqueue_time: AsyncMessageTimestamp,
    ) {
        match command {
            TokioCommand::Send { pid: _, message } => {
                let action = message.deref();
                self.recorder.emit(action)
            }
            _ => {}
        }
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        _actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        _pid: ProcessId,
    ) {
        if let None = message.redispatched_from() {
            let action = message.deref();
            self.recorder.emit(action)
        }
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, _pid: ProcessId) {
        if let None = message.redispatched_from() {
            let action = message.deref();
            self.recorder.emit(action)
        }
    }
}
