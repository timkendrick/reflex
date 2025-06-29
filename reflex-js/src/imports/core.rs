// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{create_record, Builtin, Expression, ExpressionFactory, HeapAllocator};
use reflex_stdlib::*;

pub fn import_core<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: Builtin
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
        + From<ResolveRecord>
        + From<ResolveList>
        + From<Round>
        + From<Sequence>
        + From<Slice>
        + From<Split>
        + From<StartsWith>
        + From<Subtract>
        + From<Unzip>
        + From<Values>
        + From<Zip>,
{
    create_record(
        [
            (
                factory.create_string_term(allocator.create_static_string("abs")),
                factory.create_builtin_term(Abs),
            ),
            (
                factory.create_string_term(allocator.create_static_string("add")),
                factory.create_builtin_term(Add),
            ),
            (
                factory.create_string_term(allocator.create_static_string("and")),
                factory.create_builtin_term(And),
            ),
            (
                factory.create_string_term(allocator.create_static_string("apply")),
                factory.create_builtin_term(Apply),
            ),
            (
                factory.create_string_term(allocator.create_static_string("ceil")),
                factory.create_builtin_term(Ceil),
            ),
            (
                factory.create_string_term(allocator.create_static_string("chain")),
                factory.create_builtin_term(Chain),
            ),
            (
                factory.create_string_term(allocator.create_static_string("collectConstructor")),
                factory.create_builtin_term(CollectConstructor),
            ),
            (
                factory.create_string_term(allocator.create_static_string("collectHashMap")),
                factory.create_builtin_term(CollectHashMap),
            ),
            (
                factory.create_string_term(allocator.create_static_string("collectHashSet")),
                factory.create_builtin_term(CollectHashSet),
            ),
            (
                factory.create_string_term(allocator.create_static_string("collectList")),
                factory.create_builtin_term(CollectList),
            ),
            (
                factory.create_string_term(allocator.create_static_string("collectRecord")),
                factory.create_builtin_term(CollectRecord),
            ),
            (
                factory.create_string_term(allocator.create_static_string("collectSignal")),
                factory.create_builtin_term(CollectSignal),
            ),
            (
                factory.create_string_term(allocator.create_static_string("concat")),
                factory.create_builtin_term(CollectString),
            ),
            (
                factory.create_string_term(allocator.create_static_string("contains")),
                factory.create_builtin_term(Contains),
            ),
            (
                factory.create_string_term(allocator.create_static_string("divide")),
                factory.create_builtin_term(Divide),
            ),
            (
                factory.create_string_term(allocator.create_static_string("effect")),
                factory.create_builtin_term(Effect),
            ),
            (
                factory.create_string_term(allocator.create_static_string("endsWith")),
                factory.create_builtin_term(EndsWith),
            ),
            (
                factory.create_string_term(allocator.create_static_string("eq")),
                factory.create_builtin_term(Eq),
            ),
            (
                factory.create_string_term(allocator.create_static_string("equal")),
                factory.create_builtin_term(Equal),
            ),
            (
                factory.create_string_term(allocator.create_static_string("filter")),
                factory.create_builtin_term(Filter),
            ),
            (
                factory.create_string_term(allocator.create_static_string("flatten")),
                factory.create_builtin_term(Flatten),
            ),
            (
                factory.create_string_term(allocator.create_static_string("floor")),
                factory.create_builtin_term(Floor),
            ),
            (
                factory.create_string_term(allocator.create_static_string("fold")),
                factory.create_builtin_term(Fold),
            ),
            (
                factory.create_string_term(allocator.create_static_string("get")),
                factory.create_builtin_term(Get),
            ),
            (
                factory.create_string_term(allocator.create_static_string("gt")),
                factory.create_builtin_term(Gt),
            ),
            (
                factory.create_string_term(allocator.create_static_string("gte")),
                factory.create_builtin_term(Gte),
            ),
            (
                factory.create_string_term(allocator.create_static_string("hash")),
                factory.create_builtin_term(Hash),
            ),
            (
                factory.create_string_term(allocator.create_static_string("if")),
                factory.create_builtin_term(If),
            ),
            (
                factory.create_string_term(allocator.create_static_string("ifError")),
                factory.create_lambda_term(
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(IfError),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_variable_term(1),
                                allocator.create_empty_list(),
                            ),
                            factory.create_variable_term(0),
                        ),
                    ),
                ),
            ),
            (
                factory.create_string_term(allocator.create_static_string("ifPending")),
                factory.create_lambda_term(
                    2,
                    factory.create_application_term(
                        factory.create_builtin_term(IfPending),
                        allocator.create_pair(
                            factory.create_application_term(
                                factory.create_variable_term(1),
                                allocator.create_empty_list(),
                            ),
                            factory.create_variable_term(0),
                        ),
                    ),
                ),
            ),
            (
                factory.create_string_term(allocator.create_static_string("insert")),
                factory.create_builtin_term(Insert),
            ),
            (
                factory.create_string_term(allocator.create_static_string("intersperse")),
                factory.create_builtin_term(Intersperse),
            ),
            (
                factory.create_string_term(allocator.create_static_string("keys")),
                factory.create_builtin_term(Keys),
            ),
            (
                factory.create_string_term(allocator.create_static_string("length")),
                factory.create_builtin_term(Length),
            ),
            (
                factory.create_string_term(allocator.create_static_string("lt")),
                factory.create_builtin_term(Lt),
            ),
            (
                factory.create_string_term(allocator.create_static_string("lte")),
                factory.create_builtin_term(Lte),
            ),
            (
                factory.create_string_term(allocator.create_static_string("map")),
                factory.create_builtin_term(Map),
            ),
            (
                factory.create_string_term(allocator.create_static_string("max")),
                factory.create_builtin_term(Max),
            ),
            (
                factory.create_string_term(allocator.create_static_string("merge")),
                factory.create_builtin_term(Merge),
            ),
            (
                factory.create_string_term(allocator.create_static_string("min")),
                factory.create_builtin_term(Min),
            ),
            (
                factory.create_string_term(allocator.create_static_string("multiply")),
                factory.create_builtin_term(Multiply),
            ),
            (
                factory.create_string_term(allocator.create_static_string("not")),
                factory.create_builtin_term(Not),
            ),
            (
                factory.create_string_term(allocator.create_static_string("or")),
                factory.create_builtin_term(Or),
            ),
            (
                factory.create_string_term(allocator.create_static_string("pow")),
                factory.create_builtin_term(Pow),
            ),
            (
                factory.create_string_term(allocator.create_static_string("push")),
                factory.create_builtin_term(Push),
            ),
            (
                factory.create_string_term(allocator.create_static_string("pushFront")),
                factory.create_builtin_term(PushFront),
            ),
            (
                factory.create_string_term(allocator.create_static_string("raise")),
                factory.create_builtin_term(Raise),
            ),
            (
                factory.create_string_term(allocator.create_static_string("remainder")),
                factory.create_builtin_term(Remainder),
            ),
            (
                factory.create_string_term(allocator.create_static_string("replace")),
                factory.create_builtin_term(Replace),
            ),
            (
                factory.create_string_term(allocator.create_static_string("resolveArgs")),
                factory.create_builtin_term(ResolveArgs),
            ),
            (
                factory.create_string_term(allocator.create_static_string("resolveDeep")),
                factory.create_builtin_term(ResolveDeep),
            ),
            (
                factory.create_string_term(allocator.create_static_string("resolveHashMap")),
                factory.create_builtin_term(ResolveHashMap),
            ),
            (
                factory.create_string_term(allocator.create_static_string("resolveHashSet")),
                factory.create_builtin_term(ResolveHashSet),
            ),
            (
                factory.create_string_term(allocator.create_static_string("resolveRecord")),
                factory.create_builtin_term(ResolveRecord),
            ),
            (
                factory.create_string_term(allocator.create_static_string("resolveList")),
                factory.create_builtin_term(ResolveList),
            ),
            (
                factory.create_string_term(allocator.create_static_string("round")),
                factory.create_builtin_term(Round),
            ),
            (
                factory.create_string_term(allocator.create_static_string("sequence")),
                factory.create_builtin_term(Sequence),
            ),
            (
                factory.create_string_term(allocator.create_static_string("slice")),
                factory.create_builtin_term(Slice),
            ),
            (
                factory.create_string_term(allocator.create_static_string("split")),
                factory.create_builtin_term(Split),
            ),
            (
                factory.create_string_term(allocator.create_static_string("startsWith")),
                factory.create_builtin_term(StartsWith),
            ),
            (
                factory.create_string_term(allocator.create_static_string("subtract")),
                factory.create_builtin_term(Subtract),
            ),
            (
                factory.create_string_term(allocator.create_static_string("unzip")),
                factory.create_builtin_term(Unzip),
            ),
            (
                factory.create_string_term(allocator.create_static_string("values")),
                factory.create_builtin_term(Values),
            ),
            (
                factory.create_string_term(allocator.create_static_string("zip")),
                factory.create_builtin_term(Zip),
            ),
        ],
        factory,
        allocator,
    )
}
