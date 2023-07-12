// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{
    fs,
    iter::once,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use metrics_exporter_prometheus::PrometheusBuilder;
use opentelemetry::trace::noop::NoopTracer;
use reflex_dispatcher::{Action, HandlerContext};
use reflex_engine::task::wasm_worker::WasmHeapDumpMode;
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
use reflex_protobuf::types::WellKnownTypesTranscoder;
use reflex_scheduler::threadpool::TokioRuntimeThreadPoolFactory;
use reflex_server::{
    action::ServerCliAction,
    builtins::ServerBuiltins,
    cli::{
        reflex_server::{
            cli, GraphQlWebServerMetricLabels, OpenTelemetryConfig, ReflexServerCliOptions,
        },
        task::{ServerCliTaskActor, ServerCliTaskFactory},
    },
    logger::{
        formatted::FormattedActionLogger, formatter::TimestampedLogFormatter,
        json::JsonActionLogger, messages::DefaultActionFormatter, prometheus::PrometheusLogger,
        ActionLogger, ChainLogger, EitherLogger,
    },
    scheduler_metrics::{
        NoopServerMetricsSchedulerQueueInstrumentation, ServerMetricsInstrumentation,
        ServerSchedulerMetricNames,
    },
    server::action::init::{
        InitHttpServerAction, InitOpenTelemetryAction, InitPrometheusMetricsAction,
    },
    GraphQlWebServerActorFactory, GraphQlWebServerMetricNames,
};
use reflex_server::{
    server::utils::EitherTracer, tokio_runtime_metrics_export::TokioRuntimeMonitorMetricNames,
};
use reflex_utils::reconnect::FibonacciReconnectTimeout;
use reflex_wasm::interpreter::WasmProgram;

/// Launch a GraphQL server for the provided graph root
#[derive(Parser)]
struct Args {
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
    /// Port on which to expose a GraphQL HTTP server
    #[clap(long)]
    port: u16,
    /// Port on which to expose Prometheus HTTP metrics
    #[clap(long)]
    metrics_port: Option<u16>,
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
    dump_heap_snapshot: Option<WasmHeapDumpMode>,
}
impl Into<ReflexServerCliOptions> for Args {
    fn into(self) -> ReflexServerCliOptions {
        ReflexServerCliOptions {
            address: SocketAddr::from(([0, 0, 0, 0], self.port)),
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
pub async fn main() -> Result<()> {
    type TBuiltin = ServerBuiltins;
    type T = CachedSharedTerm<TBuiltin>;
    type TFactory = SharedTermFactory<TBuiltin>;
    type TAllocator = DefaultAllocator<T>;
    type TConnect = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;
    type TReconnect = FibonacciReconnectTimeout;
    type TGrpcConfig = DefaultGrpcConfig;
    type TAction = ServerCliAction<T>;
    type TTask = ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        NoopGraphQlQueryTransform,
        NoopGraphQlQueryTransform,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        EitherTracer<NoopTracer, opentelemetry::sdk::trace::Tracer>,
    >;

    let args = Args::parse();
    let factory: TFactory = SharedTermFactory::<TBuiltin>::default();
    let allocator: TAllocator = DefaultAllocator::default();
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
    let mut logger = args.log.map(|format| match format {
        Some(LogFormat::Json) => {
            EitherLogger::Left(JsonActionLogger::<_, TAction, TTask>::stderr())
        }
        _ => EitherLogger::Right(FormattedActionLogger::<_, _, TAction, TTask>::stderr(
            TimestampedLogFormatter::rfc_3339(DefaultActionFormatter::new(factory.clone())),
        )),
    });
    if let Some(port) = args.metrics_port {
        let address = SocketAddr::from(([0, 0, 0, 0], port));
        log_server_action(
            &mut logger,
            &TAction::from(InitPrometheusMetricsAction { address }),
        );
        PrometheusBuilder::new()
            .with_http_listener(address)
            .install()
            .with_context(|| anyhow!("Failed to initialize Prometheus metrics endpoint"))?;
    }
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
    let dump_heap_snapshot = args.dump_heap_snapshot;
    let schema = if let Some(schema_path) = &args.schema {
        Some(load_graphql_schema(schema_path.as_path())?)
    } else {
        None
    };
    let effect_throttle = args.effect_throttle_ms.map(Duration::from_millis);
    let mut logger = {
        let stdout_logger = logger;
        let prometheus_logger = args
            .metrics_port
            .map(|_| PrometheusLogger::<TAction, TTask>::new(Default::default()));
        ChainLogger::new(stdout_logger, prometheus_logger)
    };
    let tracer = match OpenTelemetryConfig::parse_env(std::env::vars())? {
        None => None,
        Some(config) => {
            log_server_action(
                &mut logger,
                &TAction::from(InitOpenTelemetryAction {
                    config: config.clone(),
                }),
            );
            Some(config.into_tracer()?)
        }
    };
    let metric_names = ServerSchedulerMetricNames::default();
    let config: ReflexServerCliOptions = args.into();
    log_server_action(
        &mut logger,
        &TAction::from(InitHttpServerAction {
            address: config.address,
        }),
    );
    let server =
        cli::<TAction, TTask, T, TFactory, TAllocator, _, _, _, _, _, _, _, _, _, _, _, _, _, _>(
            config,
            wasm_module,
            graph_root_factory_export_name,
            schema,
            GraphQlWebServerActorFactory::new(|context| {
                let reconnect_timeout = FibonacciReconnectTimeout {
                    units: Duration::from_secs(1),
                    max_timeout: Duration::from_secs(30),
                };
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
            NoopGraphQlQueryTransform,
            GraphQlWebServerMetricNames::default(),
            TokioRuntimeMonitorMetricNames::default(),
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
                metric_names,
            ),
            TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current()),
            TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current()),
            effect_throttle,
            dump_heap_snapshot,
        )
        .with_context(|| anyhow!("Server startup failed"))?;
    server.await.with_context(|| anyhow!("Server error"))
}

fn log_server_action<TAction: Action>(
    logger: &mut impl ActionLogger<Action = TAction>,
    action: &TAction,
) {
    logger.log(action)
}

fn load_graphql_schema(path: &Path) -> Result<GraphQlSchema> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to load GraphQL schema: {}", path.to_string_lossy()))?;
    parse_graphql_schema(&source)
        .with_context(|| format!("Failed to load GraphQL schema: {}", path.to_string_lossy()))
}
