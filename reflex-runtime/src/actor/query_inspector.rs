// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData,
};

use reflex::core::{
    ConditionListType, ConditionType, DependencyList, EvaluationResult, Expression,
    ExpressionFactory, RefType, SignalTermType, SignalType, StateToken,
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, NoopDisposeCallback, SchedulerMode,
    SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_json::{json, JsonValue};
use reflex_macros::{dispatcher, Named};

use crate::{
    action::{
        effect::{EffectEmitAction, EffectSubscribeAction, EffectUnsubscribeAction},
        evaluate::{EvaluateResultAction, EvaluateStartAction, EvaluateStopAction},
    },
    QueryEvaluationMode, QueryInvalidationStrategy,
};

#[derive(Named, Clone)]
pub struct QueryInspector<T: Expression> {
    _expression: PhantomData<T>,
}
impl<T: Expression> Default for QueryInspector<T> {
    fn default() -> Self {
        Self {
            _expression: Default::default(),
        }
    }
}

pub struct QueryInspectorState<T: Expression> {
    active_workers: HashMap<StateToken, QueryInspectorWorkerState<T>>,
    active_effects: HashMap<StateToken, QueryInspectorEffectState<T>>,
}
impl<T: Expression> Default for QueryInspectorState<T> {
    fn default() -> Self {
        Self {
            active_workers: Default::default(),
            active_effects: Default::default(),
        }
    }
}
impl<T: Expression> QueryInspectorState<T> {
    pub fn to_json(&self, factory: &impl ExpressionFactory<T>) -> JsonValue {
        let queries = self.active_workers.iter().map(|(worker_id, worker_state)| {
            worker_state.to_json(*worker_id, &self.active_effects, factory)
        });
        let effects = self
            .active_effects
            .values()
            .filter(|effect_state| is_unresolved_effect_state(effect_state, factory))
            .map(|effect_state| {
                json!({
                    "effect": serialize_effect(&effect_state.effect),
                    "value": serialize_effect_result(effect_state.value.as_ref(), factory),
                })
            });
        json!({
            "numEffects": &self.active_effects.len(),
            "queries": queries.collect::<Vec<_>>(),
            "effects": effects.collect::<Vec<_>>()
        })
    }
}

fn serialize_query_result<T: Expression>(
    result: Option<&EvaluationResult<T>>,
    active_effects: &HashMap<StateToken, QueryInspectorEffectState<T>>,
    factory: &impl ExpressionFactory<T>,
) -> JsonValue {
    match result {
        None => JsonValue::Null,
        Some(result) => json!({
            "value": serialize_value(result.result(), factory),
            "dependencies": JsonValue::Array(get_unresolved_dependencies(result.dependencies(), active_effects, factory).map(JsonValue::from).collect()),
        }),
    }
}

fn get_unresolved_dependencies<'a, T: Expression>(
    dependencies: &'a DependencyList,
    active_effects: &'a HashMap<StateToken, QueryInspectorEffectState<T>>,
    factory: &'a impl ExpressionFactory<T>,
) -> impl Iterator<Item = StateToken> + 'a {
    dependencies
        .iter()
        .filter(|state_token| is_unresolved_dependency(state_token, active_effects, factory))
}

fn is_unresolved_dependency<T: Expression>(
    state_token: &StateToken,
    active_effects: &HashMap<StateToken, QueryInspectorEffectState<T>>,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    match active_effects.get(state_token) {
        None => true,
        Some(effect_state) => is_unresolved_effect_state(effect_state, factory),
    }
}

fn is_unresolved_effect_state<T: Expression>(
    effect_state: &QueryInspectorEffectState<T>,
    factory: &impl ExpressionFactory<T>,
) -> bool {
    match effect_state.value.as_ref() {
        None => true,
        Some(value) => factory
            .match_signal_term(value)
            .map(|term| {
                term.signals()
                    .as_deref()
                    .iter()
                    .any(|effect| is_unresolved_effect(effect.as_deref()))
            })
            .unwrap_or(false),
    }
}

fn is_unresolved_effect<T: Expression>(effect: &impl ConditionType<T>) -> bool {
    match effect.signal_type() {
        SignalType::Error { .. } => false,
        SignalType::Pending | SignalType::Custom { .. } => true,
    }
}

fn serialize_effect_result<T: Expression>(
    result: Option<&T>,
    factory: &impl ExpressionFactory<T>,
) -> JsonValue {
    match result {
        None => JsonValue::Null,
        Some(value) => serialize_value(value, factory),
    }
}

fn serialize_value<T: Expression>(value: &T, factory: &impl ExpressionFactory<T>) -> JsonValue {
    if let Some(value) = factory.match_signal_term(value) {
        JsonValue::Array(
            value
                .signals()
                .as_deref()
                .iter()
                .map(|effect| serialize_effect(effect.as_deref()))
                .collect(),
        )
    } else {
        JsonValue::Number(value.id().into())
    }
}

pub struct QueryInspectorEffectState<T: Expression> {
    effect: T::Signal,
    value: Option<T>,
}

struct QueryInspectorWorkerState<T: Expression> {
    label: String,
    #[allow(dead_code)]
    query: T,
    #[allow(dead_code)]
    evaluation_mode: QueryEvaluationMode,
    #[allow(dead_code)]
    invalidation_strategy: QueryInvalidationStrategy,
    latest_result: Option<EvaluationResult<T>>,
}
impl<T: Expression> QueryInspectorWorkerState<T> {
    fn to_json(
        &self,
        worker_id: StateToken,
        active_effects: &HashMap<StateToken, QueryInspectorEffectState<T>>,
        factory: &impl ExpressionFactory<T>,
    ) -> JsonValue {
        json!({
            "id": worker_id,
            "label": &self.label,
            "result": serialize_query_result(self.latest_result.as_ref(), active_effects, factory),
        })
    }
}

fn serialize_effect<T: Expression>(effect: &impl ConditionType<T>) -> JsonValue {
    match effect.signal_type() {
        SignalType::Custom {
            effect_type,
            payload,
            token,
        } => json!({
            "id": JsonValue::Number(effect.id().into()),
            "type": reflex_json::sanitize(&effect_type).unwrap_or_else(|_| json!({})),
            "payload": reflex_json::sanitize(&payload).unwrap_or_else(|_| json!({})),
            "token": reflex_json::sanitize(&token).unwrap_or_else(|_| json!({})),
        }),
        SignalType::Error { payload } => json!({
            "id": JsonValue::Number(effect.id().into()),
            "type": "error",
            "payload": reflex_json::sanitize(&payload).unwrap_or_else(|_| json!({})),
            "token": JsonValue::Null,
        }),
        SignalType::Pending => json!({
            "id": JsonValue::Number(effect.id().into()),
            "type": "pending",
            "payload": JsonValue::Null,
            "token": JsonValue::Null,
        }),
    }
}

dispatcher!({
    pub enum QueryInspectorAction<T: Expression> {
        Inbox(EvaluateStartAction<T>),
        Inbox(EvaluateStopAction<T>),
        Inbox(EvaluateResultAction<T>),
        Inbox(EffectSubscribeAction<T>),
        Inbox(EffectUnsubscribeAction<T>),
        Inbox(EffectEmitAction<T>),
    }

    impl<T: Expression, TAction, TTask> Dispatcher<TAction, TTask> for QueryInspector<T>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        type State = QueryInspectorState<T>;
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

        fn accept(&self, _action: &EvaluateResultAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &EvaluateResultAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &EvaluateResultAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_evaluate_result(state, action, metadata, context)
        }

        fn accept(&self, _action: &EffectSubscribeAction<T>) -> bool {
            true
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

        fn accept(&self, _action: &EffectUnsubscribeAction<T>) -> bool {
            true
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
impl<
        T: Expression,
        TAction: Action + QueryInspectorAction<T>,
        TTask: TaskFactory<TAction, TTask>,
    > TaskFactory<TAction, TTask> for QueryInspector<T>
{
    type Actor = Self;
    fn create(self) -> Self::Actor {
        self
    }
}

impl<T: Expression> QueryInspector<T> {
    fn handle_evaluate_start<TAction, TTask>(
        &self,
        state: &mut QueryInspectorState<T>,
        action: &EvaluateStartAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EvaluateStartAction {
            cache_key,
            label,
            query,
            evaluation_mode,
            invalidation_strategy,
        } = action;
        let worker_id = cache_key.id();
        match state.active_workers.entry(worker_id) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                entry.insert(QueryInspectorWorkerState {
                    label: label.clone(),
                    query: query.clone(),
                    evaluation_mode: *evaluation_mode,
                    invalidation_strategy: *invalidation_strategy,
                    latest_result: Default::default(),
                });
                None
            }
        }
    }
    fn handle_evaluate_stop<TAction, TTask>(
        &self,
        state: &mut QueryInspectorState<T>,
        action: &EvaluateStopAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EvaluateStopAction { cache_key } = action;
        let worker_id = cache_key.id();
        match state.active_workers.entry(worker_id) {
            Entry::Occupied(entry) => {
                entry.remove();
                None
            }
            Entry::Vacant(_) => None,
        }
    }
    fn handle_evaluate_result<TAction, TTask>(
        &self,
        state: &mut QueryInspectorState<T>,
        action: &EvaluateResultAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EvaluateResultAction {
            cache_key,
            state_index: _,
            result,
        } = action;
        let worker_id = cache_key.id();
        let worker_state = state.active_workers.get_mut(&worker_id)?;
        worker_state.latest_result.replace(result.clone());
        None
    }
    fn handle_effect_subscribe<TAction, TTask>(
        &self,
        state: &mut QueryInspectorState<T>,
        action: &EffectSubscribeAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectSubscribeAction {
            effect_type: _,
            effects,
        } = action;
        state.active_effects.extend(effects.iter().map(|effect| {
            (
                effect.id(),
                QueryInspectorEffectState {
                    effect: effect.clone(),
                    value: None,
                },
            )
        }));
        None
    }
    fn handle_effect_unsubscribe<TAction, TTask>(
        &self,
        state: &mut QueryInspectorState<T>,
        action: &EffectUnsubscribeAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectUnsubscribeAction {
            effect_type: _,
            effects,
        } = action;
        for state_token in effects.iter().map(|effect| effect.id()) {
            state.active_effects.remove(&state_token);
        }
        None
    }
    fn handle_effect_emit<TAction, TTask>(
        &self,
        state: &mut QueryInspectorState<T>,
        action: &EffectEmitAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        let EffectEmitAction {
            effect_types: updates,
        } = action;
        for (key, value) in updates.iter().flat_map(|batch| batch.updates.iter()) {
            let state_token = key.id();
            if let Some(effect_state) = state.active_effects.get_mut(&state_token) {
                effect_state.value.replace(value.clone());
            }
        }
        None
    }
}
