// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::RefCell,
    convert::Infallible,
    future,
    iter::empty,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::{Deref, DerefMut},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use hyper::{server::conn::AddrStream, service::make_service_fn, Server};
use reflex::{
    cache::SubstitutionCache,
    core::{Expression, ExpressionFactory, HeapAllocator, Reducible, Rewritable},
};
use reflex_dispatcher::HandlerContext;
use reflex_graphql::{imports::GraphQlImportsBuiltin, NoopGraphQlQueryTransform};
use reflex_grpc::DefaultGrpcConfig;
use reflex_handlers::actor::graphql::{GraphQlHandler, GraphQlHandlerMetricNames};
use reflex_handlers::{
    actor::HandlerActor,
    imports::HandlerImportsBuiltin,
    utils::tls::{create_https_client, hyper_rustls},
};
use reflex_js::{globals::JsGlobalsBuiltin, imports::JsImportsBuiltin, JsParserBuiltin};
use reflex_lang::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
use reflex_parser::{create_parser, syntax::js::default_js_loaders, Syntax, SyntaxParser};
use reflex_scheduler::threadpool::TokioRuntimeThreadPoolFactory;
use reflex_server::{
    action::ServerCliAction, builtins::ServerBuiltins, graphql_service, logger::NoopLogger,
    GraphQlWebServer, GraphQlWebServerMetricNames,
};
use reflex_server::{
    cli::{
        execute_query::GraphQlWebServerMetricLabels,
        task::{ServerCliTaskActor, ServerCliTaskFactory},
    },
    opentelemetry::trace::noop::NoopTracer,
    scheduler_metrics::{
        NoopServerMetricsSchedulerQueueInstrumentation, ServerMetricsInstrumentation,
    },
    GraphQlWebServerActorFactory, WasmHeapDumpEvaluationType, WasmHeapDumpMode,
    WasmHeapDumpResultType,
};
use reflex_utils::reconnect::NoopReconnectTimeout;
use reflex_wasm::{
    allocator::ArenaAllocator,
    cli::compile::WasmCompilerOptions,
    interpreter::WasmProgram,
    term_type::{LambdaTerm, TermType, TypedTerm},
    ArenaRef, Term,
};
use reflex_wasm::{
    allocator::VecAllocator, cli::compile::compile_module, factory::WasmTermFactory,
};
use tokio::sync::oneshot;

const RUNTIME_BYTES: &[u8] = include_bytes!("../../reflex-wasm/build/runtime.wasm");

#[derive(Debug)]
pub enum WasmTestError<T: Expression> {
    Parser(reflex_js::parser::ParserError),
    TranspileError(T),
    Compiler(reflex_wasm::cli::compile::WasmCompilerError),
    Server(String),
}

impl<T: Expression> std::error::Error for WasmTestError<T> {}

impl<T: Expression> std::fmt::Display for WasmTestError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parser(err) => write!(f, "Failed to parse graph definition: {err}"),
            Self::TranspileError(value) => write!(f, "Failed to translate graph node: {value}"),
            Self::Compiler(err) => write!(f, "Failed to compile graph definition: {err}"),
            Self::Server(err) => write!(f, "Failed to launch GraphQL server: {err}"),
        }
    }
}

pub fn serve_graphql(
    graph_definition: &str,
) -> Result<(SocketAddr, oneshot::Sender<()>), WasmTestError<CachedSharedTerm<ServerBuiltins>>> {
    let factory = SharedTermFactory::<ServerBuiltins>::default();
    let allocator = DefaultAllocator::default();
    let https_client = create_https_client(None).unwrap();
    let entry_point_export_name = "__graphql_root__";
    let wasm_module = compile_graphql_module(
        entry_point_export_name,
        graph_definition,
        &factory,
        &allocator,
    )?;

    type TBuiltin = ServerBuiltins;
    type T = CachedSharedTerm<TBuiltin>;
    type TFactory = SharedTermFactory<TBuiltin>;
    type TAllocator = DefaultAllocator<T>;
    type TConnect = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;
    type TReconnect = NoopReconnectTimeout;
    type TGrpcConfig = DefaultGrpcConfig;
    type TTracer = NoopTracer;
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
        TTracer,
    >;
    let tracer = NoopTracer::default();
    let logger = NoopLogger::default();
    let instrumentation = ServerMetricsInstrumentation::new(
        NoopServerMetricsSchedulerQueueInstrumentation::default(),
        Default::default(),
    );
    let async_tasks = TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current());
    let blocking_tasks = TokioRuntimeThreadPoolFactory::new(tokio::runtime::Handle::current());
    let dump_heap_snapshot = Some(WasmHeapDumpMode::new(
        WasmHeapDumpEvaluationType::Query,
        WasmHeapDumpResultType::Error,
    ));
    let app = GraphQlWebServer::<TAction, TTask>::new(
        WasmProgram::from_wasm(wasm_module),
        entry_point_export_name,
        None,
        {
            let factory = factory.clone();
            let allocator = allocator.clone();
            GraphQlWebServerActorFactory::new(move |context| {
                [(
                    context.generate_pid(),
                    ServerCliTaskActor::from(HandlerActor::GraphQlHandler(GraphQlHandler::new(
                        https_client,
                        factory,
                        allocator,
                        NoopReconnectTimeout {},
                        GraphQlHandlerMetricNames::default(),
                        context.pid(),
                    ))),
                )]
            })
        },
        factory,
        allocator,
        NoopGraphQlQueryTransform,
        NoopGraphQlQueryTransform,
        GraphQlWebServerMetricNames::default(),
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        GraphQlWebServerMetricLabels,
        tracer,
        logger,
        instrumentation.clone(),
        async_tasks,
        blocking_tasks,
        None,
        dump_heap_snapshot,
    )
    .map_err(WasmTestError::Server)?;
    let service = make_service_fn({
        let main_pid = app.main_pid();
        let app = Arc::new(app);
        let instrumentation = instrumentation.clone();
        move |_socket: &AddrStream| {
            let app = Arc::clone(&app);
            let service = graphql_service::<TAction>(app, main_pid, instrumentation.clone());
            future::ready(Ok::<_, Infallible>(service))
        }
    });
    let socket_addr = SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 0);
    let server = Server::bind(&socket_addr).serve(service);
    let (tx, rx) = oneshot::channel();
    let addr = server.local_addr().clone();
    tokio::task::spawn(async move {
        server
            .with_graceful_shutdown(async { rx.await.ok().unwrap() })
            .await
    });
    Ok((addr, tx))
}

fn compile_graphql_module<T: Expression + 'static>(
    export_name: &str,
    graph_definition: &str,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
) -> Result<Vec<u8>, WasmTestError<T>>
where
    T: Rewritable<T> + Reducible<T>,
    T::Builtin: JsParserBuiltin
        + JsGlobalsBuiltin
        + JsImportsBuiltin
        + HandlerImportsBuiltin
        + GraphQlImportsBuiltin
        + Into<reflex_wasm::stdlib::Stdlib>,
{
    let parser = create_parser(
        Syntax::JavaScript,
        Some(PathBuf::from("./index.js").as_path()),
        default_js_loaders(empty(), factory, allocator),
        empty(),
        factory,
        allocator,
    );
    let graph_root = parser
        .parse(graph_definition)
        .map_err(WasmTestError::Parser)?;

    // Abstract any free variables from any internal lambda functions within the expression
    let graph_root = graph_root
        .hoist_free_variables(factory, allocator)
        .unwrap_or(graph_root);

    // Partially-evaluate any pure expressions within the expression
    let graph_root = graph_root
        .normalize(factory, allocator, &mut SubstitutionCache::new())
        .unwrap_or(graph_root);

    let mut arena = VecAllocator::default();
    let arena = Rc::new(RefCell::new(&mut arena));
    let graph_root = WasmTermFactory::from(Rc::clone(&arena))
        .import(&graph_root, factory)
        .map_err(WasmTestError::TranspileError)?;
    let graph_factory = {
        let term = Term::new(
            TermType::Lambda(LambdaTerm {
                num_args: 0,
                body: graph_root.as_pointer(),
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

    let wasm_module = compile_module(
        [(String::from(export_name), graph_factory)],
        RUNTIME_BYTES,
        None,
        &WasmCompilerOptions::default(),
        true,
    )
    .map_err(WasmTestError::Compiler)?;
    Ok(wasm_module)
}
