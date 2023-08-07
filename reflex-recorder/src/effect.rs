// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::hash_map::Entry,
    marker::PhantomData,
    ops::Deref,
    time::{Duration, Instant, SystemTime},
};

use reflex::{
    core::{ConditionType, DependencyList, EvaluationResult, Expression, StateToken},
    hash::{HashId, IntMap},
};
use reflex_dispatcher::{Action, Matcher, MessageOffset, ProcessId, TaskFactory};
use reflex_macros::blanket_trait;
use reflex_runtime::{
    action::{
        effect::{
            EffectEmitAction, EffectSubscribeAction, EffectUnsubscribeAction, EffectUpdateBatch,
        },
        evaluate::{
            EvaluateResultAction, EvaluateStartAction, EvaluateStopAction, EvaluateUpdateAction,
        },
    },
    QueryEvaluationMode, QueryInvalidationStrategy,
};
use reflex_scheduler::tokio::{
    AsyncMessage, AsyncMessageMetadata, AsyncMessageTimestamp, TokioCommand, TokioSchedulerLogger,
};
use reflex_utils::event::EventSink;
use serde::{Deserialize, Serialize};

blanket_trait!(
    pub trait EffectLoggerAction<T: Expression>:
        Matcher<EffectSubscribeAction<T>>
        + Matcher<EffectUnsubscribeAction<T>>
        + Matcher<EffectEmitAction<T>>
        + Matcher<EvaluateStartAction<T>>
        + Matcher<EvaluateStopAction<T>>
        + Matcher<EvaluateResultAction<T>>
        + Matcher<EvaluateUpdateAction<T>>
        + From<EffectSubscribeAction<T>>
        + From<EffectUnsubscribeAction<T>>
        + From<EffectEmitAction<T>>
        + From<EvaluateStartAction<T>>
        + From<EvaluateStopAction<T>>
        + From<EvaluateResultAction<T>>
        + From<EvaluateUpdateAction<T>>
    {
    }
);

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEffectEvent<T: Expression> {
    pid: ProcessId,
    action: SerializedEffectAction<T>,
    metadata: Option<SerializedAsyncMessageMetadata>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub enum SerializedEffectAction<T: Expression> {
    EffectSubscribe(SerializedEffectSubscribeAction<T>),
    EffectUnsubscribe(SerializedEffectUnsubscribeAction<T>),
    EmitEffect(SerializedEffectEmitAction<T>),
    EvaluateStart(SerializedEvaluateStartAction<T>),
    EvaluateStop(SerializedEvaluateStopAction<T>),
    EvaluateResult(SerializedEvaluateResultAction<T>),
    EvaluateUpdate(SerializedEvaluateUpdateAction<T>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEffectSubscribeAction<T: Expression> {
    effect_type: MaybeCached<T>,
    effects: Vec<MaybeCached<T::Signal>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEffectUnsubscribeAction<T: Expression> {
    effect_type: MaybeCached<T>,
    effects: Vec<MaybeCached<T::Signal>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEffectEmitAction<T: Expression> {
    effect_types: Vec<SerializedEffectUpdateBatch<T>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEffectUpdateBatch<T: Expression> {
    effect_type: MaybeCached<T>,
    updates: Vec<(MaybeCached<T::Signal>, MaybeCached<T>)>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de> , <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEvaluateStartAction<T: Expression> {
    cache_key: MaybeCached<T::Signal>,
    label: String,
    query: MaybeCached<T>,
    evaluation_mode: QueryEvaluationMode,
    invalidation_strategy: QueryInvalidationStrategy,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de> , <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEvaluateStopAction<T: Expression> {
    cache_key: MaybeCached<T::Signal>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de> , <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEvaluateResultAction<T: Expression> {
    cache_key: MaybeCached<T::Signal>,
    state_index: Option<MessageOffset>,
    value: MaybeCached<T>,
    dependencies: Vec<StateToken>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de> , <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct SerializedEvaluateUpdateAction<T: Expression> {
    cache_key: MaybeCached<T::Signal>,
    state_index: Option<MessageOffset>,
    state_updates: Vec<(MaybeCached<T::Signal>, MaybeCached<T>)>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct SerializedAsyncMessageMetadata {
    offset: MessageOffset,
    redispatched_from: Option<MessageOffset>,
    caller: Option<(MessageOffset, ProcessId)>,
    enqueue_time: SerializedAsyncMessageTimestamp,
}

impl SerializedAsyncMessageMetadata {
    fn new(
        metadata: &AsyncMessageMetadata,
        startup_time: &(SystemTime, Instant),
    ) -> SerializedAsyncMessageMetadata {
        let AsyncMessageMetadata {
            offset,
            redispatched_from,
            caller,
            enqueue_time,
        } = metadata;
        {
            SerializedAsyncMessageMetadata {
                offset: *offset,
                redispatched_from: *redispatched_from,
                caller: *caller,
                enqueue_time: SerializedAsyncMessageTimestamp::new(enqueue_time, startup_time),
            }
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct SerializedAsyncMessageTimestamp {
    #[serde(with = "serialize_timestamp_system_time")]
    startup: SystemTime,
    #[serde(with = "serialize_timestamp_duration")]
    elapsed: Duration,
}

mod serialize_timestamp_system_time {
    use std::time::SystemTime;

    use super::serialize_timestamp_duration;

    pub fn serialize<'a, S>(value: &'a SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let duration = value
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        serialize_timestamp_duration::serialize(&duration, serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let duration = serialize_timestamp_duration::deserialize(deserializer)?;
        Ok(SystemTime::UNIX_EPOCH + duration)
    }
}

mod serialize_timestamp_duration {
    use std::time::Duration;

    use serde::Serialize;
    pub fn serialize<'a, S>(value: &'a Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (secs, nanos) = (value.as_secs(), value.subsec_nanos());
        (secs, nanos).serialize(serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (secs, nanos): (u64, u32) = serde::Deserialize::deserialize(deserializer)?;
        let duration = Duration::new(secs, nanos);
        Ok(duration)
    }
}

impl SerializedAsyncMessageTimestamp {
    fn new(
        timestamp: &AsyncMessageTimestamp,
        startup_time: &(SystemTime, Instant),
    ) -> SerializedAsyncMessageTimestamp {
        let (startup_time, startup_instant) = startup_time;
        let elapsed = timestamp.time().duration_since(*startup_instant);
        Self {
            startup: *startup_time,
            elapsed,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub enum EffectLoggerEvent<T: Expression> {
    Send {
        pid: ProcessId,
        action: SerializedEffectLoggerAction<T>,
        metadata: Option<SerializedAsyncMessageMetadata>,
    },
    Receive {
        pid: ProcessId,
        action: SerializedEffectLoggerAction<T>,
        metadata: Option<SerializedAsyncMessageMetadata>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub enum SerializedEffectLoggerAction<T: Expression> {
    EffectSubscribe(EffectSubscribeAction<T>),
    EffectUnsubscribe(EffectUnsubscribeAction<T>),
    EffectEmit(EffectEmitAction<T>),
    EvaluateStart(EvaluateStartAction<T>),
    EvaluateStop(EvaluateStopAction<T>),
    EvaluateResult(EvaluateResultAction<T>),
    EvaluateUpdate(EvaluateUpdateAction<T>),
}

#[derive(Copy, Debug)]
pub struct EffectRecorder<T: Expression, TRecorder, TAction, TTask> {
    recorder: TRecorder,
    startup_time: (SystemTime, Instant),
    _expression: PhantomData<T>,
    _action: PhantomData<TAction>,
    _task: PhantomData<TTask>,
}

impl<T: Expression, TRecorder, TAction, TTask> Clone
    for EffectRecorder<T, TRecorder, TAction, TTask>
where
    TRecorder: Clone,
{
    fn clone(&self) -> Self {
        Self {
            recorder: self.recorder.clone(),
            startup_time: self.startup_time,
            _expression: PhantomData,
            _action: PhantomData,
            _task: PhantomData,
        }
    }
}

impl<T: Expression, TRecorder, TAction, TTask> EffectRecorder<T, TRecorder, TAction, TTask> {
    pub fn new(recorder: TRecorder) -> Self {
        Self {
            recorder,
            startup_time: (SystemTime::now(), Instant::now()),
            _expression: PhantomData,
            _action: PhantomData,
            _task: PhantomData,
        }
    }
}

impl<T: Expression, TRecorder, TAction, TTask> EffectRecorder<T, TRecorder, TAction, TTask>
where
    TAction: EffectLoggerAction<T>,
{
    fn parse_action(&self, action: &TAction) -> Option<SerializedEffectLoggerAction<T>> {
        if let Option::<&EffectSubscribeAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EffectSubscribe(
                action.clone(),
            ))
        } else if let Option::<&EffectUnsubscribeAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EffectUnsubscribe(
                action.clone(),
            ))
        } else if let Option::<&EffectEmitAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EffectEmit(action.clone()))
        } else if let Option::<&EvaluateStartAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EvaluateStart(action.clone()))
        } else if let Option::<&EvaluateStopAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EvaluateStop(action.clone()))
        } else if let Option::<&EvaluateResultAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EvaluateResult(action.clone()))
        } else if let Option::<&EvaluateUpdateAction<T>>::Some(action) = action.match_type() {
            Some(SerializedEffectLoggerAction::EvaluateUpdate(action.clone()))
        } else {
            None
        }
    }
}

impl<T: Expression, TRecorder, TAction, TTask> TokioSchedulerLogger
    for EffectRecorder<T, TRecorder, TAction, TTask>
where
    TRecorder: EventSink<Event = EffectLoggerEvent<T>>,
    TAction: Action + EffectLoggerAction<T> + Clone + 'static,
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
            TokioCommand::Send { pid, message } => {
                if let None = message.redispatched_from() {
                    let metadata = message.metadata().map(|metadata| {
                        SerializedAsyncMessageMetadata::new(metadata, &self.startup_time)
                    });
                    let action = message.deref();
                    if let Some(action) = self.parse_action(action) {
                        self.recorder.emit(&EffectLoggerEvent::Send {
                            pid: *pid,
                            action,
                            metadata,
                        })
                    }
                }
            }
            _ => {}
        }
    }
    fn log_worker_message(
        &mut self,
        message: &AsyncMessage<Self::Action>,
        _actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        pid: ProcessId,
    ) {
        if let None = message.redispatched_from() {
            let metadata = message
                .metadata()
                .map(|metadata| SerializedAsyncMessageMetadata::new(metadata, &self.startup_time));
            let action = message.deref();
            if let Some(action) = self.parse_action(action) {
                self.recorder.emit(&EffectLoggerEvent::Receive {
                    pid,
                    action,
                    metadata,
                })
            }
        }
    }
    fn log_task_message(&mut self, message: &AsyncMessage<Self::Action>, pid: ProcessId) {
        if let None = message.redispatched_from() {
            let metadata = message
                .metadata()
                .map(|metadata| SerializedAsyncMessageMetadata::new(metadata, &self.startup_time));
            let action = message.deref();
            if let Some(action) = self.parse_action(action) {
                self.recorder.emit(&EffectLoggerEvent::Receive {
                    pid,
                    action,
                    metadata,
                })
            }
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectEventSerializerEventSink<T: Expression, TRecorder> {
    serializer: EffectEventSerializer<T>,
    inner: TRecorder,
}

impl<T: Expression, TRecorder> EffectEventSerializerEventSink<T, TRecorder> {
    pub fn new(inner: TRecorder) -> Self {
        Self {
            serializer: Default::default(),
            inner,
        }
    }
}

impl<T: Expression, TRecorder> EventSink for EffectEventSerializerEventSink<T, TRecorder>
where
    TRecorder: EventSink<Event = SerializedEffectEvent<T>>,
{
    type Event = EffectLoggerEvent<T>;
    fn emit(&mut self, event: &Self::Event) {
        if let Some((pid, action, metadata)) = match event {
            EffectLoggerEvent::Send {
                pid,
                action,
                metadata,
            } => Some((*pid, action, *metadata)),
            EffectLoggerEvent::Receive { .. } => None,
        } {
            let serialized_action = match action {
                SerializedEffectLoggerAction::EffectSubscribe(action) => {
                    Some(self.serializer.serialize_effect_subscribe_action(action))
                }
                SerializedEffectLoggerAction::EffectUnsubscribe(action) => {
                    Some(self.serializer.serialize_effect_unsubscribe_action(action))
                }
                SerializedEffectLoggerAction::EffectEmit(action) => {
                    Some(self.serializer.serialize_effect_emit_action(action))
                }
                SerializedEffectLoggerAction::EvaluateStart(action) => {
                    Some(self.serializer.serialize_evaluate_start_action(action))
                }
                SerializedEffectLoggerAction::EvaluateStop(action) => {
                    Some(self.serializer.serialize_evaluate_stop_action(action))
                }
                SerializedEffectLoggerAction::EvaluateResult(action) => {
                    Some(self.serializer.serialize_evaluate_result_action(action))
                }
                SerializedEffectLoggerAction::EvaluateUpdate(action) => {
                    Some(self.serializer.serialize_evaluate_update_action(action))
                }
            };
            if let Some(serialized_action) = serialized_action {
                let serialized_event = SerializedEffectEvent {
                    pid,
                    metadata,
                    action: serialized_action,
                };
                self.inner.emit(&serialized_event);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EffectEventSerializer<T: Expression> {
    cache: EffectCache<T>,
}

impl<T: Expression> Default for EffectEventSerializer<T> {
    fn default() -> Self {
        Self {
            cache: Default::default(),
        }
    }
}

impl<T: Expression> EffectEventSerializer<T> {
    pub fn serialize_actions<'a, TAction>(
        &'a mut self,
        actions: impl IntoIterator<
            Item = &'a TAction,
            IntoIter = impl Iterator<Item = &'a TAction> + 'a,
        >,
    ) -> impl Iterator<Item = SerializedEffectAction<T>> + 'a
    where
        TAction: EffectLoggerAction<T> + 'a,
    {
        actions
            .into_iter()
            .flat_map(|action| self.serialize_action(action))
    }

    fn serialize_action<TAction: EffectLoggerAction<T>>(
        &mut self,
        action: &TAction,
    ) -> Option<SerializedEffectAction<T>> {
        if let Some(action) = action.match_type() {
            Some(self.serialize_effect_subscribe_action(action))
        } else if let Some(action) = action.match_type() {
            Some(self.serialize_effect_unsubscribe_action(action))
        } else if let Some(action) = action.match_type() {
            Some(self.serialize_effect_emit_action(action))
        } else if let Some(action) = action.match_type() {
            Some(self.serialize_evaluate_start_action(action))
        } else if let Some(action) = action.match_type() {
            Some(self.serialize_evaluate_stop_action(action))
        } else if let Some(action) = action.match_type() {
            Some(self.serialize_evaluate_result_action(action))
        } else if let Some(action) = action.match_type() {
            Some(self.serialize_evaluate_update_action(action))
        } else {
            None
        }
    }

    fn serialize_effect_subscribe_action(
        &mut self,
        action: &EffectSubscribeAction<T>,
    ) -> SerializedEffectAction<T> {
        let effect_type = self.cache.store_value(&action.effect_type);
        let effects = action
            .effects
            .iter()
            .map(|condition| self.cache.store_condition(condition))
            .collect::<Vec<_>>();
        SerializedEffectAction::EffectSubscribe(SerializedEffectSubscribeAction {
            effect_type,
            effects,
        })
    }

    fn serialize_effect_unsubscribe_action(
        &mut self,
        action: &EffectUnsubscribeAction<T>,
    ) -> SerializedEffectAction<T> {
        let effect_type = self.cache.store_value(&action.effect_type);
        let effects = action
            .effects
            .iter()
            .map(|condition| self.cache.store_condition(&condition))
            .collect::<Vec<_>>();
        SerializedEffectAction::EffectUnsubscribe(SerializedEffectUnsubscribeAction {
            effect_type,
            effects,
        })
    }

    fn serialize_effect_emit_action(
        &mut self,
        action: &EffectEmitAction<T>,
    ) -> SerializedEffectAction<T> {
        let effect_types = action
            .effect_types
            .iter()
            .map(|update_batch| {
                let effect_type = self.cache.store_value(&update_batch.effect_type);
                let updates = update_batch
                    .updates
                    .iter()
                    .map(|(condition, value)| {
                        (
                            self.cache.store_condition(condition),
                            self.cache.store_value(value),
                        )
                    })
                    .collect::<Vec<_>>();
                SerializedEffectUpdateBatch {
                    effect_type,
                    updates,
                }
            })
            .collect::<Vec<_>>();
        SerializedEffectAction::EmitEffect(SerializedEffectEmitAction { effect_types })
    }

    fn serialize_evaluate_start_action(
        &mut self,
        action: &EvaluateStartAction<T>,
    ) -> SerializedEffectAction<T> {
        let cache_key = self.cache.store_condition(&action.cache_key);
        let label = action.label.clone();
        let query = self.cache.store_value(&action.query);
        let evaluation_mode = action.evaluation_mode;
        let invalidation_strategy = action.invalidation_strategy;
        SerializedEffectAction::EvaluateStart(SerializedEvaluateStartAction {
            cache_key,
            label,
            query,
            evaluation_mode,
            invalidation_strategy,
        })
    }

    fn serialize_evaluate_stop_action(
        &mut self,
        action: &EvaluateStopAction<T>,
    ) -> SerializedEffectAction<T> {
        let cache_key = self.cache.store_condition(&action.cache_key);
        SerializedEffectAction::EvaluateStop(SerializedEvaluateStopAction { cache_key })
    }

    fn serialize_evaluate_result_action(
        &mut self,
        action: &EvaluateResultAction<T>,
    ) -> SerializedEffectAction<T> {
        let cache_key = self.cache.store_condition(&action.cache_key);
        let state_index = action.state_index;
        let value = self.cache.store_value(&action.result.result());
        let dependencies = action.result.dependencies().iter().collect::<Vec<_>>();
        SerializedEffectAction::EvaluateResult(SerializedEvaluateResultAction {
            cache_key,
            state_index,
            value,
            dependencies,
        })
    }

    fn serialize_evaluate_update_action(
        &mut self,
        action: &EvaluateUpdateAction<T>,
    ) -> SerializedEffectAction<T> {
        let cache_key = self.cache.store_condition(&action.cache_key);
        let state_index = action.state_index;
        let state_updates = action
            .state_updates
            .iter()
            .map(|(condition, value)| {
                (
                    self.cache.store_condition(condition),
                    self.cache.store_value(value),
                )
            })
            .collect::<Vec<_>>();
        SerializedEffectAction::EvaluateUpdate(SerializedEvaluateUpdateAction {
            cache_key,
            state_index,
            state_updates,
        })
    }
}

#[derive(Clone, Debug)]
pub struct EffectEventDeserializer<T: Expression> {
    cache: EffectCache<T>,
}

impl<T: Expression> Default for EffectEventDeserializer<T> {
    fn default() -> Self {
        Self {
            cache: EffectCache::default(),
        }
    }
}

impl<T: Expression> EffectEventDeserializer<T> {
    pub fn deserialize_actions<'a, TAction>(
        &'a mut self,
        actions: impl IntoIterator<
            Item = &'a SerializedEffectAction<T>,
            IntoIter = impl Iterator<Item = &'a SerializedEffectAction<T>> + 'a,
        >,
    ) -> impl Iterator<Item = TAction> + 'a
    where
        TAction: EffectLoggerAction<T> + 'a,
    {
        actions
            .into_iter()
            .flat_map(|action| self.deserialize_action(action))
    }

    pub fn deserialize_action<TAction>(
        &mut self,
        action: &SerializedEffectAction<T>,
    ) -> Option<TAction>
    where
        TAction: EffectLoggerAction<T>,
    {
        if let Some(SerializedEffectAction::EffectSubscribe(action)) = action.match_type() {
            self.deserialize_effect_subscribe_action(action)
        } else if let Some(SerializedEffectAction::EffectUnsubscribe(action)) = action.match_type()
        {
            self.deserialize_effect_unsubscribe_action(action)
        } else if let Some(SerializedEffectAction::EmitEffect(action)) = action.match_type() {
            self.deserialize_effect_emit_action(action)
        } else if let Some(SerializedEffectAction::EvaluateStart(action)) = action.match_type() {
            self.deserialize_evaluate_start_action(action)
        } else if let Some(SerializedEffectAction::EvaluateStop(action)) = action.match_type() {
            self.deserialize_evaluate_stop_action(action)
        } else if let Some(SerializedEffectAction::EvaluateResult(action)) = action.match_type() {
            self.deserialize_evaluate_result_action(action)
        } else if let Some(SerializedEffectAction::EvaluateUpdate(action)) = action.match_type() {
            self.deserialize_evaluate_update_action(action)
        } else {
            None
        }
    }

    fn deserialize_effect_subscribe_action<TAction>(
        &mut self,
        action: &SerializedEffectSubscribeAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EffectSubscribeAction<T>>,
    {
        let effect_type = self.cache.retrieve_value(&action.effect_type)?;
        let effects = action
            .effects
            .iter()
            .map(|condition| self.cache.retrieve_condition(condition))
            .collect::<Option<Vec<_>>>()?;
        Some(TAction::from(EffectSubscribeAction {
            effect_type: effect_type.clone(),
            effects,
        }))
    }

    fn deserialize_effect_unsubscribe_action<TAction>(
        &mut self,
        action: &SerializedEffectUnsubscribeAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EffectUnsubscribeAction<T>>,
    {
        let effect_type = self.cache.retrieve_value(&action.effect_type)?;
        let effects = action
            .effects
            .iter()
            .map(|condition| self.cache.retrieve_condition(condition))
            .collect::<Option<Vec<_>>>()?;
        Some(TAction::from(EffectUnsubscribeAction {
            effect_type: effect_type.clone(),
            effects,
        }))
    }

    fn deserialize_effect_emit_action<TAction>(
        &mut self,
        action: &SerializedEffectEmitAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EffectEmitAction<T>>,
    {
        let effect_types = action
            .effect_types
            .iter()
            .map(|update_batch| {
                let effect_type = self.cache.retrieve_value(&update_batch.effect_type)?;
                let updates = update_batch
                    .updates
                    .iter()
                    .map(|(condition, value)| {
                        Some((
                            self.cache.retrieve_condition(condition)?,
                            self.cache.retrieve_value(value)?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(EffectUpdateBatch {
                    effect_type,
                    updates,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(TAction::from(EffectEmitAction { effect_types }))
    }

    fn deserialize_evaluate_start_action<TAction>(
        &mut self,
        action: &SerializedEvaluateStartAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EvaluateStartAction<T>>,
    {
        let cache_key = self.cache.retrieve_condition(&action.cache_key)?;
        let label = action.label.clone();
        let query = self.cache.retrieve_value(&action.query)?;
        let evaluation_mode = action.evaluation_mode;
        let invalidation_strategy = action.invalidation_strategy;
        Some(TAction::from(EvaluateStartAction {
            cache_key,
            label,
            query,
            evaluation_mode,
            invalidation_strategy,
        }))
    }

    fn deserialize_evaluate_stop_action<TAction>(
        &mut self,
        action: &SerializedEvaluateStopAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EvaluateStopAction<T>>,
    {
        let cache_key = self.cache.retrieve_condition(&action.cache_key)?;
        Some(TAction::from(EvaluateStopAction { cache_key }))
    }

    fn deserialize_evaluate_result_action<TAction>(
        &mut self,
        action: &SerializedEvaluateResultAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EvaluateResultAction<T>>,
    {
        let cache_key = self.cache.retrieve_condition(&action.cache_key)?;
        let state_index = action.state_index;
        let result = EvaluationResult::new(
            self.cache.retrieve_value(&action.value)?,
            DependencyList::from_iter(action.dependencies.iter().copied()),
        );
        Some(TAction::from(EvaluateResultAction {
            cache_key,
            state_index,
            result,
        }))
    }

    fn deserialize_evaluate_update_action<TAction>(
        &mut self,
        action: &SerializedEvaluateUpdateAction<T>,
    ) -> Option<TAction>
    where
        TAction: From<EvaluateUpdateAction<T>>,
    {
        let cache_key = self.cache.retrieve_condition(&action.cache_key)?;
        let state_index = action.state_index;
        let state_updates = action
            .state_updates
            .iter()
            .map(|(condition, value)| {
                Some((
                    self.cache.retrieve_condition(condition)?,
                    self.cache.retrieve_value(value)?,
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(TAction::from(EvaluateUpdateAction {
            cache_key,
            state_index,
            state_updates,
        }))
    }
}

#[derive(Clone, Debug)]
struct EffectCache<T: Expression> {
    conditions: IntMap<StateToken, T::Signal>,
    values: IntMap<HashId, T>,
}

impl<T: Expression> Default for EffectCache<T> {
    fn default() -> Self {
        Self {
            conditions: Default::default(),
            values: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum MaybeCached<T> {
    Cached(HashId),
    Uncached(HashId, T),
}

// TODO: Implement garbage collection for effect recorder cache
impl<T: Expression> EffectCache<T> {
    fn store_condition(&mut self, condition: &T::Signal) -> MaybeCached<T::Signal> {
        let cache_key = condition.id();
        match self.conditions.entry(cache_key) {
            Entry::Vacant(entry) => {
                let condition = condition.clone();
                entry.insert(condition.clone());
                MaybeCached::Uncached(cache_key, condition)
            }
            Entry::Occupied(_) => MaybeCached::Cached(cache_key),
        }
    }
    fn store_value(&mut self, value: &T) -> MaybeCached<T> {
        let cache_key = value.id();
        match self.values.entry(value.id()) {
            Entry::Vacant(entry) => {
                let value = value.clone();
                entry.insert(value.clone());
                MaybeCached::Uncached(cache_key, value)
            }
            Entry::Occupied(_) => MaybeCached::Cached(cache_key),
        }
    }
    fn retrieve_condition(&mut self, cache_entry: &MaybeCached<T::Signal>) -> Option<T::Signal> {
        let (cache_key, value) = match cache_entry {
            MaybeCached::Cached(cache_key) => (cache_key, None),
            MaybeCached::Uncached(cache_key, value) => (cache_key, Some(value)),
        };
        match self.conditions.entry(*cache_key) {
            Entry::Vacant(entry) => match value {
                None => None,
                Some(value) => Some(entry.insert(value.clone()).clone()),
            },
            Entry::Occupied(mut entry) => match value {
                None => Some(entry.get().clone()),
                Some(value) => {
                    entry.insert(value.clone());
                    Some(entry.get().clone())
                }
            },
        }
    }
    fn retrieve_value(&mut self, cache_entry: &MaybeCached<T>) -> Option<T> {
        let (cache_key, value) = match cache_entry {
            MaybeCached::Cached(cache_key) => (cache_key, None),
            MaybeCached::Uncached(cache_key, value) => (cache_key, Some(value)),
        };
        match self.values.entry(*cache_key) {
            Entry::Vacant(entry) => match value {
                None => None,
                Some(value) => Some(entry.insert(value.clone()).clone()),
            },
            Entry::Occupied(mut entry) => match value {
                None => Some(entry.get().clone()),
                Some(value) => {
                    entry.insert(value.clone());
                    Some(entry.get().clone())
                }
            },
        }
    }
}
