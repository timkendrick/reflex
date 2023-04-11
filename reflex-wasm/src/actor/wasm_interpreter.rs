// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    iter::once,
    marker::PhantomData,
    sync::Arc,
};

use metrics::{describe_gauge, describe_histogram, SharedString, Unit};
use reflex::{
    core::{
        ConditionListType, ConditionType, EvaluationResult, Expression, ExpressionFactory,
        HeapAllocator, RefType, SignalTermType, SignalType, StateToken,
    },
    hash::IntMap,
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, MessageOffset, NoopDisposeCallback,
    ProcessId, SchedulerCommand, SchedulerMode, SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_macros::{dispatcher, Named};
use reflex_runtime::{
    action::{
        bytecode_interpreter::{
            BytecodeInterpreterEvaluateAction, BytecodeInterpreterGcAction,
            BytecodeInterpreterGcCompleteAction, BytecodeInterpreterInitAction,
            BytecodeInterpreterResultAction, BytecodeWorkerStatistics,
        },
        evaluate::{
            EvaluateResultAction, EvaluateStartAction, EvaluateStopAction, EvaluateUpdateAction,
        },
    },
    actor::bytecode_interpreter::BytecodeInterpreterMetricLabels,
    utils::quantiles::{
        generate_quantile_metric_labels, publish_quantile_bucketed_metric, QuantileBucket,
    },
    QueryInvalidationStrategy,
};

use crate::{
    cli::compile::WasmProgram,
    task::wasm_worker::{WasmWorkerMetricNames, WasmWorkerTask, WasmWorkerTaskFactory},
};

// TODO: Allow tweaking bytecode interpreter GC trigger
const MAX_UPDATES_WITHOUT_GC: usize = 3;

#[derive(Clone, Copy, Debug)]
pub struct WasmInterpreterMetricNames {
    pub query_worker_compile_duration: &'static str,
    pub query_worker_evaluate_duration: &'static str,
    pub query_worker_gc_duration: &'static str,
    pub query_worker_state_dependency_count: &'static str,
    pub query_worker_evaluation_cache_entry_count: &'static str,
    pub query_worker_evaluation_cache_deep_size: &'static str,
}
impl WasmInterpreterMetricNames {
    pub fn init(self) -> Self {
        describe_histogram!(
            self.query_worker_compile_duration,
            Unit::Seconds,
            "Worker query compilation duration (seconds)"
        );
        describe_histogram!(
            self.query_worker_evaluate_duration,
            Unit::Seconds,
            "Worker query evaluation duration (seconds)"
        );
        describe_histogram!(
            self.query_worker_gc_duration,
            Unit::Seconds,
            "Worker garbage collection duration (seconds)"
        );
        describe_gauge!(
            self.query_worker_state_dependency_count,
            Unit::Count,
            "The number of state dependencies for the most recent worker result"
        );
        describe_gauge!(
            self.query_worker_evaluation_cache_entry_count,
            Unit::Count,
            "The number of entries in the query worker evaluation cache"
        );
        describe_gauge!(
            self.query_worker_evaluation_cache_deep_size,
            Unit::Count,
            "A full count of the number of graph nodes in all entries in the query worker evaluation cache"
        );
        self
    }
}
impl Default for WasmInterpreterMetricNames {
    fn default() -> Self {
        Self {
            query_worker_compile_duration: "query_worker_compile_duration",
            query_worker_evaluate_duration: "query_worker_evaluate_duration",
            query_worker_gc_duration: "query_worker_gc_duration",
            query_worker_state_dependency_count: "query_worker_state_dependency_count",
            query_worker_evaluation_cache_deep_size: "query_worker_evaluation_cache_deep_size",
            query_worker_evaluation_cache_entry_count: "query_worker_evaluation_cache_entry_count",
        }
    }
}

#[derive(Named, Clone)]
pub struct WasmInterpreter<T, TFactory, TAllocator, TMetricLabels>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
    TMetricLabels: BytecodeInterpreterMetricLabels,
{
    program: Arc<WasmProgram>,
    graph_root_factory_export_name: String,
    factory: TFactory,
    _allocator: TAllocator,
    metric_names: WasmInterpreterMetricNames,
    get_worker_metric_labels: TMetricLabels,
    main_pid: ProcessId,
    _expression: PhantomData<T>,
}
impl<T, TFactory, TAllocator, TMetricLabels> WasmInterpreter<T, TFactory, TAllocator, TMetricLabels>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
    TMetricLabels: BytecodeInterpreterMetricLabels,
{
    pub fn new(
        program: WasmProgram,
        graph_root_factory_export_name: impl Into<String>,
        factory: TFactory,
        allocator: TAllocator,
        metric_names: WasmInterpreterMetricNames,
        get_worker_metric_labels: TMetricLabels,
        main_pid: ProcessId,
    ) -> Self {
        Self {
            program: Arc::new(program),
            graph_root_factory_export_name: graph_root_factory_export_name.into(),
            factory,
            _allocator: allocator,
            metric_names: metric_names.init(),
            get_worker_metric_labels,
            main_pid,
            _expression: Default::default(),
        }
    }
}
pub struct WasmInterpreterState<T: Expression> {
    // TODO: Use newtypes for state hashmap keys
    workers: IntMap<StateToken, WasmInterpreterWorkerState<T>>,
    grouped_worker_metrics: HashMap<String, WorkerMetricsState<T>>,
}
impl<T: Expression> Default for WasmInterpreterState<T> {
    fn default() -> Self {
        Self {
            workers: Default::default(),
            grouped_worker_metrics: Default::default(),
        }
    }
}
impl<T, TFactory, TAllocator, TMetricLabels, TAction, TTask> TaskFactory<TAction, TTask>
    for WasmInterpreter<T, TFactory, TAllocator, TMetricLabels>
where
    T: Expression,
    TFactory: ExpressionFactory<T> + Default,
    TAllocator: HeapAllocator<T> + Default,
    TMetricLabels: BytecodeInterpreterMetricLabels,
    TAction: Action + WasmInterpreterAction<T>,
    TTask: TaskFactory<TAction, TTask> + WasmWorkerTask<T, TFactory, TAllocator>,
{
    type Actor = Self;
    fn create(self) -> Self::Actor {
        self
    }
}

struct WorkerMetricsState<T: Expression> {
    active_workers: IntMap<StateToken, T::Signal>,
    quantile_metric_labels: [Vec<(SharedString, SharedString)>; NUM_QUANTILE_BUCKETS],
}

const NUM_QUANTILE_BUCKETS: usize = 4;
const QUANTILE_BUCKETS: [QuantileBucket; NUM_QUANTILE_BUCKETS] = [
    QuantileBucket(0.5),
    QuantileBucket(0.9),
    QuantileBucket(0.99),
    QuantileBucket(1.0),
];

struct WasmInterpreterWorkerState<T: Expression> {
    pid: ProcessId,
    label: String,
    state_index: Option<MessageOffset>,
    status: WasmInterpreterWorkerStatus<T>,
    invalidation_strategy: QueryInvalidationStrategy,
    updates_since_gc: usize,
    metrics: BytecodeWorkerStatistics,
}

enum WasmInterpreterWorkerStatus<T: Expression> {
    Working(WasmWorkerUpdateQueue<T>),
    Idle,
}

enum WasmWorkerUpdateQueue<T: Expression> {
    Single(SingleWorkerUpdateQueue<T>),
    Combined(CombinedWorkerUpdateQueue<T>),
    Exact(ExactWorkerUpdateQueue<T>),
}
impl<T: Expression> WasmWorkerUpdateQueue<T> {
    fn new(invalidation_strategy: QueryInvalidationStrategy) -> Self {
        match invalidation_strategy {
            QueryInvalidationStrategy::CombineUpdateBatches => Self::Single(Default::default()),
            QueryInvalidationStrategy::Exact => Self::Exact(Default::default()),
        }
    }
    fn push_update_batch(&mut self, updates: &Vec<(T::Signal, T)>) {
        match self {
            Self::Single(queue) => {
                let initial_batch = std::mem::take(&mut queue.updates);
                *self = Self::Combined(CombinedWorkerUpdateQueue::from_iter(
                    initial_batch.into_iter().chain(
                        updates
                            .iter()
                            .map(|(key, value)| (key.clone(), value.clone())),
                    ),
                ))
            }
            Self::Combined(queue) => {
                queue.extend(
                    updates
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone())),
                );
            }
            Self::Exact(queue) => queue.batches.push_back(updates.clone()),
        }
    }
    fn pop_update_batch(&mut self) -> Vec<(T::Signal, T)> {
        match self {
            Self::Single(queue) => std::mem::take(&mut queue.updates),
            Self::Combined(queue) => std::mem::take(&mut queue.updates).into_values().collect(),
            Self::Exact(queue) => queue.batches.pop_front().unwrap_or_default(),
        }
    }
}

struct SingleWorkerUpdateQueue<T: Expression> {
    updates: Vec<(T::Signal, T)>,
}
impl<T: Expression> Default for SingleWorkerUpdateQueue<T> {
    fn default() -> Self {
        Self {
            updates: Default::default(),
        }
    }
}

struct CombinedWorkerUpdateQueue<T: Expression> {
    updates: IntMap<StateToken, (T::Signal, T)>,
}
impl<T: Expression> Default for CombinedWorkerUpdateQueue<T> {
    fn default() -> Self {
        Self {
            updates: Default::default(),
        }
    }
}
impl<T: Expression> FromIterator<(T::Signal, T)> for CombinedWorkerUpdateQueue<T> {
    fn from_iter<TIter: IntoIterator<Item = (T::Signal, T)>>(iter: TIter) -> Self {
        Self {
            updates: iter
                .into_iter()
                .map(|(key, value)| (key.id(), (key, value)))
                .collect(),
        }
    }
}
impl<T: Expression> IntoIterator for CombinedWorkerUpdateQueue<T> {
    type Item = (T::Signal, T);
    type IntoIter = std::collections::hash_map::IntoValues<StateToken, (T::Signal, T)>;
    fn into_iter(self) -> Self::IntoIter {
        self.updates.into_values()
    }
}
impl<T: Expression> Extend<(T::Signal, T)> for CombinedWorkerUpdateQueue<T> {
    fn extend<I: IntoIterator<Item = (T::Signal, T)>>(&mut self, iter: I) {
        self.updates.extend(
            iter.into_iter()
                .map(|(key, value)| (key.id(), (key, value))),
        )
    }
}

struct ExactWorkerUpdateQueue<T: Expression> {
    batches: VecDeque<Vec<(T::Signal, T)>>,
}
impl<T: Expression> Default for ExactWorkerUpdateQueue<T> {
    fn default() -> Self {
        Self {
            batches: Default::default(),
        }
    }
}

dispatcher!({
    pub enum WasmInterpreterAction<T: Expression> {
        Inbox(EvaluateStartAction<T>),
        Inbox(EvaluateUpdateAction<T>),
        Inbox(EvaluateStopAction<T>),
        Inbox(BytecodeInterpreterResultAction<T>),
        Inbox(BytecodeInterpreterGcCompleteAction<T>),

        Outbox(EvaluateResultAction<T>),
        Outbox(BytecodeInterpreterInitAction<T>),
        Outbox(BytecodeInterpreterEvaluateAction<T>),
        Outbox(BytecodeInterpreterGcAction<T>),
    }

    impl<T, TFactory, TAllocator, TMetricLabels, TAction, TTask> Dispatcher<TAction, TTask>
        for WasmInterpreter<T, TFactory, TAllocator, TMetricLabels>
    where
        T: Expression,
        TFactory: ExpressionFactory<T> + Default,
        TAllocator: HeapAllocator<T> + Default,
        TMetricLabels: BytecodeInterpreterMetricLabels,
        TAction: Action,
        TTask: TaskFactory<TAction, TTask> + WasmWorkerTask<T, TFactory, TAllocator>,
    {
        type State = WasmInterpreterState<T>;
        type Events<TInbox: TaskInbox<TAction>> = TInbox;
        type Dispose = NoopDisposeCallback;

        fn init(&self) -> Self::State {
            Default::default()
        }
        fn events<TInbox: TaskInbox<TAction>>(
            &self,
            inbox: TInbox,
        ) -> ActorEvents<TInbox, Self::Events<TInbox>, Self::Dispose> {
            ActorEvents::Sync(inbox)
        }

        fn accept(&self, _action: &EvaluateStartAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &EvaluateStartAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EvaluateStartAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_evaluate_start(state, action, metadata, context)
        }

        fn accept(&self, _action: &EvaluateUpdateAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &EvaluateUpdateAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EvaluateUpdateAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_evaluate_update(state, action, metadata, context)
        }

        fn accept(&self, _action: &EvaluateStopAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &EvaluateStopAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EvaluateStopAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_evaluate_stop(state, action, metadata, context)
        }

        fn accept(&self, _action: &BytecodeInterpreterResultAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterResultAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterResultAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.wasm_bytecode_interpreter_result(state, action, metadata, context)
        }

        fn accept(&self, _action: &BytecodeInterpreterGcCompleteAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterGcCompleteAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterGcCompleteAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_gc_complete_action(state, action, metadata, context)
        }
    }
});

impl<T, TFactory, TAllocator, TMetricLabels> WasmInterpreter<T, TFactory, TAllocator, TMetricLabels>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
    TMetricLabels: BytecodeInterpreterMetricLabels,
{
    fn handle_gc_complete_action<TAction, TTask>(
        &self,
        state: &mut WasmInterpreterState<T>,
        action: &BytecodeInterpreterGcCompleteAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterGcCompleteAction {
            cache_key,
            statistics,
        } = action;
        self.update_worker_cache_metrics(state, cache_key, *statistics);
        None
    }
    fn handle_evaluate_start<TAction, TTask>(
        &self,
        state: &mut WasmInterpreterState<T>,
        action: &EvaluateStartAction<T>,
        _metadata: &MessageData,
        context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TFactory: Default,
        TAllocator: Default,
        TAction: Action
            + From<BytecodeInterpreterInitAction<T>>
            + From<BytecodeInterpreterEvaluateAction<T>>,
        TTask: TaskFactory<TAction, TTask> + From<WasmWorkerTaskFactory<T, TFactory, TAllocator>>,
    {
        let EvaluateStartAction {
            cache_key,
            label,
            query,
            evaluation_mode,
            invalidation_strategy,
        } = action;
        let worker_id = cache_key.id();
        let actions = match state.workers.entry(worker_id) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                let task_pid = context.generate_pid();
                let current_pid = context.pid();
                entry.insert(WasmInterpreterWorkerState {
                    pid: task_pid,
                    label: label.clone(),
                    state_index: None,
                    status: WasmInterpreterWorkerStatus::Working(WasmWorkerUpdateQueue::new(
                        *invalidation_strategy,
                    )),
                    invalidation_strategy: *invalidation_strategy,
                    updates_since_gc: 0,
                    metrics: Default::default(),
                });
                Some(SchedulerTransition::new([
                    SchedulerCommand::Task(
                        task_pid,
                        WasmWorkerTaskFactory {
                            cache_key: cache_key.clone(),
                            query: query.clone(),
                            graph_root_factory_export_name: self
                                .graph_root_factory_export_name
                                .clone(),
                            evaluation_mode: *evaluation_mode,
                            wasm_module: self.program.clone(),
                            metric_names: WasmWorkerMetricNames {
                                query_worker_compile_duration: self
                                    .metric_names
                                    .query_worker_compile_duration
                                    .into(),
                                query_worker_evaluate_duration: self
                                    .metric_names
                                    .query_worker_evaluate_duration
                                    .into(),
                                query_worker_gc_duration: self
                                    .metric_names
                                    .query_worker_gc_duration
                                    .into(),
                            },
                            caller_pid: current_pid,
                            _expression: PhantomData,
                            _factory: PhantomData,
                            _allocator: PhantomData,
                        }
                        .into(),
                    ),
                    SchedulerCommand::Send(
                        task_pid,
                        BytecodeInterpreterInitAction {
                            cache_key: cache_key.clone(),
                        }
                        .into(),
                    ),
                    SchedulerCommand::Send(
                        task_pid,
                        BytecodeInterpreterEvaluateAction {
                            cache_key: cache_key.clone(),
                            state_index: None,
                            state_updates: Default::default(),
                        }
                        .into(),
                    ),
                ]))
            }
        }?;
        match state.grouped_worker_metrics.entry(label.clone()) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .active_workers
                    .insert(cache_key.id(), cache_key.clone());
            }
            Entry::Vacant(entry) => {
                let worker_labels = self.get_worker_metric_labels.labels(label.as_str());
                entry.insert(WorkerMetricsState {
                    active_workers: [(cache_key.id(), cache_key.clone())].into_iter().collect(),
                    quantile_metric_labels: generate_quantile_metric_labels(
                        &QUANTILE_BUCKETS,
                        &worker_labels,
                    ),
                });
            }
        }
        Some(actions)
    }
    fn handle_evaluate_update<TAction, TTask>(
        &self,
        state: &mut WasmInterpreterState<T>,
        action: &EvaluateUpdateAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<BytecodeInterpreterInitAction<T>>
            + From<BytecodeInterpreterEvaluateAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EvaluateUpdateAction {
            cache_key,
            state_index,
            state_updates,
        } = action;
        let worker_id = cache_key.id();
        let worker_state = state.workers.get_mut(&worker_id)?;
        worker_state.state_index = *state_index;
        let evaluate_action = match &mut worker_state.status {
            WasmInterpreterWorkerStatus::Working(update_queue) => {
                update_queue.push_update_batch(state_updates);
                None
            }
            WasmInterpreterWorkerStatus::Idle => Some(BytecodeInterpreterEvaluateAction {
                cache_key: cache_key.clone(),
                state_index: *state_index,
                state_updates: state_updates.clone(),
            }),
        }?;
        let worker_pid = worker_state.pid;
        worker_state.status = WasmInterpreterWorkerStatus::Working(WasmWorkerUpdateQueue::new(
            worker_state.invalidation_strategy,
        ));
        Some(SchedulerTransition::new(once(SchedulerCommand::Send(
            worker_pid,
            evaluate_action.into(),
        ))))
    }
    fn handle_evaluate_stop<TAction, TTask>(
        &self,
        state: &mut WasmInterpreterState<T>,
        action: &EvaluateStopAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EvaluateStopAction { cache_key } = action;
        // Reset the metrics for this worker
        self.update_worker_cache_metrics(state, cache_key, Default::default());
        // Remove the worker
        let worker_id = cache_key.id();
        let worker_state = state.workers.remove(&worker_id)?;
        let worker_pid = worker_state.pid;
        // Clean up the worker metrics
        if let Entry::Occupied(mut entry) = state.grouped_worker_metrics.entry(worker_state.label) {
            entry.get_mut().active_workers.remove(&worker_id);
            let is_final_worker_in_group = entry.get().active_workers.is_empty();
            if is_final_worker_in_group {
                entry.remove();
            }
        }
        Some(SchedulerTransition::new(once(SchedulerCommand::Kill(
            worker_pid,
        ))))
    }
    fn wasm_bytecode_interpreter_result<TAction, TTask>(
        &self,
        state: &mut WasmInterpreterState<T>,
        action: &BytecodeInterpreterResultAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<EvaluateResultAction<T>>
            + From<BytecodeInterpreterEvaluateAction<T>>
            + From<BytecodeInterpreterGcAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterResultAction {
            cache_key,
            state_index,
            result,
            statistics,
        } = action;
        self.update_worker_cache_metrics(state, cache_key, *statistics);
        let worker_id = cache_key.id();
        let worker_state = state.workers.get_mut(&worker_id)?;
        let queued_evaluation =
            match std::mem::replace(&mut worker_state.status, WasmInterpreterWorkerStatus::Idle) {
                WasmInterpreterWorkerStatus::Working(mut existing_queue) => {
                    let pending_updates = existing_queue.pop_update_batch();
                    if pending_updates.is_empty() {
                        None
                    } else {
                        Some((
                            BytecodeInterpreterEvaluateAction {
                                cache_key: cache_key.clone(),
                                state_index: worker_state.state_index,
                                state_updates: pending_updates,
                            },
                            existing_queue,
                        ))
                    }
                }
                WasmInterpreterWorkerStatus::Idle => None,
            };
        let gc_action = if is_unresolved_result(&result, &self.factory) {
            None
        } else {
            let should_gc = queued_evaluation.is_none()
                || worker_state.updates_since_gc >= MAX_UPDATES_WITHOUT_GC;
            if should_gc {
                worker_state.updates_since_gc = 0;
                Some(BytecodeInterpreterGcAction {
                    cache_key: cache_key.clone(),
                    state_index: *state_index,
                })
            } else {
                worker_state.updates_since_gc += 1;
                None
            }
        };
        Some(SchedulerTransition::new(
            once(SchedulerCommand::Send(
                self.main_pid,
                EvaluateResultAction {
                    cache_key: cache_key.clone(),
                    state_index: *state_index,
                    result: result.clone(),
                }
                .into(),
            ))
            .chain(gc_action.map(|action| {
                let worker_pid = worker_state.pid;
                SchedulerCommand::Send(worker_pid, action.into())
            }))
            .chain(queued_evaluation.map(|(action, remaining_queue)| {
                let worker_pid = worker_state.pid;
                worker_state.status = WasmInterpreterWorkerStatus::Working(remaining_queue);
                SchedulerCommand::Send(worker_pid, action.into())
            })),
        ))
    }
    fn update_worker_cache_metrics(
        &self,
        state: &mut WasmInterpreterState<T>,
        cache_key: &T::Signal,
        statistics: BytecodeWorkerStatistics,
    ) -> Option<()> {
        // Update the worker statistics
        let worker_id = cache_key.id();
        let worker_state = state.workers.get_mut(&worker_id)?;
        worker_state.metrics = statistics;
        let worker_state = state.workers.get(&worker_id)?;
        // Determine the metric labels to be applied to this worker
        let worker_metrics = state.grouped_worker_metrics.get(&worker_state.label)?;
        let metric_labels = &worker_metrics.quantile_metric_labels;
        let updated_worker_metrics =
            worker_metrics
                .active_workers
                .values()
                .filter_map(|cache_key| {
                    let worker_id = cache_key.id();
                    state.workers.get(&worker_id)
                });
        // Recompute the quantile bucket values for this worker group
        publish_quantile_bucketed_metric(
            updated_worker_metrics
                .clone()
                .map(|worker_state| worker_state.metrics.state_dependency_count as f64),
            self.metric_names.query_worker_state_dependency_count,
            &QUANTILE_BUCKETS,
            metric_labels,
        );
        publish_quantile_bucketed_metric(
            updated_worker_metrics
                .clone()
                .map(|worker_state| worker_state.metrics.evaluation_cache_entry_count as f64),
            self.metric_names.query_worker_evaluation_cache_entry_count,
            &QUANTILE_BUCKETS,
            metric_labels,
        );
        publish_quantile_bucketed_metric(
            updated_worker_metrics
                .map(|worker_state| worker_state.metrics.evaluation_cache_deep_size as f64),
            self.metric_names.query_worker_evaluation_cache_deep_size,
            &QUANTILE_BUCKETS,
            metric_labels,
        );
        Some(())
    }
}

fn is_unresolved_result<T: Expression>(
    result: &EvaluationResult<T>,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_signal_term(result.result())
        .map(|term| {
            term.signals()
                .as_deref()
                .iter()
                .any(|effect| is_unresolved_effect(effect.as_deref()))
        })
        .unwrap_or(false)
}

fn is_unresolved_effect<T: Expression<Signal = V>, V: ConditionType<T>>(effect: &V) -> bool {
    match effect.signal_type() {
        SignalType::Error => false,
        SignalType::Pending | SignalType::Custom(_) => true,
    }
}
