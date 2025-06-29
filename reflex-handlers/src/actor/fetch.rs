// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    collections::{hash_map::Entry, HashMap},
    iter::once,
    marker::PhantomData,
    ops::Deref,
    str::FromStr,
};

use bytes::Bytes;
use http::{header::HeaderName, HeaderValue};
use hyper::Body;
use metrics::{
    decrement_gauge, describe_counter, describe_gauge, increment_counter, increment_gauge, Unit,
};
use reflex::core::{
    ConditionType, Expression, ExpressionFactory, ExpressionListType, HeapAllocator, ListTermType,
    RecordTermType, RefType, SignalType, StateToken, StringTermType, StringValue,
    StructPrototypeType, Uuid,
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
    action::fetch::{FetchHandlerConnectionErrorAction, FetchHandlerFetchCompleteAction},
    task::fetch::{FetchHandlerTask, FetchHandlerTaskFactory},
    utils::fetch::FetchRequest,
};

pub const EFFECT_TYPE_FETCH: &'static str = "reflex::fetch";

pub fn is_fetch_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_FETCH)
        .unwrap_or(false)
}

pub fn create_fetch_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_FETCH))
}

#[derive(Clone, Copy, Debug)]
pub struct FetchHandlerMetricNames {
    pub fetch_effect_total_request_count: &'static str,
    pub fetch_effect_active_request_count: &'static str,
}
impl FetchHandlerMetricNames {
    fn init(self) -> Self {
        describe_counter!(
            self.fetch_effect_total_request_count,
            Unit::Count,
            "Total Fetch effect request count"
        );
        describe_gauge!(
            self.fetch_effect_active_request_count,
            Unit::Count,
            "Active Fetch effect request count"
        );
        self
    }
}
impl Default for FetchHandlerMetricNames {
    fn default() -> Self {
        Self {
            fetch_effect_total_request_count: "fetch_effect_total_request_count",
            fetch_effect_active_request_count: "fetch_effect_active_request_count",
        }
    }
}

#[derive(Named, Clone)]
pub struct FetchHandler<T, TFactory, TAllocator, TConnect>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    client: hyper::Client<TConnect, Body>,
    factory: TFactory,
    allocator: TAllocator,
    metric_names: FetchHandlerMetricNames,
    main_pid: ProcessId,
    _expression: PhantomData<T>,
}
impl<T, TFactory, TAllocator, TConnect> FetchHandler<T, TFactory, TAllocator, TConnect>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    pub fn new(
        client: hyper::Client<TConnect, Body>,
        factory: TFactory,
        allocator: TAllocator,
        metric_names: FetchHandlerMetricNames,
        main_pid: ProcessId,
    ) -> Self {
        Self {
            factory,
            allocator,
            client,
            metric_names: metric_names.init(),
            main_pid,
            _expression: Default::default(),
        }
    }
}

pub struct FetchHandlerState<T: Expression> {
    tasks: HashMap<StateToken, RequestState>,
    operation_effect_mappings: HashMap<Uuid, T::Signal>,
}
impl<T: Expression> Default for FetchHandlerState<T> {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            operation_effect_mappings: Default::default(),
        }
    }
}
impl<T: Expression> FetchHandlerState<T> {
    fn subscribe_fetch_task<TConnect>(
        &mut self,
        effect: &T::Signal,
        request: FetchRequest,
        client: &hyper::Client<TConnect, Body>,
        metric_names: &FetchHandlerMetricNames,
        context: &mut impl HandlerContext,
    ) -> Option<(ProcessId, FetchHandlerTaskFactory<TConnect>)>
    where
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let entry = match self.tasks.entry(effect.id()) {
            Entry::Vacant(entry) => Some(entry),
            Entry::Occupied(_) => None,
        }?;
        let operation_id = Uuid::new_v4();
        // TODO: Allow configurable Fetch effect metric labels
        let metric_labels = [
            ("method", request.method.clone()),
            ("url", request.url.clone()),
        ];
        increment_counter!(
            metric_names.fetch_effect_total_request_count,
            &metric_labels
        );
        increment_gauge!(
            metric_names.fetch_effect_active_request_count,
            1.0,
            &metric_labels
        );
        let (task_pid, task) = create_fetch_task(operation_id, client.clone(), request, context);
        entry.insert(RequestState {
            operation_id,
            task_pid,
            metric_labels,
        });
        self.operation_effect_mappings
            .insert(operation_id, effect.clone());
        Some((task_pid, task))
    }
    fn unsubscribe_fetch_task(
        &mut self,
        effect: &T::Signal,
        metric_names: &FetchHandlerMetricNames,
    ) -> Option<ProcessId> {
        let RequestState {
            operation_id,
            task_pid,
            metric_labels,
        } = self.tasks.remove(&effect.id())?;
        decrement_gauge!(
            metric_names.fetch_effect_active_request_count,
            1.0,
            &metric_labels
        );
        let _ = self.operation_effect_mappings.remove(&operation_id)?;
        Some(task_pid)
    }
}
struct RequestState {
    operation_id: Uuid,
    task_pid: ProcessId,
    metric_labels: [(&'static str, String); 2],
}

dispatcher!({
    pub enum FetchHandlerAction<T: Expression> {
        Inbox(EffectSubscribeAction<T>),
        Inbox(EffectUnsubscribeAction<T>),
        Inbox(FetchHandlerFetchCompleteAction),
        Inbox(FetchHandlerConnectionErrorAction),

        Outbox(EffectEmitAction<T>),
    }

    impl<T, TFactory, TAllocator, TConnect, TAction, TTask> Dispatcher<TAction, TTask>
        for FetchHandler<T, TFactory, TAllocator, TConnect>
    where
        T: AsyncExpression,
        TFactory: AsyncExpressionFactory<T>,
        TAllocator: AsyncHeapAllocator<T>,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
        TAction: Action,
        TTask: TaskFactory<TAction, TTask> + FetchHandlerTask<TConnect>,
    {
        type State = FetchHandlerState<T>;
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
            is_fetch_effect_type(&action.effect_type, &self.factory)
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
            is_fetch_effect_type(&action.effect_type, &self.factory)
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

        fn accept(&self, _action: &FetchHandlerFetchCompleteAction) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &FetchHandlerFetchCompleteAction,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &FetchHandlerFetchCompleteAction,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_fetch_handler_fetch_complete(state, action, metadata, context)
        }

        fn accept(&self, _action: &FetchHandlerConnectionErrorAction) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &FetchHandlerConnectionErrorAction,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &FetchHandlerConnectionErrorAction,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_fetch_handler_connection_error(state, action, metadata, context)
        }
    }
});

impl<T, TFactory, TAllocator, TConnect> FetchHandler<T, TFactory, TAllocator, TConnect>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn handle_effect_subscribe<TAction, TTask>(
        &self,
        state: &mut FetchHandlerState<T>,
        action: &EffectSubscribeAction<T>,
        _metadata: &MessageData,
        context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask> + From<FetchHandlerTaskFactory<TConnect>>,
    {
        let EffectSubscribeAction {
            effect_type,
            effects,
        } = action;
        if !is_fetch_effect_type(effect_type, &self.factory) {
            return None;
        }
        let (initial_values, tasks): (Vec<_>, Vec<_>) = effects
            .iter()
            .filter_map(
                |effect| match parse_fetch_effect_args(effect, &self.factory) {
                    Ok(request) => {
                        match state.subscribe_fetch_task(
                            effect,
                            request,
                            &self.client,
                            &self.metric_names,
                            context,
                        ) {
                            None => None,
                            Some((task_pid, task)) => Some((
                                (
                                    effect.clone(),
                                    create_pending_expression(&self.factory, &self.allocator),
                                ),
                                Some(SchedulerCommand::Task(task_pid, task.into())),
                            )),
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
                        effect_type: create_fetch_effect_type(&self.factory, &self.allocator),
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
        state: &mut FetchHandlerState<T>,
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
        if !is_fetch_effect_type(effect_type, &self.factory) {
            return None;
        }
        let active_pids = effects
            .iter()
            .filter_map(|effect| state.unsubscribe_fetch_task(effect, &self.metric_names));
        Some(SchedulerTransition::new(
            active_pids.map(SchedulerCommand::Kill),
        ))
    }
    fn handle_fetch_handler_fetch_complete<TAction, TTask>(
        &self,
        state: &mut FetchHandlerState<T>,
        action: &FetchHandlerFetchCompleteAction,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let FetchHandlerFetchCompleteAction {
            operation_id,
            status_code,
            body,
            ..
        } = action;
        let effect = state.operation_effect_mappings.get(operation_id).cloned()?;
        let task_pid = state.unsubscribe_fetch_task(&effect, &self.metric_names)?;
        let factory = &self.factory;
        let allocator = &self.allocator;
        let result = match String::from_utf8(body.into_iter().copied().collect()) {
            Ok(body) => factory.create_list_term(allocator.create_pair(
                factory.create_int_term(status_code.as_u16().into()),
                factory.create_string_term(allocator.create_string(body)),
            )),
            Err(err) => create_error_expression(format!("{}", err), factory, allocator),
        };
        Some(SchedulerTransition::new([
            SchedulerCommand::Kill(task_pid),
            SchedulerCommand::Send(
                self.main_pid,
                EffectEmitAction {
                    effect_types: vec![EffectUpdateBatch {
                        effect_type: create_fetch_effect_type(&self.factory, &self.allocator),
                        updates: vec![(effect, result)],
                    }],
                }
                .into(),
            ),
        ]))
    }
    fn handle_fetch_handler_connection_error<TAction, TTask>(
        &self,
        state: &mut FetchHandlerState<T>,
        action: &FetchHandlerConnectionErrorAction,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let FetchHandlerConnectionErrorAction {
            operation_id,
            message,
            ..
        } = action;
        let effect = state.operation_effect_mappings.get(operation_id).cloned()?;
        let task_pid = state.unsubscribe_fetch_task(&effect, &self.metric_names)?;
        let result = create_error_expression(message.clone(), &self.factory, &self.allocator);
        Some(SchedulerTransition::new([
            SchedulerCommand::Kill(task_pid),
            SchedulerCommand::Send(
                self.main_pid,
                EffectEmitAction {
                    effect_types: vec![EffectUpdateBatch {
                        effect_type: create_fetch_effect_type(&self.factory, &self.allocator),
                        updates: vec![(effect, result)],
                    }],
                }
                .into(),
            ),
        ]))
    }
}

fn create_fetch_task<TConnect>(
    operation_id: Uuid,
    client: hyper::Client<TConnect, Body>,
    request: FetchRequest,
    context: &mut impl HandlerContext,
) -> (ProcessId, FetchHandlerTaskFactory<TConnect>)
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let task_pid = context.generate_pid();
    let current_pid = context.pid();
    let task = FetchHandlerTaskFactory {
        operation_id,
        client,
        request,
        caller_pid: current_pid,
    };
    (task_pid, task)
}

fn parse_fetch_effect_args<T: Expression>(
    effect: &T::Signal,
    factory: &impl ExpressionFactory<T>,
) -> Result<FetchRequest, String> {
    let payload = match effect.signal_type() {
        SignalType::Custom { payload, .. } => Ok(payload),
        _ => Err(format!("Invalid {EFFECT_TYPE_FETCH} signal: {effect}")),
    }?;
    let args = factory
        .match_list_term(&payload)
        .filter(|args| args.items().as_deref().len() == 4)
        .ok_or_else(|| {
            format!("Invalid {EFFECT_TYPE_FETCH} signal: Expected 4 arguments, received {payload}")
        })?;
    let args = args.items();
    let mut args = args.as_deref().iter().map(|item| item.as_deref().clone());
    let url = args.next().unwrap();
    let method = args.next().unwrap();
    let headers = args.next().unwrap();
    let body = args.next().unwrap();
    let url = parse_string_arg(&url, factory);
    let method = parse_string_arg(&method, factory);
    let headers = parse_key_values_arg(&headers, factory);
    let body = parse_optional_string_arg(&body, factory);
    match (method, url, headers, body) {
        (Some(method), Some(url), Some(headers), Some(body)) => {
            let headers = format_request_headers(headers)?;
            Ok(FetchRequest {
                method,
                url,
                headers,
                body: body.map(Bytes::from),
            })
        }
        _ => Err(format!(
            "Invalid {EFFECT_TYPE_FETCH} signal arguments: {payload}",
        )),
    }
}

fn format_request_headers(
    headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
) -> Result<Vec<(HeaderName, HeaderValue)>, String> {
    headers
        .into_iter()
        .map(|(key, value)| {
            let key: String = key.into();
            let value: String = value.into();
            let key = HeaderName::from_str(key.as_str())
                .map_err(|_| format!("Invalid fetch header name: {}", key))?;
            let value = HeaderValue::from_str(value.as_str())
                .map_err(|_| format!("Invalid value for fetch header {}: {}", key, value))?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn parse_string_arg<T: Expression>(
    value: &T,
    factory: &impl ExpressionFactory<T>,
) -> Option<String> {
    match factory.match_string_term(value) {
        Some(term) => Some(String::from(term.value().as_deref().as_str().deref())),
        _ => None,
    }
}

fn parse_optional_string_arg<T: Expression>(
    value: &T,
    factory: &impl ExpressionFactory<T>,
) -> Option<Option<String>> {
    match factory.match_string_term(value) {
        Some(term) => Some(Some(String::from(term.value().as_deref().as_str().deref()))),
        _ => match factory.match_nil_term(value) {
            Some(_) => Some(None),
            _ => None,
        },
    }
}

fn parse_key_values_arg<T: Expression>(
    value: &T,
    factory: &impl ExpressionFactory<T>,
) -> Option<Vec<(String, String)>> {
    if let Some(value) = factory.match_record_term(value) {
        value
            .prototype()
            .as_deref()
            .keys()
            .as_deref()
            .iter()
            .zip(value.values().as_deref().iter())
            .map(|(key, value)| {
                match (
                    factory.match_string_term(key.as_deref()),
                    factory.match_string_term(value.as_deref()),
                ) {
                    (Some(key), Some(value)) => Some((
                        String::from(key.value().as_deref().as_str().deref()),
                        String::from(value.value().as_deref().as_str().deref()),
                    )),
                    _ => None,
                }
            })
            .collect::<Option<Vec<_>>>()
    } else {
        None
    }
}

fn create_pending_expression<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_signal_term(
        allocator.create_signal_list(once(allocator.create_signal(SignalType::Pending))),
    )
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
