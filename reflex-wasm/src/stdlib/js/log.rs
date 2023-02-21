// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Log;
impl Log {
    pub const UUID: Uuid = uuid!("c0f30755-91b9-4252-b038-1aa3cd6f267c");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Eager],
        optional: [],
        variadic: Some(ArgType::Eager),
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Log {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
