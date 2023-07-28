// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ApplicationTermType, ArgType, Arity, DependencyList, Expression, GraphNode, LambdaTermType,
    SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompiledFunctionCall, CompiledFunctionCallArgs, CompiledFunctionId,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ConstValue, Eagerness,
        Internable, LazyExpression, MaybeLazyExpression, ParamsSignature, Strictness,
        TypeSignature, ValueType,
    },
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    term_type::{
        list::compile_list, BuiltinTerm, ConstructorTerm, LambdaTerm, ListTerm, TypedTerm,
        WasmExpression,
    },
    ArenaPointer, ArenaRef, Term,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct ApplicationTerm {
    pub target: ArenaPointer,
    pub args: ArenaPointer,
}

impl TermSize for ApplicationTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ApplicationTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let target_hash = arena.read_value::<Term, _>(self.target, |term| term.id());
        let args_hash = arena.read_value::<Term, _>(self.args, |term| term.id());
        hasher.hash(&target_hash, arena).hash(&args_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<ApplicationTerm, A> {
    pub fn target(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|value| value.target))
    }
    pub fn args(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|value| value.args),
        )
    }
}

impl<A: Arena + Clone> ApplicationTermType<WasmExpression<A>> for ArenaRef<ApplicationTerm, A> {
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

impl<A: Arena + Clone> ApplicationTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<ApplicationTerm>, A>
{
    fn target<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        <ArenaRef<ApplicationTerm, A> as ApplicationTermType<WasmExpression<A>>>::target(
            &self.as_inner(),
        )
    }
    fn args<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionListRef<'a>
    where
        WasmExpression<A>: 'a,
        <WasmExpression<A> as Expression>::ExpressionList: 'a,
    {
        <ArenaRef<ApplicationTerm, A> as ApplicationTermType<WasmExpression<A>>>::args(
            &self.as_inner(),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<ApplicationTerm, A> {
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
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<ApplicationTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!("Unable to patch terms: {}, {}", self, target))
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<ApplicationTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target() && self.args() == other.args()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<ApplicationTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<ApplicationTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<ApplicationTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<apply:{}:({})>",
            self.target(),
            self.args()
                .as_inner()
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<_>>()
                .join(", ")
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

impl<A: Arena + Clone> Internable for ArenaRef<ApplicationTerm, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        matches!(eager, Eagerness::Lazy)
            && self.target().is_static()
            && self.target().should_intern(eager)
            && self.args().as_inner().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ApplicationTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let args = self.args();
        // Determine the application target and combined set of arguments, taking into account any
        // potentially-nested partial applications
        let application_type = ApplicationFunctionCall::parse(target, args);
        // Compile the application according to the type of application
        application_type.compile(stack, state, options)
    }
}

#[derive(Debug, Clone)]
enum ApplicationFunctionCall<A: Arena + Clone> {
    /// Unknown application target (e.g. return value of another application)
    Generic(GenericCompiledFunctionCall<A>),
    /// Stdlib application target
    Builtin(BuiltinCompiledFunctionCall<A>),
    /// Lambda application target
    Lambda(LambdaCompiledFunctionCall<A>),
    /// Constructor application target
    Constructor(ConstructorCompiledFunctionCall<A>),
}

impl<A: Arena + Clone> ApplicationFunctionCall<A> {
    pub fn parse(target: WasmExpression<A>, args: ArenaRef<TypedTerm<ListTerm>, A>) -> Self {
        // Collect a list of partially-applied arguments, in the order in which the arguments will be passed to the function
        let (target, combined_args) = {
            // Combine arguments across multiple levels of nested partial terms
            let mut target = target;
            let mut arg_lists = Vec::new();
            while let Some(partial) = target.as_partial_term() {
                let partial = partial.as_inner();
                target = partial.target();
                let partial_args = partial.args();
                if partial_args.as_inner().len() > 0 {
                    arg_lists.push(partial_args);
                }
            }
            // Partially-applied arguments are applied deepest-first, so reverse the list
            arg_lists.reverse();
            (target, arg_lists)
        };
        // Push the provided argument list onto the collection of partially-applied arguments
        let combined_args = if args.as_inner().len() > 0 {
            let mut combined_args = combined_args;
            combined_args.push(args);
            combined_args
        } else {
            combined_args
        };
        let args = CompiledFunctionCallArgs {
            args: combined_args,
        };
        if let Some(target) = target.as_builtin_term().cloned() {
            ApplicationFunctionCall::Builtin(BuiltinCompiledFunctionCall { target, args })
        } else if let Some(target) = target.as_constructor_term().cloned() {
            ApplicationFunctionCall::Constructor(ConstructorCompiledFunctionCall { target, args })
        } else if let Some(target) = target.as_lambda_term().cloned() {
            ApplicationFunctionCall::Lambda(LambdaCompiledFunctionCall { target, args })
        } else {
            // Unknown function application target, e.g. a lambda returned from another function
            ApplicationFunctionCall::Generic(GenericCompiledFunctionCall { target, args })
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ApplicationFunctionCall<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        match self {
            Self::Generic(inner) => inner.compile(stack, state, options),
            Self::Builtin(inner) => inner.compile(stack, state, options),
            Self::Lambda(inner) => inner.compile(stack, state, options),
            Self::Constructor(inner) => inner.compile(stack, state, options),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GenericCompiledFunctionCall<A: Arena + Clone> {
    target: ArenaRef<Term, A>,
    args: CompiledFunctionCallArgs<A>,
}

impl<A: Arena + Clone> CompileWasm<A> for GenericCompiledFunctionCall<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let block = CompiledBlockBuilder::new(stack);
        // Push the application target onto the stack
        // => [Term]
        let block = block.append_inner(|stack| target.compile(stack, state, options))?;
        // Break out of the current control flow stack if the application target is a signal
        // => [Term]
        let block = block.push(instruction::runtime::BreakOnSignal { target_block: 0 });
        // Yield the argument list onto the stack
        // => [Term, ListTerm]
        let block = {
            // If this function call comprises a single argument list (taking into account partially-applied arguments),
            // and that argument list is eligible for static term inlining, delegate to the underlying implementation
            if let Some(args) = args.as_internable(Eagerness::Eager) {
                block.append_inner(|stack| args.as_term().compile(stack, state, options))
            } else {
                // Otherwise compile the combined argument sequence into a list according to the compiler eagerness strategy
                if options.lazy_function_args {
                    block.append_inner(|stack| {
                        compile_list(
                            args.iter()
                                .map(|arg| (LazyExpression::new(arg), Strictness::NonStrict)),
                            stack,
                            state,
                            options,
                        )
                    })
                } else {
                    block.append_inner(|stack| {
                        compile_list(
                            args.iter().map(|item| {
                                // Skip signal-testing for list items that are already fully evaluated to a non-signal value
                                let strictness =
                                    if item.is_atomic() && item.as_signal_term().is_none() {
                                        Strictness::NonStrict
                                    } else {
                                        Strictness::Strict
                                    };
                                (item, strictness)
                            }),
                            stack,
                            state,
                            options,
                        )
                    })
                }
            }
        }?;
        // Apply the target to the arguments
        // => [Term]
        let block = block.push(instruction::runtime::Apply);
        // Evaluate the result
        // => [Term]
        let block = block.push(instruction::runtime::Evaluate);
        block.finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LambdaCompiledFunctionCall<A: Arena + Clone> {
    target: ArenaRef<TypedTerm<LambdaTerm>, A>,
    args: CompiledFunctionCallArgs<A>,
}

impl<A: Arena + Clone> CompileWasm<A> for LambdaCompiledFunctionCall<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let target_term = target.as_inner();
        let num_args = target_term.num_args() as usize;
        // If insufficient arguments were provided, return a compiler error
        if args.len() < num_args {
            return Err(CompilerError::InvalidFunctionArgs {
                target: target.as_term().clone(),
                arity: Arity::lazy(num_args, 0, false),
                args: args.iter().collect(),
            });
        }
        // If this is an immediately-invoked nullary lambda, inline the function body directly
        if num_args == 0 {
            return target.body().compile(stack, state, options);
        }
        // Ensure that the target lambda is compiled
        // (note that all lambda functions are extracted out and linked in a later compiler phase)
        let _ = target_term.compile(stack.clone(), state, options)?;
        let compiled_function_id = CompiledFunctionId::from(&target_term);
        let block = CompiledBlockBuilder::new(stack);
        // Push each argument onto the stack
        // => [Term...]
        let block = args
            .iter()
            .fold(Result::<_, CompilerError<_>>::Ok(block), |block, arg| {
                let block = block?;
                // Push the argument onto the stack
                // => [Term...]
                let block = if options.lazy_function_args {
                    block.append_inner(|stack| {
                        LazyExpression::new(arg).compile(stack, state, options)
                    })
                } else {
                    block.append_inner(|stack| arg.compile(stack, state, options))
                }?;
                Ok(block)
            })?;
        // Call the compiled lambda function
        // => [Term]
        let block = block.push(instruction::runtime::CallCompiledFunction {
            signature: TypeSignature::new(
                ParamsSignature::from_iter(args.iter().map(|_| ValueType::HeapPointer)),
                ValueType::HeapPointer,
            ),
            target: compiled_function_id,
        });
        // Evaluate the result
        // => [Term]
        let block = block.push(instruction::runtime::Evaluate);
        block.finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ConstructorCompiledFunctionCall<A: Arena + Clone> {
    target: ArenaRef<TypedTerm<ConstructorTerm>, A>,
    args: CompiledFunctionCallArgs<A>,
}

impl<A: Arena + Clone> CompileWasm<A> for ConstructorCompiledFunctionCall<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let term = target.as_inner();
        let keys = term.keys();
        let block = CompiledBlockBuilder::new(stack);
        // Yield the key list onto the stack
        // => [ListTerm]
        let block = if keys.as_term().should_intern(Eagerness::Eager) {
            block.append_inner(|stack| keys.as_term().compile(stack, state, options))
        } else {
            block.append_inner(|stack| {
                compile_list(
                    keys.as_inner()
                        .iter()
                        .map(|key| (key, Strictness::NonStrict)),
                    stack,
                    state,
                    options,
                )
            })
        }?;
        // Yield the value list onto the stack
        // => [ListTerm, ListTerm]
        let block = if let Some(values) = args.as_internable(Eagerness::Eager) {
            block.append_inner(|stack| values.as_term().compile(stack, state, options))
        } else {
            if options.lazy_constructors {
                block.append_inner(|stack| {
                    compile_list(
                        args.iter()
                            .map(|value| (LazyExpression::new(value), Strictness::NonStrict)),
                        stack,
                        state,
                        options,
                    )
                })
            } else {
                block
                    .append_inner(|stack| {
                        compile_list(
                            args.iter().map(|item| {
                                // Skip signal-testing for list items that are already fully evaluated to a non-signal value
                                let strictness =
                                    if item.is_atomic() && item.as_signal_term().is_none() {
                                        Strictness::NonStrict
                                    } else {
                                        Strictness::Strict
                                    };
                                (item, strictness)
                            }),
                            stack,
                            state,
                            options,
                        )
                    })
                    .map(|block| {
                        block.push(instruction::runtime::BreakOnSignal { target_block: 0 })
                    })
            }
        }?;
        // Invoke the term constructor
        // => [RecordTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateRecord,
        });
        block.finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BuiltinCompiledFunctionCall<A: Arena + Clone> {
    target: ArenaRef<TypedTerm<BuiltinTerm>, A>,
    args: CompiledFunctionCallArgs<A>,
}

impl<A: Arena + Clone> CompileWasm<A> for BuiltinCompiledFunctionCall<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let term = target.as_inner();
        let builtin_target = term.target();
        // Retrieve the corresponding target function
        let builtin = Stdlib::try_from(builtin_target)
            .map_err(|_| CompilerError::InvalidFunctionTarget(builtin_target))?;
        let target = target.as_term().clone();
        match builtin {
            // Certain builtin stdlib functions have special-case compilation strategies
            Stdlib::And(builtin) => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(stack, state, options),
            Stdlib::If(builtin) => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(stack, state, options),
            Stdlib::Or(builtin) => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(stack, state, options),
            // All other builtins are compiled as generic stdlib function calls
            builtin => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(stack, state, options),
        }
    }
}

impl<'a, A: Arena + Clone> CompileWasm<A> for CompiledFunctionCall<'a, A, Stdlib> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let Self {
            builtin,
            target,
            args,
        } = self;
        let arity = builtin.arity();
        // If insufficient arguments were provided, return a compiler error
        if args.len() < arity.required().len() {
            return Err(CompilerError::InvalidFunctionArgs {
                target: target.clone(),
                arity,
                args: args.iter().collect(),
            });
        }
        let args_with_eagerness = args.iter().zip(arity.iter());
        let num_provided_args = args.len();
        let num_positional_args = arity.required().len() + arity.optional().len();
        let num_variadic_args = num_provided_args.saturating_sub(num_positional_args);
        let variadic_args_offset = arity.variadic().map(|_| num_positional_args);
        let num_strict_args = args_with_eagerness
            .clone()
            .filter(|(_, arg_type)| arg_type.is_strict())
            .count();
        let has_multiple_strict_args = num_strict_args > 1;
        let block = CompiledBlockBuilder::new(stack);
        // Any strict arguments need to be tested for signals, so while assigning the arguments we keep track of a
        // combined signal term pointer (or null pointer) containing the aggregate of all signals encountered amongst the
        // strict arguments, where the null pointer will be used as a placeholder if there were no signals
        // encountered amongst the strict arguments. This check will be omitted if there are no strict arguments.
        //
        // Assign each argument in turn, and for each strict argument, create a new temporary lexical scope containing the
        // combined signal result with the accumulated signal result from all previous strict arguments.
        // If there are variadic arguments, collect these into a list term as the final item on the operand stack
        // => [Term..., {ListTerm}]
        let (block, num_signal_scopes) = args_with_eagerness
            .enumerate()
            .map(|(index, (arg, arg_type))| {
                // Determine whether the argument should be evaluated before being being passed into the function
                let eagerness = match arg_type {
                    ArgType::Strict | ArgType::Eager => Eagerness::Eager,
                    ArgType::Lazy => Eagerness::Lazy,
                };
                // Determine whether to short-circuit any signals encountered when evaluating this argument
                let strictness = match arg_type {
                    ArgType::Strict => {
                        // Skip signal-testing for arguments that are already fully evaluated to a non-signal value
                        if arg.is_atomic() && arg.as_signal_term().is_none() {
                            Strictness::NonStrict
                        } else {
                            Strictness::Strict
                        }
                    }
                    ArgType::Eager | ArgType::Lazy => Strictness::NonStrict,
                };
                // Determine the index of this argument within the set of variadic arguments
                let variadic_arg_index = variadic_args_offset.and_then(|variadic_args_offset| {
                    if index >= variadic_args_offset {
                        Some(index - variadic_args_offset)
                    } else {
                        None
                    }
                });
                (
                    MaybeLazyExpression::new(arg, eagerness),
                    strictness,
                    variadic_arg_index,
                )
            })
            .fold(
                Result::<_, CompilerError<_>>::Ok((block, 0usize)),
                |result, (arg, strictness, variadic_arg_index)| {
                    let (block, num_signal_scopes) = result?;
                    // If this is a variadic argument, we need to add it to an argument list rather than leaving it on the stack
                    let block = if let Some(variadic_arg_index) = variadic_arg_index {
                        // If this is the first variadic argument, construct a list term to hold the variadic arguments
                        let block = if variadic_arg_index == 0 {
                            // Push the list capacity onto the stack
                            // => [Term..., u32]
                            let block = block.push(instruction::core::Const {
                                value: ConstValue::U32(num_variadic_args as u32),
                            });
                            // Allocate the list term
                            // => [Term..., ListTerm]
                            let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                                target: RuntimeBuiltin::AllocateList,
                            });
                            block
                        } else {
                            block
                        };
                        // Duplicate the argument list term pointer onto the stack
                        // => [Term..., ListTerm, ListTerm]
                        let block = block.push(instruction::core::Duplicate {
                            value_type: ValueType::HeapPointer,
                        });
                        // Push the item index onto the stack
                        // => [Term..., ListTerm, ListTerm, u32]
                        let block = block.push(instruction::core::Const {
                            value: ConstValue::U32(variadic_arg_index as u32),
                        });
                        block
                    } else {
                        // Otherwise if this is a standard positional argument, yield the item directly onto the stack
                        block
                    };
                    // Yield the argument onto the stack
                    // => [Term..., {ListTerm, ListTerm, u32}, Term]
                    let block = block.append_inner(|stack| {
                        let should_create_signal_boundary = match strictness {
                            // If the argument is not strict, we need to add a block boundary around this argument to
                            // prevent any inner signals from breaking out of the current scope
                            Strictness::NonStrict => true,
                            // Otherwise if this is a strict argument, if this is the only strict argument it can bubble
                            // directly to the call site, however if there are multiple strict arguments, we need to add
                            // a block boundary around each strict argument to allow combining signals before bubbling
                            // (this ensures that if signals are encountered across multiple arguments, signals will be
                            // 'caught' at their respective block boundaries, to be combined into a single signal result,
                            // rather than the first signal short-circuiting all the way to the top level)
                            Strictness::Strict => has_multiple_strict_args,
                            // Variadic arguments also need a block boundary to prevent inner signals from breaking out
                            // of a half-constructed variadic argument list
                        } || variadic_arg_index.is_some();
                        if should_create_signal_boundary {
                            let block_type = TypeSignature {
                                params: ParamsSignature::Void,
                                results: ParamsSignature::Single(ValueType::HeapPointer),
                            };
                            let inner_stack = stack.enter_block(&block_type)?;
                            let block = CompiledBlockBuilder::new(stack);
                            let block = block.push(instruction::core::Block {
                                block_type,
                                body: arg.compile(inner_stack, state, options)?,
                            });
                            block.finish()
                        } else {
                            arg.compile(stack, state, options)
                        }
                    })?;
                    // If this argument needs to be tested for signals, combine the argument's signal result with the accumulated signal result
                    // => [Term..., {ListTerm, ListTerm, u32}, Term]
                    let (block, num_signal_scopes) = if strictness.is_strict() {
                        // Pop the argument from the top of the stack and assign to a temporary lexical scope variable
                        // => [Term..., {ListTerm, ListTerm, u32}]
                        let block = block.push(instruction::core::ScopeStart {
                            value_type: ValueType::HeapPointer,
                        });
                        // Push the argument back onto the top of the stack
                        // => [Term..., {ListTerm, ListTerm, u32}, Term]
                        let block = block.push(instruction::core::GetScopeValue {
                            scope_offset: 0,
                            value_type: ValueType::HeapPointer,
                        });
                        let has_preceding_signal_args = num_signal_scopes > 0;
                        // If this is not the first strict argument, push the accumulated signal term onto the top of the stack
                        // (the combined signal result from processing the previous argument will be in the penultimate lexical scope)
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}]
                        let block = if has_preceding_signal_args {
                            block.push(instruction::core::GetScopeValue {
                                scope_offset: 1,
                                value_type: ValueType::HeapPointer,
                            })
                        } else {
                            block
                        };
                        // Push another copy of the argument back onto the top of the stack
                        // (this will be used as the 'true' branch of the signal-testing select instruction)
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}, Term]
                        let block = block.push(instruction::core::GetScopeValue {
                            scope_offset: 0,
                            value_type: ValueType::HeapPointer,
                        });
                        // Push a null pointer onto the top of the stack
                        // (this will be used as the 'false' branch of the signal-testing select instruction)
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}, Term, NULL]
                        let block = block.push(instruction::runtime::NullPointer);
                        // Push another copy of the argument onto the top of the stack
                        // (this will be used to test whether the term is a signal term)
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}, Term, NULL, Term]
                        let block = block.push(instruction::core::GetScopeValue {
                            scope_offset: 0,
                            value_type: ValueType::HeapPointer,
                        });
                        // Determine whether the argument is a signal term
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}, Term, NULL, bool]
                        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                            target: RuntimeBuiltin::IsSignal,
                        });
                        // Determine whether the argument is a signal term
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}, Option<SignalTerm>]
                        let block = block.push(instruction::core::Select {
                            value_type: ValueType::HeapPointer,
                        });
                        // Dispose the temporary lexical scope variable used to store the argument
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, {Option<SignalTerm>}, Option<SignalTerm>]
                        let block = block.push(instruction::core::ScopeEnd {
                            value_type: ValueType::HeapPointer,
                        });
                        // If there have been previous strict arguments, combine this argument's optional signal term pointer
                        // with the accumulated signal term pointer that has already been added to the stack,
                        // => [Term..., {ListTerm, ListTerm, u32}, Term, Option<SignalTerm>]
                        let block = if has_preceding_signal_args {
                            block.push(instruction::runtime::CallRuntimeBuiltin {
                                target: RuntimeBuiltin::CombineSignals,
                            })
                        } else {
                            // If there were no previous strict arguments, the stack will already be in the correct state
                            block
                        };
                        // Pop the combined signal result from the stack and assign it to a new lexical scope that tracks
                        // the latest accumulated signal result (SSA equivalent of mutating an accumulator variable).
                        // While this may appear to require many more locals than mutating a single accumulator
                        // variable, in practice the chain of nested SSA scopes can be optimized away during a later
                        // compiler optimization pass
                        // => [Term..., {ListTerm, ListTerm, u32}, Term]
                        let block = block.push(instruction::core::ScopeStart {
                            value_type: ValueType::HeapPointer,
                        });
                        (block, num_signal_scopes + 1)
                    } else {
                        (block, num_signal_scopes)
                    };
                    // If this was a variadic argument, assign the argument to the variadic argument list
                    // (if this is the case, the argument list term pointer and index will already be present on the stack)
                    let block = if variadic_arg_index.is_some() {
                        // Set the argument list term's value at the given index to the child item, leaving the list on top of the stack
                        // => [Term..., ListTerm]
                        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                            target: RuntimeBuiltin::SetListItem,
                        });
                        block
                    } else {
                        block
                    };
                    Ok((block, num_signal_scopes))
                },
            )?;
        // If a variadic argument list was allocated, initialize the list term with the correct length
        let block = if arity.variadic().is_some() {
            // Push the list length onto the stack
            // => [Term..., ListTerm, u32]
            let block = block.push(instruction::core::Const {
                value: ConstValue::U32(num_variadic_args as u32),
            });
            // Initialize the list term with the length that is on the stack
            // => [Term..., ListTerm]
            let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                target: RuntimeBuiltin::InitList,
            });
            block
        } else {
            block
        };
        // If there were strict arguments and one or more of the strict arguments was a signal, short-circuit the
        // combined signal result, otherwise continue
        // => [Term..., {ListTerm}]
        let block = if num_signal_scopes > 0 {
            // The combined signal result from processing the final argument will be in the most recently-declared lexical scope
            let combined_signal_scope_offset = 0;
            // Push the combined signal result onto the stack
            // => [Term..., {ListTerm}, Option<SignalTerm>]
            let block = block.push(instruction::core::GetScopeValue {
                scope_offset: combined_signal_scope_offset,
                value_type: ValueType::HeapPointer,
            });
            // Push another copy of the combined signal result onto the stack for comparing against the null pointer
            // => [Term..., {ListTerm}, Option<SignalTerm>, Option<SignalTerm>]
            let block = block.push(instruction::core::GetScopeValue {
                scope_offset: combined_signal_scope_offset,
                value_type: ValueType::HeapPointer,
            });
            // Push a null pointer onto the stack to use for comparing against the combined signal term result
            // => [Term..., {ListTerm}, Option<SignalTerm>, Option<SignalTerm>, NULL]
            let block = block.push(instruction::runtime::NullPointer);
            // Determine whether the combined signal result is not equal to the null pointer
            // => [Term..., {ListTerm}, Option<SignalTerm>, bool]
            let block = block.push(instruction::core::Ne {
                value_type: ValueType::HeapPointer,
            });
            // Dispose any temporary signal-testing lexical scopes
            // => [Term..., {ListTerm}, Option<SignalTerm>, bool]
            let block = (0..num_signal_scopes).fold(block, |block, _| {
                block.push(instruction::core::ScopeEnd {
                    value_type: ValueType::HeapPointer,
                })
            });
            // Break out of the current control flow block if a signal term was encountered
            // => [Term..., {ListTerm}, Option<SignalTerm>]
            let block = block.push(instruction::core::ConditionalBreak {
                target_block: 0,
                result_type: ParamsSignature::Single(ValueType::HeapPointer),
            });
            // Otherwise drop the null signal result
            // => [Term..., {ListTerm}]
            let block = block.push(instruction::core::Drop {
                value_type: ValueType::HeapPointer,
            });
            block
        } else {
            block
        };
        // Now that the arguments are laid out in the correct order on the stack, apply the builtin function
        // => [Term..., {ListTerm}]
        let block = block.push(instruction::runtime::CallStdlib { target: *builtin });
        // Evaluate the result
        // => [Term]
        let block = block.push(instruction::runtime::Evaluate);
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn application() {
        assert_eq!(
            TermType::Application(ApplicationTerm {
                target: ArenaPointer(0x54321),
                args: ArenaPointer(0x98765),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Application as u32, 0x54321, 0x98765],
        );
    }
}
