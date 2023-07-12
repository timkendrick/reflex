// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use reflex::core::{ConditionType, EvaluationResult, Expression};
use reflex_dispatcher::{Action, MessageOffset, Named, SerializableAction, SerializedAction};
use reflex_json::{JsonMap, JsonValue};
use reflex_macros::Named;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum BytecodeInterpreterActions<T: Expression> {
    #[serde(bound(
        serialize = "<T as Expression>::Signal: Serialize",
        deserialize = "<T as Expression>::Signal: Deserialize<'de>"
    ))]
    Init(BytecodeInterpreterInitAction<T>),
    Evaluate(BytecodeInterpreterEvaluateAction<T>),
    Result(BytecodeInterpreterResultAction<T>),
    Gc(BytecodeInterpreterGcAction<T>),
    GcComplete(BytecodeInterpreterGcCompleteAction<T>),
}
impl<T: Expression> Named for BytecodeInterpreterActions<T> {
    fn name(&self) -> &'static str {
        match self {
            Self::Init(action) => action.name(),
            Self::Evaluate(action) => action.name(),
            Self::Result(action) => action.name(),
            Self::Gc(action) => action.name(),
            Self::GcComplete(action) => action.name(),
        }
    }
}
impl<T: Expression> Action for BytecodeInterpreterActions<T> {}
impl<T: Expression> SerializableAction for BytecodeInterpreterActions<T> {
    fn to_json(&self) -> SerializedAction {
        match self {
            Self::Init(action) => action.to_json(),
            Self::Evaluate(action) => action.to_json(),
            Self::Result(action) => action.to_json(),
            Self::Gc(action) => action.to_json(),
            Self::GcComplete(action) => action.to_json(),
        }
    }
}

impl<T: Expression> From<BytecodeInterpreterInitAction<T>> for BytecodeInterpreterActions<T> {
    fn from(value: BytecodeInterpreterInitAction<T>) -> Self {
        Self::Init(value)
    }
}
impl<T: Expression> From<BytecodeInterpreterActions<T>>
    for Option<BytecodeInterpreterInitAction<T>>
{
    fn from(value: BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Init(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a BytecodeInterpreterActions<T>>
    for Option<&'a BytecodeInterpreterInitAction<T>>
{
    fn from(value: &'a BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Init(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<BytecodeInterpreterEvaluateAction<T>> for BytecodeInterpreterActions<T> {
    fn from(value: BytecodeInterpreterEvaluateAction<T>) -> Self {
        Self::Evaluate(value)
    }
}
impl<T: Expression> From<BytecodeInterpreterActions<T>>
    for Option<BytecodeInterpreterEvaluateAction<T>>
{
    fn from(value: BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Evaluate(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a BytecodeInterpreterActions<T>>
    for Option<&'a BytecodeInterpreterEvaluateAction<T>>
{
    fn from(value: &'a BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Evaluate(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<BytecodeInterpreterResultAction<T>> for BytecodeInterpreterActions<T> {
    fn from(value: BytecodeInterpreterResultAction<T>) -> Self {
        Self::Result(value)
    }
}
impl<T: Expression> From<BytecodeInterpreterActions<T>>
    for Option<BytecodeInterpreterResultAction<T>>
{
    fn from(value: BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Result(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a BytecodeInterpreterActions<T>>
    for Option<&'a BytecodeInterpreterResultAction<T>>
{
    fn from(value: &'a BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Result(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<BytecodeInterpreterGcAction<T>> for BytecodeInterpreterActions<T> {
    fn from(value: BytecodeInterpreterGcAction<T>) -> Self {
        Self::Gc(value)
    }
}
impl<T: Expression> From<BytecodeInterpreterActions<T>> for Option<BytecodeInterpreterGcAction<T>> {
    fn from(value: BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Gc(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a BytecodeInterpreterActions<T>>
    for Option<&'a BytecodeInterpreterGcAction<T>>
{
    fn from(value: &'a BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::Gc(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<BytecodeInterpreterGcCompleteAction<T>> for BytecodeInterpreterActions<T> {
    fn from(value: BytecodeInterpreterGcCompleteAction<T>) -> Self {
        Self::GcComplete(value)
    }
}
impl<T: Expression> From<BytecodeInterpreterActions<T>>
    for Option<BytecodeInterpreterGcCompleteAction<T>>
{
    fn from(value: BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::GcComplete(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a BytecodeInterpreterActions<T>>
    for Option<&'a BytecodeInterpreterGcCompleteAction<T>>
{
    fn from(value: &'a BytecodeInterpreterActions<T>) -> Self {
        match value {
            BytecodeInterpreterActions::GcComplete(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "<T as Expression>::Signal: Serialize",
    deserialize = "<T as Expression>::Signal: Deserialize<'de>"
))]
pub struct BytecodeInterpreterInitAction<T: Expression> {
    pub cache_key: T::Signal,
}
impl<T: Expression> Action for BytecodeInterpreterInitAction<T> {}
impl<T: Expression> SerializableAction for BytecodeInterpreterInitAction<T> {
    fn to_json(&self) -> SerializedAction {
        SerializedAction::from_iter([("cache_id", JsonValue::from(self.cache_key.id()))])
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct BytecodeInterpreterEvaluateAction<T: Expression> {
    pub cache_key: T::Signal,
    pub state_index: Option<MessageOffset>,
    pub state_updates: Vec<(T::Signal, T)>,
}
impl<T: Expression> Action for BytecodeInterpreterEvaluateAction<T> {}
impl<T: Expression> Named for BytecodeInterpreterEvaluateAction<T> {
    fn name(&self) -> &'static str {
        "BytecodeInterpreterEvaluateAction"
    }
}
impl<T: Expression> SerializableAction for BytecodeInterpreterEvaluateAction<T> {
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

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct BytecodeInterpreterResultAction<T: Expression> {
    pub cache_key: T::Signal,
    pub state_index: Option<MessageOffset>,
    pub result: EvaluationResult<T>,
    pub statistics: BytecodeWorkerStatistics,
}
impl<T: Expression> Action for BytecodeInterpreterResultAction<T> {}
impl<T: Expression> Named for BytecodeInterpreterResultAction<T> {
    fn name(&self) -> &'static str {
        "BytecodeInterpreterResultAction"
    }
}
impl<T: Expression> SerializableAction for BytecodeInterpreterResultAction<T> {
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
            (
                "statistics",
                JsonValue::Object(JsonMap::from_iter([
                    (
                        String::from("state_dependency_count"),
                        JsonValue::from(self.statistics.state_dependency_count),
                    ),
                    (
                        String::from("evaluation_cache_entry_count"),
                        JsonValue::from(self.statistics.evaluation_cache_entry_count),
                    ),
                    (
                        String::from("evaluation_cache_deep_size"),
                        JsonValue::from(self.statistics.evaluation_cache_deep_size),
                    ),
                ])),
            ),
        ])
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "<T as Expression>::Signal: Serialize",
    deserialize = "<T as Expression>::Signal: Deserialize<'de>"
))]
pub struct BytecodeInterpreterGcAction<T: Expression> {
    pub cache_key: T::Signal,
    pub state_index: Option<MessageOffset>,
}
impl<T: Expression> Action for BytecodeInterpreterGcAction<T> {}
impl<T: Expression> SerializableAction for BytecodeInterpreterGcAction<T> {
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
        ])
    }
}

#[derive(Named, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "<T as Expression>::Signal: Serialize",
    deserialize = "<T as Expression>::Signal: Deserialize<'de>"
))]
pub struct BytecodeInterpreterGcCompleteAction<T: Expression> {
    pub cache_key: T::Signal,
    pub statistics: BytecodeWorkerStatistics,
}
impl<T: Expression> Action for BytecodeInterpreterGcCompleteAction<T> {}
impl<T: Expression> SerializableAction for BytecodeInterpreterGcCompleteAction<T> {
    fn to_json(&self) -> SerializedAction {
        SerializedAction::from_iter([
            ("cache_id", JsonValue::from(self.cache_key.id())),
            (
                "statistics",
                JsonValue::Object(JsonMap::from_iter([
                    (
                        String::from("state_dependency_count"),
                        JsonValue::from(self.statistics.state_dependency_count),
                    ),
                    (
                        String::from("evaluation_cache_entry_count"),
                        JsonValue::from(self.statistics.evaluation_cache_entry_count),
                    ),
                    (
                        String::from("evaluation_cache_deep_size"),
                        JsonValue::from(self.statistics.evaluation_cache_deep_size),
                    ),
                ])),
            ),
        ])
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct BytecodeWorkerStatistics {
    pub state_dependency_count: usize,
    pub evaluation_cache_entry_count: usize,
    pub evaluation_cache_deep_size: usize,
}
