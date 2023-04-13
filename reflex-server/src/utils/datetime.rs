// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::time::SystemTime;

use chrono::{DateTime, Utc};

pub fn format_datetime_utc(timestamp: SystemTime, format: &'_ str) -> impl std::fmt::Display + '_ {
    let datetime: DateTime<Utc> = timestamp.into();
    datetime.format(format)
}
