// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct IfError;
impl IfError {
    pub const UUID: Uuid = uuid!("ca015984-8f33-49c5-b821-aca8fb122ee8");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Eager, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for IfError {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
