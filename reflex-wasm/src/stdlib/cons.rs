// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Cons;
impl Cons {
    pub const UUID: Uuid = uuid!("6dafb566-229c-441d-85e4-951a9e2b5a60");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Cons {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
