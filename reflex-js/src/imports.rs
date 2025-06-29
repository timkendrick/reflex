// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use reflex::core::{Builtin, Expression, ExpressionFactory, HeapAllocator};
use reflex_stdlib::*;

use crate::stdlib::*;

pub mod core;
pub mod utils;

pub use self::core::import_core;
pub use self::utils::import_utils;

pub trait JsImportsBuiltin:
    Builtin
    + From<Abs>
    + From<Add>
    + From<And>
    + From<Apply>
    + From<Ceil>
    + From<Chain>
    + From<CollectConstructor>
    + From<CollectHashMap>
    + From<CollectHashSet>
    + From<CollectList>
    + From<CollectRecord>
    + From<CollectSignal>
    + From<CollectString>
    + From<Contains>
    + From<Divide>
    + From<Effect>
    + From<EndsWith>
    + From<Eq>
    + From<Equal>
    + From<Filter>
    + From<Flatten>
    + From<Floor>
    + From<Fold>
    + From<Get>
    + From<Gt>
    + From<Gte>
    + From<Hash>
    + From<If>
    + From<IfError>
    + From<IfPending>
    + From<Insert>
    + From<Intersperse>
    + From<Keys>
    + From<Length>
    + From<Log>
    + From<Lt>
    + From<Lte>
    + From<Map>
    + From<Max>
    + From<Merge>
    + From<Min>
    + From<Multiply>
    + From<Not>
    + From<Or>
    + From<Pow>
    + From<Push>
    + From<PushFront>
    + From<Raise>
    + From<Remainder>
    + From<Replace>
    + From<ResolveArgs>
    + From<ResolveDeep>
    + From<ResolveHashMap>
    + From<ResolveHashSet>
    + From<ResolveList>
    + From<ResolveRecord>
    + From<Round>
    + From<Sequence>
    + From<Slice>
    + From<Split>
    + From<StartsWith>
    + From<Subtract>
    + From<Unzip>
    + From<Values>
    + From<Zip>
{
}
impl<T> JsImportsBuiltin for T where
    T: Builtin
        + From<Abs>
        + From<Add>
        + From<And>
        + From<Apply>
        + From<Ceil>
        + From<Chain>
        + From<CollectConstructor>
        + From<CollectHashMap>
        + From<CollectHashSet>
        + From<CollectList>
        + From<CollectRecord>
        + From<CollectSignal>
        + From<CollectString>
        + From<Contains>
        + From<Divide>
        + From<Effect>
        + From<EndsWith>
        + From<Eq>
        + From<Equal>
        + From<Filter>
        + From<Flatten>
        + From<Floor>
        + From<Fold>
        + From<Get>
        + From<Gt>
        + From<Gte>
        + From<Hash>
        + From<If>
        + From<IfError>
        + From<IfPending>
        + From<Insert>
        + From<Intersperse>
        + From<Keys>
        + From<Length>
        + From<Log>
        + From<Lt>
        + From<Lte>
        + From<Map>
        + From<Max>
        + From<Merge>
        + From<Min>
        + From<Multiply>
        + From<Not>
        + From<Or>
        + From<Pow>
        + From<Push>
        + From<PushFront>
        + From<Raise>
        + From<Remainder>
        + From<Replace>
        + From<ResolveArgs>
        + From<ResolveDeep>
        + From<ResolveHashMap>
        + From<ResolveHashSet>
        + From<ResolveList>
        + From<ResolveRecord>
        + From<Round>
        + From<Sequence>
        + From<Slice>
        + From<Split>
        + From<StartsWith>
        + From<Subtract>
        + From<Unzip>
        + From<Values>
        + From<Zip>
{
}

pub fn builtin_imports<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> Vec<(String, T)>
where
    T::Builtin: JsImportsBuiltin,
{
    vec![
        (
            String::from("reflex::core"),
            import_core(factory, allocator),
        ),
        (
            String::from("reflex::utils"),
            import_utils(factory, allocator),
        ),
    ]
}
