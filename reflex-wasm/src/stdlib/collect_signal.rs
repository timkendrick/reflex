// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct CollectSignal;
impl CollectSignal {
    pub const UUID: Uuid = uuid!("d4ea9c07-b2da-4da7-89c0-352eb5abd35f");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Eager),
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for CollectSignal {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
