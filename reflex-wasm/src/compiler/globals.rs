// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum RuntimeGlobal {
    NullPointer,
}

impl RuntimeGlobal {
    pub fn name(self) -> &'static str {
        match self {
            RuntimeGlobal::NullPointer => "NULL",
        }
    }
}
