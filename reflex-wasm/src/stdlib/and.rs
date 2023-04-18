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

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct And;
impl And {
    pub const UUID: Uuid = uuid!("223539c0-3858-4257-a53d-55fa93e2e7ba");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Lazy],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for And {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}

impl<'a, A: Arena + Clone> CompileWasm<A> for CompiledFunctionCall<'a, A, And> {
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
        let (condition, consequent) = {
            let mut arg_list = args.iter();
            let condition = arg_list.next();
            let consequent = arg_list.next();
            match (condition, consequent) {
                (Some(condition), Some(consequent)) => Ok((condition, consequent)),
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
        // Pop the condition term off the stack and store in a temporary lexical scope
        // => []
        let block = block.push(instruction::core::ScopeStart {
            value_type: ValueType::HeapPointer,
        });
        // Push a copy of the condition term onto the stack (for truthiness-testing)
        // => [Term]
        let block = block.push(instruction::core::GetScopeValue {
            scope_offset: 0,
            value_type: ValueType::HeapPointer,
        });
        // Invoke the runtime builtin to determine whether the condition is truthy
        // => [bool]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::IsTruthy,
        });
        // If the condition was truthy, evaluate the consequent, otherwise return the falsy condition
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
                    let block = CompiledBlockBuilder::new(alternative_stack);
                    // Push the stored result onto the stack
                    // => [Term]
                    let block = block.push(instruction::core::GetScopeValue {
                        value_type: ValueType::HeapPointer,
                        scope_offset: 0,
                    });
                    block.finish::<CompilerError<_>>()
                }?,
            });
            block.finish::<CompilerError<_>>()
        })?;
        // Drop the temporary lexical scope
        // => [Term]
        let block = block.push(instruction::core::ScopeEnd {
            value_type: ValueType::HeapPointer,
        });
        block.finish()
    }
}
