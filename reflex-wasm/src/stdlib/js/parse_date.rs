// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ParseDate;
impl ParseDate {
    pub const UUID: Uuid = uuid!("c63f7c30-b28c-42dd-aabc-3a228cca40e2");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ParseDate {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
