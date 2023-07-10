// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Hash;
impl Hash {
    pub const UUID: Uuid = uuid!("8312114e-8e8d-4ab0-a4d6-7a58f4e51f1d");
    const ARITY: FunctionArity<0, 0> = FunctionArity {
        required: [],
        optional: [],
        variadic: Some(ArgType::Strict),
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for Hash {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
