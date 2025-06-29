// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{marker::PhantomData, ops::Deref};

use metrics::{describe_counter, increment_counter, Unit};
use reflex_dispatcher::{Action, Named, ProcessId, TaskFactory};
use reflex_scheduler::tokio::{
    AsyncMessage, AsyncMessageTimestamp, TokioCommand, TokioSchedulerLogger,
};

use crate::logger::ActionLogger;

#[derive(Clone, Copy, Debug)]
pub struct PrometheusLoggerMetricNames {
    pub total_action_count: &'static str,
}
impl PrometheusLoggerMetricNames {
    fn init(self) -> Self {
        describe_counter!(self.total_action_count, Unit::Count, "Total action count");
        self
    }
}
impl Default for PrometheusLoggerMetricNames {
    fn default() -> Self {
        Self {
            total_action_count: "total_action_count",
        }
    }
}

pub trait PrometheusLoggerAction: Named {}
impl<_Self> PrometheusLoggerAction for _Self where Self: Named {}

#[derive(Clone)]
pub struct PrometheusLogger<TAction: Action, TTask: TaskFactory<TAction, TTask>> {
    metric_names: PrometheusLoggerMetricNames,
    _action: PhantomData<TAction>,
    _task: PhantomData<TTask>,
}
impl<TAction: Action + PrometheusLoggerAction, TTask: TaskFactory<TAction, TTask>>
    PrometheusLogger<TAction, TTask>
{
    pub fn new(metric_names: PrometheusLoggerMetricNames) -> Self {
        Self {
            metric_names: metric_names.init(),
            _action: PhantomData,
            _task: PhantomData,
        }
    }
    fn log(&self, action: &TAction) {
        let metric_labels = [("type", action.name())];
        increment_counter!(self.metric_names.total_action_count, &metric_labels);
    }
}
impl<TAction, TTask> ActionLogger for PrometheusLogger<TAction, TTask>
where
    TAction: Action + PrometheusLoggerAction,
    TTask: TaskFactory<TAction, TTask>,
{
    type Action = TAction;
    fn log(&mut self, action: &Self::Action) {
        PrometheusLogger::log(self, action);
    }
}
impl<TAction, TTask> TokioSchedulerLogger for PrometheusLogger<TAction, TTask>
where
    TAction: Action + PrometheusLoggerAction,
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
                if message.redispatched_from().is_none() {
                    let action = message.deref();
                    PrometheusLogger::log(self, action)
                }
            }
            _ => {}
        }
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
