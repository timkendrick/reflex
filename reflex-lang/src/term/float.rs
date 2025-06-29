// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use reflex::core::{
    as_integer, DependencyList, FloatTermType, FloatValue, GraphNode, SerializeJson, StackOffset,
};

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct FloatTerm {
    value: FloatValue,
}
impl Eq for FloatTerm {}
impl std::hash::Hash for FloatTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.value.to_le_bytes())
    }
}
impl FloatTerm {
    pub fn new(value: FloatValue) -> Self {
        Self { value }
    }
}
impl FloatTermType for FloatTerm {
    fn value(&self) -> FloatValue {
        self.value
    }
}
impl GraphNode for FloatTerm {
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

impl std::fmt::Display for FloatTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match as_integer(self.value) {
            Some(value) => write!(f, "{}.0", value),
            None => write!(f, "{}", self.value),
        }
    }
}
impl std::fmt::Debug for FloatTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl SerializeJson for FloatTerm {
    fn to_json(&self) -> Result<JsonValue, String> {
        match serde_json::Number::from_f64(self.value) {
            Some(number) => Ok(JsonValue::Number(number)),
            None => Err(format!(
                "Unable to serialize float non-finite float as JSON value: {}",
                self
            )),
        }
    }

    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.value == target.value {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}
