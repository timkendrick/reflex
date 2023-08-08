// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    iter::{empty, once},
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
    pin::Pin,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use futures::{Future, Stream, StreamExt};
use metrics::SharedString;
use pin_project::pin_project;
use reflex::{
    cache::SubstitutionCache,
    core::{
        Applicable, ArgType, ConditionType, Expression, ExpressionFactory, HeapAllocator,
        Reducible, Rewritable, StateCache,
    },
};
use reflex_cli::{builtins::CliBuiltins, format_signal_result, repl};
use reflex_dispatcher::{
    Action, Actor, ActorEvents, AsyncScheduler, Handler, HandlerContext, Matcher, MessageData,
    Named, ProcessId, Redispatcher, SchedulerMode, SchedulerTransition, SerializableAction,
    SerializedAction, TaskFactory, TaskInbox, Worker,
};
use reflex_engine::{
    actor::{
        bytecode_interpreter::BytecodeInterpreterMetricLabels,
        wasm_interpreter::{WasmInterpreter, WasmInterpreterAction, WasmInterpreterMetricNames},
    },
    task::wasm_worker::{
        WasmHeapDumpMode, WasmWorkerAction, WasmWorkerTask, WasmWorkerTaskFactory,
    },
};
use reflex_grpc::{
    action::*,
    actor::{GrpcHandler, GrpcHandlerAction, GrpcHandlerMetricNames},
    load_grpc_services,
    task::{GrpcHandlerConnectionTaskAction, GrpcHandlerConnectionTaskFactory, GrpcHandlerTask},
    DefaultGrpcConfig, GrpcConfig,
};
use reflex_handlers::{
    action::{
        fetch::{
            FetchHandlerActions, FetchHandlerConnectionErrorAction, FetchHandlerFetchCompleteAction,
        },
        graphql::*,
        timeout::{TimeoutHandlerActions, TimeoutHandlerTimeoutAction},
        timestamp::{TimestampHandlerActions, TimestampHandlerUpdateAction},
    },
    actor::{HandlerAction, HandlerActor, HandlerActorBuiltin, HandlerTask},
    default_handler_actors, hyper,
    task::{
        fetch::FetchHandlerTaskFactory,
        graphql::{
            GraphQlHandlerHttpFetchTaskFactory, GraphQlHandlerWebSocketConnectionTaskFactory,
        },
        timeout::TimeoutHandlerTaskFactory,
        timestamp::TimestampHandlerTaskFactory,
        DefaultHandlersTaskAction, DefaultHandlersTaskFactory,
    },
    utils::tls::{create_https_client, hyper_rustls},
    DefaultHandlerMetricNames,
};
use reflex_json::{JsonMap, JsonValue};
use reflex_lang::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
use reflex_macros::{blanket_trait, task_factory_enum, Matcher, Named};
use reflex_parser::{create_parser, syntax::js::default_js_loaders, Syntax};
use reflex_protobuf::types::WellKnownTypesTranscoder;
use reflex_runtime::{
    action::{bytecode_interpreter::*, effect::*, evaluate::*, query::*, RuntimeActions},
    actor::{
        evaluate_handler::{
            create_evaluate_effect, create_evaluate_effect_type, is_evaluate_effect_type,
            parse_evaluate_effect_result,
        },
        RuntimeAction, RuntimeActor, RuntimeMetricNames,
    },
    runtime_actors,
    task::{
        evaluate_handler::EffectThrottleTaskFactory, RuntimeTask, RuntimeTaskAction,
        RuntimeTaskFactory,
    },
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator, QueryEvaluationMode,
    QueryInvalidationStrategy,
};
use reflex_scheduler::threadpool::TokioRuntimeThreadPoolFactory;
use reflex_scheduler::tokio::{
    AsyncMessage, AsyncMessageTimestamp, NoopTokioSchedulerInstrumentation, TokioCommand,
    TokioSchedulerBuilder, TokioSchedulerLogger,
};
use reflex_utils::reconnect::{NoopReconnectTimeout, ReconnectTimeout};
use reflex_wasm::{
    cli::compile::{
        parse_and_compile_module, CompilerRootConfig, ExpressionFactoryEntryPoint,
        JavaScriptCompilerRootConfig, JsonCompilerRootConfig, LispCompilerRootConfig,
        ModuleEntryPoint, RuntimeEntryPointSyntax, WasmCompilerOptions, WasmCompilerRuntimeOptions,
    },
    compiler::CompilerOptions,
    interpreter::WasmProgram,
};

const RUNTIME_BYTES: &'static [u8] = include_bytes!("../../../reflex-wasm/build/runtime.wasm");

/// Reflex runtime evaluator
#[derive(Parser)]
struct Args {
    /// Optional entry point module to evaluate (defaults to REPL)
    input_path: Option<PathBuf>,
    /// Entry point module syntax (defaults to inferring based on entry point module file extension)
    #[clap(long)]
    syntax: Option<RuntimeEntryPointSyntax>,
    /// Name of entry point function within WebAssembly module (only valid for WASM entry points)
    #[clap(long)]
    entry_point: Option<ModuleEntryPoint>,
    /// Path to custom TLS certificate
    #[clap(long)]
    tls_cert: Option<PathBuf>,
    /// Paths of compiled gRPC service definition protobufs
    #[clap(long)]
    grpc_service: Vec<PathBuf>,
    /// Throttle stateful effect updates
    #[clap(long)]
    effect_throttle_ms: Option<u64>,
    /// Log runtime actions
    #[clap(long)]
    log: bool,
    /// Skip compiler optimizations
    #[clap(long)]
    unoptimized: bool,
    /// Compile array items as lazily-evaluated expressions
    #[clap(long)]
    lazy_list_items: bool,
    /// Compile record field values as lazily-evaluated expressions
    #[clap(long)]
    lazy_record_values: bool,
    /// Compile function call arguments as lazily-evaluated expressions
    #[clap(long)]
    lazy_function_args: bool,
    /// Compile variable initializer values as lazily-evaluated expressions
    #[clap(long)]
    lazy_variable_initializers: bool,
    /// Compile lambda arguments as lazily-evaluated expressions
    #[clap(long)]
    lazy_lambda_args: bool,
    /// Compile constructor arguments as lazily-evaluated expressions
    #[clap(long)]
    lazy_constructors: bool,
    /// Wrap compiled lambdas in argument memoization wrappers
    #[clap(long)]
    memoize_lambdas: bool,
    /// Dump heap snapshots for any queries that return error results
    #[clap(long)]
    dump_heap_snapshot: Option<WasmHeapDumpMode>,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    type TBuiltin = CliBuiltins;
    type T = CachedSharedTerm<TBuiltin>;
    type TFactory = SharedTermFactory<TBuiltin>;
    type TAllocator = DefaultAllocator<T>;
    type TConnect = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;
    type TReconnect = NoopReconnectTimeout;
    type TGrpcConfig = DefaultGrpcConfig;
    type TAction = CliActions<T>;
    type TMetricLabels = CliMetricLabels;
    type TTask =
        CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>;
    type TInstrumentation = NoopTokioSchedulerInstrumentation<TAction, TTask>;
    let args = Args::parse();
    let unoptimized = args.unoptimized;
    let dump_heap_snapshot = args.dump_heap_snapshot;
    let effect_throttle = args.effect_throttle_ms.map(Duration::from_millis);
    let input_path = &args.input_path;
    let factory: TFactory = SharedTermFactory::<TBuiltin>::default();
    let allocator: TAllocator = DefaultAllocator::default();
    let https_client: hyper::Client<TConnect> = create_https_client(None)?;
    let grpc_services = load_grpc_services(args.grpc_service.iter())
        .with_context(|| "Failed to load gRPC service descriptor")?;
    let grpc_config = DefaultGrpcConfig::default();
    let grpc_max_operations_per_connection =
        match std::env::var("GRPC_MAX_OPERATIONS_PER_CONNECTION") {
            Ok(value) => str::parse::<usize>(&value)
                .with_context(|| "Invalid value for GRPC_MAX_OPERATIONS_PER_CONNECTION")
                .map(Some),
            _ => Ok(None),
        }?;
    let compiler_options = {
        let defaults = WasmCompilerOptions::default();
        WasmCompilerOptions {
            compiler: {
                let defaults = CompilerOptions::default();
                CompilerOptions {
                    lazy_record_values: match args.lazy_record_values {
                        true => ArgType::Lazy,
                        false => defaults.lazy_record_values,
                    },
                    lazy_list_items: match args.lazy_list_items {
                        true => ArgType::Lazy,
                        false => defaults.lazy_list_items,
                    },
                    lazy_variable_initializers: match args.lazy_variable_initializers {
                        true => ArgType::Lazy,
                        false => defaults.lazy_variable_initializers,
                    },
                    lazy_function_args: match args.lazy_function_args {
                        true => true,
                        false => defaults.lazy_function_args,
                    },
                    lazy_lambda_args: match args.lazy_lambda_args {
                        true => ArgType::Lazy,
                        false => defaults.lazy_lambda_args,
                    },
                    lazy_constructors: match args.lazy_constructors {
                        true => ArgType::Lazy,
                        false => defaults.lazy_constructors,
                    },
                    ..defaults
                }
            },
            runtime: {
                let defaults = WasmCompilerRuntimeOptions::default();
                WasmCompilerRuntimeOptions {
                    memoize_lambdas: args.memoize_lambdas,
                    ..defaults
                }
            },
            ..defaults
        }
    };
    match input_path {
        None => {
            let syntax = args
                .syntax
                .ok_or_else(|| anyhow!("If no input file is specified, syntax must be specified"))
                .and_then(|syntax| match syntax {
                    RuntimeEntryPointSyntax::Source(syntax) => Ok(syntax),
                    _ => Err(anyhow!("Invalid REPL input syntax")),
                })?;
            let state = StateCache::default();
            let mut cache = SubstitutionCache::new();
            let parser = create_parser(
                syntax,
                None,
                default_js_loaders(empty(), &factory, &allocator),
                empty(),
                &factory,
                &allocator,
            );
            repl::run(parser, &state, &factory, &allocator, &mut cache)?;
        }
        Some(input_path) => {
            let syntax = match args.syntax {
                Some(syntax) => Ok(syntax),
                None => {
                    let file_extension = input_path.extension().ok_or_else(|| {
                        anyhow!("Unable to determine entry point filename extension")
                    })?;
                    RuntimeEntryPointSyntax::infer(file_extension).ok_or_else(|| {
                        anyhow!("Unable to infer entry point syntax based on filename")
                    })
                }
            }?;
            let (wasm_module, entry_point_name) = match syntax {
                RuntimeEntryPointSyntax::Wasm => {
                    let entry_point_name = args.entry_point.as_ref().cloned().unwrap_or_default();
                    read_wasm_module(input_path)
                        .map(WasmProgram::from_wasm)
                        .map(|module| (module, entry_point_name))
                }
                RuntimeEntryPointSyntax::PrecompiledWasm => {
                    let entry_point_name = args.entry_point.as_ref().cloned().unwrap_or_default();
                    read_wasm_module(input_path)
                        .map(WasmProgram::from_cwasm)
                        .map(|module| (module, entry_point_name))
                }
                RuntimeEntryPointSyntax::Source(syntax) => {
                    let entry_point_name = ModuleEntryPoint::default();
                    let root = match syntax {
                        Syntax::Lisp => CompilerRootConfig::Lisp(LispCompilerRootConfig::from(
                            input_path.to_owned(),
                        )),
                        Syntax::Json => CompilerRootConfig::Json(JsonCompilerRootConfig::from(
                            input_path.to_owned(),
                        )),
                        Syntax::JavaScript => CompilerRootConfig::JavaScript(
                            JavaScriptCompilerRootConfig::from(input_path.to_owned()),
                        ),
                    };
                    let entry_point =
                        ExpressionFactoryEntryPoint::new(entry_point_name.clone(), root);
                    parse_and_compile_module(
                        [&entry_point],
                        default_js_loaders(empty(), &factory, &allocator),
                        std::env::vars(),
                        RUNTIME_BYTES,
                        &factory,
                        &allocator,
                        &compiler_options,
                        unoptimized,
                    )
                    .with_context(|| "Failed to compile entry point: {input_path}")
                    .map(WasmProgram::from_wasm)
                    .map(move |module| (module, entry_point_name))
                }
            }?;
            let logger = if args.log {
                Some(CliActionLogger::stderr())
            } else {
                None
            };
            let instrumentation = NoopTokioSchedulerInstrumentation::default();
            let async_tasks = TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current());
            let blocking_tasks =
                TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current());
            let (evaluate_effect, subscribe_action) = create_root_query(&factory, &allocator);
            let (scheduler, main_pid) = {
                let mut builder = TokioSchedulerBuilder::<TAction, TTask, _, _, _, _>::new(
                    logger,
                    instrumentation,
                    async_tasks,
                    blocking_tasks,
                );
                let main_pid = builder.generate_pid();
                let actors = {
                    runtime_actors(
                        factory.clone(),
                        allocator.clone(),
                        effect_throttle,
                        RuntimeMetricNames::default(),
                        main_pid,
                    )
                    .into_iter()
                    .map(CliActor::Runtime)
                }
                .chain(once(CliActor::WasmInterpreter(WasmInterpreter::new(
                    wasm_module,
                    entry_point_name,
                    factory.clone(),
                    allocator.clone(),
                    WasmInterpreterMetricNames::default(),
                    CliMetricLabels,
                    main_pid,
                    dump_heap_snapshot,
                ))))
                .chain(
                    default_handler_actors::<
                        TAction,
                        TTask,
                        T,
                        TFactory,
                        TAllocator,
                        TConnect,
                        TReconnect,
                    >(
                        https_client,
                        &factory,
                        &allocator,
                        NoopReconnectTimeout,
                        DefaultHandlerMetricNames::default(),
                        main_pid,
                    )
                    .into_iter()
                    .map(|actor| CliActor::Handler(actor)),
                )
                .chain(once(CliActor::Grpc(GrpcHandler::new(
                    grpc_services,
                    WellKnownTypesTranscoder,
                    factory.clone(),
                    allocator.clone(),
                    NoopReconnectTimeout,
                    grpc_max_operations_per_connection,
                    grpc_config,
                    GrpcHandlerMetricNames::default(),
                    main_pid,
                ))))
                .map(|actor| (builder.generate_pid(), actor))
                .collect::<Vec<_>>();
                let actor_pids = actors.iter().map(|(pid, _)| *pid);
                builder.worker(main_pid, CliActor::Main(Redispatcher::new(actor_pids)));
                for (pid, actor) in actors {
                    builder.worker(pid, actor);
                }
                builder.send(main_pid, TAction::from(subscribe_action));
                let runtime = builder.build();
                (runtime, main_pid)
            };
            let mut results_stream = tokio::spawn(scheduler.subscribe(main_pid, {
                let factory = factory.clone();
                move |action: &CliActions<CachedSharedTerm<CliBuiltins>>| {
                    let EffectEmitAction { effect_types } = action.match_type()?;
                    let update = effect_types
                        .iter()
                        .filter(|batch| is_evaluate_effect_type(&batch.effect_type, &factory))
                        .flat_map(|batch| batch.updates.iter())
                        .filter(|(key, _)| key.id() == evaluate_effect.id())
                        .filter_map({
                            let factory = factory.clone();
                            move |(_, value)| parse_evaluate_effect_result(value, &factory)
                        })
                        .next()?;
                    Some(update.result().clone())
                }
            }))
            .await
            .unwrap();
            while let Some(value) = results_stream.next().await {
                let output = match factory.match_signal_term(&value) {
                    None => format!("{}", value),
                    Some(signal) => format_signal_result(signal),
                };
                println!("{}{}", clear_escape_sequence(), output);
            }
        }
    }
    Ok(())
}

fn create_root_query<
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T>,
    TAllocator: AsyncHeapAllocator<T>,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> (T::Signal, EffectSubscribeAction<T>) {
    let query = factory.create_lambda_term(1, factory.create_variable_term(0));
    let evaluate_effect = create_evaluate_effect(
        String::from("<anonymous>"),
        query,
        QueryEvaluationMode::Query,
        QueryInvalidationStrategy::Exact,
        factory,
        allocator,
    );
    let subscribe_action = EffectSubscribeAction {
        effect_type: create_evaluate_effect_type(factory, allocator),
        effects: vec![evaluate_effect.clone()],
    };
    (evaluate_effect, subscribe_action)
}

fn read_wasm_module(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| {
        format!(
            "Failed to load WebAssemby module: {}",
            path.to_string_lossy()
        )
    })
}

struct CliMetricLabels;
impl BytecodeInterpreterMetricLabels for CliMetricLabels {
    fn labels(&self, query_name: &str) -> Vec<(SharedString, SharedString)> {
        vec![("worker".into(), String::from(query_name).into())]
    }
}

pub struct CliActionLogger<
    TOut: std::io::Write,
    TAction: Action,
    TTask: TaskFactory<TAction, TTask>,
> {
    output: TOut,
    _action: PhantomData<TAction>,
    _task: PhantomData<TTask>,
}
impl<TOut: std::io::Write, TAction: Action, TTask: TaskFactory<TAction, TTask>>
    CliActionLogger<TOut, TAction, TTask>
{
    pub fn new(output: TOut) -> Self {
        Self {
            output,
            _action: PhantomData,
            _task: PhantomData,
        }
    }
    fn log(&mut self, action: &impl SerializableAction) {
        let serialized_args = JsonValue::Object(JsonMap::from_iter(action.to_json()));
        let _ = writeln!(
            self.output,
            "[{}] {}",
            action.name(),
            serialized_args.to_string()
        );
    }
}
impl<TAction: Action, TTask: TaskFactory<TAction, TTask>>
    CliActionLogger<std::io::Stderr, TAction, TTask>
{
    pub fn stderr() -> Self {
        Self::new(std::io::stderr())
    }
}
impl<TAction: Action, TTask: TaskFactory<TAction, TTask>> Clone
    for CliActionLogger<std::io::Stderr, TAction, TTask>
{
    fn clone(&self) -> Self {
        Self::new(std::io::stderr())
    }
}
impl<TAction: Action, TTask: TaskFactory<TAction, TTask>>
    CliActionLogger<std::io::Stdout, TAction, TTask>
{
    pub fn stdout() -> Self {
        Self::new(std::io::stdout())
    }
}
impl<TAction: Action, TTask: TaskFactory<TAction, TTask>> Clone
    for CliActionLogger<std::io::Stdout, TAction, TTask>
{
    fn clone(&self) -> Self {
        Self::new(std::io::stdout())
    }
}
impl<TOut: std::io::Write, TAction: Action, TTask: TaskFactory<TAction, TTask>> TokioSchedulerLogger
    for CliActionLogger<TOut, TAction, TTask>
where
    TAction: SerializableAction,
{
    type Action = TAction;
    type Task = TTask;
    fn log_scheduler_command(
        &mut self,
        command: &TokioCommand<Self::Action, Self::Task>,
        _enqueue_time: AsyncMessageTimestamp,
    ) {
        match command {
            TokioCommand::Send { pid: _, message } if message.redispatched_from().is_none() => {
                let action = message.deref();
                self.log(action);
            }
            _ => {}
        }
    }
    fn log_worker_message(
        &mut self,
        _message: &AsyncMessage<Self::Action>,
        _actor: &<Self::Task as TaskFactory<Self::Action, Self::Task>>::Actor,
        _pid: ProcessId,
    ) {
    }
    fn log_task_message(&mut self, _message: &AsyncMessage<Self::Action>, _pid: ProcessId) {}
}

fn clear_escape_sequence() -> &'static str {
    "\x1b[2J\x1b[H"
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum CliActions<T: Expression> {
    Runtime(RuntimeActions<T>),
    WasmInterpreter(BytecodeInterpreterActions<T>),
    FetchHandler(FetchHandlerActions),
    GraphQlHandler(GraphQlHandlerActions),
    TimeoutHandler(TimeoutHandlerActions),
    TimestampHandler(TimestampHandlerActions),
    GrpcHandler(GrpcHandlerActions),
}
impl<T: Expression> Named for CliActions<T> {
    fn name(&self) -> &'static str {
        match self {
            Self::Runtime(action) => action.name(),
            Self::WasmInterpreter(action) => action.name(),
            Self::FetchHandler(action) => action.name(),
            Self::GraphQlHandler(action) => action.name(),
            Self::TimeoutHandler(action) => action.name(),
            Self::TimestampHandler(action) => action.name(),
            Self::GrpcHandler(action) => action.name(),
        }
    }
}
impl<T: Expression> Action for CliActions<T> {}
impl<T: Expression> SerializableAction for CliActions<T> {
    fn to_json(&self) -> SerializedAction {
        match self {
            Self::Runtime(action) => action.to_json(),
            Self::WasmInterpreter(action) => action.to_json(),
            Self::FetchHandler(action) => action.to_json(),
            Self::GraphQlHandler(action) => action.to_json(),
            Self::TimeoutHandler(action) => action.to_json(),
            Self::TimestampHandler(action) => action.to_json(),
            Self::GrpcHandler(action) => action.to_json(),
        }
    }
}

blanket_trait!(
    trait CliAction<T: Expression>:
        SerializableAction
        + RuntimeAction<T>
        + HandlerAction<T>
        + GrpcHandlerAction<T>
        + WasmInterpreterAction<T>
        + CliTaskAction<T>
    {
    }
);

blanket_trait!(
    trait CliTask<T, TFactory, TAllocator, TConnect>:
        RuntimeTask
        + HandlerTask<TConnect>
        + WasmWorkerTask<T, TFactory, TAllocator>
        + GrpcHandlerTask<WellKnownTypesTranscoder>
    where
        T: Expression,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
    }
);

enum CliActor<
    T,
    TFactory,
    TAllocator,
    TConnect,
    TReconnect,
    TGrpcConfig,
    TMetricLabels,
    TAction,
    TTask,
> where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliTaskAction<T> + Send + 'static,
    TTask: TaskFactory<TAction, TTask>,
{
    Runtime(RuntimeActor<T, TFactory, TAllocator>),
    Handler(HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect>),
    Grpc(GrpcHandler<T, TFactory, TAllocator, WellKnownTypesTranscoder, TGrpcConfig, TReconnect>),
    WasmInterpreter(WasmInterpreter<T, TFactory, TAllocator, TMetricLabels>),
    Main(Redispatcher),
    Task(CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask>),
}
impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels, TAction, TTask>
    Named
    for CliActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliTaskAction<T> + Send + 'static,
    TTask: TaskFactory<TAction, TTask>,
{
    fn name(&self) -> &'static str {
        match self {
            Self::Runtime(inner) => inner.name(),
            Self::Handler(inner) => inner.name(),
            Self::Grpc(inner) => inner.name(),
            Self::WasmInterpreter(inner) => inner.name(),
            Self::Main(inner) => inner.name(),
            Self::Task(inner) => inner.name(),
        }
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels, TAction, TTask>
    Actor<TAction, TTask>
    for CliActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    type Events<TInbox: TaskInbox<TAction>> = CliEvents<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TInbox,
        TAction,
        TTask,
    >;
    type Dispose = CliDispose<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >;
    fn init(&self) -> Self::State {
        match self {
            Self::Runtime(actor) => {
                CliActorState::Runtime(<RuntimeActor<T, TFactory, TAllocator> as Actor<
                    TAction,
                    TTask,
                >>::init(actor))
            }
            Self::Handler(actor) => {
                CliActorState::Handler(
                    <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Actor<
                        TAction,
                        TTask,
                    >>::init(actor),
                )
            }
            Self::Grpc(actor) => {
                CliActorState::Grpc(<GrpcHandler<
                    T,
                    TFactory,
                    TAllocator,
                    WellKnownTypesTranscoder,
                    TGrpcConfig,
                    TReconnect,
                > as Actor<TAction, TTask>>::init(actor))
            }
            Self::WasmInterpreter(actor) => {
                CliActorState::WasmInterpreter(<WasmInterpreter<
                    T,
                    TFactory,
                    TAllocator,
                    TMetricLabels,
                > as Actor<TAction, TTask>>::init(
                    actor
                ))
            }
            Self::Main(actor) => {
                CliActorState::Main(<Redispatcher as Actor<TAction, TTask>>::init(actor))
            }
            Self::Task(actor) => {
                CliActorState::Task(<CliTaskActor<
                    T,
                    TFactory,
                    TAllocator,
                    TConnect,
                    TAction,
                    TTask,
                > as Actor<TAction, TTask>>::init(actor))
            }
        }
    }
    fn events<TInbox: TaskInbox<TAction>>(
        &self,
        inbox: TInbox,
    ) -> ActorEvents<TInbox, Self::Events<TInbox>, Self::Dispose> {
        match self {
            Self::Runtime(actor) => <RuntimeActor<T, TFactory, TAllocator> as Actor<
                TAction,
                TTask,
            >>::events(actor, inbox)
            .map(|(events, dispose)| {
                (CliEvents::Runtime(events), dispose.map(CliDispose::Runtime))
            }),
            Self::Handler(actor) => {
                <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Actor<
                    TAction,
                    TTask,
                >>::events(actor, inbox)
                .map(|(events, dispose)| {
                    (CliEvents::Handler(events), dispose.map(CliDispose::Handler))
                })
            }
            Self::Grpc(actor) => <GrpcHandler<
                T,
                TFactory,
                TAllocator,
                WellKnownTypesTranscoder,
                TGrpcConfig,
                TReconnect,
            > as Actor<TAction, TTask>>::events(actor, inbox)
            .map(|(events, dispose)| (CliEvents::Grpc(events), dispose.map(CliDispose::Grpc))),
            Self::WasmInterpreter(actor) => {
                <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Actor<
                    TAction,
                    TTask,
                >>::events(actor, inbox)
                .map(|(events, dispose)| {
                    (
                        CliEvents::WasmInterpreter(events),
                        dispose.map(CliDispose::WasmInterpreter),
                    )
                })
            }
            Self::Main(actor) => <Redispatcher as Actor<TAction, TTask>>::events(actor, inbox)
                .map(|(events, dispose)| (CliEvents::Main(events), dispose.map(CliDispose::Main))),
            Self::Task(actor) => {
                <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Actor<
                    TAction,
                    TTask,
                >>::events(actor, inbox)
                .map(|(events, dispose)| (CliEvents::Task(events), dispose.map(CliDispose::Task)))
            }
        }
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels, TAction, TTask>
    Worker<TAction, SchedulerTransition<TAction, TTask>>
    for CliActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    fn accept(&self, message: &TAction) -> bool {
        match self {
            Self::Runtime(actor) => <RuntimeActor<T, TFactory, TAllocator> as Worker<
                TAction,
                SchedulerTransition<TAction, TTask>,
            >>::accept(actor, message),
            Self::Handler(actor) => {
                <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::accept(actor, message)
            }
            Self::Grpc(actor) => <GrpcHandler<
                T,
                TFactory,
                TAllocator,
                WellKnownTypesTranscoder,
                TGrpcConfig,
                TReconnect,
            > as Worker<TAction, SchedulerTransition<TAction, TTask>>>::accept(
                actor, message
            ),
            Self::WasmInterpreter(actor) => {
                <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::accept(actor, message)
            }
            Self::Main(actor) => <Redispatcher as Worker<
                TAction,
                SchedulerTransition<TAction, TTask>,
            >>::accept(actor, message),
            Self::Task(actor) => {
                <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::accept(actor, message)
            }
        }
    }
    fn schedule(&self, message: &TAction, state: &Self::State) -> Option<SchedulerMode> {
        match (self, state) {
            (Self::Runtime(actor), CliActorState::Runtime(state)) => {
                <RuntimeActor<T, TFactory, TAllocator> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::schedule(actor, message, state)
            }
            (Self::Handler(actor), CliActorState::Handler(state)) => {
                <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::schedule(actor, message, state)
            }
            (Self::Grpc(actor), CliActorState::Grpc(state)) => {
                <GrpcHandler<
                    T,
                    TFactory,
                    TAllocator,
                    WellKnownTypesTranscoder,
                    TGrpcConfig,
                    TReconnect,
                > as Worker<TAction, SchedulerTransition<TAction, TTask>>>::schedule(
                    actor, message, state,
                )
            }
            (Self::WasmInterpreter(actor), CliActorState::WasmInterpreter(state)) => {
                <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::schedule(actor, message, state)
            }
            (Self::Main(actor), CliActorState::Main(state)) => {
                <Redispatcher as Worker<TAction, SchedulerTransition<TAction, TTask>>>::schedule(
                    actor, message, state,
                )
            }
            (Self::Task(actor), CliActorState::Task(state)) => {
                <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Worker<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::schedule(actor, message, state)
            }
            _ => unreachable!(),
        }
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels, TAction, TTask>
    Handler<TAction, SchedulerTransition<TAction, TTask>>
    for CliActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    type State = CliActorState<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >;
    fn handle(
        &self,
        state: &mut Self::State,
        action: &TAction,
        metadata: &MessageData,
        context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>> {
        match (self, state) {
            (Self::Runtime(actor), CliActorState::Runtime(state)) => {
                <RuntimeActor<T, TFactory, TAllocator> as Handler<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::handle(actor, state, action, metadata, context)
            }
            (Self::Handler(actor), CliActorState::Handler(state)) => {
                <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Handler<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::handle(actor, state, action, metadata, context)
            }
            (Self::Grpc(actor), CliActorState::Grpc(state)) => {
                <GrpcHandler<
                    T,
                    TFactory,
                    TAllocator,
                    WellKnownTypesTranscoder,
                    TGrpcConfig,
                    TReconnect,
                > as Handler<TAction, SchedulerTransition<TAction, TTask>>>::handle(
                    actor, state, action, metadata, context,
                )
            }
            (Self::WasmInterpreter(actor), CliActorState::WasmInterpreter(state)) => {
                <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Handler<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::handle(actor, state, action, metadata, context)
            }
            (Self::Main(actor), CliActorState::Main(state)) => {
                <Redispatcher as Handler<TAction, SchedulerTransition<TAction, TTask>>>::handle(
                    actor, state, action, metadata, context,
                )
            }
            (Self::Task(actor), CliActorState::Task(state)) => {
                <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Handler<
                    TAction,
                    SchedulerTransition<TAction, TTask>,
                >>::handle(actor, state, action, metadata, context)
            }
            _ => unreachable!(),
        }
    }
}

enum CliActorState<
    T,
    TFactory,
    TAllocator,
    TConnect,
    TReconnect,
    TGrpcConfig,
    TMetricLabels,
    TAction,
    TTask,
> where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    Runtime(
        <RuntimeActor<T, TFactory, TAllocator> as Handler<
            TAction,
            SchedulerTransition<TAction, TTask>,
        >>::State,
    ),
    Handler(
        <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Handler<
            TAction,
            SchedulerTransition<TAction, TTask>,
        >>::State,
    ),
    Grpc(
        <GrpcHandler<T, TFactory, TAllocator, WellKnownTypesTranscoder, TGrpcConfig, TReconnect> as Handler<
            TAction,
            SchedulerTransition<TAction, TTask>,
        >>::State,
    ),
    WasmInterpreter(
        <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Handler<
            TAction,
            SchedulerTransition<TAction, TTask>,
        >>::State,
    ),
    Main(<Redispatcher as Handler<TAction, SchedulerTransition<TAction, TTask>>>::State),
    Task(
        <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Handler<
            TAction,
            SchedulerTransition<TAction, TTask>,
        >>::State,
    ),
}

#[pin_project(project = CliEventsVariant)]
enum CliEvents<
    T,
    TFactory,
    TAllocator,
    TConnect,
    TReconnect,
    TGrpcConfig,
    TMetricLabels,
    TInbox,
    TAction,
    TTask,
> where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TInbox: TaskInbox<TAction>,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    Runtime(
        #[pin] <RuntimeActor<T, TFactory, TAllocator> as Actor<TAction, TTask>>::Events<TInbox>,
    ),
    Handler(
        #[pin]
        <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Actor<
                TAction,
                TTask,
            >>::Events<TInbox>,
    ),
    Grpc(
        #[pin]
        <GrpcHandler<T, TFactory, TAllocator, WellKnownTypesTranscoder, TGrpcConfig, TReconnect> as Actor<
                TAction,
                TTask,
            >>::Events<TInbox>,
    ),
    WasmInterpreter(
        #[pin]
        <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Actor<
                TAction,
                TTask,
            >>::Events<TInbox>,
    ),
    Main(#[pin] <Redispatcher as Actor<TAction, TTask>>::Events<TInbox>),
    Task(
        #[pin]
        <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Actor<
            TAction,
            TTask,
        >>::Events<TInbox>,
    ),
}
impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TInbox,
        TAction,
        TTask,
    > Stream
    for CliEvents<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TInbox,
        TAction,
        TTask,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TInbox: TaskInbox<TAction>,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    type Item = TInbox::Message;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.project() {
            CliEventsVariant::Runtime(inner) => inner.poll_next(cx),
            CliEventsVariant::Handler(inner) => inner.poll_next(cx),
            CliEventsVariant::Grpc(inner) => inner.poll_next(cx),
            CliEventsVariant::WasmInterpreter(inner) => inner.poll_next(cx),
            CliEventsVariant::Main(inner) => inner.poll_next(cx),
            CliEventsVariant::Task(inner) => inner.poll_next(cx),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Runtime(inner) => inner.size_hint(),
            Self::Handler(inner) => inner.size_hint(),
            Self::Grpc(inner) => inner.size_hint(),
            Self::WasmInterpreter(inner) => inner.size_hint(),
            Self::Main(inner) => inner.size_hint(),
            Self::Task(inner) => inner.size_hint(),
        }
    }
}

#[pin_project(project = CliDisposeVariant)]
enum CliDispose<
    T,
    TFactory,
    TAllocator,
    TConnect,
    TReconnect,
    TGrpcConfig,
    TMetricLabels,
    TAction,
    TTask,
> where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    Runtime(#[pin] <RuntimeActor<T, TFactory, TAllocator> as Actor<TAction, TTask>>::Dispose),
    Handler(
        #[pin]
        <HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect> as Actor<
                TAction,
                TTask,
            >>::Dispose,
    ),
    Grpc(
        #[pin]
        <GrpcHandler<T, TFactory, TAllocator, WellKnownTypesTranscoder, TGrpcConfig, TReconnect> as Actor<
                TAction,
                TTask,
            >>::Dispose,
    ),
    WasmInterpreter(
        #[pin]
        <WasmInterpreter<T, TFactory, TAllocator, TMetricLabels> as Actor<
                TAction,
                TTask,
            >>::Dispose,
    ),
    Main(#[pin] <Redispatcher as Actor<TAction, TTask>>::Dispose),
    Task(
        #[pin]
        <CliTaskActor<T, TFactory, TAllocator, TConnect, TAction, TTask> as Actor<
            TAction,
            TTask,
        >>::Dispose,
    ),
}
impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels, TAction, TTask>
    Future
    for CliDispose<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        TTask,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
    TTask:
        TaskFactory<TAction, TTask> + CliTask<T, TFactory, TAllocator, TConnect> + Send + 'static,
{
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.project() {
            CliDisposeVariant::Runtime(inner) => inner.poll(cx),
            CliDisposeVariant::Handler(inner) => inner.poll(cx),
            CliDisposeVariant::Grpc(inner) => inner.poll(cx),
            CliDisposeVariant::WasmInterpreter(inner) => inner.poll(cx),
            CliDisposeVariant::Main(inner) => inner.poll(cx),
            CliDisposeVariant::Task(inner) => inner.poll(cx),
        }
    }
}

blanket_trait!(
    trait CliTaskAction<T: Expression>:
        Action
        + RuntimeTaskAction
        + WasmWorkerAction<T>
        + DefaultHandlersTaskAction
        + GrpcHandlerConnectionTaskAction
    {
    }
);

task_factory_enum!({
    #[derive(Matcher, Clone)]
    enum CliTaskFactory<T, TFactory, TAllocator, TConnect>
    where
        T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
        TFactory: AsyncExpressionFactory<T> + Default,
        TAllocator: AsyncHeapAllocator<T> + Default,
        T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        Runtime(RuntimeTaskFactory),
        WasmWorker(WasmWorkerTaskFactory<T, TFactory, TAllocator>),
        DefaultHandlers(DefaultHandlersTaskFactory<TConnect>),
        GrpcHandler(GrpcHandlerConnectionTaskFactory),
    }

    impl<T, TFactory, TAllocator, TConnect, TAction, TTask> TaskFactory<TAction, TTask>
        for CliTaskFactory<T, TFactory, TAllocator, TConnect>
    where
        T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
        TFactory: AsyncExpressionFactory<T> + Default,
        TAllocator: AsyncHeapAllocator<T> + Default,
        T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
        TAction: Action + CliTaskAction<T> + Send + 'static,
        TTask: TaskFactory<TAction, TTask>,
    {
    }
});

#[derive(Named, Clone)]
struct CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    inner: CliTaskFactory<T, TFactory, TAllocator, TConnect>,
    _reconnect: PhantomData<TReconnect>,
    _grpc_config: PhantomData<TGrpcConfig>,
    _metric_labels: PhantomData<TMetricLabels>,
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<CliTaskFactory<T, TFactory, TAllocator, TConnect>>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: CliTaskFactory<T, TFactory, TAllocator, TConnect>) -> Self {
        Self {
            inner: value,
            _reconnect: PhantomData,
            _grpc_config: PhantomData,
            _metric_labels: PhantomData,
        }
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels, TAction>
    TaskFactory<TAction, Self>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
    TAction: Action + CliAction<T> + Send + 'static,
{
    type Actor = CliActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TMetricLabels,
        TAction,
        Self,
    >;
    fn create(self) -> Self::Actor {
        CliActor::Task(<CliTaskFactory<T, TFactory, TAllocator, TConnect> as TaskFactory<TAction, Self>>::create(self.inner))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<WasmWorkerTaskFactory<T, TFactory, TAllocator>>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: WasmWorkerTaskFactory<T, TFactory, TAllocator>) -> Self {
        Self::from(CliTaskFactory::WasmWorker(value))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<EffectThrottleTaskFactory>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: EffectThrottleTaskFactory) -> Self {
        Self::from(CliTaskFactory::Runtime(RuntimeTaskFactory::from(value)))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<FetchHandlerTaskFactory<TConnect>>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: FetchHandlerTaskFactory<TConnect>) -> Self {
        Self::from(CliTaskFactory::DefaultHandlers(
            DefaultHandlersTaskFactory::from(value),
        ))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<GraphQlHandlerHttpFetchTaskFactory<TConnect>>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: GraphQlHandlerHttpFetchTaskFactory<TConnect>) -> Self {
        Self::from(CliTaskFactory::DefaultHandlers(
            DefaultHandlersTaskFactory::from(value),
        ))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<GraphQlHandlerWebSocketConnectionTaskFactory>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: GraphQlHandlerWebSocketConnectionTaskFactory) -> Self {
        Self::from(CliTaskFactory::DefaultHandlers(
            DefaultHandlersTaskFactory::from(value),
        ))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<TimeoutHandlerTaskFactory>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: TimeoutHandlerTaskFactory) -> Self {
        Self::from(CliTaskFactory::DefaultHandlers(
            DefaultHandlersTaskFactory::from(value),
        ))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<TimestampHandlerTaskFactory>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: TimestampHandlerTaskFactory) -> Self {
        Self::from(CliTaskFactory::DefaultHandlers(
            DefaultHandlersTaskFactory::from(value),
        ))
    }
}

impl<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
    From<GrpcHandlerConnectionTaskFactory>
    for CliActorFactory<T, TFactory, TAllocator, TConnect, TReconnect, TGrpcConfig, TMetricLabels>
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: HandlerActorBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + Clone + 'static,
    TMetricLabels: BytecodeInterpreterMetricLabels + Send + 'static,
{
    fn from(value: GrpcHandlerConnectionTaskFactory) -> Self {
        Self::from(CliTaskFactory::GrpcHandler(value))
    }
}

impl<T: Expression> From<RuntimeActions<T>> for CliActions<T> {
    fn from(value: RuntimeActions<T>) -> Self {
        Self::Runtime(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<RuntimeActions<T>> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::Runtime(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a RuntimeActions<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::Runtime(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<BytecodeInterpreterActions<T>> for CliActions<T> {
    fn from(value: BytecodeInterpreterActions<T>) -> Self {
        Self::WasmInterpreter(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<BytecodeInterpreterActions<T>> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::WasmInterpreter(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a BytecodeInterpreterActions<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::WasmInterpreter(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<FetchHandlerActions> for CliActions<T> {
    fn from(value: FetchHandlerActions) -> Self {
        Self::FetchHandler(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<FetchHandlerActions> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::FetchHandler(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a FetchHandlerActions> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::FetchHandler(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<GraphQlHandlerActions> for CliActions<T> {
    fn from(value: GraphQlHandlerActions) -> Self {
        Self::GraphQlHandler(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerActions> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::GraphQlHandler(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GraphQlHandlerActions> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::GraphQlHandler(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<TimeoutHandlerActions> for CliActions<T> {
    fn from(value: TimeoutHandlerActions) -> Self {
        Self::TimeoutHandler(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<TimeoutHandlerActions> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::TimeoutHandler(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a TimeoutHandlerActions> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::TimeoutHandler(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<TimestampHandlerActions> for CliActions<T> {
    fn from(value: TimestampHandlerActions) -> Self {
        Self::TimestampHandler(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<TimestampHandlerActions> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::TimestampHandler(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a TimestampHandlerActions> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::TimestampHandler(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<GrpcHandlerActions> for CliActions<T> {
    fn from(value: GrpcHandlerActions) -> Self {
        Self::GrpcHandler(value)
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerActions> {
    fn from(value: CliActions<T>) -> Self {
        match value {
            CliActions::GrpcHandler(value) => Some(value),
            _ => None,
        }
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerActions> {
    fn from(value: &'a CliActions<T>) -> Self {
        match value {
            CliActions::GrpcHandler(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Expression> From<EffectActions<T>> for CliActions<T> {
    fn from(value: EffectActions<T>) -> Self {
        RuntimeActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EffectActions<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<RuntimeActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EffectActions<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a RuntimeActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EffectSubscribeAction<T>> for CliActions<T> {
    fn from(value: EffectSubscribeAction<T>) -> Self {
        EffectActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EffectSubscribeAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EffectSubscribeAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EffectUnsubscribeAction<T>> for CliActions<T> {
    fn from(value: EffectUnsubscribeAction<T>) -> Self {
        EffectActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EffectUnsubscribeAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EffectUnsubscribeAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EffectEmitAction<T>> for CliActions<T> {
    fn from(value: EffectEmitAction<T>) -> Self {
        EffectActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EffectEmitAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EffectEmitAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EffectThrottleEmitAction> for CliActions<T> {
    fn from(value: EffectThrottleEmitAction) -> Self {
        EffectActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EffectThrottleEmitAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EffectThrottleEmitAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EffectActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EvaluateActions<T>> for CliActions<T> {
    fn from(value: EvaluateActions<T>) -> Self {
        RuntimeActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EvaluateActions<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<RuntimeActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EvaluateActions<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a RuntimeActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EvaluateStartAction<T>> for CliActions<T> {
    fn from(value: EvaluateStartAction<T>) -> Self {
        EvaluateActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EvaluateStartAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EvaluateStartAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EvaluateUpdateAction<T>> for CliActions<T> {
    fn from(value: EvaluateUpdateAction<T>) -> Self {
        EvaluateActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EvaluateUpdateAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EvaluateUpdateAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EvaluateStopAction<T>> for CliActions<T> {
    fn from(value: EvaluateStopAction<T>) -> Self {
        EvaluateActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EvaluateStopAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EvaluateStopAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<EvaluateResultAction<T>> for CliActions<T> {
    fn from(value: EvaluateResultAction<T>) -> Self {
        EvaluateActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<EvaluateResultAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a EvaluateResultAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a EvaluateActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<QueryActions<T>> for CliActions<T> {
    fn from(value: QueryActions<T>) -> Self {
        RuntimeActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<QueryActions<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<RuntimeActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a QueryActions<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a RuntimeActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<QuerySubscribeAction<T>> for CliActions<T> {
    fn from(value: QuerySubscribeAction<T>) -> Self {
        QueryActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<QuerySubscribeAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<QueryActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a QuerySubscribeAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a QueryActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<QueryUnsubscribeAction<T>> for CliActions<T> {
    fn from(value: QueryUnsubscribeAction<T>) -> Self {
        QueryActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<QueryUnsubscribeAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<QueryActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a QueryUnsubscribeAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a QueryActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<QueryEmitAction<T>> for CliActions<T> {
    fn from(value: QueryEmitAction<T>) -> Self {
        QueryActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<QueryEmitAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<QueryActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a QueryEmitAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a QueryActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<BytecodeInterpreterInitAction<T>> for CliActions<T> {
    fn from(value: BytecodeInterpreterInitAction<T>) -> Self {
        BytecodeInterpreterActions::<T>::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<BytecodeInterpreterInitAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a BytecodeInterpreterInitAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<BytecodeInterpreterEvaluateAction<T>> for CliActions<T> {
    fn from(value: BytecodeInterpreterEvaluateAction<T>) -> Self {
        BytecodeInterpreterActions::<T>::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<BytecodeInterpreterEvaluateAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a BytecodeInterpreterEvaluateAction<T>>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<BytecodeInterpreterGcCompleteAction<T>> for CliActions<T> {
    fn from(value: BytecodeInterpreterGcCompleteAction<T>) -> Self {
        BytecodeInterpreterActions::<T>::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<BytecodeInterpreterGcCompleteAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a BytecodeInterpreterGcCompleteAction<T>>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<BytecodeInterpreterResultAction<T>> for CliActions<T> {
    fn from(value: BytecodeInterpreterResultAction<T>) -> Self {
        BytecodeInterpreterActions::<T>::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<BytecodeInterpreterResultAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a BytecodeInterpreterResultAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<BytecodeInterpreterGcAction<T>> for CliActions<T> {
    fn from(value: BytecodeInterpreterGcAction<T>) -> Self {
        BytecodeInterpreterActions::<T>::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<BytecodeInterpreterGcAction<T>> {
    fn from(value: CliActions<T>) -> Self {
        Option::<BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a BytecodeInterpreterGcAction<T>> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a BytecodeInterpreterActions<T>>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<FetchHandlerFetchCompleteAction> for CliActions<T> {
    fn from(value: FetchHandlerFetchCompleteAction) -> Self {
        FetchHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<FetchHandlerFetchCompleteAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<FetchHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a FetchHandlerFetchCompleteAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a FetchHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<FetchHandlerConnectionErrorAction> for CliActions<T> {
    fn from(value: FetchHandlerConnectionErrorAction) -> Self {
        FetchHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<FetchHandlerConnectionErrorAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<FetchHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a FetchHandlerConnectionErrorAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a FetchHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerHttpFetchCompleteAction> for CliActions<T> {
    fn from(value: GraphQlHandlerHttpFetchCompleteAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerHttpFetchCompleteAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerHttpFetchCompleteAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerHttpConnectionErrorAction> for CliActions<T> {
    fn from(value: GraphQlHandlerHttpConnectionErrorAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerHttpConnectionErrorAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerHttpConnectionErrorAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerWebSocketConnectSuccessAction> for CliActions<T> {
    fn from(value: GraphQlHandlerWebSocketConnectSuccessAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerWebSocketConnectSuccessAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerWebSocketConnectSuccessAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerWebSocketClientMessageAction> for CliActions<T> {
    fn from(value: GraphQlHandlerWebSocketClientMessageAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerWebSocketClientMessageAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerWebSocketClientMessageAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerWebSocketServerMessageAction> for CliActions<T> {
    fn from(value: GraphQlHandlerWebSocketServerMessageAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerWebSocketServerMessageAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerWebSocketServerMessageAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerWebSocketConnectionErrorAction> for CliActions<T> {
    fn from(value: GraphQlHandlerWebSocketConnectionErrorAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GraphQlHandlerWebSocketConnectionErrorAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerWebSocketConnectionErrorAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GraphQlHandlerWebSocketConnectionTerminateAction> for CliActions<T> {
    fn from(value: GraphQlHandlerWebSocketConnectionTerminateAction) -> Self {
        GraphQlHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>>
    for Option<GraphQlHandlerWebSocketConnectionTerminateAction>
{
    fn from(value: CliActions<T>) -> Self {
        Option::<GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GraphQlHandlerWebSocketConnectionTerminateAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GraphQlHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<TimeoutHandlerTimeoutAction> for CliActions<T> {
    fn from(value: TimeoutHandlerTimeoutAction) -> Self {
        TimeoutHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<TimeoutHandlerTimeoutAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<TimeoutHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a TimeoutHandlerTimeoutAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a TimeoutHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<TimestampHandlerUpdateAction> for CliActions<T> {
    fn from(value: TimestampHandlerUpdateAction) -> Self {
        TimestampHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<TimestampHandlerUpdateAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<TimestampHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a TimestampHandlerUpdateAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a TimestampHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerConnectSuccessAction> for CliActions<T> {
    fn from(value: GrpcHandlerConnectSuccessAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerConnectSuccessAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerConnectSuccessAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerConnectErrorAction> for CliActions<T> {
    fn from(value: GrpcHandlerConnectErrorAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerConnectErrorAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerConnectErrorAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerRequestStartAction> for CliActions<T> {
    fn from(value: GrpcHandlerRequestStartAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerRequestStartAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerRequestStartAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerRequestStopAction> for CliActions<T> {
    fn from(value: GrpcHandlerRequestStopAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerRequestStopAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerRequestStopAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerSuccessResponseAction> for CliActions<T> {
    fn from(value: GrpcHandlerSuccessResponseAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerSuccessResponseAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerSuccessResponseAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerErrorResponseAction> for CliActions<T> {
    fn from(value: GrpcHandlerErrorResponseAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerErrorResponseAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerErrorResponseAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerTransportErrorAction> for CliActions<T> {
    fn from(value: GrpcHandlerTransportErrorAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerTransportErrorAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerTransportErrorAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerAbortRequestAction> for CliActions<T> {
    fn from(value: GrpcHandlerAbortRequestAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerAbortRequestAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>> for Option<&'a GrpcHandlerAbortRequestAction> {
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}

impl<T: Expression> From<GrpcHandlerConnectionTerminateAction> for CliActions<T> {
    fn from(value: GrpcHandlerConnectionTerminateAction) -> Self {
        GrpcHandlerActions::from(value).into()
    }
}
impl<T: Expression> From<CliActions<T>> for Option<GrpcHandlerConnectionTerminateAction> {
    fn from(value: CliActions<T>) -> Self {
        Option::<GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
impl<'a, T: Expression> From<&'a CliActions<T>>
    for Option<&'a GrpcHandlerConnectionTerminateAction>
{
    fn from(value: &'a CliActions<T>) -> Self {
        Option::<&'a GrpcHandlerActions>::from(value).and_then(|value| value.into())
    }
}
