// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Builtin, Expression, ExpressionFactory, HeapAllocator};
use reflex_macros::blanket_trait;

use crate::imports::{
    http::{import_http, HttpImportBuiltin},
    invalidation::{import_invalidation, InvalidationImportBuiltin},
    loader::{import_loader, LoaderImportBuiltin},
    state::{import_state, StateImportBuiltin},
    time::{import_time, TimeImportBuiltin},
};

pub mod http;
pub mod invalidation;
pub mod loader;
pub mod state;
pub mod time;

blanket_trait!(
    pub trait HandlerImportsBuiltin:
        Builtin
        + HttpImportBuiltin
        + InvalidationImportBuiltin
        + LoaderImportBuiltin
        + StateImportBuiltin
        + TimeImportBuiltin
    {
    }
);

pub fn handler_imports<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> Vec<(String, T)>
where
    T::Builtin: HandlerImportsBuiltin,
{
    vec![
        (
            String::from("reflex::http"),
            import_http(factory, allocator),
        ),
        (
            String::from("reflex::invalidation"),
            import_invalidation(factory, allocator),
        ),
        (
            String::from("reflex::loader"),
            import_loader(factory, allocator),
        ),
        (
            String::from("reflex::state"),
            import_state(factory, allocator),
        ),
        (
            String::from("reflex::time"),
            import_time(factory, allocator),
        ),
    ]
}
