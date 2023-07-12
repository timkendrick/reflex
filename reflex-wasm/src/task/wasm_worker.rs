// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::ArenaAllocator,
    cli::compile::WasmProgram,
    factory::WasmTermFactory,
    interpreter::{InterpreterError, UnboundEvaluationResult, WasmInterpreter},
    term_type::{
        symbol::SymbolTerm, ApplicationTerm, ConditionTerm, HashmapTerm, ListTerm, TermType,
        TreeTerm, TypedTerm, WasmExpression,
    },
    ArenaPointer, ArenaRef, Term,
};
use metrics::histogram;
use reflex::{
    core::{
        ConditionListType, ConditionType, DependencyList, EvaluationResult, Expression,
        ExpressionFactory, HeapAllocator, RefType, SignalTermType, SignalType, StateToken,
    },
    hash::IntMap,
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, MessageOffset, NoopDisposeCallback,
    ProcessId, SchedulerCommand, SchedulerMode, SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_engine::task::bytecode_worker::BytecodeWorkerAction;
use reflex_macros::{blanket_trait, dispatcher, Named};
use reflex_runtime::{
    action::bytecode_interpreter::{
        BytecodeInterpreterEvaluateAction, BytecodeInterpreterGcAction,
        BytecodeInterpreterInitAction, BytecodeInterpreterResultAction,
    },
    action::bytecode_interpreter::{BytecodeInterpreterGcCompleteAction, BytecodeWorkerStatistics},
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator, QueryEvaluationMode,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow, cell::RefCell, collections::hash_map::Entry, iter::once, marker::PhantomData,
    ops::Deref, rc::Rc, sync::Arc, time::Instant,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmWorkerMetricNames {
    pub query_worker_compile_duration: Cow<'static, str>,
    pub query_worker_evaluate_duration: Cow<'static, str>,
    pub query_worker_gc_duration: Cow<'static, str>,
}

blanket_trait!(
    pub trait WasmWorkerTaskAction<T: Expression>: BytecodeWorkerAction<T> {}
);

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
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct WasmWorkerTaskFactory<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    pub cache_key: T::Signal,
    pub query: T,
    pub graph_root_factory_export_name: String,
    pub evaluation_mode: QueryEvaluationMode,
    pub wasm_module: Arc<WasmProgram>,
    pub metric_names: WasmWorkerMetricNames,
    pub caller_pid: ProcessId,
    pub dump_query_errors: bool,
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
    T::Builtin: Into<crate::stdlib::Stdlib>,
    TAction: Action + WasmWorkerAction<T> + Send + 'static,
    TTask: TaskFactory<TAction, TTask>,
{
    type Actor = WasmWorker<T, TFactory, TAllocator>;
    fn create(self) -> Self::Actor {
        let Self {
            cache_key,
            query,
            graph_root_factory_export_name,
            evaluation_mode,
            wasm_module,
            metric_names,
            caller_pid,
            dump_query_errors,
            _expression,
            _factory,
            _allocator,
        } = self;
        let factory = TFactory::default();
        let allocator = TAllocator::default();
        WasmWorker {
            cache_key,
            query,
            graph_root_factory_export_name,
            evaluation_mode,
            wasm_module,
            factory,
            allocator,
            metric_names,
            caller_pid,
            dump_query_errors,
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
    cache_key: T::Signal,
    query: T,
    graph_root_factory_export_name: String,
    evaluation_mode: QueryEvaluationMode,
    wasm_module: Arc<WasmProgram>,
    factory: TFactory,
    allocator: TAllocator,
    metric_names: WasmWorkerMetricNames,
    caller_pid: ProcessId,
    dump_query_errors: bool,
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
        Inbox(BytecodeInterpreterInitAction<T>),
        Inbox(BytecodeInterpreterEvaluateAction<T>),
        Inbox(BytecodeInterpreterGcAction<T>),

        Outbox(BytecodeInterpreterResultAction<T>),
        Outbox(BytecodeInterpreterGcCompleteAction<T>),
    }

    impl<T, TFactory, TAllocator, TAction, TTask> Dispatcher<TAction, TTask>
        for WasmWorker<T, TFactory, TAllocator>
    where
        T: Expression,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
        T::Builtin: Into<crate::stdlib::Stdlib>,
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

        fn accept(&self, _action: &BytecodeInterpreterInitAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterInitAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterInitAction<T>,
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

        fn accept(&self, _action: &BytecodeInterpreterGcAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterGcAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Blocking)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterGcAction<T>,
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
    InvalidGraphDefinition,
    InterpreterError(InterpreterError),
    SerializationError(T),
}

impl<T: Expression + std::fmt::Display> std::fmt::Display for WasmWorkerError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unititialized => write!(f, "WebAssembly module not initialized"),
            Self::InvalidGraphDefinition => write!(f, "Invalid graph definition"),
            Self::InterpreterError(err) => {
                write!(f, "WebAssembly interpreter error: {}", err)
            }
            Self::SerializationError(term) => write!(
                f,
                "WebAssembly serialization error: unable to serialize term: {}",
                term
            ),
        }
    }
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
    T::Builtin: Into<crate::stdlib::Stdlib>,
{
    fn handle_bytecode_interpreter_init<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterInitAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<BytecodeInterpreterResultAction<T>>
            + From<BytecodeInterpreterGcCompleteAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterInitAction { cache_key } = action;
        if cache_key.id() != self.cache_key.id() {
            return None;
        }
        match state {
            WasmWorkerState::Uninitialized => {
                *state = match {
                    let compiler_start_time = Instant::now();
                    WasmInterpreter::instantiate(&self.wasm_module, "memory")
                        .map_err(WasmWorkerError::InterpreterError)
                        .and_then(|mut instance| {
                            let graph_root = instance
                                .call::<u32, (u32, u32)>(
                                    &self.graph_root_factory_export_name,
                                    u32::from(ArenaPointer::null()),
                                )
                                .map_err(WasmWorkerError::InterpreterError)
                                .and_then(|(graph_root, dependencies)| {
                                    if ArenaPointer::from(dependencies).is_null() {
                                        Ok(ArenaPointer::from(graph_root))
                                    } else {
                                        Err(WasmWorkerError::InvalidGraphDefinition)
                                    }
                                })?;
                            let mut wasm_factory =
                                WasmTermFactory::from(Rc::new(RefCell::new(&mut instance)));
                            let query = match self.evaluation_mode {
                                QueryEvaluationMode::Query => compile_graphql_query(
                                    graph_root,
                                    &self.query,
                                    &self.factory,
                                    &mut wasm_factory,
                                ),
                                QueryEvaluationMode::Standalone => compile_wasm_expression(
                                    &self.query,
                                    &self.factory,
                                    &mut wasm_factory,
                                ),
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
            + From<BytecodeInterpreterGcCompleteAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterEvaluateAction {
            cache_key,
            state_index,
            state_updates,
        } = action;
        if cache_key.id() != self.cache_key.id() {
            return None;
        }
        let result = match state {
            WasmWorkerState::Uninitialized => {
                Err(MaybeOwned::Owned(WasmWorkerError::Unititialized))
            }
            WasmWorkerState::Error(err) => Err(MaybeOwned::Borrowed(&*err)),
            WasmWorkerState::Initialized(state) => {
                let state_index = *state_index;
                state.state_index = state_index;
                let state_update_status =
                    state_updates.iter().fold(Ok(()), |result, (key, value)| {
                        let _ = result?;
                        let wasm_factory =
                            WasmTermFactory::from(Rc::new(RefCell::new(&mut state.instance)));
                        let state_token = key.id();
                        let key_pointer = import_wasm_condition(key, &self.factory, &wasm_factory)
                            .map_err(WasmWorkerError::SerializationError)?;
                        let value_pointer =
                            import_wasm_expression(value, &self.factory, &wasm_factory)
                                .map_err(WasmWorkerError::SerializationError)?;
                        match state.state_values.entry(state_token) {
                            Entry::Occupied(mut entry) => {
                                let (_key, state_value) = entry.get_mut();
                                *state_value = value_pointer;
                            }
                            Entry::Vacant(entry) => {
                                entry.insert((key_pointer, value_pointer));
                            }
                        }
                        Ok(())
                    });
                match state_update_status {
                    Err(err) => Err(MaybeOwned::Owned(err)),
                    Ok(_) => {
                        let runtime_state = if state.state_values.is_empty() {
                            ArenaPointer::null()
                        } else {
                            HashmapTerm::allocate(
                                state
                                    .state_values
                                    .values()
                                    .map(|(key, value)| (*key, *value)),
                                &mut state.instance,
                            )
                        };
                        let initial_heap_snapshot = match self.dump_query_errors {
                            true => Some(state.instance.dump_heap()),
                            false => None,
                        }
                        .unwrap_or_default();
                        let start_time = Instant::now();
                        let result = state.instance.evaluate(state.entry_point, runtime_state);
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
                                    if self.dump_query_errors {
                                        if let Some(signal) = self
                                            .factory
                                            .match_signal_term(result.result())
                                            .filter(|signal| {
                                                signal.signals().as_deref().iter().any(|effect| {
                                                    matches!(
                                                        effect.as_deref().signal_type(),
                                                        SignalType::Error
                                                    )
                                                })
                                            })
                                        {
                                            println!("Query error: {result_pointer:?}");
                                            for err in
                                                signal.signals().as_deref().iter().filter_map(
                                                    |effect| {
                                                        let effect = effect.as_deref();
                                                        match effect.signal_type() {
                                                            SignalType::Error => Some(
                                                                effect.payload().as_deref().clone(),
                                                            ),
                                                            _ => None,
                                                        }
                                                    },
                                                )
                                            {
                                                println!(" {err}");
                                            }
                                            if let Some(pointer) = dependencies_pointer {
                                                println!(" Dependencies: {dependencies_pointer:?}");
                                                let dependencies =
                                                    ArenaRef::<TypedTerm<TreeTerm>, _>::new(
                                                        Rc::clone(&arena),
                                                        pointer,
                                                    );
                                                for condition in dependencies.as_inner().nodes() {
                                                    let condition_pointer = condition.as_pointer();
                                                    println!(
                                                        "  {condition_pointer:?}: {condition}"
                                                    );
                                                }
                                            }
                                            let output_filename = format!(
                                                "{}_{}_{}_{}.bin",
                                                cache_key.id(),
                                                state_index.map(usize::from).unwrap_or(0),
                                                u32::from(state.entry_point),
                                                u32::from(runtime_state)
                                            );
                                            println!("Dumping heap to {output_filename}...");
                                            std::fs::write(output_filename, initial_heap_snapshot)
                                                .expect("Failed to dump heap");
                                            println!("Heap dump complete");
                                            println!(
                                                "Invoking function evaluate({}, {})",
                                                u32::from(state.entry_point),
                                                u32::from(runtime_state)
                                            );
                                        }
                                    }
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
                        cache_key: cache_key.clone(),
                        state_index: *state_index,
                        result,
                        statistics,
                    }
                    .into(),
                ))))
            }
            Err(err) => {
                let message = format!("{}", err.deref());
                let result = EvaluationResult::new(
                    create_error_expression(message, &self.factory, &self.allocator),
                    Default::default(),
                );
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterResultAction {
                        cache_key: cache_key.clone(),
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
        action: &BytecodeInterpreterGcAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<BytecodeInterpreterGcCompleteAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterGcAction {
            cache_key,
            state_index,
        } = action;
        if cache_key.id() != self.cache_key.id() {
            return None;
        }
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
                        cache_key: cache_key.clone(),
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
    let (result, dependencies) = export_wasm_expression(result, factory, allocator, arena)
        .and_then(|result| {
            let dependencies = dependencies
                .map(|dependencies| {
                    parse_wasm_interpreter_result_dependencies(
                        dependencies,
                        factory,
                        allocator,
                        arena,
                    )
                })
                .transpose()?;
            Ok((result, dependencies))
        })
        .unwrap_or_else(|term| {
            (
                create_error_expression(
                    if let Some(condition) = term.as_condition_term() {
                        format!("{}", condition)
                    } else {
                        format!("Unable to translate evaluation result: {}", term)
                    },
                    factory,
                    allocator,
                ),
                None,
            )
        });
    EvaluationResult::new(result, dependencies.unwrap_or_default())
}

fn parse_wasm_interpreter_result_dependencies<'heap, T: Expression>(
    dependencies: &ArenaRef<TypedTerm<TreeTerm>, Rc<RefCell<&'heap mut WasmInterpreter>>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
) -> Result<DependencyList, WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>> {
    dependencies
        .as_inner()
        .nodes()
        .map(|dependency| match dependency.as_condition_term() {
            None => Err(dependency),
            Some(condition) => export_wasm_condition(condition, factory, allocator, arena)
                .map(|effect| effect.id()),
        })
        .collect::<Result<DependencyList, _>>()
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
    graph_root_factory: ArenaPointer,
    query: &T,
    factory: &impl ExpressionFactory<T>,
    arena: &mut WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<crate::stdlib::Stdlib>,
{
    let compiled_query_function = compile_wasm_expression(query, factory, arena)?;
    let graph_root = {
        // Graph root factory evaluates to a 1-argument function that takes the query token as an argument
        let query_token = arena.allocate(Term::new(
            TermType::Symbol(SymbolTerm {
                id: (query.id() & 0x00000000FFFFFFFF) as u32,
            }),
            arena,
        ));
        let factory_args = ListTerm::allocate([query_token], arena);
        let factory_call = arena.allocate(Term::new(
            TermType::Application(ApplicationTerm {
                target: graph_root_factory,
                args: factory_args,
                cache: Default::default(),
            }),
            arena,
        ));
        factory_call
    };
    let query = {
        // Create an expression that applies the query function to the graph root
        let query_term = Term::new(
            TermType::Application(ApplicationTerm {
                target: compiled_query_function,
                args: ListTerm::allocate([graph_root], arena),
                cache: Default::default(),
            }),
            &*arena,
        );
        arena.allocate(query_term)
    };
    Ok(query)
}

fn compile_wasm_expression<'heap, T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<crate::stdlib::Stdlib>,
{
    import_wasm_expression(expression, factory, wasm_factory)
}

fn import_wasm_expression<'heap, T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<crate::stdlib::Stdlib>,
{
    let term = wasm_factory.import(expression, factory)?;
    Ok(term.as_pointer())
}

fn import_wasm_condition<'heap, T: Expression>(
    condition: &T::Signal,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<crate::stdlib::Stdlib>,
{
    let term = wasm_factory.import_condition(condition, factory)?;
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

fn export_wasm_condition<'heap, T: Expression>(
    condition: &ArenaRef<TypedTerm<ConditionTerm>, Rc<RefCell<&'heap mut WasmInterpreter>>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
) -> Result<T::Signal, WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>> {
    let wasm_factory = WasmTermFactory::from(Rc::clone(arena));
    wasm_factory.export_condition(condition, factory, allocator)
}
