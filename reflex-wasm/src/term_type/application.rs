// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    ApplicationTermType, ArgType, Arity, DependencyList, Eagerness, Expression, GraphNode,
    Internable, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledFunctionCall,
        CompiledFunctionCallArgs, CompiledFunctionId, CompiledInstruction, CompilerError,
        CompilerOptions, CompilerResult, CompilerStack, CompilerStackValue, CompilerState,
        CompilerVariableBindings, MaybeLazyExpression, ParamsSignature, TypeSignature, ValueType,
    },
    hash::{TermHash, TermHashState, TermHasher, TermSize},
    stdlib::Stdlib,
    term_type::{
        BuiltinTerm, ConstructorTerm, LambdaTerm, ListTerm, TreeTerm, TypedTerm, WasmExpression,
    },
    utils::{chunks_to_u64, u64_to_chunks},
    ArenaPointer, ArenaRef, PointerIter, Term,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ApplicationTerm {
    pub target: ArenaPointer,
    pub args: ArenaPointer,
    pub cache: ApplicationCache,
}

#[derive(Clone, Copy, Debug, reflex_macros::PointerIter)]
#[repr(C)]
pub struct ApplicationCache {
    pub value: ArenaPointer,
    pub dependencies: ArenaPointer,
    pub overall_state_hash: [u32; 2],
    pub minimal_state_hash: [u32; 2],
}

pub type ApplicationTermPointerIter =
    std::iter::Chain<std::array::IntoIter<ArenaPointer, 2>, ApplicationCachePointerIter>;

impl<A: Arena + Clone> PointerIter for ArenaRef<ApplicationTerm, A> {
    type Iter<'a> = ApplicationTermPointerIter
    where
        Self: 'a;
    fn iter<'a>(&self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        let pointers = [
            self.inner_pointer(|term| &term.target),
            self.inner_pointer(|term| &term.args),
        ];
        let cache = self.inner_ref::<ApplicationCache>(|term| &term.cache);
        let cache_pointers: ApplicationCachePointerIter = PointerIter::iter(&cache);
        pointers.into_iter().chain(cache_pointers)
    }
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

impl TermSize for ApplicationCache {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Default for ApplicationCache {
    fn default() -> Self {
        Self {
            value: ArenaPointer::null(),
            dependencies: ArenaPointer::null(),
            overall_state_hash: u64_to_chunks(0xFFFFFFFFFFFFFFFF),
            minimal_state_hash: u64_to_chunks(0xFFFFFFFFFFFFFFFF),
        }
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
    pub fn cache(&self) -> ArenaRef<ApplicationCache, A> {
        self.inner_ref(|value| &value.cache)
    }
}

impl<A: Arena + Clone> ArenaRef<ApplicationCache, A> {
    pub fn value(&self) -> Option<ArenaRef<Term, A>> {
        let pointer = self.read_value(|value| value.value).as_non_null()?;
        Some(ArenaRef::<Term, _>::new(self.arena.clone(), pointer))
    }
    pub fn dependencies(&self) -> Option<ArenaRef<TypedTerm<TreeTerm>, A>> {
        let pointer = self.read_value(|value| value.dependencies).as_non_null()?;
        Some(ArenaRef::<TypedTerm<TreeTerm>, _>::new(
            self.arena.clone(),
            pointer,
        ))
    }
    pub fn overall_state_hash(&self) -> Option<TermHashState> {
        let value = self.read_value(|value| chunks_to_u64(value.overall_state_hash));
        if value == 0xFFFFFFFFFFFFFFFF {
            None
        } else {
            Some(TermHashState::from(value))
        }
    }
    pub fn minimal_state_hash(&self) -> Option<TermHashState> {
        let value = self.read_value(|value| chunks_to_u64(value.minimal_state_hash));
        if value == 0xFFFFFFFFFFFFFFFF {
            None
        } else {
            Some(TermHashState::from(value))
        }
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
        eager == Eagerness::Lazy
            && self.target().is_static()
            && self.target().should_intern(eager)
            && self.args().as_inner().should_intern(eager)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ApplicationTerm, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let target = self.target();
        let args = self.args();
        // Determine the application target and combined set of arguments, taking into account any
        // potentially-nested partial applications
        let application_type = ApplicationFunctionCall::parse(target, args);
        // Compile the application according to the type of application
        application_type.compile(state, bindings, options, stack)
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
        let mut target = target;
        let mut partial_args = Vec::new();
        while let Some(partial) = target.as_partial_term() {
            let partial = partial.as_inner();
            target = partial.target();
            let partial_arg_list = partial.args().as_inner();
            // Prepend the partially-appied args to any existing partially-applied args
            let existing_partial_args = std::mem::take(&mut partial_args);
            let added_args = partial_arg_list.iter();
            partial_args = added_args.chain(existing_partial_args).collect();
        }
        let args = CompiledFunctionCallArgs {
            partial_args,
            args: args.as_inner(),
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        match self {
            Self::Generic(inner) => inner.compile(state, bindings, options, stack),
            Self::Builtin(inner) => inner.compile(state, bindings, options, stack),
            Self::Lambda(inner) => inner.compile(state, bindings, options, stack),
            Self::Constructor(inner) => inner.compile(state, bindings, options, stack),
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let mut instructions = CompiledBlock::default();
        // Push the application target onto the stack
        // => [Term]
        instructions.append_block(target.compile(state, bindings, options, stack)?);
        let stack = stack.push_strict();
        // Duplicate the application target onto the stack to test whether it is a signal
        // => [Term, Term]
        instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
        // Invoke the builtin function to determine whether the value is a signal
        // => [Term, bool]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::IsSignal,
        ));
        // Short circuit if a signal term was encountered
        // TODO: Consolidate signal-testing code across multiple use cases
        // => [Term]
        instructions.push(CompiledInstruction::ConditionalBreak {
            // Retain the evaluated target term pointer on the stack
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
                    // Pop the signal term pointer from the top of the stack and store in a new temporary lexical scope
                    instructions.push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                    let stack = stack.pop();
                    // Discard any preceding stack arguments that had been captured for use in the continuation block closure
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
                    // Drop the temporary lexical scope
                    instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                }
                instructions
            },
        });
        // Yield the argument list onto the stack
        // => [Term, ListTerm]
        instructions.append_block(args.compile(state, bindings, options, &stack)?);
        // Apply the target to the arguments
        // => [Term]
        instructions.push(CompiledInstruction::Apply);
        // Evaluate the result
        // => [Term]
        instructions.push(CompiledInstruction::Evaluate);
        Ok(instructions)
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let target_term = target.as_inner();
        let num_args = target_term.num_args() as usize;
        // Ensure that the target lambda is compiled
        // (note that all lambda functions are extracted out and linked in a later compiler phase)
        let _ = target_term.compile(state, bindings, options, stack);
        let compiled_function_id = CompiledFunctionId::from(&target_term);
        // If insufficient arguments were provided, return a compiler error
        if args.len() < num_args {
            return Err(CompilerError::InvalidFunctionArgs {
                target: target.as_term().clone(),
                arity: Arity::lazy(num_args, 0, false),
                args: args.iter().collect(),
            });
        }
        let mut instructions = CompiledBlock::default();
        // Push each argument onto the stack
        // => [Term...]
        let _ = args.iter().fold(Ok(stack.clone()), |stack, arg| {
            let stack = stack?;
            // Push the argument onto the stack
            // => [Term...]
            instructions.append_block(arg.compile(state, bindings, options, &stack)?);
            // Push the current argument onto the accumulated list of captured stack values
            // TODO: Support eager lambda arguments
            Ok(stack.push_lazy(ValueType::HeapPointer))
        })?;
        // Apply the lambda function
        // => [Term]
        instructions.push(CompiledInstruction::CallCompiledFunction {
            signature: TypeSignature::new(
                ParamsSignature::from_iter(args.iter().map(|_| ValueType::HeapPointer)),
                ValueType::HeapPointer,
            ),
            target: compiled_function_id,
        });
        // Evaluate the result
        // => [Term]
        instructions.push(CompiledInstruction::Evaluate);
        Ok(instructions)
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let Self { target, args } = self;
        let term = target.as_inner();
        let keys = term.keys().as_inner();
        let mut instructions = CompiledBlock::default();
        // Yield the key list onto the stack as-is
        // => [ListTerm]
        instructions.append_block(keys.compile(state, bindings, options, stack)?);
        let stack = stack.push_lazy(ValueType::HeapPointer);
        // Yield the value list onto the stack
        // => [ListTerm, ListTerm]
        instructions.append_block(args.compile(state, bindings, options, &stack)?);
        // Invoke the term constructor
        // => [RecordTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateRecord,
        ));
        Ok(instructions)
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
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
            .compile(state, bindings, options, stack),
            Stdlib::If(builtin) => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(state, bindings, options, stack),
            Stdlib::Or(builtin) => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(state, bindings, options, stack),
            // All other builtins are compiled as generic stdlib function calls
            builtin => CompiledFunctionCall {
                builtin,
                target,
                args,
            }
            .compile(state, bindings, options, stack),
        }
    }
}

impl<'a, A: Arena + Clone> CompileWasm<A> for CompiledFunctionCall<'a, A, Stdlib> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
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
        let args_with_eagerness = args.iter().zip(arity.iter()).collect::<Vec<_>>();
        let mut instructions = CompiledBlock::default();
        let num_provided_args = args.len();
        let num_positional_args = arity.required().len() + arity.optional().len();
        let num_variadic_args = num_provided_args.saturating_sub(num_positional_args);
        let variadic_args_offset = arity.variadic().map(|_| num_positional_args);
        let num_existing_stack_values = stack.depth();
        // Any strict arguments need to be tested for signals, so we yield all the arguments onto the operand stack,
        // followed by a signal term pointer (or Nil term pointer) containing the aggregate of all signals encountered
        // amongst the strict arguments, where the Nil term will be used as a placeholder if there were no signals
        // encountered amongst the strict arguments. This check will be omitted if there are no strict arguments.
        //
        // Yield each argument in turn onto the stack, and for each strict argument, create a new temporary lexical scope
        // containing the combined signal result with the accumulated signal result from all previous strict arguments
        let (stack, num_signal_scopes) = args_with_eagerness
            .into_iter()
            .enumerate()
            .map(|(index, (arg, arg_type))| {
                // Determine whether the argument should be evaluated before being being passed into the function
                let eagerness = match arg_type {
                    ArgType::Strict | ArgType::Eager => Eagerness::Eager,
                    ArgType::Lazy => Eagerness::Lazy,
                };
                // Determine whether to short-circuit any signals encountered when evaluating this argument
                let is_strict =
                    arg_type.is_strict() && !arg.is_static() && arg.as_signal_term().is_none();
                // Determine the index of this argument within the set of variadic arguments
                let variadic_arg_index = variadic_args_offset.and_then(|variadic_args_offset| {
                    if index >= variadic_args_offset {
                        Some(index - variadic_args_offset)
                    } else {
                        None
                    }
                });
                (arg, eagerness, is_strict, variadic_arg_index)
            })
            .fold(
                Ok((stack.clone(), 0usize)),
                |results, (arg, eagerness, is_strict, variadic_arg_index)| {
                    let (stack, num_signal_scopes) = results?;
                    // Compile the argument
                    let compiled_arg = {
                        // Compile the argument in eager or lazy mode as appropriate
                        // Ensure any variable references within the arguments skip over any intermediate signal-testing locals,
                        let inner_bindings = bindings.offset(num_signal_scopes);
                        // If this is a variadic argument, the compiled argument will be injected at a later point in the block
                        // with preceding operand stack entries for the variadic argument list
                        // (this is necessary because the compiled argument block will be injected at a different stack state
                        // depending on whether the argument is a variadic argument)
                        // FIXME: This is confusing; extract into a function that takes a stack argument
                        let arg_stack = match variadic_arg_index {
                            Some(variadic_arg_index) => match variadic_arg_index {
                                // The first variadic argument will have the initial argument list instance,
                                // as well as racked-up operands for setting the argument list index
                                0 => Some(
                                    stack
                                        .push_lazy(ValueType::HeapPointer)
                                        .push_lazy(ValueType::HeapPointer)
                                        .push_lazy(ValueType::U32),
                                ),
                                // Any subsequent variadic arguments will already have had the argument list instance
                                // added to the stack,
                                // so they only need to rack up the operands for setting the argument list index
                                _ => Some(
                                    stack
                                        .push_lazy(ValueType::HeapPointer)
                                        .push_lazy(ValueType::U32),
                                ),
                            },
                            _ => None,
                        };
                        let mut instructions = MaybeLazyExpression::new(arg, eagerness).compile(
                            state,
                            &inner_bindings,
                            options,
                            arg_stack.as_ref().unwrap_or(&stack),
                        )?;
                        // If this is a strict argument, combine any signal result with the existing accumulated signal result from previous arguments
                        if is_strict {
                            // Pop the result from the stack and assign as the variable of a short-lived lexical scope
                            // to use for testing whether this argument's value is a signal
                            // => []
                            instructions
                                .push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                            // Push a copy of the result onto back onto the stack to be passed as the function argument
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
                            // depending on whether the argument value is a signal term pointer
                            // => [Term, Option<SignalTerm>]
                            instructions.push(CompiledInstruction::Select(ValueType::HeapPointer));
                            // Discard this argument's temporary signal-testing lexical scope
                            instructions
                                .push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                            // If there have been previous strict arguments, combine the current signal result with the
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
                            // variable), leaving the evaluated argument on top of the stack.
                            // While this may appear to require many more locals than mutating a single accumulator
                            // variable, in practice the chain of nested SSA scopes can be optimized away during a later
                            // compiler optimization pass
                            // => [Term]
                            instructions
                                .push(CompiledInstruction::ScopeStart(ValueType::HeapPointer));
                        }
                        instructions
                    };
                    // If this is a variadic argument, we need to add it to an argument list rather than leaving it on the stack
                    let stack = if let Some(variadic_arg_index) = variadic_arg_index {
                        // If this is the first variadic argument, construct a list term to hold the variadic arguments
                        let stack = if variadic_arg_index == 0 {
                            // Push the list capacity onto the stack
                            // => [capacity]
                            instructions
                                .push(CompiledInstruction::u32_const(num_variadic_args as u32));
                            // Allocate the list term
                            // => [ListTerm]
                            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                                RuntimeBuiltin::AllocateList,
                            ));
                            // Ensure the argument list stack value is captured when processing subsequent arguments
                            // FIXME: Verify signal short-circuiting behavior for nested variadic function calls
                            stack.push_lazy(ValueType::HeapPointer)
                        } else {
                            stack
                        };
                        // Duplicate the argument list term pointer onto the stack
                        // => [ListTerm, ListTerm]
                        instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
                        // Push the item index onto the stack
                        // => [ListTerm, ListTerm, index]
                        instructions
                            .push(CompiledInstruction::u32_const(variadic_arg_index as u32));
                        // Yield the item onto the stack
                        // => [ListTerm, ListTerm, index, Term]
                        instructions.append_block(compiled_arg);
                        // Set the argument list term's value at the given index to the child item, leaving the list on top of the stack
                        // => [ListTerm]
                        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                            RuntimeBuiltin::SetListItem,
                        ));
                        stack
                    } else {
                        // Otherwise if this is a standard positional argument, yield the item directly onto the stack
                        // => [Term]
                        instructions.append_block(compiled_arg);
                        // Ensure the argument stack value is captured when processing subsequent arguments
                        if is_strict {
                            stack.push_strict()
                        } else {
                            stack.push_lazy(ValueType::HeapPointer)
                        }
                    };
                    // If this is a strict argument, we will have created a temporary lexical scope to keep track of the
                    // combined signal result for this argument (taking into account the accumulated signal result from
                    // previous arguments)
                    // We need to keep track of how many temporary lexical scopes have been created, so we know how many to
                    // discard once we're done building up the accumulated signal
                    if is_strict {
                        Ok((stack, num_signal_scopes + 1))
                    } else {
                        // If this was not a strict argument, no extra signal scopes introduced
                        Ok((stack, num_signal_scopes))
                    }
                },
            )?;
        let num_arg_stack_values = stack.depth() - num_existing_stack_values;
        // Now that all the arguments have been put on the stack, short-circuit if a signal was encountered while
        // evaluating the strict arguments, leaving the arguments on top of the stack
        // => [Term...]
        if num_signal_scopes > 0 {
            // If there are strict arguments, we want to push a combined signal result onto the stack,
            // ensuring there is a valid term on top of the stack by replacing the potential null signal pointer
            // with a Nil placeholder term (this prevents null pointer exceptions when testing whether to break)
            //
            // Push a placeholder Nil term pointer onto the stack (true branch)
            // => [Term..., NilTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::CreateNil,
            ));
            // Push the combined signal result onto the stack (false branch)
            // (the combined signal result from processing the final argument will be in the most recent lexical scope)
            // => [Term..., NilTerm, Option<SignalTerm>]
            instructions.push(CompiledInstruction::GetScopeValue {
                value_type: ValueType::HeapPointer,
                scope_offset: 0,
            });
            // Push another copy of the signal result onto the stack for comparing against the null pointer
            // => [Term..., NilTerm, Option<SignalTerm>, Option<SignalTerm>]
            instructions.push(CompiledInstruction::GetScopeValue {
                value_type: ValueType::HeapPointer,
                scope_offset: 0,
            });
            // Push a null pointer onto the stack
            // => [Term..., NilTerm, Option<SignalTerm>, Option<SignalTerm>, NULL]
            instructions.push(CompiledInstruction::NullPointer);
            // Invoke the builtin function to determine whether the signal result is equal to the null pointer
            // => [Term..., NilTerm, Option<SignalTerm>, bool]
            instructions.push(CompiledInstruction::Eq(ValueType::HeapPointer));
            // Select either the Nil placeholder term pointer or the signal result term pointer respectively,
            // based on whether the signal result was null
            // => [Term..., Term]
            instructions.push(CompiledInstruction::Select(ValueType::HeapPointer));
            let stack = stack.push_strict();
            // Now that we have the final accumulated signal, we can drop all the temporary lexical scopes that were
            // used to store all the intermediate combined signals
            for _ in 0..num_signal_scopes {
                instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
            }
            // Duplicate the signal result onto the stack
            // => [Term..., Term, Term]
            instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
            // Push a boolean onto the stack indicating whether a signal term was encountered
            // => [Term..., Term, bool]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::IsSignal,
            ));
            // Short circuit if a signal term was encountered amongst the strict arguments, retaining the
            // arguments and signal result on the stack
            // TODO: Consolidate signal-testing code across multiple use cases
            // => [Term..., Term]
            instructions.push(CompiledInstruction::ConditionalBreak {
                // Retain the argument term pointers followed by the signal result term pointer on the stack
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
                        // Drop the temporary operand stack values holding the function argument values
                        // (any signals encountered when evaluating strict arguments have already been collected)
                        let stack = (0..num_arg_stack_values).fold(stack, |stack, _| {
                            instructions.push(CompiledInstruction::Drop(ValueType::HeapPointer));
                            stack.pop()
                        });
                        // Discard any preceding stack arguments that had been captured for use in the continuation block closure
                        let num_signal_scopes =
                            stack
                                .rev()
                                .fold(Ok(0usize), |num_signal_scopes, stack_value| {
                                    let num_signal_scopes = num_signal_scopes?;
                                    match stack_value {
                                        CompilerStackValue::Lazy(value_type) => {
                                            // If the captured stack value does not need to be checked for signals,
                                            // pop it from the operand stack and move on
                                            instructions
                                                .push(CompiledInstruction::Drop(value_type));
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
                                            instructions.push(
                                                CompiledInstruction::CallRuntimeBuiltin(
                                                    RuntimeBuiltin::IsSignal,
                                                ),
                                            );
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
                                            instructions.push(
                                                CompiledInstruction::CallRuntimeBuiltin(
                                                    RuntimeBuiltin::CombineSignals,
                                                ),
                                            );
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
                            instructions
                                .push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                        }
                        // Drop the temporary signal result term lexical scope
                        instructions.push(CompiledInstruction::ScopeEnd(ValueType::HeapPointer));
                    }
                    instructions
                },
            });
            // Drop the Nil signal placeholder term pointer, leaving just the arguments
            // => [Term...]
            instructions.push(CompiledInstruction::Drop(ValueType::HeapPointer));
        }
        // If a variadic argument list was allocated, initialize the list term with the correct length
        if num_variadic_args > 0 {
            // Push the list length onto the stack
            // => [Term..., ListTerm, length]
            instructions.push(CompiledInstruction::u32_const(num_variadic_args as u32));
            // Initialize the list term with the length that is on the stack
            // => [Term..., ListTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::InitList,
            ));
        }
        // Now that the arguments are laid out in the correct order on the stack, apply the builtin function
        // => [Term]
        instructions.push(CompiledInstruction::CallStdlib(*builtin));
        // Evaluate the result
        // => [Term]
        instructions.push(CompiledInstruction::Evaluate);
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        term_type::{TermType, TermTypeDiscriminants},
        utils::u64_to_chunks,
    };

    use super::*;

    #[test]
    fn application() {
        assert_eq!(
            TermType::Application(ApplicationTerm {
                target: ArenaPointer(0x54321),
                args: ArenaPointer(0x98765),
                cache: ApplicationCache {
                    value: ArenaPointer::null(),
                    dependencies: ArenaPointer::null(),
                    overall_state_hash: u64_to_chunks(0xFFFFFFFFFFFFFFFF),
                    minimal_state_hash: u64_to_chunks(0xFFFFFFFFFFFFFFFF),
                },
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::Application as u32,
                0x54321,
                0x98765,
                0xFFFFFFFF,
                0xFFFFFFFF,
                0xFFFFFFFF,
                0xFFFFFFFF,
                0xFFFFFFFF,
                0xFFFFFFFF,
            ],
        );
    }
}
