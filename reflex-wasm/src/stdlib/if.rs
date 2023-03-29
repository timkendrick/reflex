// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

use crate::{
    allocator::Arena,
    compiler::{
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompiledFunctionCall, CompilerOptions, CompilerResult, CompilerStack,
        CompilerState, ParamsSignature, TypeSignature, ValueType,
    },
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct If;
impl If {
    pub const UUID: Uuid = uuid!("9c8fc3a1-2d55-420e-bf81-3098932f8cf0");
    const ARITY: FunctionArity<3, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Lazy, ArgType::Lazy],
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
        // Invoke the runtime builtin to determine whether the condition is truthy
        // => [bool]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::IsTruthy,
        });
        // If the condition was truthy, evaluate the consequent, otherwise evaluate the alternative
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
                    // Yield the consequent onto the stack
                    // => [Term]
                    consequent.compile(consequent_stack, state, options)
                }?,
                alternative: {
                    // Yield the alternative onto the stack
                    // => [Term]
                    alternative.compile(alternative_stack, state, options)
                }?,
            });
            block.finish::<CompilerError<_>>()
        })?;
        block.finish()
    }
}
