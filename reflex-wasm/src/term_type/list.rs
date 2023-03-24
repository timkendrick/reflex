// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::{
    core::{
        DependencyList, Eagerness, Expression, ExpressionListType, GraphNode, Internable,
        ListTermType, SerializeJson, StackOffset, StructPrototypeType,
    },
    hash::HashId,
};
use reflex_utils::{json::is_empty_json_object, MapIntoIterator};
use serde_json::Value as JsonValue;

use crate::{
    allocator::{Arena, ArenaAllocator},
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerOptions,
        CompilerResult, CompilerStack, CompilerStackValue, CompilerState, CompilerVariableBindings,
        LazyExpression, ParamsSignature, TypeSignature, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TermType, TypedTerm, WasmExpression},
    ArenaArrayIter, ArenaPointer, ArenaRef, Array, IntoArenaRefIter, PointerIter, Term,
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
    pub fn iter(&self) -> IntoArenaRefIter<'_, Term, A, ArenaArrayIter<'_, ArenaPointer, A>> {
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
        IntoArenaRefIter<'a, Term, A, ArenaArrayIter<'a, ArenaPointer, A>>,
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
    fn should_intern(&self, eager: Eagerness) -> bool {
        self.iter().all(|item| item.should_intern(eager))
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ListTerm, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let items = self.iter();
        if options.lazy_list_items {
            compile_list(
                items.map(|item| (LazyExpression::new(item), Eagerness::Lazy)),
                state,
                bindings,
                options,
                stack,
            )
        } else {
            compile_list(
                items.map(|item| {
                    let eagerness = if item.is_static() && item.as_signal_term().is_none() {
                        Eagerness::Lazy
                    } else {
                        Eagerness::Eager
                    };
                    (item, eagerness)
                }),
                state,
                bindings,
                options,
                stack,
            )
        }
    }
}

pub(crate) fn compile_list<A: Arena + Clone, T: CompileWasm<A>>(
    items: impl IntoIterator<
        Item = (T, Eagerness),
        IntoIter = impl ExactSizeIterator<Item = (T, Eagerness)>,
    >,
    state: &mut CompilerState,
    bindings: &CompilerVariableBindings,
    options: &CompilerOptions,
    stack: &CompilerStack,
) -> CompilerResult<A> {
    let items = items.into_iter();
    let num_items = items.len();
    let mut instructions = CompiledBlock::default();
    // Push the list capacity onto the stack
    // => [capacity]
    instructions.push(CompiledInstruction::u32_const(num_items as u32));
    // Allocate the list term
    // => [ListTerm]
    instructions.push(CompiledInstruction::CallRuntimeBuiltin(
        RuntimeBuiltin::AllocateList,
    ));
    let stack = stack.push_lazy(ValueType::HeapPointer);
    // Any strict list items need to be tested for signals, so we yield all the list items onto the operand stack,
    // followed by a signal term pointer (or Nil term pointer) containing the aggregate of all signals encountered
    // amongst the strict list items, where the Nil term will be used as a placeholder if there were no signals
    // encountered amongst the strict list items. This check will be omitted if there are no strict list items.
    //
    // Yield each list item in turn onto the stack, and for each strict list item, create a new temporary lexical scope
    // containing the combined signal result with the accumulated signal result from all previous strict list items
    let num_signal_scopes =
        items
            .enumerate()
            .fold(Ok(0usize), |results, (index, (item, eagerness))| {
                let num_signal_scopes = results?;
                let is_strict = matches!(eagerness, Eagerness::Eager);
                // Compile the list item
                let compiled_item = {
                    let mut instructions = CompiledBlock::default();
                    // Duplicate the list term pointer onto the stack
                    // => [ListTerm, ListTerm]
                    instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
                    let stack = stack.push_lazy(ValueType::HeapPointer);
                    // Push the item index onto the stack
                    // => [ListTerm, ListTerm, index]
                    instructions.push(CompiledInstruction::u32_const(index as u32));
                    let stack = stack.push_lazy(ValueType::U32);
                    // Compile the list item in eager or lazy mode as appropriate, ensuring any variables encountered
                    // within the list items skip over any intermediate signal-testing locals
                    // => [ListTerm, ListTerm, index, Term]
                    let item_bindings = bindings.offset(num_signal_scopes);
                    instructions.append_block(item.compile(
                        state,
                        &item_bindings,
                        options,
                        &stack,
                    )?);
                    // If this is a strict list item, combine any signal result with the existing accumulated signal result from previous list items
                    if is_strict {
                        // Pop the result from the stack and assign as the variable of a short-lived lexical scope
                        // to use for testing whether this list item's value is a signal
                        // => []
                        instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                        // Push a copy of the result onto back onto the stack to be added to the list
                        // => [Term]
                        instructions.push(CompiledInstruction::GetScopeValue {
                            value_type: ValueType::HeapPointer,
                            scope_offset: 0,
                        });
                        // Push a copy of the result onto the stack (true case)
                        // => [Term, Term]
                        instructions.push(CompiledInstruction::GetScopeValue {
                            value_type: ValueType::HeapPointer,
                            scope_offset: 0,
                        });
                        // Push the null pointer onto the stack (false case)
                        // => [Term, Term, NULL]
                        instructions.push(CompiledInstruction::NullPointer);
                        // Push another copy of the result onto the stack
                        // => [Term, Term, NULL, Term]
                        instructions.push(CompiledInstruction::GetScopeValue {
                            value_type: ValueType::HeapPointer,
                            scope_offset: 0,
                        });
                        // Invoke the builtin method to determine whether the item is a signal or not
                        // => [Term, Term, NULL, bool]
                        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                            RuntimeBuiltin::IsSignal,
                        ));
                        // Pop the comparison result from the stack and select one of the two preceding values, leaving
                        // either the signal term pointer (true case) or the null pointer (false case) on the stack
                        // depending on whether the list item value is a signal term pointer
                        // => [Term, Option<SignalTerm>]
                        instructions.push(CompiledInstruction::Select(ValueType::HeapPointer));
                        // Discard this list item's temporary signal-testing lexical scope
                        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                        // If there have been previous strict list items, combine the current signal result with the
                        // accumulated signal result
                        if num_signal_scopes > 0 {
                            // Push the existing accumulated signal result onto the stack
                            // => [Term, Option<SignalTerm>, Option<SignalTerm>]
                            instructions.push(CompiledInstruction::GetScopeValue {
                                value_type: ValueType::HeapPointer,
                                scope_offset: 0,
                            });
                            // Invoke the builtin method to combine the existing accumulated signal with the current signal
                            // => [Term, Option<SignalTerm>]
                            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                                RuntimeBuiltin::CombineSignals,
                            ));
                        }
                        // Pop the combined signal result off the stack and assign it to a new lexical scope that
                        // tracks the latest accumulated signal result (SSA equivalent of mutating an accumulator
                        // variable), leaving the evaluated list item on top of the stack.
                        // While this may appear to require many more locals than mutating a single accumulator
                        // variable, in practice the chain of nested SSA scopes can be optimized away during a later
                        // compiler optimization pass
                        // => [Term]
                        instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                    }
                    instructions
                };
                // Yield the item onto the stack
                // => [ListTerm, ListTerm, index, Term]
                instructions.append_block(compiled_item);
                // Set the list term's value at the given index to the child item
                // => [ListTerm]
                instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                    RuntimeBuiltin::SetListItem,
                ));
                // If this is a strict list item, we will have created a temporary lexical scope to keep track of the
                // combined signal result for this list item (taking into account the accumulated signal result from
                // previous list items)
                // We need to keep track of how many temporary lexical scopes have been created, so we know how many to
                // discard once we're done building up the accumulated signal
                if is_strict {
                    Ok(num_signal_scopes + 1)
                } else {
                    // If this was not a strict list item, no extra signal scopes introduced
                    Ok(num_signal_scopes)
                }
            })?;
    // Now that all the list items have been added to the list, short-circuit if a signal was encountered while
    // evaluating the strict list items, leaving the list on top of the stack
    // => [ListTerm]
    if num_signal_scopes > 0 {
        // If there are strict list items, we want to push a combined signal result onto the stack,
        // ensuring there is a valid term on top of the stack by replacing the potential null signal pointer
        // with a Nil placeholder term (this prevents null pointer exceptions when testing whether to break)
        //
        // Push a placeholder Nil term pointer onto the stack (true branch)
        // => [ListTerm, NilTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateNil,
        ));
        // Push the combined signal result onto the stack (false branch)
        // (the combined signal result from processing the final list item will be in the most recent lexical scope)
        // => [ListTerm, NilTerm, Option<SignalTerm>]
        instructions.push(CompiledInstruction::GetScopeValue {
            value_type: ValueType::HeapPointer,
            scope_offset: 0,
        });
        // Push another copy of the signal result onto the stack for comparing against the null pointer
        // => [ListTerm, NilTerm, Option<SignalTerm>, Option<SignalTerm>]
        instructions.push(CompiledInstruction::GetScopeValue {
            value_type: ValueType::HeapPointer,
            scope_offset: 0,
        });
        // Push a null pointer onto the stack
        // => [ListTerm, NilTerm, Option<SignalTerm>, Option<SignalTerm>, NULL]
        instructions.push(CompiledInstruction::NullPointer);
        // Invoke the builtin function to determine whether the signal result is equal to the null pointer
        // => [ListTerm, NilTerm, Option<SignalTerm>, bool]
        instructions.push(CompiledInstruction::Eq(ValueType::HeapPointer));
        // Select either the Nil placeholder term pointer or the signal result term pointer respectively,
        // based on whether the signal result was null
        // => [ListTerm, Term]
        instructions.push(CompiledInstruction::Select(ValueType::HeapPointer));
        let stack = stack.push_strict();
        // Now that we have the final accumulated signal, we can drop all the temporary lexical scopes that were
        // used to store all the intermediate combined signals
        for _ in 0..num_signal_scopes {
            instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
        }
        // Duplicate the signal result onto the stack
        // => [ListTerm, Term, Term]
        instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
        // Push a boolean onto the stack indicating whether a signal term was encountered
        // => [ListTerm, Term, bool]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::IsSignal,
        ));
        // Short circuit if a signal term was encountered amongst the strict list items, retaining the
        // list items and signal result on the stack
        // TODO: Consolidate signal-testing code across multiple use cases
        // => [ListTerm, Term]
        instructions.push(CompiledInstruction::ConditionalBreak {
            // Retain the list item term pointers followed by the signal result term pointer on the stack
            block_type: TypeSignature {
                params: ParamsSignature::from_iter(stack.value_types()),
                results: ParamsSignature::Single(ValueType::HeapPointer),
            },
            // Return the signal term
            handler: {
                let mut instructions = CompiledBlock::default();
                // If there were any captured values saved onto the operand stack we need to discard them and then
                // push the signal term pointer back on top of the stack
                if stack.depth() > 1 {
                    // Pop the signal result term pointer from the top of the stack and store in a new temporary lexical scope
                    instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                    let stack = stack.pop();
                    // Discard any preceding stack values that had been captured for use in the continuation block closure
                    let num_signal_scopes =
                        stack
                            .rev()
                            .fold(Ok(0usize), |num_signal_scopes, stack_value| {
                                let num_signal_scopes = num_signal_scopes?;
                                match stack_value {
                                    CompilerStackValue::Lazy(value_type) => {
                                        // If the captured stack value does not need to be checked for signals,
                                        // pop it from the operand stack and move on
                                        instructions.push(CompiledInstruction::Drop(value_type));
                                        Ok(num_signal_scopes)
                                    }
                                    CompilerStackValue::Strict => {
                                        // Pop the captured value from the operand stack and store it in a temporary scope for signal-testing
                                        instructions.push(CompiledInstruction::ScopeStart(
                                            ValueType::HeapPointer,
                                        ));
                                        // Reinstate a copy of the captured value on the operand stack (true branch)
                                        instructions.push(CompiledInstruction::GetScopeValue {
                                            value_type: ValueType::HeapPointer,
                                            scope_offset: 0,
                                        });
                                        // Push a null pointer onto the operand stack (false branch)
                                        instructions.push(CompiledInstruction::NullPointer);
                                        // Push another copy of the captured value onto the operand stack for signal comparison
                                        instructions.push(CompiledInstruction::GetScopeValue {
                                            value_type: ValueType::HeapPointer,
                                            scope_offset: 0,
                                        });
                                        // Dispose the temporary signal-testing scope
                                        instructions.push(CompiledInstruction::ScopeEnd(
                                            ValueType::HeapPointer,
                                        ));
                                        // Determine whether the captured value is a signal (condition)
                                        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                                            RuntimeBuiltin::IsSignal,
                                        ));
                                        // Select either the captured value or the null pointer depending on whether the captured value is a signal
                                        instructions.push(CompiledInstruction::Select(
                                            ValueType::HeapPointer,
                                        ));
                                        // Push the existing accumulated signal onto the operand stack
                                        instructions.push(CompiledInstruction::GetScopeValue {
                                            value_type: ValueType::HeapPointer,
                                            scope_offset: 0,
                                        });
                                        // Combine with the existing accumulated signal
                                        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                                            RuntimeBuiltin::CombineSignals,
                                        ));
                                        // Create a new lexical scope containing the accumulated signal result
                                        instructions.push(CompiledInstruction::ScopeStart(
                                            ValueType::HeapPointer,
                                        ));
                                        Ok(num_signal_scopes + 1)
                                    }
                                }
                            })?;
                    // Push the accumulated signal term pointer onto the top of the stack
                    instructions.push(CompiledInstruction::GetScopeValue {
                        value_type: ValueType::HeapPointer,
                        scope_offset: 0,
                    });
                    // Drop the temporary signal-testing scopes
                    for _ in 0..num_signal_scopes {
                        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                    }
                    // Drop the temporary signal result term lexical scope
                    instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                }
                instructions
            },
        });
        // Drop the Nil signal placeholder term pointer, leaving just the list term
        // => [ListTerm]
        instructions.push(CompiledInstruction::Drop(ValueType::HeapPointer));
    }
    // Now that all the items have been added, push the list length onto the stack
    // => [ListTerm, length]
    instructions.push(CompiledInstruction::u32_const(num_items as u32));
    // Initialize the list term with the length that is on the stack
    // => [ListTerm]
    instructions.push(CompiledInstruction::CallRuntimeBuiltin(
        RuntimeBuiltin::InitList,
    ));
    Ok(instructions)
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
