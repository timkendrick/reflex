// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashSet;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::core::{
    DependencyList, GraphNode, SerializeJson, StackOffset, TimestampTermType, TimestampValue,
};

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct TimestampTerm {
    millis: TimestampValue,
}
impl TimestampTerm {
    pub fn new(millis: TimestampValue) -> Self {
        Self { millis }
    }
}
impl TimestampTermType for TimestampTerm {
    fn millis(&self) -> TimestampValue {
        self.millis
    }
}
impl GraphNode for TimestampTerm {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::new()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl std::fmt::Display for TimestampTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", format_datetime_utc(self.millis))
    }
}
impl std::fmt::Debug for TimestampTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl SerializeJson for TimestampTerm {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::String(format!(
            "{}",
            format_datetime_utc(self.millis)
        )))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        target.to_json().map(Some)
    }
}

pub fn format_datetime_utc(millis: TimestampValue) -> impl std::fmt::Display {
    let datetime: DateTime<Utc> = DateTime::from_utc(
        NaiveDateTime::from_timestamp_millis(millis).unwrap_or_default(),
        Utc,
    );
    datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ")
}
