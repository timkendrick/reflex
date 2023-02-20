// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use metrics::histogram;
use reflex::{
    core::{
        DependencyList, EvaluationResult, Expression, ExpressionFactory, HeapAllocator,
        IntTermType, SignalType, StateToken,
    },
    hash::{HashId, IntMap},
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, MessageOffset, NoopDisposeCallback,
    ProcessId, SchedulerCommand, SchedulerMode, SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_macros::{dispatcher, Named};
use reflex_wasm::{
    allocator::{Arena, ArenaAllocator},
    factory::WasmTermFactory,
    interpreter::{InterpreterError, UnboundEvaluationResult, WasmInterpreter},
    term_type::{
        ApplicationTerm, HashmapTerm, IntTerm, ListTerm, TermType, TreeTerm, TypedTerm,
        WasmExpression,
    },
    utils::{from_twos_complement_i64, into_twos_complement_i64},
    ArenaPointer, ArenaRef, Term,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow, cell::RefCell, collections::hash_map::Entry, iter::once, marker::PhantomData,
    ops::Deref, rc::Rc, sync::Arc, time::Instant,
};

use crate::{
    action::bytecode_interpreter::{
        BytecodeInterpreterEvaluateAction, BytecodeInterpreterGcAction,
        BytecodeInterpreterInitAction, BytecodeInterpreterResultAction,
    },
    actor::wasm_interpreter::WasmProgram,
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator,
};
use crate::{
    action::bytecode_interpreter::{BytecodeInterpreterGcCompleteAction, BytecodeWorkerStatistics},
    QueryEvaluationMode,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmWorkerMetricNames {
    pub query_worker_compile_duration: Cow<'static, str>,
    pub query_worker_evaluate_duration: Cow<'static, str>,
    pub query_worker_gc_duration: Cow<'static, str>,
}

pub trait WasmWorkerTask<T, TFactory, TAllocator>:
    From<WasmWorkerTaskFactory<T, TFactory, TAllocator>>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
{
}
impl<_Self, T, TFactory, TAllocator> WasmWorkerTask<T, TFactory, TAllocator> for _Self
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
    Self: From<WasmWorkerTaskFactory<T, TFactory, TAllocator>>,
{
}

#[derive(Named, Clone, Serialize, Deserialize)]
pub struct WasmWorkerTaskFactory<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    pub cache_id: HashId,
    pub query: T,
    pub graph_root_factory_export_name: String,
    pub evaluation_mode: QueryEvaluationMode,
    pub graph_root: Arc<WasmProgram>,
    pub metric_names: WasmWorkerMetricNames,
    pub caller_pid: ProcessId,
    pub _expression: PhantomData<T>,
    pub _factory: PhantomData<TFactory>,
    pub _allocator: PhantomData<TAllocator>,
}

impl<T, TFactory, TAllocator, TAction, TTask> TaskFactory<TAction, TTask>
    for WasmWorkerTaskFactory<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TAction: Action + WasmWorkerAction<T> + Send + 'static,
    TTask: TaskFactory<TAction, TTask>,
{
    type Actor = WasmWorker<T, TFactory, TAllocator>;
    fn create(self) -> Self::Actor {
        let Self {
            cache_id,
            query,
            graph_root_factory_export_name,
            evaluation_mode,
            graph_root,
            metric_names,
            caller_pid,
            _expression,
            _factory,
            _allocator,
        } = self;
        let factory = TFactory::default();
        let allocator = TAllocator::default();
        WasmWorker {
            cache_id,
            query,
            graph_root_factory_export_name,
            evaluation_mode,
            program: graph_root,
            factory,
            allocator,
            metric_names,
            caller_pid,
        }
    }
}

#[derive(Named, Clone)]
pub struct WasmWorker<T, TFactory, TAllocator>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
{
    cache_id: HashId,
    query: T,
    graph_root_factory_export_name: String,
    evaluation_mode: QueryEvaluationMode,
    program: Arc<WasmProgram>,
    factory: TFactory,
    allocator: TAllocator,
    metric_names: WasmWorkerMetricNames,
    caller_pid: ProcessId,
}

pub enum WasmWorkerState<T: Expression> {
    Uninitialized,
    Initialized(WasmWorkerInitializedState<T>),
    Error(WasmWorkerError<T>),
}

impl<T: Expression> Default for WasmWorkerState<T> {
    fn default() -> Self {
        Self::Uninitialized
    }
}

pub struct WasmWorkerInitializedState<T: Expression> {
    instance: WasmInterpreter,
    entry_point: ArenaPointer,
    state_index: Option<MessageOffset>,
    state_values: IntMap<StateToken, (ArenaPointer, ArenaPointer)>,
    latest_result: Option<EvaluationResult<T>>,
}

impl<T: Expression> WasmWorkerInitializedState<T> {
    fn get_statistics(&self) -> BytecodeWorkerStatistics {
        BytecodeWorkerStatistics {
            state_dependency_count: self
                .latest_result
                .as_ref()
                .map(|result| result.dependencies().len())
                .unwrap_or(0),
            evaluation_cache_entry_count: 0,
            evaluation_cache_deep_size: 0,
        }
    }
}

dispatcher!({
    pub enum WasmWorkerAction<T: Expression> {
        Inbox(BytecodeInterpreterInitAction),
        Inbox(BytecodeInterpreterEvaluateAction<T>),
        Inbox(BytecodeInterpreterGcAction),

        Outbox(BytecodeInterpreterResultAction<T>),
        Outbox(BytecodeInterpreterGcCompleteAction),
    }

    impl<T, TFactory, TAllocator, TAction, TTask> Dispatcher<TAction, TTask>
        for WasmWorker<T, TFactory, TAllocator>
    where
        T: Expression,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        type State = WasmWorkerState<T>;
        type Events<TInbox: TaskInbox<TAction>> = TInbox;
        type Dispose = NoopDisposeCallback;

        fn init(&self) -> Self::State {
            Default::default()
        }
        fn events<TInbox: TaskInbox<TAction>>(
            &self,
            inbox: TInbox,
        ) -> ActorEvents<TInbox, Self::Events<TInbox>, Self::Dispose> {
            ActorEvents::Sync(inbox)
        }

        fn accept(&self, _action: &BytecodeInterpreterInitAction) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterInitAction,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterInitAction,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_bytecode_interpreter_init(state, action, metadata, context)
        }

        fn accept(&self, _action: &BytecodeInterpreterEvaluateAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterEvaluateAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Blocking)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterEvaluateAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_bytecode_interpreter_evaluate(state, action, metadata, context)
        }

        fn accept(&self, _action: &BytecodeInterpreterGcAction) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterGcAction,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Blocking)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterGcAction,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_bytecode_interpreter_gc(state, action, metadata, context)
        }
    }
});

#[derive(Debug)]
pub enum WasmWorkerError<T: Expression> {
    Unititialized,
    InterpreterError(InterpreterError),
    SerializationError(T),
}

enum MaybeOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}
impl<'a, T: 'a> Deref for MaybeOwned<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(value) => &value,
            Self::Borrowed(value) => value,
        }
    }
}

impl<T, TFactory, TAllocator> WasmWorker<T, TFactory, TAllocator>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
{
    fn handle_bytecode_interpreter_init<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterInitAction,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<BytecodeInterpreterResultAction<T>>
            + From<BytecodeInterpreterGcCompleteAction>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterInitAction { cache_id: _ } = action;
        match state {
            WasmWorkerState::Uninitialized => {
                *state = match {
                    let compiler_start_time = Instant::now();
                    self.program
                        .instantiate("memory")
                        .map_err(WasmWorkerError::InterpreterError)
                        .and_then(|mut instance| {
                            let graph_root = instance
                                .call::<(), u32>(&self.graph_root_factory_export_name, ())
                                .map_err(WasmWorkerError::InterpreterError)?;
                            let mut wasm_factory =
                                WasmTermFactory::from(Rc::new(RefCell::new(&mut instance)));
                            let query = match self.evaluation_mode {
                                QueryEvaluationMode::Query => compile_graphql_query(
                                    ArenaPointer::from(graph_root),
                                    &self.query,
                                    &self.factory,
                                    &mut wasm_factory,
                                ),
                                QueryEvaluationMode::Standalone => todo!(),
                            };
                            let elapsed_time = compiler_start_time.elapsed();
                            {
                                match &self.metric_names.query_worker_compile_duration {
                                    Cow::Borrowed(metric_name) => {
                                        histogram!(*metric_name, elapsed_time.as_secs_f64())
                                    }
                                    Cow::Owned(metric_name) => {
                                        histogram!(metric_name.clone(), elapsed_time.as_secs_f64())
                                    }
                                }
                            }
                            query
                                .map_err(WasmWorkerError::SerializationError)
                                .map(|query| (instance, query))
                        })
                } {
                    Ok((instance, entry_point)) => {
                        WasmWorkerState::Initialized(WasmWorkerInitializedState {
                            instance,
                            entry_point,
                            state_index: Default::default(),
                            state_values: Default::default(),
                            latest_result: Default::default(),
                        })
                    }
                    Err(err) => WasmWorkerState::Error(err),
                };
                None
            }
            WasmWorkerState::Error(_) | WasmWorkerState::Initialized(_) => None,
        }
    }
    fn handle_bytecode_interpreter_evaluate<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterEvaluateAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<BytecodeInterpreterResultAction<T>>
            + From<BytecodeInterpreterGcCompleteAction>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterEvaluateAction {
            cache_id: _,
            state_index,
            state_updates,
        } = action;
        let result = match state {
            WasmWorkerState::Uninitialized => {
                Err(MaybeOwned::Owned(WasmWorkerError::Unititialized))
            }
            WasmWorkerState::Error(err) => Err(MaybeOwned::Borrowed(&*err)),
            WasmWorkerState::Initialized(state) => {
                let state_index = *state_index;
                state.state_index = state_index;
                let state_update_status =
                    state_updates
                        .iter()
                        .fold(Ok(()), |result, (state_token, value)| {
                            let _ = result?;
                            let mut wasm_factory =
                                WasmTermFactory::from(Rc::new(RefCell::new(&mut state.instance)));
                            let value_pointer =
                                import_wasm_expression(value, &self.factory, &mut wasm_factory)
                                    .map_err(WasmWorkerError::SerializationError)?;
                            match state.state_values.entry(*state_token) {
                                Entry::Occupied(mut entry) => {
                                    let (_key, state_value) = entry.get_mut();
                                    *state_value = value_pointer;
                                }
                                Entry::Vacant(entry) => {
                                    let key_pointer = {
                                        let term = Term::new(
                                            TermType::Int(IntTerm::from(from_twos_complement_i64(
                                                *state_token,
                                            ))),
                                            &state.instance,
                                        );
                                        state.instance.allocate(term)
                                    };
                                    entry.insert((key_pointer, value_pointer));
                                }
                            }
                            Ok(())
                        });
                match state_update_status {
                    Err(err) => Err(MaybeOwned::Owned(err)),
                    Ok(_) => {
                        let runtime_state = HashmapTerm::allocate(
                            state
                                .state_values
                                .values()
                                .map(|(key, value)| (*key, *value)),
                            &mut state.instance,
                        );
                        let start_time = Instant::now();
                        let result = state.instance.interpret(state.entry_point, runtime_state);
                        let elapsed_time = start_time.elapsed();
                        match &self.metric_names.query_worker_evaluate_duration {
                            Cow::Borrowed(metric_name) => {
                                histogram!(*metric_name, elapsed_time.as_secs_f64())
                            }
                            Cow::Owned(metric_name) => {
                                histogram!(metric_name.clone(), elapsed_time.as_secs_f64())
                            }
                        }
                        match result {
                            Ok(UnboundEvaluationResult {
                                result_pointer,
                                dependencies_pointer,
                            }) => {
                                let result = {
                                    let arena = Rc::new(RefCell::new(&mut state.instance));
                                    let result = {
                                        let value = ArenaRef::<Term, _>::new(
                                            Rc::clone(&arena),
                                            result_pointer,
                                        );
                                        let dependencies = match dependencies_pointer {
                                            None => None,
                                            Some(pointer) => {
                                                Some(ArenaRef::<TypedTerm<TreeTerm>, _>::new(
                                                    Rc::clone(&arena),
                                                    pointer,
                                                ))
                                            }
                                        };
                                        parse_wasm_interpreter_result(
                                            &value,
                                            dependencies.as_ref(),
                                            &self.factory,
                                            &self.allocator,
                                            &arena,
                                        )
                                    };
                                    result
                                };
                                state.latest_result = Some(result.clone());
                                Ok((result, state.get_statistics()))
                            }
                            Err(err) => {
                                Err(MaybeOwned::Owned(WasmWorkerError::InterpreterError(err)))
                            }
                        }
                    }
                }
            }
        };
        match result {
            Ok((result, statistics)) => {
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterResultAction {
                        cache_id: self.cache_id,
                        state_index: *state_index,
                        result,
                        statistics,
                    }
                    .into(),
                ))))
            }
            Err(err) => {
                let message = match err.deref() {
                    WasmWorkerError::Unititialized => {
                        String::from("WebAssembly module not initialized")
                    }
                    WasmWorkerError::InterpreterError(err) => {
                        format!("WebAssembly interpreter error: {}", err)
                    }
                    WasmWorkerError::SerializationError(term) => format!(
                        "WebAssembly serialization error: unable to serialize term: {}",
                        term
                    ),
                };
                let result = EvaluationResult::new(
                    create_error_expression(message, &self.factory, &self.allocator),
                    Default::default(),
                );
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterResultAction {
                        cache_id: self.cache_id,
                        state_index: *state_index,
                        result,
                        statistics: Default::default(),
                    }
                    .into(),
                ))))
            }
        }
    }
    fn handle_bytecode_interpreter_gc<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterGcAction,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<BytecodeInterpreterGcCompleteAction>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterGcAction {
            cache_id,
            state_index,
        } = action;
        match state {
            WasmWorkerState::Uninitialized | WasmWorkerState::Error(_) => None,
            WasmWorkerState::Initialized(state) => {
                let latest_state_index = &state.state_index;
                if state_index < latest_state_index {
                    return None;
                }
                let start_time = Instant::now();
                // FIXME: perform GC on WASM VM heap
                let empty_dependencies = DependencyList::default();
                let retained_keys = state
                    .latest_result
                    .as_ref()
                    .map(|result| result.dependencies())
                    .unwrap_or(&empty_dependencies);
                if retained_keys.len() < state.state_values.len() {
                    // FIXME: perform GC on state value cache
                }
                let elapsed_time = start_time.elapsed();
                match &self.metric_names.query_worker_gc_duration {
                    Cow::Borrowed(metric_name) => {
                        histogram!(*metric_name, elapsed_time.as_secs_f64())
                    }
                    Cow::Owned(metric_name) => {
                        histogram!(metric_name.clone(), elapsed_time.as_secs_f64())
                    }
                }
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterGcCompleteAction {
                        cache_id: *cache_id,
                        statistics: state.get_statistics(),
                    }
                    .into(),
                ))))
            }
        }
    }
}

fn parse_wasm_interpreter_result<'heap, T: Expression>(
    result: &WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>,
    dependencies: Option<&ArenaRef<TypedTerm<TreeTerm>, Rc<RefCell<&'heap mut WasmInterpreter>>>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
) -> EvaluationResult<T> {
    EvaluationResult::new(
        export_wasm_expression(result, factory, allocator, arena).unwrap_or_else(|term| {
            create_error_expression(
                format!("Unable to serialize term: {}", term),
                factory,
                allocator,
            )
        }),
        dependencies
            .map(|dependencies| {
                parse_wasm_interpreter_result_dependencies(&dependencies.as_inner())
            })
            .unwrap_or_default(),
    )
}

fn parse_wasm_interpreter_result_dependencies<A: Arena + Clone>(
    dependencies: &ArenaRef<TreeTerm, A>,
) -> DependencyList {
    DependencyList::from_iter(dependencies.nodes().filter_map(|dependency| {
        dependency
            .as_int_term()
            .map(|term| into_twos_complement_i64(term.value()))
    }))
}

fn create_error_expression<T: Expression>(
    message: String,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_signal_term(allocator.create_signal_list(once(allocator.create_signal(
        SignalType::Error,
        factory.create_string_term(allocator.create_string(message)),
        factory.create_nil_term(),
    ))))
}

fn compile_graphql_query<'heap, T: Expression>(
    graph_root: ArenaPointer,
    query: &T,
    factory: &impl ExpressionFactory<T>,
    arena: &mut WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T> {
    let compiled_query_pointer = compile_wasm_expression(query, factory, arena)?;
    let term = Term::new(
        TermType::Application(ApplicationTerm {
            target: compiled_query_pointer,
            args: ListTerm::allocate([graph_root], arena),
            cache: Default::default(),
        }),
        &*arena,
    );
    Ok(arena.allocate(term))
}

fn compile_wasm_expression<'heap, T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &mut WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T> {
    import_wasm_expression(expression, factory, wasm_factory)
}

fn import_wasm_expression<'heap, T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &mut WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T> {
    let term = wasm_factory.import(expression, factory)?;
    Ok(term.as_pointer())
}

fn export_wasm_expression<'heap, T: Expression>(
    expression: &WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
) -> Result<T, WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>> {
    let wasm_factory = WasmTermFactory::from(Rc::clone(arena));
    wasm_factory.export(expression, factory, allocator)
}
