// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

use crate::{
    allocator::Arena,
    compiler::{
        compile_variable_declarations, error::CompilerError, instruction,
        runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder, CompiledFunctionCall,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ParamsSignature,
        TypeSignature, ValueType,
    },
    term_type::{LambdaTerm, ListTerm, TypedTerm, WasmExpression},
    ArenaRef,
};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct If;
impl If {
    pub const UUID: Uuid = uuid!("9c8fc3a1-2d55-420e-bf81-3098932f8cf0");
    const ARITY: FunctionArity<3, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for If {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}

impl<'a, A: Arena + Clone> CompileWasm<A> for CompiledFunctionCall<'a, A, If> {
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
            ..
        } = self;
        let block = CompiledBlockBuilder::new(stack);
        let (condition, consequent, alternative) = {
            let mut arg_list = args.iter();
            let condition = arg_list.next();
            let consequent = arg_list.next();
            let alternative = arg_list.next();
            match (condition, consequent, alternative) {
                (Some(condition), Some(consequent), Some(alternative)) => {
                    Ok((condition, consequent, alternative))
                }
                _ => Err(CompilerError::InvalidFunctionArgs {
                    target: target.clone(),
                    arity: builtin.arity(),
                    args: args.iter().collect(),
                }),
            }
        }?;
        // Yield the condition onto the stack
        // => [Term]
        let block = block.append_inner(|stack| condition.compile(stack, state, options))?;
        // If the condition evaluated to a signal, break out of the current control flow block, otherwise continue
        // => [Term]
        let block = block.push(instruction::runtime::BreakOnSignal { target_block: 0 });
        // TODO: Break on non-boolean condition terms
        // Invoke the runtime builtin to determine whether the condition is truthy
        // => [bool]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::GetBooleanValue,
        });
        // If the condition was truthy, apply the consequent branch factory, otherwise apply the alternative branch factory
        // => []
        let block = block.append_inner(|stack| {
            let block_type = TypeSignature {
                params: ParamsSignature::Void,
                results: ParamsSignature::Single(ValueType::HeapPointer),
            };
            let inner_stack = stack.enter_block(&block_type)?;
            let (consequent_stack, alternative_stack) = (inner_stack.clone(), inner_stack);
            let block = CompiledBlockBuilder::new(stack);
            let block = block.push(instruction::core::If {
                block_type,
                consequent: {
                    // Yield the consequent branch onto the stack
                    // => [Term]
                    compile_conditional_branch(&consequent, consequent_stack, state, options)
                }?,
                alternative: {
                    // Yield the alternative branch onto the stack
                    // => [Term]
                    compile_conditional_branch(&alternative, alternative_stack, state, options)
                }?,
            });
            block.finish::<CompilerError<_>>()
        })?;
        block.finish()
    }
}

pub(crate) fn compile_conditional_branch<A: Arena + Clone>(
    factory: &WasmExpression<A>,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    if let Some(factory) = match_nullary_lambda_branch(factory) {
        compile_lambda_branch(&factory.as_inner().body(), stack, state, options)
    } else if let Some((closure_args, closure)) = match_nullary_closure_branch(factory) {
        compile_closure_branch(
            closure_args.as_inner().iter(),
            &closure.as_inner().body(),
            stack,
            state,
            options,
        )
    } else {
        compile_generic_factory_branch(factory, stack, state, options)
    }
}

fn match_nullary_lambda_branch<A: Arena + Clone>(
    factory: &WasmExpression<A>,
) -> Option<&ArenaRef<TypedTerm<LambdaTerm>, A>> {
    factory
        .as_lambda_term()
        .filter(|term| term.as_inner().num_args() == 0)
}

fn match_nullary_closure_branch<A: Arena + Clone>(
    factory: &WasmExpression<A>,
) -> Option<(
    ArenaRef<TypedTerm<ListTerm>, A>,
    ArenaRef<TypedTerm<LambdaTerm>, A>,
)> {
    factory.as_partial_term().and_then(|term| {
        let term = term.as_inner();
        let closure_args = term.args();
        let inner_factory = term
            .target()
            .as_lambda_term()
            .filter(|term| term.as_inner().num_args() as usize == closure_args.as_inner().len())
            .cloned()?;
        Some((closure_args, inner_factory))
    })
}

fn compile_lambda_branch<A: Arena + Clone>(
    body: &WasmExpression<A>,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    // Yield the function body onto the stack
    // => [Term]
    body.compile(stack, state, options)
}

fn compile_closure_branch<A: Arena + Clone>(
    closure_initializers: impl IntoIterator<
        Item = WasmExpression<A>,
        IntoIter = impl Iterator<Item = WasmExpression<A>> + ExactSizeIterator,
    >,
    body: &WasmExpression<A>,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    let closure_initializers = closure_initializers.into_iter();
    let num_variable_scopes = closure_initializers.len();
    let block = CompiledBlockBuilder::new(stack);
    // Declare lexical scopes for each of the closed-over variable initializer values
    // => []
    let block = block.append_inner(|stack| {
        compile_variable_declarations(
            closure_initializers
                .into_iter()
                .map(|initializer| (ValueType::HeapPointer, initializer)),
            stack,
            state,
            options,
        )
    })?;
    // Yield the function body onto the stack
    // => [Term]
    let block = block.append_inner(|stack| body.compile(stack, state, options))?;
    // Dispose of the lexical scopes created for the closed-over variables
    // => [Term]
    let block = (0..num_variable_scopes).fold(block, |block, _| {
        block.push(instruction::core::ScopeEnd {
            value_type: ValueType::HeapPointer,
        })
    });
    block.finish()
}

fn compile_generic_factory_branch<A: Arena + Clone>(
    factory: &WasmExpression<A>,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    let block = CompiledBlockBuilder::new(stack);
    // Yield the factory onto the stack
    // => [Term]
    let block = block.append_inner(|stack| factory.compile(stack, state, options))?;
    // Yield a zero-length argument list onto the stack
    // => [Term, ListTerm]
    let block = block.push(instruction::runtime::CallRuntimeBuiltin {
        target: RuntimeBuiltin::CreateEmptyList,
    });
    // Apply the factory, leaving the result on the stack
    // => [Term]
    let block = block.push(instruction::runtime::Apply);
    // Return the completed block
    block.finish()
}
