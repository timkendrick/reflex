// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ToString;
impl ToString {
    pub const UUID: Uuid = uuid!("7f651286-8d00-4854-a956-0a54dfe662d0");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: Some(ArgType::Lazy),
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ToString {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
