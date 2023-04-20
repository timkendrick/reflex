// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};

use crate::stdlib::ParseDate;

pub fn global_date<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    _allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: From<ParseDate>,
{
    factory.create_builtin_term(ParseDate)
}
