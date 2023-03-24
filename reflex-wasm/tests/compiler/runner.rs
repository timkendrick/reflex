// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{cell::RefCell, rc::Rc};

use reflex::{
    core::{ConditionType, ExpressionFactory, HeapAllocator, RefType, SignalType},
    hash::IntMap,
};
use reflex_lang::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
use reflex_wasm::{
    allocator::{Arena, ArenaAllocator, VecAllocator},
    cli::compile::WasmCompilerOptions,
    compiler::{
        tests::{
            evaluate_compiled, validate_bytecode, CompilerTestError, WasmDependencyList,
            WasmEvaluationResult,
        },
        TypeSignature, TypedStackType, ValueType,
    },
    factory::WasmTermFactory,
    term_type::{ConditionTerm, SignalTerm, TypedTerm, WasmExpression},
    ArenaRef,
};

use crate::WasmTestScenario;

pub(crate) fn run_scenario(
    scenario: &impl WasmTestScenario<
        CachedSharedTerm<reflex_wasm::stdlib::Stdlib>,
        SharedTermFactory<reflex_wasm::stdlib::Stdlib>,
    >,
) -> Result<
    (
        WasmEvaluationResult<Rc<RefCell<VecAllocator>>>,
        WasmEvaluationResult<Rc<RefCell<VecAllocator>>>,
    ),
    CompilerTestError<CachedSharedTerm<reflex_wasm::stdlib::Stdlib>>,
> {
    let factory = SharedTermFactory::<reflex_wasm::stdlib::Stdlib>::default();
    let allocator = DefaultAllocator::<CachedSharedTerm<reflex_wasm::stdlib::Stdlib>>::default();
    let compiler_options = WasmCompilerOptions {
        compiler: scenario.options(),
        generator: Default::default(),
    };
    let expression = scenario.input(&factory, &allocator);
    let state = scenario.state(&factory, &allocator);
    let expected = scenario.expected(&factory, &allocator);
    match validate_bytecode(&expression, &factory, &compiler_options.compiler) {
        Err(err) => panic!("{}", err),
        Ok(block_type) => assert_eq!(
            block_type,
            TypedStackType::from(TypeSignature::new((), ValueType::HeapPointer)),
        ),
    };
    let actual = evaluate_compiled(expression, state, &factory, &compiler_options)?;
    let actual = {
        let (result, dependencies) = actual.into_parts();
        if let Some(signal) = result.as_signal_term() {
            WasmEvaluationResult {
                result: normalize_signal_term(signal),
                dependencies,
            }
        } else {
            WasmEvaluationResult {
                result,
                dependencies,
            }
        }
    };
    let expected = {
        let arena = actual.result.arena();
        let wasm_factory = WasmTermFactory::from(Rc::clone(arena));
        let (result, dependencies) = expected;
        wasm_factory
            .import(&result, &factory)
            .map_err(CompilerTestError::Allocator)
            .map(|result| WasmExpression::new(Rc::clone(arena), result.as_pointer()))
            .map(|result| {
                if let Some(signal) = result.as_signal_term() {
                    normalize_signal_term(signal)
                } else {
                    result
                }
            })
            .and_then(|result| {
                let dependencies = dependencies
                    .into_iter()
                    .map(|dependency| {
                        let condition = {
                            let dependency = dependency;
                            let effect_type = match dependency.signal_type() {
                                SignalType::Custom(effect_type) => wasm_factory
                                    .import(&effect_type, &factory)
                                    .map(SignalType::Custom),
                                SignalType::Pending => Ok(SignalType::Pending),
                                SignalType::Error => Ok(SignalType::Error),
                            }?;
                            let payload =
                                wasm_factory.import(dependency.payload().as_deref(), &factory)?;
                            let token =
                                wasm_factory.import(dependency.token().as_deref(), &factory)?;
                            wasm_factory.create_signal(effect_type, payload, token)
                        };
                        Ok(WasmExpression::new(
                            Rc::clone(arena),
                            condition.as_pointer(),
                        ))
                    })
                    .collect::<Result<WasmDependencyList<_>, _>>()
                    .map_err(CompilerTestError::Allocator)?;
                Ok(WasmEvaluationResult {
                    result,
                    dependencies,
                })
            })
    }?;
    Ok((actual, expected))
}

fn normalize_signal_term<A: Arena>(
    signal: &ArenaRef<TypedTerm<SignalTerm>, Rc<RefCell<A>>>,
) -> WasmExpression<Rc<RefCell<A>>>
where
    A: ArenaAllocator,
    Rc<RefCell<A>>: Arena,
{
    let arena = signal.arena();
    let wasm_factory = WasmTermFactory::from(Rc::clone(arena));
    let conditions = signal
        .as_inner()
        .conditions()
        .as_inner()
        .iter()
        .map(|pointer| {
            let condition =
                ArenaRef::<TypedTerm<ConditionTerm>, _>::new(wasm_factory.clone(), pointer);
            (condition.id(), condition)
        })
        .collect::<IntMap<_, _>>();
    // Sort the conditions by their hash ID
    let signal_list = wasm_factory.create_signal_list(conditions.into_values());
    let signal = wasm_factory.create_signal_term(signal_list);
    WasmExpression::new(Rc::clone(arena), signal.as_pointer())
}
