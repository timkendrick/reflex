// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    collections::{hash_map::Entry, HashMap},
    iter::once,
    marker::PhantomData,
    ops::Deref,
};

use metrics::{describe_counter, describe_gauge, gauge, increment_gauge, SharedString, Unit};
use reflex::core::{
    ConditionType, Expression, ExpressionFactory, ExpressionListType, HeapAllocator, ListTermType,
    RefType, SignalType, StateToken, StringTermType, StringValue,
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, NoopDisposeCallback, ProcessId,
    SchedulerCommand, SchedulerMode, SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_macros::{blanket_trait, dispatcher, Named};
use reflex_runtime::{
    action::effect::{
        EffectEmitAction, EffectSubscribeAction, EffectUnsubscribeAction, EffectUpdateBatch,
    },
    actor::evaluate_handler::{
        create_evaluate_effect, create_evaluate_effect_type, is_evaluate_effect_type,
        parse_evaluate_effect_result,
    },
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator, QueryEvaluationMode,
    QueryInvalidationStrategy,
};
use reflex_stdlib::{Apply, CollectList, ResolveDeep};

pub const EFFECT_TYPE_SCAN: &'static str = "reflex::scan";

blanket_trait!(
    pub trait ScanHandlerBuiltin: From<Apply> + From<ResolveDeep> + From<CollectList> {}
);

pub fn is_scan_effect_type<T: Expression>(
    effect_type: &T,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    factory
        .match_string_term(effect_type)
        .map(|effect_type| effect_type.value().as_deref().as_str().deref() == EFFECT_TYPE_SCAN)
        .unwrap_or(false)
}

pub fn create_scan_effect_type<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_string_term(allocator.create_static_string(EFFECT_TYPE_SCAN))
}

#[derive(Clone, Copy, Debug)]
pub struct ScanHandlerMetricNames {
    pub scan_effect_iteration_count: &'static str,
    pub scan_effect_result_count: &'static str,
    pub scan_effect_state_size: &'static str,
}
impl ScanHandlerMetricNames {
    fn init(self) -> Self {
        describe_counter!(
            self.scan_effect_iteration_count,
            Unit::Count,
            "Scan effect iteration count"
        );
        describe_counter!(
            self.scan_effect_result_count,
            Unit::Count,
            "Scan effect result count"
        );
        describe_gauge!(
            self.scan_effect_state_size,
            Unit::Count,
            "Scan effect accumulated state size"
        );
        self
    }
}
impl Default for ScanHandlerMetricNames {
    fn default() -> Self {
        Self {
            scan_effect_iteration_count: "scan_effect_iteration_count",
            scan_effect_result_count: "scan_effect_result_count",
            scan_effect_state_size: "scan_effect_state_size",
        }
    }
}

const EVENT_TYPE_SCAN_SOURCE: &'static str = "reflex::scan::source";
const EVENT_TYPE_SCAN_STATE: &'static str = "reflex::scan::state";

#[derive(Named, Clone)]
pub struct ScanHandler<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
{
    factory: TFactory,
    allocator: TAllocator,
    metric_names: ScanHandlerMetricNames,
    main_pid: ProcessId,
    _expression: PhantomData<T>,
}
impl<T, TFactory, TAllocator> ScanHandler<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
{
    pub fn new(
        factory: TFactory,
        allocator: TAllocator,
        metric_names: ScanHandlerMetricNames,
        main_pid: ProcessId,
    ) -> Self {
        Self {
            factory,
            allocator,
            metric_names: metric_names.init(),
            main_pid,
            _expression: Default::default(),
        }
    }
}

pub struct ScanHandlerState<T: Expression> {
    effect_state: HashMap<StateToken, ScanHandlerReducerState<T>>,
    /// Maps the child evaluate effect ID to the parent state effect
    effect_mappings: HashMap<StateToken, T::Signal>,
}
impl<T: Expression> Default for ScanHandlerState<T> {
    fn default() -> Self {
        Self {
            effect_state: Default::default(),
            effect_mappings: Default::default(),
        }
    }
}

struct ScanHandlerReducerState<T: Expression> {
    metric_labels: [(SharedString, SharedString); 2],
    source_effect: T::Signal,
    source_value_effect: T::Signal,
    source_value: Option<T>,
    state_value_effect: T::Signal,
    state_value: T,
    result_effect: T::Signal,
}

dispatcher!({
    pub enum ScanHandlerAction<T: Expression> {
        Inbox(EffectSubscribeAction<T>),
        Inbox(EffectEmitAction<T>),
        Inbox(EffectUnsubscribeAction<T>),

        Outbox(EffectSubscribeAction<T>),
        Outbox(EffectEmitAction<T>),
        Outbox(EffectUnsubscribeAction<T>),
    }

    impl<T, TFactory, TAllocator, TAction, TTask> Dispatcher<TAction, TTask>
        for ScanHandler<T, TFactory, TAllocator>
    where
        T: AsyncExpression,
        TFactory: AsyncExpressionFactory<T>,
        TAllocator: AsyncHeapAllocator<T>,
        T::Builtin: ScanHandlerBuiltin,
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        type State = ScanHandlerState<T>;
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
            is_scan_effect_type(&action.effect_type, &self.factory)
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
            is_scan_effect_type(&action.effect_type, &self.factory)
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

        fn accept(&self, _action: &EffectEmitAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &EffectEmitAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EffectEmitAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_effect_emit(state, action, metadata, context)
        }
    }
});

impl<T, TFactory, TAllocator> ScanHandler<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
    T::Builtin: ScanHandlerBuiltin,
{
    fn handle_effect_subscribe<TAction, TTask>(
        &self,
        state: &mut ScanHandlerState<T>,
        action: &EffectSubscribeAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectSubscribeAction<T>> + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectSubscribeAction {
            effect_type,
            effects,
        } = action;
        if !is_scan_effect_type(effect_type, &self.factory) {
            return None;
        }
        let (initial_values, tasks): (Vec<_>, Vec<_>) = effects
            .iter()
            .filter_map(
                |effect| match parse_scan_effect_args(effect, &self.factory) {
                    Ok(args) => {
                        if let Some(action) = self.subscribe_scan_effect(state, effect, args) {
                            Some((
                                (
                                    effect.clone(),
                                    create_pending_expression(&self.factory, &self.allocator),
                                ),
                                Some(SchedulerCommand::Send(self.main_pid, action)),
                            ))
                        } else {
                            None
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
                        effect_type: create_scan_effect_type(&self.factory, &self.allocator),
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
        state: &mut ScanHandlerState<T>,
        action: &EffectUnsubscribeAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectUnsubscribeAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectUnsubscribeAction {
            effect_type,
            effects,
        } = action;
        if !is_scan_effect_type(effect_type, &self.factory) {
            return None;
        }
        let unsubscribe_actions = effects.iter().filter_map(|effect| {
            if let Some(operation) = self.unsubscribe_scan_effect(state, effect) {
                Some(SchedulerCommand::Send(self.main_pid, operation))
            } else {
                None
            }
        });
        Some(SchedulerTransition::new(unsubscribe_actions))
    }
    fn handle_effect_emit<TAction, TTask>(
        &self,
        state: &mut ScanHandlerState<T>,
        action: &EffectEmitAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<EffectEmitAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectEmitAction {
            effect_types: updates,
        } = action;
        if state.effect_state.is_empty() {
            return None;
        }
        let updates = updates
            .iter()
            .filter(|batch| is_evaluate_effect_type(&batch.effect_type, &self.factory))
            .flat_map(|batch| batch.updates.iter())
            .filter_map(|(updated_effect, update)| {
                let scan_effect = state.effect_mappings.get(&updated_effect.id())?;
                let reducer_state = state.effect_state.get_mut(&scan_effect.id())?;
                let updated_state_token = updated_effect.id();
                if updated_state_token == reducer_state.source_effect.id() {
                    // The source input has emitted, so trigger the next reducer iteration
                    {
                        increment_gauge!(
                            self.metric_names.scan_effect_iteration_count,
                            1.0,
                            &reducer_state.metric_labels
                        );
                    }
                    let (value, _) =
                        parse_evaluate_effect_result(update, &self.factory)?.into_parts();
                    reducer_state.source_value.replace(value.clone());
                    // Assign values for both reducer arguments (this will trigger the reducer query to be re-evaluated)
                    Some([
                        (reducer_state.source_value_effect.clone(), value),
                        (
                            reducer_state.state_value_effect.clone(),
                            reducer_state.state_value.clone(),
                        ),
                    ])
                } else if updated_state_token == reducer_state.result_effect.id() {
                    // The reducer has emitted a result, so emit a new result and reset the reducer to a pending state
                    // while we wait for the next input value to arrive
                    let (value, _) =
                        parse_evaluate_effect_result(update, &self.factory)?.into_parts();
                    reducer_state.state_value = value.clone();
                    increment_gauge!(
                        self.metric_names.scan_effect_result_count,
                        1.0,
                        &reducer_state.metric_labels
                    );
                    gauge!(
                        self.metric_names.scan_effect_state_size,
                        value.size() as f64,
                        &reducer_state.metric_labels
                    );
                    // Emit a result for the overall scan effect, resetting the reducer state to a pending value
                    // (this effectively blocks the reducer query from being prematurely re-evaluated with a stale state
                    // when the next source value arrives)
                    Some([
                        (
                            reducer_state.state_value_effect.clone(),
                            create_pending_expression(&self.factory, &self.allocator),
                        ),
                        (scan_effect.clone(), value),
                    ])
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();
        let update_action = if updates.is_empty() {
            None
        } else {
            Some(SchedulerCommand::Send(
                self.main_pid,
                EffectEmitAction {
                    effect_types: vec![EffectUpdateBatch {
                        effect_type: create_scan_effect_type(&self.factory, &self.allocator),
                        updates,
                    }],
                }
                .into(),
            ))
        };
        Some(SchedulerTransition::new(update_action))
    }
    fn subscribe_scan_effect<TAction>(
        &self,
        state: &mut ScanHandlerState<T>,
        effect: &T::Signal,
        args: ScanEffectArgs<T>,
    ) -> Option<TAction>
    where
        TAction: Action + From<EffectSubscribeAction<T>>,
    {
        let (source_effect, result_effect) = match state.effect_state.entry(effect.id()) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                let ScanEffectArgs {
                    target,
                    seed,
                    iteratee,
                } = args;
                let source_value_effect = self.allocator.create_signal(SignalType::Custom {
                    effect_type: self.factory.create_string_term(
                        self.allocator.create_static_string(EVENT_TYPE_SCAN_SOURCE),
                    ),
                    payload: self.factory.create_list_term(self.allocator.create_triple(
                        target.clone(),
                        seed.clone(),
                        iteratee.clone(),
                    )),
                    token: self.factory.create_nil_term(),
                });
                let state_value_effect = self.allocator.create_signal(SignalType::Custom {
                    effect_type: self.factory.create_string_term(
                        self.allocator.create_static_string(EVENT_TYPE_SCAN_STATE),
                    ),
                    payload: self.factory.create_list_term(self.allocator.create_triple(
                        target.clone(),
                        seed.clone(),
                        iteratee.clone(),
                    )),
                    token: self.factory.create_nil_term(),
                });
                let source_label = format!("{}:{} [scan]", target.id(), iteratee.id());
                let reducer_label = format!("{}:{} [reducer]", target.id(), iteratee.id());
                let metric_labels = [
                    (
                        SharedString::borrowed("source"),
                        SharedString::owned(format!("{}", target.id())),
                    ),
                    (
                        SharedString::borrowed("reducer"),
                        SharedString::owned(format!("{}", iteratee.id())),
                    ),
                ];
                let source_effect = create_evaluate_effect(
                    source_label,
                    self.factory.create_application_term(
                        self.factory.create_builtin_term(ResolveDeep),
                        self.allocator.create_unit_list(
                            self.factory.create_application_term(
                                target,
                                self.allocator.create_empty_list(),
                            ),
                        ),
                    ),
                    QueryEvaluationMode::Standalone,
                    QueryInvalidationStrategy::Exact,
                    &self.factory,
                    &self.allocator,
                );
                let result_effect = create_evaluate_effect(
                    reducer_label,
                    self.factory.create_application_term(
                        self.factory.create_builtin_term(ResolveDeep),
                        self.allocator.create_unit_list(
                            self.factory.create_application_term(
                                self.factory.create_builtin_term(Apply),
                                self.allocator.create_pair(
                                    iteratee,
                                    self.factory.create_application_term(
                                        self.factory.create_builtin_term(CollectList),
                                        self.allocator.create_pair(
                                            self.factory
                                                .create_effect_term(state_value_effect.clone()),
                                            self.factory
                                                .create_effect_term(source_value_effect.clone()),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                    QueryEvaluationMode::Standalone,
                    QueryInvalidationStrategy::Exact,
                    &self.factory,
                    &self.allocator,
                );
                let reducer_state = ScanHandlerReducerState {
                    metric_labels,
                    source_effect: source_effect.clone(),
                    source_value_effect,
                    source_value: None,
                    state_value_effect,
                    state_value: seed,
                    result_effect: result_effect.clone(),
                };
                gauge!(
                    self.metric_names.scan_effect_iteration_count,
                    0.0,
                    &reducer_state.metric_labels
                );
                gauge!(
                    self.metric_names.scan_effect_result_count,
                    0.0,
                    &reducer_state.metric_labels
                );
                gauge!(
                    self.metric_names.scan_effect_state_size,
                    0.0,
                    &reducer_state.metric_labels
                );
                entry.insert(reducer_state);
                Some((source_effect, result_effect))
            }
        }?;
        state
            .effect_mappings
            .insert(source_effect.id(), effect.clone());
        state
            .effect_mappings
            .insert(result_effect.id(), effect.clone());
        Some(
            EffectSubscribeAction {
                effect_type: create_evaluate_effect_type(&self.factory, &self.allocator),
                effects: vec![source_effect, result_effect],
            }
            .into(),
        )
    }
    fn unsubscribe_scan_effect<TAction>(
        &self,
        state: &mut ScanHandlerState<T>,
        effect: &T::Signal,
    ) -> Option<TAction>
    where
        TAction: Action + From<EffectUnsubscribeAction<T>>,
    {
        if let Entry::Occupied(entry) = state.effect_state.entry(effect.id()) {
            let reducer_state = entry.remove();
            gauge!(
                self.metric_names.scan_effect_iteration_count,
                0.0,
                &reducer_state.metric_labels
            );
            gauge!(
                self.metric_names.scan_effect_result_count,
                0.0,
                &reducer_state.metric_labels
            );
            gauge!(
                self.metric_names.scan_effect_state_size,
                0.0,
                &reducer_state.metric_labels
            );
            let ScanHandlerReducerState {
                metric_labels: _,
                source_effect,
                source_value_effect: _,
                source_value: _,
                state_value_effect: _,
                state_value: _,
                result_effect,
            } = reducer_state;
            state.effect_mappings.remove(&source_effect.id());
            state.effect_mappings.remove(&result_effect.id());
            Some(
                EffectUnsubscribeAction {
                    effect_type: create_evaluate_effect_type(&self.factory, &self.allocator),
                    effects: vec![source_effect, result_effect],
                }
                .into(),
            )
        } else {
            None
        }
    }
}

struct ScanEffectArgs<T: Expression> {
    target: T,
    seed: T,
    iteratee: T,
}

fn parse_scan_effect_args<T: Expression>(
    effect: &T::Signal,
    factory: &impl ExpressionFactory<T>,
) -> Result<ScanEffectArgs<T>, String> {
    let payload = match effect.signal_type() {
        SignalType::Custom { payload, .. } => Ok(payload),
        _ => Err(format!("Invalid {EFFECT_TYPE_SCAN} signal: {effect}")),
    }?;
    let args = factory
        .match_list_term(&payload)
        .filter(|args| args.items().as_deref().len() == 3)
        .ok_or_else(|| {
            format!("Invalid {EFFECT_TYPE_SCAN} signal: Expected 3 arguments, received {payload}",)
        })?;
    let args = args.items();
    let mut args = args.as_deref().iter().map(|item| item.as_deref().clone());
    let target = args.next().unwrap();
    let seed = args.next().unwrap();
    let iteratee = args.next().unwrap();
    Ok(ScanEffectArgs {
        target,
        seed,
        iteratee,
    })
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
