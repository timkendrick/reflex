// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    collections::{hash_map::Entry, HashMap},
    iter::once,
    marker::PhantomData,
    ops::Deref,
    time::{Duration, SystemTime},
};

use reflex::core::{
    ConditionType, Expression, ExpressionFactory, ExpressionListType, FloatTermType, HeapAllocator,
    IntTermType, ListTermType, RefType, SignalType, StateToken, StringTermType, StringValue, Uuid,
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, NoopDisposeCallback, ProcessId,
    SchedulerCommand, SchedulerMode, SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_macros::{dispatcher, Named};
use reflex_runtime::{
    action::effect::{
        EffectEmitAction, EffectSubscribeAction, EffectUnsubscribeAction, EffectUpdateBatch,
    },
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator,
};

use crate::{
    action::timestamp::TimestampHandlerUpdateAction,
    task::timestamp::{TimestampHandlerTask, TimestampHandlerTaskFactory},
    utils::timestamp::get_timestamp_millis,
};

pub const EFFECT_TYPE_TIMESTAMP: &'static str = "reflex::timestamp";

pub fn is_timestamp_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_TIMESTAMP)
        .unwrap_or(false)
}

pub fn create_timestamp_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_TIMESTAMP))
}

#[derive(Named, Clone)]
pub struct TimestampHandler<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
{
    factory: TFactory,
    allocator: TAllocator,
    main_pid: ProcessId,
    _expression: PhantomData<T>,
}
impl<T, TFactory, TAllocator> TimestampHandler<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
{
    pub fn new(factory: TFactory, allocator: TAllocator, main_pid: ProcessId) -> Self {
        Self {
            factory,
            allocator,
            main_pid,
            _expression: Default::default(),
        }
    }
}

pub struct TimestampHandlerState<T: Expression> {
    active_operations: HashMap<StateToken, (Uuid, ProcessId)>,
    operation_effect_mappings: HashMap<Uuid, T::Signal>,
}
impl<T: Expression> Default for TimestampHandlerState<T> {
    fn default() -> Self {
        Self {
            active_operations: Default::default(),
            operation_effect_mappings: Default::default(),
        }
    }
}
impl<T: Expression> TimestampHandlerState<T> {
    fn subscribe_timestamp_task(
        &mut self,
        effect: &T::Signal,
        duration: Duration,
        context: &mut impl HandlerContext,
    ) -> Option<(ProcessId, TimestampHandlerTaskFactory)> {
        let entry = match self.active_operations.entry(effect.id()) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => Some(entry),
        }?;
        let operation_id = Uuid::new_v4();
        let (task_pid, task) = create_timestamp_task(operation_id, duration, context);
        self.operation_effect_mappings
            .insert(operation_id, effect.clone());
        entry.insert((operation_id, task_pid));
        Some((task_pid, task))
    }
    fn unsubscribe_timestamp_task(&mut self, effect: &T::Signal) -> Option<ProcessId> {
        let (operation_id, task_pid) = self.active_operations.remove(&effect.id())?;
        let _ = self.operation_effect_mappings.remove(&operation_id)?;
        Some(task_pid)
    }
}

dispatcher!({
    pub enum TimestampHandlerAction<T: Expression> {
        Inbox(EffectSubscribeAction<T>),
        Inbox(EffectUnsubscribeAction<T>),
        Inbox(TimestampHandlerUpdateAction),

        Outbox(EffectEmitAction<T>),
    }

    impl<T, TFactory, TAllocator, TAction, TTask> Dispatcher<TAction, TTask>
        for TimestampHandler<T, TFactory, TAllocator>
    where
        T: AsyncExpression,
        TFactory: AsyncExpressionFactory<T>,
        TAllocator: AsyncHeapAllocator<T>,
        TAction: Action,
        TTask: TaskFactory<TAction, TTask> + From<TimestampHandlerTaskFactory>,
    {
        type State = TimestampHandlerState<T>;
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

        fn accept(&self, action: &EffectSubscribeAction<T>) -> bool {
            is_timestamp_effect_type(&action.effect_type, &self.factory)
        }
        fn schedule(
            &self,
            _action: &EffectSubscribeAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EffectSubscribeAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_effect_subscribe(state, action, metadata, context)
        }

        fn accept(&self, action: &EffectUnsubscribeAction<T>) -> bool {
            is_timestamp_effect_type(&action.effect_type, &self.factory)
        }
        fn schedule(
            &self,
            _action: &EffectUnsubscribeAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EffectUnsubscribeAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_effect_unsubscribe(state, action, metadata, context)
        }

        fn accept(&self, _action: &TimestampHandlerUpdateAction) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &TimestampHandlerUpdateAction,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &TimestampHandlerUpdateAction,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_timestamp_handler_update(state, action, metadata, context)
        }
    }
});

impl<T, TFactory, TAllocator> TimestampHandler<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
{
    fn handle_effect_subscribe<TAction, TTask>(
        &self,
        state: &mut TimestampHandlerState<T>,
        action: &EffectSubscribeAction<T>,
        _metadata: &MessageData,
        context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask> + TimestampHandlerTask,
    {
        let EffectSubscribeAction {
            effect_type,
            effects,
        } = action;
        if !is_timestamp_effect_type(effect_type, &self.factory) {
            return None;
        }
        let (initial_values, tasks): (Vec<_>, Vec<_>) = effects
            .iter()
            .filter_map(
                |effect| match parse_timestamp_effect_args(effect, &self.factory) {
                    Ok(interval) => {
                        match state.subscribe_timestamp_task(effect, interval, context) {
                            None => None,
                            Some((task_pid, task)) => {
                                let initial_value = self.factory.create_timestamp_term(
                                    get_timestamp_millis(SystemTime::now()) as i64,
                                );
                                Some((
                                    (effect.clone(), initial_value),
                                    Some(SchedulerCommand::Task(task_pid, task.into())),
                                ))
                            }
                        }
                    }
                    Err(err) => Some((
                        (
                            effect.clone(),
                            create_error_expression(err, &self.factory, &self.allocator),
                        ),
                        None,
                    )),
                },
            )
            .unzip();
        let initial_values_action = if initial_values.is_empty() {
            None
        } else {
            Some(SchedulerCommand::Send(
                self.main_pid,
                EffectEmitAction {
                    effect_types: vec![EffectUpdateBatch {
                        effect_type: create_timestamp_effect_type(&self.factory, &self.allocator),
                        updates: initial_values,
                    }],
                }
                .into(),
            ))
        };
        Some(SchedulerTransition::new(
            initial_values_action
                .into_iter()
                .chain(tasks.into_iter().flatten()),
        ))
    }
    fn handle_effect_unsubscribe<TAction, TTask>(
        &self,
        state: &mut TimestampHandlerState<T>,
        action: &EffectUnsubscribeAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectUnsubscribeAction {
            effect_type,
            effects,
        } = action;
        if !is_timestamp_effect_type(effect_type, &self.factory) {
            return None;
        }
        let active_pids = effects
            .iter()
            .filter_map(|effect| state.unsubscribe_timestamp_task(effect));
        Some(SchedulerTransition::new(
            active_pids.map(SchedulerCommand::Kill),
        ))
    }
    fn handle_timestamp_handler_update<TAction, TTask>(
        &self,
        state: &mut TimestampHandlerState<T>,
        action: &TimestampHandlerUpdateAction,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let TimestampHandlerUpdateAction {
            operation_id,
            timestamp,
        } = action;
        let effect = state.operation_effect_mappings.get(operation_id)?;
        let result = self
            .factory
            .create_timestamp_term(get_timestamp_millis(*timestamp) as i64);
        Some(SchedulerTransition::new(once(SchedulerCommand::Send(
            self.main_pid,
            EffectEmitAction {
                effect_types: vec![EffectUpdateBatch {
                    effect_type: create_timestamp_effect_type(&self.factory, &self.allocator),
                    updates: vec![(effect.clone(), result.clone())],
                }],
            }
            .into(),
        ))))
    }
}

fn create_timestamp_task(
    operation_id: Uuid,
    interval: Duration,
    context: &mut impl HandlerContext,
) -> (ProcessId, TimestampHandlerTaskFactory) {
    let task_pid = context.generate_pid();
    let current_pid = context.pid();
    let task = TimestampHandlerTaskFactory {
        operation_id,
        interval,
        caller_pid: current_pid,
    };
    (task_pid, task)
}

fn parse_timestamp_effect_args<T: Expression>(
    effect: &T::Signal,
    factory: &impl ExpressionFactory<T>,
) -> Result<Duration, String> {
    let payload = match effect.signal_type() {
        SignalType::Custom { payload, .. } => Ok(payload),
        _ => Err(format!("Invalid {EFFECT_TYPE_TIMESTAMP} signal: {effect}")),
    }?;
    let args = factory
        .match_list_term(&payload)
        .filter(|args| args.items().as_deref().len() == 1)
        .ok_or_else(|| {
            format!(
                "Invalid {EFFECT_TYPE_TIMESTAMP} signal: Expected 1 argument, received {payload}",
            )
        })?;
    let args = args.items();
    let mut args = args.as_deref().iter().map(|item| item.as_deref().clone());
    let interval = args.next().unwrap();
    let interval = parse_duration_millis_arg(&interval, factory);
    match interval {
        Some(interval) if interval.as_millis() >= 1 => Ok(interval),
        _ => Err(format!(
            "Invalid {EFFECT_TYPE_TIMESTAMP} signal arguments: {payload}",
        )),
    }
}

fn parse_duration_millis_arg<T: Expression>(
    value: &T,
    factory: &impl ExpressionFactory<T>,
) -> Option<Duration> {
    match factory.match_int_term(value) {
        Some(term) => {
            let value = term.value();
            if value >= 0 {
                Some(Duration::from_millis(value as u64))
            } else {
                None
            }
        }
        _ => match factory.match_float_term(value) {
            Some(term) => {
                let value = term.value();
                if value >= 0.0 {
                    Some(Duration::from_millis(value.trunc() as u64))
                } else {
                    None
                }
            }
            _ => None,
        },
    }
}

fn create_error_expression<T: Expression>(
    message: String,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_signal_term(allocator.create_signal_list(once(allocator.create_signal(
        SignalType::Error {
            payload: factory.create_string_term(allocator.create_string(message)),
        },
    ))))
}
