// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::{
    core::{
        ArgType, DependencyList, Expression, ExpressionListType, GraphNode, ListTermType,
        SerializeJson, StackOffset, StructPrototypeType,
    },
    hash::HashId,
};
use reflex_utils::{json::is_empty_json_object, MapIntoIterator};
use serde_json::Value as JsonValue;

use crate::{
    allocator::{Arena, ArenaAllocator},
    compiler::{
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompilerOptions, CompilerResult, CompilerStack, CompilerState,
        ConstValue, Internable, MaybeBlockWrappedExpression, MaybeLazyExpression, ParamsSignature,
        ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermType, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Array, ArrayValueIter, IntoArenaRefIter, PointerIter, Term,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ListTerm {
    pub items: Array<ArenaPointer>,
}

pub type ListTermPointerIter = std::vec::IntoIter<ArenaPointer>;

impl<A: Arena> PointerIter for ArenaRef<ListTerm, A> {
    type Iter<'a> = ListTermPointerIter
    where
        Self: 'a;

    fn iter<'a>(&self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        self.read_value(|term| {
            let ptr = term as *const ListTerm as u32;

            term.items
                .items()
                .map(|item| {
                    self.pointer
                        .offset(item as *const ArenaPointer as u32 - ptr)
                })
                .collect::<Vec<_>>()
        })
        .into_iter()
    }
}

impl TermSize for ListTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<Array<ArenaPointer>>()
            + self.items.size_of()
    }
}
impl TermHash for ListTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let hasher = hasher.write_u32(self.items.length);
        self.items.items().copied().fold(hasher, |hasher, item| {
            let item_hash = arena.read_value::<Term, _>(item, |term| term.id());
            hasher.hash(&item_hash, arena)
        })
    }
}
impl ListTerm {
    pub fn allocate(
        values: impl IntoIterator<
            Item = ArenaPointer,
            IntoIter = impl ExactSizeIterator<Item = ArenaPointer>,
        >,
        arena: &mut impl ArenaAllocator,
    ) -> ArenaPointer {
        let values = values.into_iter();
        let term = Term::new(
            TermType::List(ListTerm {
                items: Default::default(),
            }),
            arena,
        );
        let term_size = term.size_of();
        let instance = arena.allocate(term);
        let list = instance.offset((term_size - std::mem::size_of::<Array<ArenaPointer>>()) as u32);
        Array::<ArenaPointer>::extend(list, values, arena);
        let hash = arena.read_value::<Term, _>(instance, |term| {
            TermHasher::default().hash(term, arena).finish()
        });
        arena.write::<u64>(Term::get_hash_pointer(instance), u64::from(hash));
        instance
    }
}

impl<A: Arena + Clone> ArenaRef<ListTerm, A> {
    fn items_pointer(&self) -> ArenaPointer {
        self.inner_pointer(|value| &value.items)
    }
    pub fn items(&self) -> ArenaRef<Array<ArenaPointer>, A> {
        ArenaRef::<Array<ArenaPointer>, _>::new(self.arena.clone(), self.items_pointer())
    }
    pub fn iter(&self) -> IntoArenaRefIter<'_, Term, A, ArrayValueIter<'_, ArenaPointer, A>> {
        IntoArenaRefIter::new(
            &self.arena,
            Array::<ArenaPointer>::iter(self.items_pointer(), &self.arena),
        )
    }
    pub fn len(&self) -> usize {
        self.items().len()
    }
    pub fn get(&self, index: usize) -> Option<ArenaRef<Term, A>> {
        self.read_value(|term| term.items.get(index).copied())
            .map(|pointer| ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<ListTerm, A> {
    fn size(&self) -> usize {
        1 + self.iter().map(|term| term.size()).sum::<usize>()
    }
    fn capture_depth(&self) -> StackOffset {
        self.iter()
            .map(|term| term.capture_depth())
            .max()
            .unwrap_or(0)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.iter().flat_map(|term| term.free_variables()).collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.iter()
            .map(|term| term.count_variable_usages(offset))
            .sum()
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.iter()
                .flat_map(|term| term.dynamic_dependencies(deep))
                .collect()
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.iter().any(|term| term.has_dynamic_dependencies(deep))
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.iter().all(|term| term.is_atomic())
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> ListTermType<WasmExpression<A>> for ArenaRef<TypedTerm<ListTerm>, A> {
    fn items<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
        WasmExpression<A>: 'a,
    {
        self.clone()
    }
}

impl<A: Arena + Clone> StructPrototypeType<WasmExpression<A>> for ArenaRef<TypedTerm<ListTerm>, A> {
    fn keys<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
        WasmExpression<A>: 'a,
    {
        self.clone()
    }
}

impl<A: Arena + Clone> ExpressionListType<WasmExpression<A>> for ArenaRef<TypedTerm<ListTerm>, A> {
    type Iterator<'a> = MapIntoIterator<
        IntoArenaRefIter<'a, Term, A, ArrayValueIter<'a, ArenaPointer, A>>,
        ArenaRef<Term, A>,
        <WasmExpression<A> as Expression>::ExpressionRef<'a>,
    >
    where
        WasmExpression<A>: 'a,
        Self: 'a;
    fn id(&self) -> HashId {
        self.read_value(|term| term.id())
    }
    fn len(&self) -> usize {
        self.as_inner().items().len()
    }
    fn get<'a>(
        &'a self,
        index: usize,
    ) -> Option<<WasmExpression<A> as Expression>::ExpressionRef<'a>>
    where
        WasmExpression<A>: 'a,
    {
        self.as_inner()
            .items()
            .get(index)
            .map(|pointer| ArenaRef::<Term, _>::new(self.arena.clone(), pointer).into())
    }
    fn iter<'a>(&'a self) -> Self::Iterator<'a>
    where
        WasmExpression<A>: 'a,
    {
        MapIntoIterator::new(IntoArenaRefIter::new(
            &self.arena,
            Array::<ArenaPointer>::iter(self.as_inner().items_pointer(), &self.arena),
        ))
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<ListTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        self.iter()
            .map(|key| key.to_json())
            .collect::<Result<Vec<_>, String>>()
            .map(|values| JsonValue::Array(values))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        let updates = target
            .iter()
            .zip(self.iter())
            .map(|(current, previous)| previous.patch(&current))
            .chain(
                target
                    .iter()
                    .skip(self.len())
                    .map(|item| item.to_json().map(Some)),
            )
            .collect::<Result<Vec<_>, _>>()?;
        let updates = reflex_utils::json::json_object(
            updates
                .into_iter()
                .enumerate()
                .filter_map(|(index, item)| item.map(|value| (index.to_string(), value)))
                .chain(if target.len() != self.len() {
                    Some((String::from("length"), JsonValue::from(target.len())))
                } else {
                    None
                }),
        );
        if is_empty_json_object(&updates) {
            Ok(None)
        } else {
            Ok(Some(updates))
        }
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<ListTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Clarify PartialEq implementations for container terms
        // This assumes that lists with the same length and hash are almost certainly identical
        self.len() == other.len()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<ListTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<ListTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<ListTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_displayed_items = 100;
        let items = self.iter();
        let num_items = items.len();
        write!(
            f,
            "[{}]",
            if num_items <= max_displayed_items {
                items
                    .map(|item| format!("{}", item))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                items
                    .take(max_displayed_items - 1)
                    .map(|item| format!("{}", item))
                    .chain(once(format!(
                        "...{} more items",
                        num_items - (max_displayed_items - 1)
                    )))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<ListTerm, A> {
    fn should_intern(&self, eager: ArgType) -> bool {
        self.iter().all(|item| item.should_intern(eager))
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ListTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let items = self.iter();
        let eagerness = options.lazy_list_items;
        compile_list(items.map(|item| (item, eagerness)), stack, state, options)
    }
}

pub(crate) fn compile_list<A: Arena + Clone>(
    items: impl IntoIterator<
        Item = (WasmExpression<A>, ArgType),
        IntoIter = impl ExactSizeIterator<Item = (WasmExpression<A>, ArgType)>,
    >,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    collect_compiled_list_values(
        items.into_iter().map(|(item, eagerness)| {
            let strictness = match eagerness {
                ArgType::Strict => Strictness::Strict,
                ArgType::Eager | ArgType::Lazy => Strictness::NonStrict,
            };
            let expression = match eagerness {
                // All signals encountered when evaluating strict list items must be collected into a single aggregated
                // signal before short-circuiting (this ensures that if signals are encountered across multiple
                // arguments, signals will be 'caught' at their respective block boundaries to be combined into a single
                // signal result, rather than the first signal short-circuiting all the way to the top level)
                ArgType::Strict => {
                    MaybeBlockWrappedExpression::wrapped(MaybeLazyExpression::Strict(item))
                }
                eagerness => MaybeBlockWrappedExpression::unwrapped(MaybeLazyExpression::new(
                    item, eagerness,
                )),
            };
            (expression, strictness)
        }),
        stack,
        state,
        options,
    )
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) enum Strictness {
    Strict,
    NonStrict,
}

pub(crate) fn collect_compiled_list_values<A: Arena + Clone, T: CompileWasm<A>>(
    items: impl IntoIterator<
        Item = (T, Strictness),
        IntoIter = impl ExactSizeIterator<Item = (T, Strictness)>,
    >,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    let items = items.into_iter();
    let num_items = items.len();
    if num_items == 0 {
        let block = CompiledBlockBuilder::new(stack);
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateEmptyList,
        });
        return block.finish();
    }
    let block = CompiledBlockBuilder::new(stack);
    // Push the list capacity onto the stack
    // => [u32]
    let block = block.push(instruction::core::Const {
        value: ConstValue::U32(num_items as u32),
    });
    // Allocate the list term
    // => [ListTerm]
    let block = block.push(instruction::runtime::CallRuntimeBuiltin {
        target: RuntimeBuiltin::AllocateList,
    });
    // Any strict list items need to be tested for signals, so while assigning the list items we keep track of a
    // combined signal term pointer (or null pointer) containing the aggregate of all signals encountered amongst the
    // strict list items, where the null pointer will be used as a placeholder if there were no signals
    // encountered amongst the strict list items. This check will be omitted if there are no strict list items.
    //
    // Assign each list item in turn, and for each strict list item, create a new temporary lexical scope containing the
    // combined signal result with the accumulated signal result from all previous strict list items
    // => [ListTerm]
    let (block, num_signal_scopes) = items.enumerate().fold(
        Result::<_, CompilerError<_>>::Ok((block, 0usize)),
        |result, (index, (item, strictness))| {
            let (block, num_signal_scopes) = result?;
            // Push a copy of the list term pointer onto the stack
            // => [ListTerm, ListTerm]
            let block = block.push(instruction::core::Duplicate {
                value_type: ValueType::HeapPointer,
            });
            // Push the item index onto the stack
            // => [ListTerm, ListTerm, index]
            let block = block.push(instruction::core::Const {
                value: ConstValue::U32(index as u32),
            });
            // Yield the list item onto the stack
            // => [ListTerm, ListTerm, index, Term]
            let block = block.append_inner(|stack| item.compile(stack, state, options))?;
            // If this item needs to be tested for signals, combine the item's signal result with the accumulated signal result
            // => [ListTerm, ListTerm, index, Term]
            let (block, num_signal_scopes) = if matches!(strictness, Strictness::Strict) {
                // Pop the list item from the top of the stack and assign to a temporary lexical scope variable
                // => [ListTerm, ListTerm, index]
                let block = block.push(instruction::core::ScopeStart {
                    value_type: ValueType::HeapPointer,
                });
                // Push the list item back onto the top of the stack
                // => [ListTerm, ListTerm, index, Term]
                let block = block.push(instruction::core::GetScopeValue {
                    scope_offset: 0,
                    value_type: ValueType::HeapPointer,
                });
                // If this is not the first strict item, push the accumulated signal term onto the top of the stack
                // (the combined signal result from processing the previous list item will be in the penultimate lexical scope)
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}]
                let block = if num_signal_scopes > 0 {
                    block.push(instruction::core::GetScopeValue {
                        scope_offset: 1,
                        value_type: ValueType::HeapPointer,
                    })
                } else {
                    block
                };
                // Push another copy of the list item back onto the top of the stack
                // (this will be used as the 'true' branch of the signal-testing select instruction)
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}, Term]
                let block = block.push(instruction::core::GetScopeValue {
                    scope_offset: 0,
                    value_type: ValueType::HeapPointer,
                });
                // Push a null pointer onto the top of the stack
                // (this will be used as the 'false' branch of the signal-testing select instruction)
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}, Term, NULL]
                let block = block.push(instruction::runtime::NullPointer);
                // Push another copy of the list item onto the top of the stack
                // (this will be used to test whether the term is a signal term)
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}, Term, NULL, Term]
                let block = block.push(instruction::core::GetScopeValue {
                    scope_offset: 0,
                    value_type: ValueType::HeapPointer,
                });
                // Determine whether the list item is a signal term
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}, Term, NULL, bool]
                let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                    target: RuntimeBuiltin::IsSignal,
                });
                // Determine whether the list item is a signal term
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}, Option<SignalTerm>]
                let block = block.push(instruction::core::Select {
                    value_type: ValueType::HeapPointer,
                });
                // Dispose the temporary lexical scope variable used to store the list item
                // => [ListTerm, ListTerm, index, Term, {Option<SignalTerm>}, Option<SignalTerm>]
                let block = block.push(instruction::core::ScopeEnd {
                    value_type: ValueType::HeapPointer,
                });
                // If there have been previous strict items, combine this item's optional signal term pointer
                // with the accumulated signal term pointer that has already been added to the stack,
                // => [ListTerm, ListTerm, index, Term, Option<SignalTerm>]
                let block = if num_signal_scopes > 0 {
                    block.push(instruction::runtime::CallRuntimeBuiltin {
                        target: RuntimeBuiltin::CombineSignals,
                    })
                } else {
                    block
                };
                // Pop the combined signal result from the stack and assign it to a new lexical scope that tracks
                // the latest accumulated signal result (SSA equivalent of mutating an accumulator variable).
                // While this may appear to require many more locals than mutating a single accumulator
                // variable, in practice the chain of nested SSA scopes can be optimized away during a later
                // compiler optimization pass
                // => [ListTerm, ListTerm, index, Term]
                let block = block.push(instruction::core::ScopeStart {
                    value_type: ValueType::HeapPointer,
                });
                (block, num_signal_scopes + 1)
            } else {
                (block, num_signal_scopes)
            };
            // Set the list term's value at the given index to the child item
            // => [ListTerm]
            let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                target: RuntimeBuiltin::SetListItem,
            });
            Ok((block, num_signal_scopes))
        },
    )?;
    // Push the list length onto the stack
    // => [ListTerm, u32]
    let block = block.push(instruction::core::Const {
        value: ConstValue::U32(num_items as u32),
    });
    // Initialize the list term
    // => [ListTerm]
    let block = block.push(instruction::runtime::CallRuntimeBuiltin {
        target: RuntimeBuiltin::InitList,
    });
    // If there were strict list items and one or more of the strict list items was a signal,
    // replace the list term on top of the operand stack with the combined signal result term
    // => [Term]
    let block = if num_signal_scopes > 0 {
        // The combined signal result from processing the final list item will be in the most recently-declared lexical scope
        let combined_signal_scope_offset = 0;
        // Push the combined signal result onto the stack
        // (this will be used as the 'false' branch of the select instruction)
        // => [ListTerm, Option<SignalTerm>]
        let block = block.push(instruction::core::GetScopeValue {
            scope_offset: combined_signal_scope_offset,
            value_type: ValueType::HeapPointer,
        });
        // Push another copy of the combined signal result onto the stack for comparing against the null pointer
        // => [ListTerm, Option<SignalTerm>, Option<SignalTerm>]
        let block = block.push(instruction::core::GetScopeValue {
            value_type: ValueType::HeapPointer,
            scope_offset: combined_signal_scope_offset,
        });
        // Dispose any temporary signal-testing lexical scopes
        // => [Term]
        let block = (0..num_signal_scopes).fold(block, |block, _| {
            block.push(instruction::core::ScopeEnd {
                value_type: ValueType::HeapPointer,
            })
        });
        // Push a null pointer onto the stack to use for comparing against the combined signal term result
        // => [ListTerm, Option<SignalTerm>, Option<SignalTerm>, NULL]
        let block = block.push(instruction::runtime::NullPointer);
        // Determine whether the combined signal result is not equal to the null pointer
        // => [ListTerm, Option<SignalTerm>, bool]
        let block = block.push(instruction::core::Ne {
            value_type: ValueType::HeapPointer,
        });
        // If a combined signal result was encountered, break out of the current control flow block, otherwise continue
        // => [ListTerm, NULL]
        let block = block.push(instruction::core::ConditionalBreak {
            target_block: 0,
            result_type: ParamsSignature::Single(ValueType::HeapPointer),
        });
        // Dispose of the combined signal null pointer
        // => [ListTerm]
        let block = block.push(instruction::core::Drop {
            value_type: ValueType::HeapPointer,
        });
        block
    } else {
        block
    };
    block.finish()
}

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        term_type::{IntTerm, TermType, TermTypeDiscriminants},
        utils::chunks_to_u64,
    };

    use super::*;

    #[test]
    fn list() {
        assert_eq!(
            TermType::List(ListTerm {
                items: Default::default()
            })
            .as_bytes(),
            [TermTypeDiscriminants::List as u32, 0, 0],
        );
        let mut allocator = VecAllocator::default();
        {
            let first_item =
                allocator.allocate(Term::new(TermType::Int(IntTerm::from(3)), &allocator));
            let second_item =
                allocator.allocate(Term::new(TermType::Int(IntTerm::from(4)), &allocator));
            let entries = [first_item, second_item];
            let instance = ListTerm::allocate(entries, &mut allocator);
            let result = allocator.get_ref::<Term>(instance).as_bytes();
            // TODO: Test term hashing
            let _hash = chunks_to_u64([result[0], result[1]]);
            let discriminant = result[2];
            let data_length = result[3];
            let data_capacity = result[4];
            let data = &result[5..];
            assert_eq!(discriminant, TermTypeDiscriminants::List as u32);
            assert_eq!(data_length, entries.len() as u32);
            assert_eq!(data_capacity, entries.len() as u32);
            assert_eq!(data, [u32::from(first_item), u32::from(second_item)]);
        }
    }
}
