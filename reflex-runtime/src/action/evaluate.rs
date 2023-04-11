// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{ConditionType, EvaluationResult, Expression};
use reflex_dispatcher::{Action, MessageOffset, Named, SerializableAction, SerializedAction};
use reflex_json::JsonValue;
use reflex_macros::Named;
use serde::{Deserialize, Serialize};

use crate::{QueryEvaluationMode, QueryInvalidationStrategy};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub enum EvaluateActions<T: Expression> {
    Start(EvaluateStartAction<T>),
    Update(EvaluateUpdateAction<T>),
    Stop(EvaluateStopAction<T>),
    Result(EvaluateResultAction<T>),
}
impl<T: Expression> Action for EvaluateActions<T> {}
impl<T: Expression> Named for EvaluateActions<T> {
    fn name(&self) -> &'static str {
        match self {
            Self::Start(action) => action.name(),
            Self::Update(action) => action.name(),
            Self::Stop(action) => action.name(),
            Self::Result(action) => action.name(),
        }
    }
}
impl<T: Expression> SerializableAction for EvaluateActions<T> {
    fn to_json(&self) -> SerializedAction {
        match self {
            Self::Start(action) => action.to_json(),
            Self::Update(action) => action.to_json(),
            Self::Stop(action) => action.to_json(),
            Self::Result(action) => action.to_json(),
        }
    }
}

impl<T: Expression> From<EvaluateStartAction<T>> for EvaluateActions<T> {
    fn from(value: EvaluateStartAction<T>) -> Self {
        Self::Start(value)
    }
}
impl<T: Expression> From<EvaluateActions<T>> for Option<EvaluateStartAction<T>> {
    fn from(value: EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Start(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a EvaluateActions<T>> for Option<&'a EvaluateStartAction<T>> {
    fn from(value: &'a EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Start(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<EvaluateUpdateAction<T>> for EvaluateActions<T> {
    fn from(value: EvaluateUpdateAction<T>) -> Self {
        Self::Update(value)
    }
}
impl<T: Expression> From<EvaluateActions<T>> for Option<EvaluateUpdateAction<T>> {
    fn from(value: EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Update(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a EvaluateActions<T>> for Option<&'a EvaluateUpdateAction<T>> {
    fn from(value: &'a EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Update(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<EvaluateStopAction<T>> for EvaluateActions<T> {
    fn from(value: EvaluateStopAction<T>) -> Self {
        Self::Stop(value)
    }
}
impl<T: Expression> From<EvaluateActions<T>> for Option<EvaluateStopAction<T>> {
    fn from(value: EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Stop(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a EvaluateActions<T>> for Option<&'a EvaluateStopAction<T>> {
    fn from(value: &'a EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Stop(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<EvaluateResultAction<T>> for EvaluateActions<T> {
    fn from(value: EvaluateResultAction<T>) -> Self {
        Self::Result(value)
    }
}
impl<T: Expression> From<EvaluateActions<T>> for Option<EvaluateResultAction<T>> {
    fn from(value: EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Result(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a EvaluateActions<T>> for Option<&'a EvaluateResultAction<T>> {
    fn from(value: &'a EvaluateActions<T>) -> Self {
        match value {
            EvaluateActions::Result(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct EvaluateStartAction<T: Expression> {
    pub cache_key: T::Signal,
    pub label: String,
    pub query: T,
    pub evaluation_mode: QueryEvaluationMode,
    pub invalidation_strategy: QueryInvalidationStrategy,
}
impl<T: Expression> Action for EvaluateStartAction<T> {}
impl<T: Expression> SerializableAction for EvaluateStartAction<T> {
    fn to_json(&self) -> SerializedAction {
        SerializedAction::from_iter([
            ("cache_id", JsonValue::from(self.cache_key.id())),
            ("label", JsonValue::from(self.label.clone())),
            ("query_id", JsonValue::from(self.query.id())),
            (
                "standalone",
                JsonValue::from(match self.evaluation_mode {
                    QueryEvaluationMode::Standalone => true,
                    QueryEvaluationMode::Query => false,
                }),
            ),
            (
                "batch_updates",
                JsonValue::from(match self.invalidation_strategy {
                    QueryInvalidationStrategy::CombineUpdateBatches => true,
                    QueryInvalidationStrategy::Exact => false,
                }),
            ),
        ])
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct EvaluateUpdateAction<T: Expression> {
    pub cache_key: T::Signal,
    pub state_index: Option<MessageOffset>,
    pub state_updates: Vec<(T::Signal, T)>,
}
impl<T: Expression> Action for EvaluateUpdateAction<T> {}
impl<T: Expression> SerializableAction for EvaluateUpdateAction<T> {
    fn to_json(&self) -> SerializedAction {
        SerializedAction::from_iter([
            ("cache_id", JsonValue::from(self.cache_key.id())),
            (
                "state_index",
                match self.state_index {
                    None => JsonValue::Null,
                    Some(value) => value.into(),
                },
            ),
            (
                "num_updates",
                JsonValue::Number(self.state_updates.len().into()),
            ),
        ])
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "<T as Expression>::Signal: Serialize",
    deserialize = "<T as Expression>::Signal: Deserialize<'de>"
))]
pub struct EvaluateStopAction<T: Expression> {
    pub cache_key: T::Signal,
}
impl<T: Expression> SerializableAction for EvaluateStopAction<T> {
    fn to_json(&self) -> SerializedAction {
        SerializedAction::from_iter([("cache_id", JsonValue::from(self.cache_key.id()))])
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct EvaluateResultAction<T: Expression> {
    pub cache_key: T::Signal,
    pub state_index: Option<MessageOffset>,
    pub result: EvaluationResult<T>,
}
impl<T: Expression> Action for EvaluateResultAction<T> {}
impl<T: Expression> SerializableAction for EvaluateResultAction<T> {
    fn to_json(&self) -> SerializedAction {
        SerializedAction::from_iter([
            ("cache_id", JsonValue::from(self.cache_key.id())),
            (
                "state_index",
                match self.state_index {
                    None => JsonValue::Null,
                    Some(value) => value.into(),
                },
            ),
            ("result_id", JsonValue::from(self.result.result().id())),
        ])
    }
}
