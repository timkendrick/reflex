// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct FormatErrorMessage;
impl FormatErrorMessage {
    pub const UUID: Uuid = uuid!("3f88d05d-47f1-4b49-b16e-b8dc2f4ee61c");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: Some(ArgType::Lazy),
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for FormatErrorMessage {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
