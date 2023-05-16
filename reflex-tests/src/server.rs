// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use std::{convert::Infallible, future, iter::empty, net::SocketAddr, path::PathBuf, sync::Arc};

use hyper::{server::conn::AddrStream, service::make_service_fn, Server};
use reflex::core::{
    Applicable, Expression, ExpressionFactory, HeapAllocator, InstructionPointer, Reducible,
    Rewritable,
};
use reflex_dispatcher::HandlerContext;
use reflex_grpc::DefaultGrpcConfig;
use reflex_handlers::{actor::HandlerActor, utils::tls::create_https_client};
use reflex_parser::{
    create_parser, syntax::js::default_js_loaders, ParserBuiltin, Syntax, SyntaxParser,
};
use reflex_scheduler::threadpool::TokioRuntimeThreadPoolFactory;
use reflex_server::{
    cli::{
        execute_query::GraphQlWebServerMetricLabels,
        task::{ServerCliTaskActor, ServerCliTaskFactory},
    },
    opentelemetry::trace::noop::NoopTracer,
    scheduler_metrics::{
        NoopServerMetricsSchedulerQueueInstrumentation, ServerMetricsInstrumentation,
    },
    GraphQlWebServerActorFactory,
};
use tokio::sync::oneshot;

use reflex_graphql::NoopGraphQlQueryTransform;
use reflex_handlers::{
    actor::graphql::{GraphQlHandler, GraphQlHandlerMetricNames},
    utils::tls::hyper_rustls,
};
use reflex_interpreter::{
    compiler::{compile_graph_root, Compile, CompiledProgram, CompilerMode, CompilerOptions},
    InterpreterOptions,
};
use reflex_lang::{allocator::DefaultAllocator, CachedSharedTerm, SharedTermFactory};
use reflex_server::{
    action::ServerCliAction, builtins::ServerBuiltins, graphql_service, logger::NoopLogger,
    GraphQlWebServer, GraphQlWebServerMetricNames,
};
use reflex_utils::reconnect::NoopReconnectTimeout;

pub fn serve_graphql(input: &str) -> (SocketAddr, oneshot::Sender<()>) {
    let factory = SharedTermFactory::<ServerBuiltins>::default();
    let allocator = DefaultAllocator::default();
    let https_client = create_https_client(None).unwrap();
    let compiler_options = CompilerOptions::default();
    let interpreter_options = InterpreterOptions::default();
    let graph_root = {
        compile_graphql_module(input, empty(), &factory, &allocator, &compiler_options).unwrap()
    };

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
    let app = GraphQlWebServer::<TAction, TTask>::new(
        graph_root,
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
        compiler_options,
        interpreter_options,
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
    )
    .unwrap();
    let socket_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
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
    let server = Server::bind(&socket_addr).serve(service);
    let (tx, rx) = oneshot::channel();
    let addr = server.local_addr().clone();
    tokio::task::spawn(async move {
        server
            .with_graceful_shutdown(async { rx.await.ok().unwrap() })
            .await
    });
    (addr, tx)
}

fn compile_graphql_module<T: Expression + 'static>(
    graph_definition: &str,
    env_vars: impl IntoIterator<Item = (String, String)>,
    factory: &(impl ExpressionFactory<T> + Clone + 'static),
    allocator: &(impl HeapAllocator<T> + Clone + 'static),
    compiler_options: &CompilerOptions,
) -> Result<(CompiledProgram, InstructionPointer), String>
where
    // TODO: Remove unnecessary trait bounds
    T: Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::Builtin: ParserBuiltin,
{
    let parser = create_parser(
        Syntax::JavaScript,
        Some(PathBuf::from("./index.js").as_path()),
        default_js_loaders(empty(), factory, allocator),
        env_vars,
        factory,
        allocator,
    );
    let expression = parser.parse(graph_definition)?;
    compile_graph_root(
        &expression,
        factory,
        allocator,
        compiler_options,
        CompilerMode::Function,
    )
}
