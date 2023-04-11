// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use reflex::core::{
    ConditionListType, ConditionType, Expression, RefType, SignalTermType, SignalType,
};

pub mod builtins;
pub mod repl;

pub fn format_signal_result<T: Expression<SignalTerm = V>, V: SignalTermType<T>>(
    result: &V,
) -> String {
    result
        .signals()
        .as_deref()
        .iter()
        .map(|signal| format_signal(signal.as_deref()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_signal<T: Expression<Signal = V>, V: ConditionType<T>>(signal: &V) -> String {
    match signal.signal_type() {
        SignalType::Error { payload } => {
            format!("Error: {payload}")
        }
        SignalType::Custom {
            effect_type,
            payload,
            ..
        } => format!("<{effect_type}> {payload}",),
        SignalType::Pending => String::from("<pending>"),
    }
}
