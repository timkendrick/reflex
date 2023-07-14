// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use debug_ignore::DebugIgnore;
use derivative::Derivative;
use reflex::{
    core::{ConditionType, Expression, ExpressionFactory, HeapAllocator, SignalType},
    hash::{HashId, IntMap},
};
use reflex_lang::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
use reflex_wasm::{
    allocator::{Arena, ArenaAllocator, VecAllocator},
    cli::compile::{
        compile_module, parse_inline_memory_snapshot, ModuleEntryPoint, WasmCompilerError,
        WasmCompilerOptions,
    },
    compiler::{
        error::TypedStackError, CompileWasm, CompilerOptions, CompilerStack, CompilerState,
        ParamsSignature, TypeSignature, TypedCompilerBlock, ValueType,
    },
    factory::WasmTermFactory,
    interpreter::{InterpreterError, WasmInterpreter, WasmProgram},
    stdlib::Stdlib,
    term_type::{
        condition::ConditionTerm, hashmap::HashmapTerm, lambda::LambdaTerm, signal::SignalTerm,
        tree::TreeTerm, TermType, TypedTerm, WasmExpression,
    },
    ArenaPointer, ArenaRef, Term,
};

use crate::WasmTestScenario;

const RUNTIME_BYTES: &[u8] = include_bytes!("../../build/runtime.wasm");

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
        runtime: Default::default(),
        generator: Default::default(),
    };
    let expression = scenario.input(&factory, &allocator);
    let state = scenario.state(&factory, &allocator);
    let expected = scenario.expected(&factory, &allocator);
    match validate_bytecode(
        &expression,
        &factory,
        CompilerStack::default(),
        &compiler_options.compiler,
    ) {
        Err(err) => panic!("{}", err),
        Ok(stack) => assert_eq!(
            ParamsSignature::from_iter(stack.operands()),
            ParamsSignature::Single(ValueType::HeapPointer),
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
                            let signal_type = match dependency.signal_type() {
                                SignalType::Custom {
                                    effect_type,
                                    payload,
                                    token,
                                } => {
                                    let effect_type =
                                        wasm_factory.import(&effect_type, &factory)?;
                                    let payload = wasm_factory.import(&payload, &factory)?;
                                    let token = wasm_factory.import(&token, &factory)?;
                                    SignalType::Custom {
                                        effect_type,
                                        payload,
                                        token,
                                    }
                                }
                                SignalType::Pending => SignalType::Pending,
                                SignalType::Error { payload } => {
                                    let payload = wasm_factory.import(&payload, &factory)?;
                                    SignalType::Error { payload }
                                }
                            };
                            wasm_factory.create_signal(signal_type)
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

#[derive(Derivative)]
#[derivative(Default(bound = ""), Debug(bound = ""), Clone(bound = ""))]
pub struct WasmDependencyList<A: Arena + Clone> {
    dependencies: IntMap<HashId, WasmExpression<A>>,
}

impl<A: Arena + Clone> WasmDependencyList<A> {
    fn iter(&self) -> std::collections::hash_map::Values<'_, HashId, WasmExpression<A>> {
        self.dependencies.values()
    }
}

impl<A: Arena + Clone> IntoIterator for WasmDependencyList<A> {
    type Item = WasmExpression<A>;
    type IntoIter = std::collections::hash_map::IntoValues<HashId, WasmExpression<A>>;
    fn into_iter(self) -> Self::IntoIter {
        self.dependencies.into_values()
    }
}

impl<A: Arena + Clone> FromIterator<WasmExpression<A>> for WasmDependencyList<A> {
    fn from_iter<T: IntoIterator<Item = WasmExpression<A>>>(iter: T) -> Self {
        Self {
            dependencies: iter
                .into_iter()
                .filter_map(|expression| {
                    expression
                        .as_condition_term()
                        .map(|condition| (condition.id()))
                        .map(|state_token| (state_token, expression))
                })
                .collect(),
        }
    }
}

impl<A: Arena + Clone> PartialEq for WasmDependencyList<A> {
    fn eq(&self, other: &Self) -> bool {
        self.dependencies.len() == other.dependencies.len()
            && self.dependencies.iter().all(|(key, value)| {
                other
                    .dependencies
                    .get(key)
                    .map(|other_value| value == other_value)
                    .unwrap_or(false)
            })
    }
}

impl<A: Arena + Clone> Eq for WasmDependencyList<A> {}

#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct WasmEvaluationResult<A: Arena + Clone> {
    pub result: WasmExpression<A>,
    pub dependencies: WasmDependencyList<A>,
}

impl<A: Arena + Clone> WasmEvaluationResult<A> {
    pub fn into_parts(self) -> (WasmExpression<A>, WasmDependencyList<A>) {
        let Self {
            result,
            dependencies,
        } = self;
        (result, dependencies)
    }
}

impl<A: Arena + Clone> std::fmt::Display for WasmEvaluationResult<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.result,
            self.dependencies
                .iter()
                .map(|term| format!("{}", term))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub enum CompilerTestError<T: Expression> {
    Allocator(T),
    Compiler(WasmCompilerError),
    Interpreter(InterpreterError, DebugIgnore<WasmProgram>),
}

impl<T: Expression> std::error::Error for CompilerTestError<T> {}

impl<T: Expression> std::fmt::Display for CompilerTestError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocator(term) => write!(f, "Failed to allocate expression: {term}"),
            Self::Compiler(err) => write!(f, "Failed to compile expression: {err}"),
            Self::Interpreter(err, _) => write!(f, "Failed to interpret expression: {err}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BytecodeValidationError<T: Expression> {
    Deserialize(T),
    Compiler(String),
    StackError(TypedStackError),
}

impl<T: Expression + std::fmt::Display> std::fmt::Display for BytecodeValidationError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deserialize(term) => write!(f, "Failed to deserialize expression: {term}"),
            Self::Compiler(err) => write!(f, "Compiler error: {err}"),
            Self::StackError(err) => write!(f, "Stack error: {err}"),
        }
    }
}

fn validate_bytecode<T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    stack: CompilerStack,
    compiler_options: &CompilerOptions,
) -> Result<CompilerStack, BytecodeValidationError<T>>
where
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
{
    let mut allocator = VecAllocator::default();
    let arena = Rc::new(RefCell::new(&mut allocator));
    let wasm_factory = WasmTermFactory::from(Rc::clone(&arena));
    let expression = wasm_factory
        .import(expression, factory)
        .map_err(BytecodeValidationError::Deserialize)?;
    let mut compiler_state = CompilerState::from_heap_snapshot::<Term>(arena.borrow().as_bytes());
    let block_stack = stack
        .enter_block(&TypeSignature {
            params: ParamsSignature::Void,
            results: ParamsSignature::Single(ValueType::HeapPointer),
        })
        .map_err(BytecodeValidationError::StackError)?;
    let compiled_expression = expression
        .compile(block_stack.clone(), &mut compiler_state, compiler_options)
        .map_err(|err| format!("{}", err))
        .map_err(BytecodeValidationError::Compiler)?;
    let result_stack = compiled_expression
        .get_type(&block_stack)
        .map_err(BytecodeValidationError::StackError)?;
    Ok(result_stack)
}

fn evaluate_compiled<T: Expression>(
    expression: T,
    state: impl IntoIterator<Item = (T::Signal, T)>,
    factory: &impl ExpressionFactory<T>,
    compiler_options: &WasmCompilerOptions,
) -> Result<WasmEvaluationResult<Rc<RefCell<VecAllocator>>>, CompilerTestError<T>>
where
    T::Builtin: Into<Stdlib>,
{
    let export_name = "__root__";
    let initial_heap_snapshot = parse_inline_memory_snapshot(RUNTIME_BYTES).unwrap();
    let mut allocator = VecAllocator::from_bytes(&initial_heap_snapshot);
    let mut arena = Rc::new(RefCell::new(&mut allocator));
    let wasm_factory = WasmTermFactory::from(Rc::clone(&arena));
    let wasm_expression = wasm_factory
        .import(&expression, factory)
        .map_err(CompilerTestError::Allocator)?;
    let entry_point = {
        let term = Term::new(
            TermType::Lambda(LambdaTerm {
                num_args: 0,
                body: wasm_expression.as_pointer(),
            }),
            &arena,
        );
        let pointer = arena
            .deref()
            .borrow_mut()
            .deref_mut()
            .deref_mut()
            .allocate(term);
        ArenaRef::<TypedTerm<LambdaTerm>, _>::new(Rc::clone(&arena), pointer)
    };
    let state_entries = state
        .into_iter()
        .map(|(key, value)| {
            let key = wasm_factory.import_condition(&key, factory)?;
            let value = wasm_factory.import(&value, factory)?;
            Ok((key.as_pointer(), value.as_pointer()))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(CompilerTestError::Allocator)?;
    let state = if state_entries.is_empty() {
        ArenaPointer::null()
    } else {
        HashmapTerm::allocate(state_entries, &mut arena)
    };
    let linear_memory = Vec::from(arena.deref().borrow().deref().deref().as_bytes());
    let wasm_module = compile_module(
        [(&ModuleEntryPoint::from(export_name), entry_point)],
        &RUNTIME_BYTES,
        Some(&linear_memory),
        compiler_options,
        true,
    )
    .map_err(CompilerTestError::Compiler)?;
    let wasm_program = WasmProgram::from_wasm(wasm_module);

    let (interpreter, result, dependencies) = WasmInterpreter::instantiate(&wasm_program, "memory")
        .and_then(|mut interpreter| {
            interpreter
                .call::<u32, (u32, u32)>(export_name, u32::from(state))
                .map(|(result, dependencies)| (interpreter, result, dependencies))
        })
        .map_err(|err| CompilerTestError::Interpreter(err, wasm_program.into()))?;
    let allocator = VecAllocator::from_bytes(&interpreter.dump_heap());
    let arena = Rc::new(RefCell::new(allocator));
    let dependencies = ArenaPointer::from(dependencies)
        .as_non_null()
        .map(|pointer| ArenaRef::<TypedTerm<TreeTerm>, _>::new(Rc::clone(&arena), pointer))
        .map(|dependency_tree| {
            dependency_tree
                .as_inner()
                .nodes()
                .collect::<WasmDependencyList<_>>()
        })
        .unwrap_or_default();
    let result = ArenaRef::<Term, _>::new(arena, ArenaPointer::from(result));
    Ok(WasmEvaluationResult {
        result,
        dependencies,
    })
}
