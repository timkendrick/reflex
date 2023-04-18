// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Debug;
impl Debug {
    pub const UUID: Uuid = uuid!("6fa88d06-dca9-4e6c-b552-342445705f88");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Eager],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Debug {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
