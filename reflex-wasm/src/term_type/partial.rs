// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ArgType, Arity, DependencyList, Eagerness, Expression, GraphNode, Internable,
    PartialApplicationTermType, SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use reflex_utils::WithExactSizeIterator;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        get_compiled_function_arity, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompilerOptions, CompilerResult, CompilerStack, CompilerState,
        LazyExpression, MaybeLazyExpression, Strictness,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{list::compile_list, ListTerm, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct PartialTerm {
    pub target: ArenaPointer,
    pub args: ArenaPointer,
}
impl TermSize for PartialTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for PartialTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let target_hash = arena.read_value::<Term, _>(self.target, |term| term.id());
        let args_hash = arena.read_value::<Term, _>(self.args, |term| term.id());
        hasher.hash(&target_hash, arena).hash(&args_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<PartialTerm, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.target))
    }
    pub fn args(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.args),
        )
    }
    pub fn arity(&self) -> Option<Arity> {
        self.target()
            .arity()
            .map(|arity| arity.partial(self.args().as_inner().len()))
    }
}

impl<A: Arena + Clone> PartialApplicationTermType<WasmExpression<A>> for ArenaRef<PartialTerm, A> {
    fn target<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        self.target().into()
    }
    fn args<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<A>: 'a,
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
    {
        self.args().into()
    }
}

impl<A: Arena + Clone> PartialApplicationTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<PartialTerm>, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<PartialTerm, A> as PartialApplicationTermType<WasmExpression<A>>>::target(
            &self.as_inner(),
        )
    }
    fn args<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<A>: 'a,
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
    {
        <ArenaRef<PartialTerm, A> as PartialApplicationTermType<WasmExpression<A>>>::args(
            &self.as_inner(),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<PartialTerm, A> {
    fn size(&self) -> usize {
        1 + self.target().size() + self.args().size()
    }
    fn capture_depth(&self) -> StackOffset {
        let target_depth = self.target().capture_depth();
        let arg_depth = self.args().capture_depth();
        target_depth.max(arg_depth)
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        let target_free_variables = self.target().free_variables();
        let args_free_variables = self.args().free_variables();
        if target_free_variables.is_empty() {
            args_free_variables
        } else if args_free_variables.is_empty() {
            target_free_variables
        } else {
            let mut combined = target_free_variables;
            combined.extend(args_free_variables);
            combined
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.target().count_variable_usages(offset) + self.args().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        let target_dependencies = self.target().dynamic_dependencies(deep);
        if deep {
            target_dependencies.union(self.args().dynamic_dependencies(deep))
        } else {
            match self.target().arity() {
                None => target_dependencies,
                Some(arity) => get_eager_args(self.args().as_inner().iter(), &arity).fold(
                    target_dependencies,
                    |combined_dependencies, arg| {
                        combined_dependencies.union(arg.dynamic_dependencies(deep))
                    },
                ),
            }
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        self.target().has_dynamic_dependencies(deep)
            || (if deep {
                self.args().has_dynamic_dependencies(deep)
            } else {
                match self.target().arity() {
                    None => false,
                    Some(arity) => get_eager_args(self.args().as_inner().iter(), &arity)
                        .any(|arg| arg.has_dynamic_dependencies(deep)),
                }
            })
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.target().is_atomic() && self.args().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<PartialTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<PartialTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<PartialTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<PartialTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<PartialTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<partial:{}:{}>",
            self.args().as_inner().len(),
            self.target()
        )
    }
}

fn get_eager_args<T>(args: impl IntoIterator<Item = T>, arity: &Arity) -> impl Iterator<Item = T> {
    arity
        .iter()
        .zip(args)
        .filter_map(|(arg_type, arg)| match arg_type {
            ArgType::Strict | ArgType::Eager => Some(arg),
            ArgType::Lazy => None,
        })
}

impl<A: Arena + Clone> Internable for ArenaRef<PartialTerm, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        self.target().should_intern(eager) && self.args().as_inner().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<PartialTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let args = self.args();
        let num_partial_args = args.as_inner().len();
        if num_partial_args == 0 {
            return target.compile(stack, state, options);
        }
        let block = CompiledBlockBuilder::new(stack);
        // Push the partial application target onto the stack
        // => [Term]
        let block = block.append_inner(|stack| target.compile(stack, state, options))?;
        // Push the partial application arguments onto the stack
        // => [Term, ListTerm]
        let block = if args.as_term().should_intern(Eagerness::Eager) {
            block.append_inner(|stack| args.as_term().compile(stack, state, options))
        } else {
            // Attempt to determine the application target arity according to the compiler eagerness strategy
            // (this is only possible for static function calls)
            let target_arity = get_compiled_function_arity(&target, options);
            // If the target arity is known, drop any excess partially-applied arguments
            let num_partial_args = match target_arity {
                None => num_partial_args,
                Some(arity) => {
                    num_partial_args.min((0..num_partial_args).zip(arity.iter()).count())
                }
            };
            // Compile the partial application argumens into a list according to the target arity and compiler eagerness strategy
            block.append_inner(|stack| {
                compile_list(
                    WithExactSizeIterator::new(
                        num_partial_args,
                        args.as_inner()
                            .iter()
                            .zip(
                                target_arity
                                    .unwrap_or(if options.lazy_function_args {
                                        Arity::lazy(num_partial_args, 0, false)
                                    } else {
                                        Arity::strict(num_partial_args, 0, false)
                                    })
                                    .iter(),
                            )
                            .map(|(arg, arg_type)| match arg_type {
                                ArgType::Strict => {
                                    let strictness =
                                        if arg.is_atomic() && arg.as_signal_term().is_none() {
                                            Strictness::Strict
                                        } else {
                                            Strictness::NonStrict
                                        };
                                    (MaybeLazyExpression::Eager(arg), strictness)
                                }
                                ArgType::Eager => {
                                    (MaybeLazyExpression::Eager(arg), Strictness::Strict)
                                }
                                ArgType::Lazy => (
                                    MaybeLazyExpression::Lazy(LazyExpression::new(arg)),
                                    Strictness::Strict,
                                ),
                            }),
                    ),
                    stack,
                    state,
                    options,
                )
            })
        }?;
        // Invoke the term constructor
        // => [PartialTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreatePartial,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn partial() {
        assert_eq!(
            TermType::Partial(PartialTerm {
                target: ArenaPointer(0x54321),
                args: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Partial as u32, 0x54321, 0x98765],
        );
    }
}
