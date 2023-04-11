// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{
    fs,
    iter::once,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use opentelemetry::trace::noop::NoopTracer;
use reflex_dispatcher::HandlerContext;
use reflex_graphql::{parse_graphql_schema, GraphQlSchema, NoopGraphQlQueryTransform};
use reflex_grpc::{
    actor::{GrpcHandler, GrpcHandlerMetricNames},
    load_grpc_services, DefaultGrpcConfig,
};
use reflex_handlers::{
    default_handler_actors,
    utils::tls::{create_https_client, hyper_rustls},
    DefaultHandlerMetricNames,
};
use reflex_lang::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
use reflex_parser::DefaultModuleLoader;
use reflex_protobuf::types::WellKnownTypesTranscoder;
use reflex_scheduler::threadpool::TokioRuntimeThreadPoolFactory;
use reflex_server::{
    action::ServerCliAction,
    builtins::ServerBuiltins,
    cli::{
        execute_query::{
            cli, ExecuteQueryCliOptions, GraphQlWebServerMetricLabels, NoopHttpMiddleware,
        },
        task::{ServerCliTaskActor, ServerCliTaskFactory},
    },
    logger::{
        formatted::FormattedActionLogger, formatter::PrefixedLogFormatter, json::JsonActionLogger,
        messages::DefaultActionFormatter, EitherLogger,
    },
    scheduler_metrics::{
        NoopServerMetricsSchedulerQueueInstrumentation, ServerMetricsInstrumentation,
    },
    server::{utils::EitherTracer, NoopWebSocketGraphQlServerQueryTransform},
    GraphQlWebServerActorFactory, GraphQlWebServerMetricNames,
};
use reflex_server::{
    cli::reflex_server::OpenTelemetryConfig,
    tokio_runtime_metrics_export::TokioRuntimeMonitorMetricNames,
};
use reflex_utils::reconnect::NoopReconnectTimeout;
use reflex_wasm::cli::compile::WasmProgram;

/// Execute a GraphQL query against the provided graph root
#[derive(Parser)]
pub struct Args {
    /// Path to runtime WebAssembly module
    #[clap(long)]
    module: PathBuf,
    /// Whether the provided WebAssembly module has been precompiled via Cranelift
    #[clap(long)]
    precompiled: bool,
    /// Name of graph root entry point function within runtime WebAssembly module
    #[clap(long)]
    entry_point: String,
    /// Path to GraphQL schema SDL
    #[clap(long)]
    schema: Option<PathBuf>,
    /// GraphQL query
    #[clap(long)]
    query: String,
    /// JSON-formatted GraphQL query variables
    #[clap(long)]
    variables: Option<String>,
    /// Paths of compiled gRPC service definition protobufs
    #[clap(long)]
    grpc_service: Vec<PathBuf>,
    /// Throttle stateful effect updates
    #[clap(long)]
    effect_throttle_ms: Option<u64>,
    /// Log runtime actions
    #[clap(long)]
    log: Option<Option<LogFormat>>,
    /// Dump heap snapshots for any queries that return error results
    #[clap(long)]
    dump_query_errors: bool,
}
impl Into<ExecuteQueryCliOptions> for Args {
    fn into(self) -> ExecuteQueryCliOptions {
        ExecuteQueryCliOptions {
            query: self.query,
            variables: self.variables,
            headers: None,
            effect_throttle: self.effect_throttle_ms.map(Duration::from_millis),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LogFormat {
    Json,
}
impl FromStr for LogFormat {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            _ => Err(anyhow!("Unrecognized log format: {}", input)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    type TBuiltin = ServerBuiltins;
    type T = CachedSharedTerm<TBuiltin>;
    type TFactory = SharedTermFactory<TBuiltin>;
    type TAllocator = DefaultAllocator<T>;
    type TLoader = DefaultModuleLoader<T>;
    type TConnect = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;
    type TReconnect = NoopReconnectTimeout;
    type TGrpcConfig = DefaultGrpcConfig;
    type TTransformHttp = NoopGraphQlQueryTransform;
    type TTransformWs = NoopWebSocketGraphQlServerQueryTransform;
    type TMetricLabels = GraphQlWebServerMetricLabels;
    type TTracer = EitherTracer<NoopTracer, opentelemetry::sdk::trace::Tracer>;
    type TAction = ServerCliAction<T>;
    type TTask = ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TMetricLabels,
        TMetricLabels,
        TMetricLabels,
        TMetricLabels,
        TMetricLabels,
        TTracer,
    >;

    let args = Args::parse();
    let wasm_module = fs::read(&args.module).with_context(|| {
        format!(
            "Failed to load WebAssemby module: {}",
            args.module.to_string_lossy()
        )
    })?;
    let wasm_module = if args.precompiled {
        WasmProgram::from_cwasm(wasm_module)
    } else {
        WasmProgram::from_wasm(wasm_module)
    };
    let graph_root_factory_export_name = args.entry_point.clone();
    let schema = if let Some(schema_path) = &args.schema {
        Some(load_graphql_schema(schema_path.as_path())?)
    } else {
        None
    };
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
    let dump_query_errors = args.dump_query_errors;
    let factory: TFactory = SharedTermFactory::<TBuiltin>::default();
    let allocator: TAllocator = DefaultAllocator::default();
    let tracer = match OpenTelemetryConfig::parse_env(std::env::vars())? {
        None => None,
        Some(config) => Some(config.into_tracer()?),
    };
    let logger = args.log.map(|format| match format {
        Some(LogFormat::Json) => {
            EitherLogger::Left(JsonActionLogger::<_, TAction, TTask>::stderr())
        }
        None => EitherLogger::Right(FormattedActionLogger::<_, _, TAction, TTask>::stderr(
            PrefixedLogFormatter::new("server", DefaultActionFormatter::new(factory.clone())),
        )),
    });
    cli::<TAction, TTask, T, TFactory, TAllocator, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _>(
        args.into(),
        wasm_module,
        graph_root_factory_export_name,
        schema,
        GraphQlWebServerActorFactory::new(|context| {
            let reconnect_timeout = NoopReconnectTimeout;
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
                reconnect_timeout,
                DefaultHandlerMetricNames::default(),
                context.pid(),
            )
            .into_iter()
            .map(ServerCliTaskActor::Handler)
            .chain(once(ServerCliTaskActor::Grpc(GrpcHandler::new(
                grpc_services,
                WellKnownTypesTranscoder,
                factory.clone(),
                allocator.clone(),
                reconnect_timeout,
                grpc_max_operations_per_connection,
                grpc_config,
                GrpcHandlerMetricNames::default(),
                context.pid(),
            ))))
            .map(|actor| (context.generate_pid(), actor))
            .collect::<Vec<_>>()
        }),
        &factory,
        &allocator,
        NoopGraphQlQueryTransform,
        NoopWebSocketGraphQlServerQueryTransform,
        NoopHttpMiddleware,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        match tracer {
            None => EitherTracer::Left(NoopTracer::default()),
            Some(tracer) => EitherTracer::Right(tracer),
        },
        logger,
        ServerMetricsInstrumentation::new(
            NoopServerMetricsSchedulerQueueInstrumentation::default(),
            Default::default(),
        ),
        GraphQlWebServerMetricNames::default(),
        TokioRuntimeMonitorMetricNames::default(),
        TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current()),
        TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current()),
        dump_query_errors,
    )
    .await
    .map(|response| println!("{}", response))
}

fn load_graphql_schema(path: &Path) -> Result<GraphQlSchema> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to load GraphQL schema: {}", path.to_string_lossy()))?;
    parse_graphql_schema(&source)
        .with_context(|| format!("Failed to load GraphQL schema: {}", path.to_string_lossy()))
}
