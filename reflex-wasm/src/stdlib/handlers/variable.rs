// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct GetVariable;
impl GetVariable {
    pub const UUID: Uuid = uuid!("fb7bbe51-fa38-4c79-a361-c90607db2736");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Lazy],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for GetVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct SetVariable;
impl SetVariable {
    pub const UUID: Uuid = uuid!("c7a1b8e5-1045-4ce7-bd1a-4125bdb8d647");
    const ARITY: FunctionArity<3, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Lazy, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for SetVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct IncrementVariable;
impl IncrementVariable {
    pub const UUID: Uuid = uuid!("c73a5dfa-61fb-4a19-956e-752f34526718");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for IncrementVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DecrementVariable;
impl DecrementVariable {
    pub const UUID: Uuid = uuid!("75b1d997-91ff-43e6-bbb6-b3f9d6c47a34");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for DecrementVariable {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
