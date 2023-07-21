// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};

use crate::stdlib::IsTruthy;

pub fn global_boolean<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    _allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: From<IsTruthy>,
{
    factory.create_builtin_term(IsTruthy)
}
