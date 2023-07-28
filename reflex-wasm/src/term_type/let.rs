// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Expression, GraphNode, LetTermType, SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        error::CompilerError, instruction, CompileWasm, CompiledBlockBuilder, CompilerOptions,
        CompilerResult, CompilerStack, CompilerState, Eagerness, Internable, LazyExpression,
        ParamsSignature, Strictness, TypeSignature, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct LetTerm {
    pub initializer: ArenaPointer,
    pub body: ArenaPointer,
}
impl TermSize for LetTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LetTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let initializer_hash = arena.read_value::<Term, _>(self.initializer, |term| term.id());
        let body_hash = arena.read_value::<Term, _>(self.body, |term| term.id());
        hasher
            .hash(&initializer_hash, arena)
            .hash(&body_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<LetTerm, A> {
    pub fn initializer(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.initializer))
    }
    pub fn body(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.body))
    }
}

impl<A: Arena + Clone> LetTermType<WasmExpression<A>> for ArenaRef<LetTerm, A> {
    fn initializer<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.initializer().into()
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.body().into()
    }
}

impl<A: Arena + Clone> LetTermType<WasmExpression<A>> for ArenaRef<TypedTerm<LetTerm>, A> {
    fn initializer<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LetTerm, A> as LetTermType<WasmExpression<A>>>::initializer(&self.as_inner())
    }
    fn body<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<LetTerm, A> as LetTermType<WasmExpression<A>>>::body(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<LetTerm, A> {
    fn size(&self) -> usize {
        1 + self.initializer().size() + self.body().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.initializer()
            .capture_depth()
            .max(self.body().capture_depth().saturating_sub(1))
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.initializer()
            .free_variables()
            .into_iter()
            .chain(
                self.body()
                    .free_variables()
                    .into_iter()
                    .filter_map(|offset| if offset == 0 { None } else { Some(offset - 1) }),
            )
            .collect()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.initializer().count_variable_usages(offset)
            + self.body().count_variable_usages(offset + 1)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        // TODO: Verify shallow dynamic dependencies for Let term
        self.initializer()
            .dynamic_dependencies(deep)
            .union(self.body().dynamic_dependencies(deep))
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        // TODO: Verify shallow dynamic dependencies for Let term
        self.initializer().has_dynamic_dependencies(deep)
            || self.body().has_dynamic_dependencies(deep)
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<LetTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<LetTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.initializer() == other.initializer() && self.body() == other.body()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<LetTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<LetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<LetTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<let:{}:{}>", self.initializer(), self.body())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<LetTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        false
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LetTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let initializer = self.initializer();
        let body = self.body();
        // If the variable initializer is an alias to a variable declared in a parent scope, handle that as a special case
        if let Some(alias) = initializer.as_variable_term() {
            // Determine the scope offset of the aliased target variable
            let target_offset = alias.as_inner().stack_offset();
            // Determine the stack machine offset of the target variable
            let target_scope_offset = stack
                .lookup_variable(target_offset)
                .ok_or_else(|| CompilerError::UnboundVariable(target_offset))?;
            let block = CompiledBlockBuilder::new(stack);
            // Load the aliased variable from the target scope
            // => [Term]
            let block = block.push(instruction::core::GetScopeValue {
                value_type: ValueType::HeapPointer,
                scope_offset: target_scope_offset,
            });
            // Create a new temporary lexical scope that includes the local variable
            // => []
            let block = block.push(instruction::core::ScopeStart {
                value_type: ValueType::HeapPointer,
            });
            // Yield the expression body onto the stack (this will be evaluated within the new scope)
            // => [Term]
            let block = block.append_inner(|stack| body.compile(stack, state, options))?;
            // Drop the temporary lexical scope, leaving the result on the stack
            // => [Term]
            let block = block.push(instruction::core::ScopeEnd {
                value_type: ValueType::HeapPointer,
            });
            block.finish()
        } else {
            let (eagerness, strictness) = if options.lazy_variable_initializers {
                (Eagerness::Lazy, Strictness::NonStrict)
            } else {
                (
                    Eagerness::Eager,
                    // Skip signal-testing for variable initializers that are already fully evaluated to a non-signal value
                    if initializer.is_atomic() && initializer.as_signal_term().is_none() {
                        Strictness::NonStrict
                    } else {
                        Strictness::Strict
                    },
                )
            };
            let block = CompiledBlockBuilder::new(stack);
            // Yield the initializer term onto the stack, according to its eagerness and strictness
            // => [Term]
            let block = match eagerness {
                // If this is a lazy variable initializer, compile it as a lazy thunk within the current block
                Eagerness::Lazy => block.append_inner(|stack| {
                    LazyExpression::new(initializer).compile(stack, state, options)
                }),
                Eagerness::Eager => match strictness {
                    // If this is a strict eager variable initializer, compile it directly into the current block
                    Strictness::Strict => {
                        block.append_inner(|stack| initializer.compile(stack, state, options))
                    }
                    // If this is a non-strict eager variable initializer, compile it within a new control flow block
                    // (this ensures that any signals encountered when evaluating the variable initializer will not
                    // break out of the current block; the signal will only short-circuit when the variable is
                    // referenced in a strict context)
                    Strictness::NonStrict => block.append_inner(|stack| {
                        let block_type = TypeSignature {
                            params: ParamsSignature::Void,
                            results: ParamsSignature::Single(ValueType::HeapPointer),
                        };
                        let inner_stack = stack.enter_block(&block_type)?;
                        let block = CompiledBlockBuilder::new(stack);
                        let block = block.push(instruction::core::Block {
                            block_type,
                            body: initializer.compile(inner_stack, state, options)?,
                        });
                        block.finish::<CompilerError<_>>()
                    }),
                },
            }?;
            // Pop the initializer term and assign it to a lexical scope
            // => []
            let block = block.push(instruction::runtime::DeclareVariable {
                value_type: ValueType::HeapPointer,
            });
            // If the initializer is being evaluated in strict mode, test for signals
            // => []
            let block = if strictness.is_strict() {
                // Push a copy of the initializer result onto the stack
                // => [Term]
                let block = block.push(instruction::core::GetScopeValue {
                    value_type: ValueType::HeapPointer,
                    scope_offset: 0,
                });
                // If the initializer result is a signal, break out of the current block
                // => [Term]
                let block = block.push(instruction::runtime::BreakOnSignal { target_block: 0 });
                // Otherwise drop the temporary signal-testing value from the operand stack
                // => []
                let block = block.push(instruction::core::Drop {
                    value_type: ValueType::HeapPointer,
                });
                block
            } else {
                block
            };
            // Yield the expression body onto the stack (this will be evaluated within the new scope)
            // => [Term]
            let block = block.append_inner(|stack| body.compile(stack, state, options))?;
            // Drop the lexical scope, leaving the result on the stack
            // => [Term]
            let block = block.push(instruction::core::ScopeEnd {
                value_type: ValueType::HeapPointer,
            });
            block.finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn r#let() {
        assert_eq!(
            TermType::Let(LetTerm {
                initializer: ArenaPointer(0x54321),
                body: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Let as u32, 0x54321, 0x98765],
        );
    }
}
