// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Round;
impl Round {
    pub const UUID: Uuid = uuid!("8abe6c68-60e8-45e7-8497-d91b845d6e7e");
    const ARITY: FunctionArity<1, 0> = FunctionArity {
        required: [ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Round {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
