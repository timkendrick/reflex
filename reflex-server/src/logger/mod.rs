// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    marker::PhantomData,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use reflex_dispatcher::{Action, ProcessId, TaskFactory};
use reflex_scheduler::tokio::{
    AsyncMessage, AsyncMessageTimestamp, TokioCommand, TokioSchedulerLogger,
};
use reflex_utils::event::EventSink;

pub mod async_channel;
pub mod formatted;
pub mod formatter;
pub mod json;
pub mod messages;
pub mod prometheus;

pub trait ActionLogger {
    type Action: Action;
    fn log(&mut self, action: &Self::Action);
}

#[derive(Copy, Debug)]
pub struct NoopLogger<TAction, TTask>
where
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    _action: PhantomData<TAction>,
    _task: PhantomData<TTask>,
}
impl<TAction, TTask> Clone for NoopLogger<TAction, TTask>
where
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn clone(&self) -> Self {
        Self {
            _action: PhantomData,
            _task: PhantomData,
        }
    }
}
impl<TAction, TTask> Default for NoopLogger<TAction, TTask>
where
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn default() -> Self {
        Self {
            _action: PhantomData,
            _task: PhantomData,
        }
    }
}
impl<TAction, TTask> ActionLogger for NoopLogger<TAction, TTask>
where
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    fn log(&mut self, _action: &Self::Action) {}
}
impl<TAction, TTask> TokioSchedulerLogger for NoopLogger<TAction, TTask>
where
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        _command: &TokioCommand<Self::Action, Self::Task>,
        _enqueue_time: AsyncMessageTimestamp,
    ) {
    }
    fn log_worker_message(
        &mut self,
        _message: &AsyncMessage<Self::Action>,
        _actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        _pid: ProcessId,
    ) {
    }
    fn log_task_message(&mut self, _message: &AsyncMessage<Self::Action>, _pid: ProcessId) {}
}

impl<T, TAction> ActionLogger for Option<T>
where
    T: ActionLogger<Action = TAction>,
    TAction: Action,
{
    type Action = TAction;
    fn log(&mut self, action: &Self::Action) {
        if let Some(inner) = self {
            inner.log(action);
        }
    }
}

#[derive(Debug)]
pub enum EitherLogger<T1, T2> {
    Left(T1),
    Right(T2),
}
impl<T1, T2> Clone for EitherLogger<T1, T2>
where
    T1: Clone,
    T2: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Left(inner) => Self::Left(inner.clone()),
            Self::Right(inner) => Self::Right(inner.clone()),
        }
    }
}
impl<T1, T2, TAction> ActionLogger for EitherLogger<T1, T2>
where
    T1: ActionLogger<Action = TAction>,
    T2: ActionLogger<Action = TAction>,
    TAction: Action,
{
    type Action = TAction;
    fn log(&mut self, action: &Self::Action) {
        match self {
            Self::Left(logger) => logger.log(action),
            Self::Right(logger) => logger.log(action),
        }
    }
}
impl<T1, T2, TAction, TTask> TokioSchedulerLogger for EitherLogger<T1, T2>
where
    T1: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    T2: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        enqueue_time: AsyncMessageTimestamp,
    ) {
        match self {
            Self::Left(inner) => inner.log_scheduler_command(command, enqueue_time),
            Self::Right(inner) => inner.log_scheduler_command(command, enqueue_time),
        }
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        pid: ProcessId,
    ) {
        match self {
            Self::Left(inner) => inner.log_worker_message(message, actor, pid),
            Self::Right(inner) => inner.log_worker_message(message, actor, pid),
        }
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, pid: ProcessId) {
        match self {
            Self::Left(inner) => inner.log_task_message(message, pid),
            Self::Right(inner) => inner.log_task_message(message, pid),
        }
    }
}

#[derive(Debug)]
pub struct ChainLogger<T1, T2> {
    left: T1,
    right: T2,
}
impl<T1, T2> ChainLogger<T1, T2> {
    pub fn new(left: T1, right: T2) -> Self {
        Self { left, right }
    }
}
impl<T1, T2> Clone for ChainLogger<T1, T2>
where
    T1: Clone,
    T2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}
impl<T1, T2, TAction> ActionLogger for ChainLogger<T1, T2>
where
    T1: ActionLogger<Action = TAction>,
    T2: ActionLogger<Action = TAction>,
    TAction: Action,
{
    type Action = TAction;
    fn log(&mut self, action: &Self::Action) {
        self.left.log(action);
        self.right.log(action);
    }
}
impl<T1, T2, TAction, TTask> TokioSchedulerLogger for ChainLogger<T1, T2>
where
    T1: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    T2: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        enqueue_time: AsyncMessageTimestamp,
    ) {
        self.left.log_scheduler_command(command, enqueue_time);
        self.right.log_scheduler_command(command, enqueue_time);
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        pid: ProcessId,
    ) {
        self.left.log_worker_message(message, actor, pid);
        self.right.log_worker_message(message, actor, pid);
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, pid: ProcessId) {
        self.left.log_task_message(message, pid);
        self.right.log_task_message(message, pid);
    }
}

#[derive(Debug)]
pub struct SharedLogger<TInner> {
    inner: Arc<Mutex<TInner>>,
}
impl<TInner> SharedLogger<TInner> {
    pub fn new(inner: TInner) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}
impl<TInner> Clone for SharedLogger<TInner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<TInner> EventSink for SharedLogger<TInner>
where
    TInner: EventSink,
{
    type Event = TInner::Event;
    fn emit(&mut self, event: &Self::Event) {
        match self.inner.lock() {
            Ok(mut inner) => inner.deref_mut().emit(event),
            Err(_) => {}
        };
    }
}
impl<TInner, TAction> ActionLogger for SharedLogger<TInner>
where
    TInner: ActionLogger<Action = TAction>,
    TAction: Action,
{
    type Action = TAction;
    fn log(&mut self, action: &Self::Action) {
        match self.inner.lock() {
            Ok(mut inner) => inner.deref_mut().log(action),
            Err(_) => {}
        };
    }
}
impl<TInner, TAction, TTask> TokioSchedulerLogger for SharedLogger<TInner>
where
    TInner: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        enqueue_time: AsyncMessageTimestamp,
    ) {
        match self.inner.lock() {
            Ok(mut inner) => {
                inner
                    .deref_mut()
                    .log_scheduler_command(command, enqueue_time);
            }
            Err(_) => {}
        }
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        pid: ProcessId,
    ) {
        match self.inner.lock() {
            Ok(mut inner) => {
                inner.deref_mut().log_worker_message(message, actor, pid);
            }
            Err(_) => {}
        }
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, pid: ProcessId) {
        match self.inner.lock() {
            Ok(mut inner) => {
                inner.deref_mut().log_task_message(message, pid);
            }
            Err(_) => {}
        }
    }
}

pub trait FilteredLoggerPredicate<TAction: Action, TTask: TaskFactory<TAction, TTask>> {
    fn accept_scheduler_command(&self, command: &TokioCommand<TAction, TTask>) -> bool;
    fn accept_worker_message(
        &self,
        message: &AsyncMessage<TAction>,
        actor: &TTask::Actor,
        pid: ProcessId,
    ) -> bool;
    fn accept_task_message(&self, message: &AsyncMessage<TAction>, pid: ProcessId) -> bool;
}

#[derive(Debug)]
pub struct FilteredLogger<T, TPredicate, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TPredicate: FilteredLoggerPredicate<TAction, TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    inner: T,
    predicate: TPredicate,
}
impl<T, TPredicate, TAction, TTask> Clone for FilteredLogger<T, TPredicate, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask> + Clone,
    TPredicate: FilteredLoggerPredicate<TAction, TTask> + Clone,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            predicate: self.predicate.clone(),
        }
    }
}
impl<T, TPredicate, TAction, TTask> FilteredLogger<T, TPredicate, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TPredicate: FilteredLoggerPredicate<TAction, TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    pub fn new(inner: T, predicate: TPredicate) -> Self {
        Self { inner, predicate }
    }
}
impl<T, TPredicate, TAction, TTask> TokioSchedulerLogger
    for FilteredLogger<T, TPredicate, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TPredicate: FilteredLoggerPredicate<TAction, TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        enqueue_time: AsyncMessageTimestamp,
    ) {
        if self.predicate.accept_scheduler_command(command) {
            self.inner.log_scheduler_command(command, enqueue_time);
        }
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        pid: ProcessId,
    ) {
        if self.predicate.accept_worker_message(message, actor, pid) {
            self.inner.log_worker_message(message, actor, pid);
        }
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, pid: ProcessId) {
        if self.predicate.accept_task_message(message, pid) {
            self.inner.log_task_message(message, pid);
        }
    }
}

#[derive(Debug)]
pub struct SkipRedispatchedActionsLogger<TInner, TAction, TTask>(
    FilteredLogger<TInner, SkipRedispatchedActionsLoggerPredicate, TAction, TTask>,
)
where
    TInner: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>;
impl<T, TAction, TTask> SkipRedispatchedActionsLogger<T, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    pub fn new(inner: T) -> Self {
        Self(FilteredLogger::new(
            inner,
            SkipRedispatchedActionsLoggerPredicate,
        ))
    }
}
impl<T, TAction, TTask> Clone for SkipRedispatchedActionsLogger<T, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask> + Clone,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn clone(&self) -> Self {
        let Self(inner) = self;
        Self(inner.clone())
    }
}
impl<T, TAction, TTask> TokioSchedulerLogger for SkipRedispatchedActionsLogger<T, TAction, TTask>
where
    T: TokioSchedulerLogger<Action = TAction, Task = TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        enqueue_time: AsyncMessageTimestamp,
    ) {
        let Self(inner) = self;
        inner.log_scheduler_command(command, enqueue_time)
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        pid: ProcessId,
    ) {
        let Self(inner) = self;
        inner.log_worker_message(message, actor, pid)
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, pid: ProcessId) {
        let Self(inner) = self;
        inner.log_task_message(message, pid)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct SkipRedispatchedActionsLoggerPredicate;
impl<TAction, TTask> FilteredLoggerPredicate<TAction, TTask>
    for SkipRedispatchedActionsLoggerPredicate
where
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn accept_scheduler_command(&self, command: &TokioCommand<TAction, TTask>) -> bool {
        match command {
            TokioCommand::Send { pid: _, message } => message.redispatched_from().is_none(),
            _ => true,
        }
    }
    fn accept_worker_message(
        &self,
        message: &AsyncMessage<TAction>,
        _actor: &TTask::Actor,
        _pid: ProcessId,
    ) -> bool {
        match message.redispatched_from() {
            Some(_) => false,
            None => true,
        }
    }
    fn accept_task_message(&self, message: &AsyncMessage<TAction>, _pid: ProcessId) -> bool {
        match message.redispatched_from() {
            Some(_) => false,
            None => true,
        }
    }
}

impl<TInner, TAction, TTask> FilteredLoggerPredicate<TAction, TTask> for Option<TInner>
where
    TInner: FilteredLoggerPredicate<TAction, TTask>,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
{
    fn accept_scheduler_command(&self, command: &TokioCommand<TAction, TTask>) -> bool {
        match self {
            Some(inner) => inner.accept_scheduler_command(command),
            None => true,
        }
    }
    fn accept_worker_message(
        &self,
        message: &AsyncMessage<TAction>,
        actor: &TTask::Actor,
        pid: ProcessId,
    ) -> bool {
        match self {
            Some(inner) => inner.accept_worker_message(message, actor, pid),
            None => true,
        }
    }
    fn accept_task_message(&self, message: &AsyncMessage<TAction>, pid: ProcessId) -> bool {
        match self {
            Some(inner) => inner.accept_task_message(message, pid),
            None => true,
        }
    }
}
