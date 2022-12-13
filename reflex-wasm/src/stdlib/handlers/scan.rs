// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Scan;
impl Scan {
    pub const UUID: Uuid = uuid!("9c9f5a15-45a7-484d-a910-c6f114a8bced");
    const ARITY: FunctionArity<3, 0> = FunctionArity {
        required: [ArgType::Lazy, ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Scan {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
