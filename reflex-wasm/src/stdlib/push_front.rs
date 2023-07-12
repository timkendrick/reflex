// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct PushFront;
impl PushFront {
    pub const UUID: Uuid = uuid!("5029b060-3f1c-4df8-8a7f-6135590c5c1a");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for PushFront {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
