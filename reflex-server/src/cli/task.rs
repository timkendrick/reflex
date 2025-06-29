// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use opentelemetry::trace::Tracer;
use reflex::core::{Applicable, Expression, Reducible, Rewritable};
use reflex_dispatcher::{Action, Redispatcher};
use reflex_engine::{
    actor::{
        bytecode_interpreter::{BytecodeInterpreterAction, BytecodeInterpreterMetricLabels},
        wasm_interpreter::WasmInterpreter,
    },
    task::wasm_worker::{WasmWorkerTask, WasmWorkerTaskAction, WasmWorkerTaskFactory},
};
use reflex_graphql::GraphQlParserBuiltin;
use reflex_grpc::{
    actor::{GrpcHandler, GrpcHandlerAction},
    task::GrpcHandlerConnectionTaskFactory,
    GrpcConfig,
};
use reflex_handlers::{
    actor::{HandlerAction, HandlerActor, HandlerActorBuiltin},
    task::{
        fetch::FetchHandlerTaskFactory,
        graphql::{
            GraphQlHandlerHttpFetchTaskFactory, GraphQlHandlerWebSocketConnectionTaskFactory,
        },
        timeout::TimeoutHandlerTaskFactory,
        timestamp::TimestampHandlerTaskFactory,
        DefaultHandlersTaskFactory,
    },
};
use reflex_interpreter::compiler::Compile;
use reflex_macros::{blanket_trait, task_factory_enum, Matcher};
use reflex_protobuf::types::WellKnownTypesTranscoder;
use reflex_runtime::{
    task::{evaluate_handler::EffectThrottleTaskFactory, RuntimeTask, RuntimeTaskFactory},
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator,
};
use reflex_utils::reconnect::ReconnectTimeout;

use crate::{
    actor::{ServerAction, ServerActor},
    server::{
        task::websocket_graphql_server::{
            WebSocketGraphQlServerTaskFactory, WebSocketGraphQlServerThrottleTimeoutTaskFactory,
        },
        GraphQlServerOperationMetricLabels, GraphQlServerQueryLabel,
        HttpGraphQlServerQueryMetricLabels, HttpGraphQlServerQueryTransform,
        WebSocketGraphQlServerConnectionMetricLabels, WebSocketGraphQlServerQueryTransform,
    },
    task::{ServerTask, ServerTaskAction, ServerTaskFactory},
    GraphQlWebServerTask,
};

blanket_trait!(
    pub trait ServerCliTaskAction<T: Expression>:
        ServerAction<T>
        + BytecodeInterpreterAction<T>
        + WasmWorkerTaskAction<T>
        + ServerTaskAction
        + HandlerAction<T>
        + GrpcHandlerAction<T>
    {
    }
);

blanket_trait!(
    pub trait ServerCliTask<T, TFactory, TAllocator, TConnect>:
        RuntimeTask
        + WasmWorkerTask<T, TFactory, TAllocator>
        + ServerTask<TConnect>
        + GraphQlWebServerTask<T, TFactory, TAllocator>
    where
        T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
        T::String: Send,
        T::Builtin: Send,
        T::Signal: Send,
        T::SignalList: Send,
        T::StructPrototype: Send,
        T::ExpressionList: Send,
        T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
        TFactory: AsyncExpressionFactory<T> + Default,
        TAllocator: AsyncHeapAllocator<T> + Default,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
    }
);

task_factory_enum!({
    #[derive(Matcher, Clone)]
    pub enum ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
    where
        T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
        T::String: Send,
        T::Builtin: Send,
        T::Signal: Send,
        T::SignalList: Send,
        T::StructPrototype: Send,
        T::ExpressionList: Send,
        T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
        TFactory: AsyncExpressionFactory<T> + Default,
        TAllocator: AsyncHeapAllocator<T> + Default,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
        TReconnect: ReconnectTimeout + Send + Clone + 'static,
        TGrpcConfig: GrpcConfig + Send + 'static,
        TTransformHttp: HttpGraphQlServerQueryTransform,
        TTransformWs: WebSocketGraphQlServerQueryTransform,
        TGraphQlQueryLabel: GraphQlServerQueryLabel,
        THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
        TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
        TOperationMetricLabels: GraphQlServerOperationMetricLabels,
        TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
        TTracer: Tracer,
        TTracer::Span: Send + Sync + 'static,
    {
        Runtime(EffectThrottleTaskFactory),
        ServerTask(ServerTaskFactory<TConnect>),
        Server(
            ServerActor<
                T,
                TFactory,
                TAllocator,
                TTransformHttp,
                TTransformWs,
                TGraphQlQueryLabel,
                THttpMetricLabels,
                TConnectionMetricLabels,
                TOperationMetricLabels,
                TTracer,
            >,
        ),
        WasmInterpreter(WasmInterpreter<T, TFactory, TAllocator, TWorkerMetricLabels>),
        WasmWorkerTask(WasmWorkerTaskFactory<T, TFactory, TAllocator>),
        Handler(HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect>),
        Grpc(
            GrpcHandler<T, TFactory, TAllocator, WellKnownTypesTranscoder, TGrpcConfig, TReconnect>,
        ),
        Main(Redispatcher),
    }

    impl<
            T,
            TFactory,
            TAllocator,
            TConnect,
            TReconnect,
            TGrpcConfig,
            TTransformHttp,
            TTransformWs,
            TGraphQlQueryLabel,
            THttpMetricLabels,
            TConnectionMetricLabels,
            TWorkerMetricLabels,
            TOperationMetricLabels,
            TTracer,
            TAction,
        >
        TaskFactory<
            TAction,
            ServerCliTaskFactory<
                T,
                TFactory,
                TAllocator,
                TConnect,
                TReconnect,
                TGrpcConfig,
                TTransformHttp,
                TTransformWs,
                TGraphQlQueryLabel,
                THttpMetricLabels,
                TConnectionMetricLabels,
                TWorkerMetricLabels,
                TOperationMetricLabels,
                TTracer,
            >,
        >
        for ServerCliTaskFactory<
            T,
            TFactory,
            TAllocator,
            TConnect,
            TReconnect,
            TGrpcConfig,
            TTransformHttp,
            TTransformWs,
            TGraphQlQueryLabel,
            THttpMetricLabels,
            TConnectionMetricLabels,
            TWorkerMetricLabels,
            TOperationMetricLabels,
            TTracer,
        >
    where
        T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
        T::String: Send,
        T::Builtin: Send,
        T::Signal: Send,
        T::SignalList: Send,
        T::StructPrototype: Send,
        T::ExpressionList: Send,
        T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
        TFactory: AsyncExpressionFactory<T> + Default,
        TAllocator: AsyncHeapAllocator<T> + Default,
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
        TReconnect: ReconnectTimeout + Send + Clone + 'static,
        TGrpcConfig: GrpcConfig + Send + 'static,
        TTransformHttp: HttpGraphQlServerQueryTransform,
        TTransformWs: WebSocketGraphQlServerQueryTransform,
        TGraphQlQueryLabel: GraphQlServerQueryLabel,
        THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
        TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
        TOperationMetricLabels: GraphQlServerOperationMetricLabels,
        TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
        TTracer: Tracer,
        TTracer::Span: Send + Sync + 'static,
        TAction: Action + ServerCliTaskAction<T> + Send + 'static,
    {
    }
});

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    >
    From<
        ServerActor<
            T,
            TFactory,
            TAllocator,
            TTransformHttp,
            TTransformWs,
            TGraphQlQueryLabel,
            THttpMetricLabels,
            TConnectionMetricLabels,
            TOperationMetricLabels,
            TTracer,
        >,
    >
    for ServerCliTaskActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
    TAction: Action + ServerCliTaskAction<T> + Send + 'static,
{
    fn from(
        value: ServerActor<
            T,
            TFactory,
            TAllocator,
            TTransformHttp,
            TTransformWs,
            TGraphQlQueryLabel,
            THttpMetricLabels,
            TConnectionMetricLabels,
            TOperationMetricLabels,
            TTracer,
        >,
    ) -> Self {
        Self::Server(value)
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    > From<WasmInterpreter<T, TFactory, TAllocator, TWorkerMetricLabels>>
    for ServerCliTaskActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
    TAction: Action + ServerCliTaskAction<T> + Send + 'static,
{
    fn from(value: WasmInterpreter<T, TFactory, TAllocator, TWorkerMetricLabels>) -> Self {
        Self::WasmInterpreter(value)
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    > From<HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect>>
    for ServerCliTaskActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
    TAction: Action + ServerCliTaskAction<T> + Send + 'static,
{
    fn from(value: HandlerActor<T, TFactory, TAllocator, TConnect, TReconnect>) -> Self {
        Self::Handler(value)
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    > From<GrpcHandler<T, TFactory, TAllocator, WellKnownTypesTranscoder, TGrpcConfig, TReconnect>>
    for ServerCliTaskActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
    TAction: Action + ServerCliTaskAction<T> + Send + 'static,
{
    fn from(
        value: GrpcHandler<
            T,
            TFactory,
            TAllocator,
            WellKnownTypesTranscoder,
            TGrpcConfig,
            TReconnect,
        >,
    ) -> Self {
        Self::Grpc(value)
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    > From<Redispatcher>
    for ServerCliTaskActor<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
        TAction,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
    TAction: Action + ServerCliTaskAction<T> + Send + 'static,
{
    fn from(value: Redispatcher) -> Self {
        Self::Main(value)
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<RuntimeTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: RuntimeTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<DefaultHandlersTaskFactory<TConnect>>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: DefaultHandlersTaskFactory<TConnect>) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<WebSocketGraphQlServerTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: WebSocketGraphQlServerTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<FetchHandlerTaskFactory<TConnect>>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: FetchHandlerTaskFactory<TConnect>) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<GraphQlHandlerHttpFetchTaskFactory<TConnect>>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: GraphQlHandlerHttpFetchTaskFactory<TConnect>) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<GraphQlHandlerWebSocketConnectionTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: GraphQlHandlerWebSocketConnectionTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<TimeoutHandlerTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: TimeoutHandlerTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<TimestampHandlerTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: TimestampHandlerTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<WebSocketGraphQlServerThrottleTimeoutTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: WebSocketGraphQlServerThrottleTimeoutTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}

impl<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    > From<GrpcHandlerConnectionTaskFactory>
    for ServerCliTaskFactory<
        T,
        TFactory,
        TAllocator,
        TConnect,
        TReconnect,
        TGrpcConfig,
        TTransformHttp,
        TTransformWs,
        TGraphQlQueryLabel,
        THttpMetricLabels,
        TConnectionMetricLabels,
        TWorkerMetricLabels,
        TOperationMetricLabels,
        TTracer,
    >
where
    T: AsyncExpression + Rewritable<T> + Reducible<T> + Applicable<T> + Compile<T>,
    T::String: Send,
    T::Builtin: Send,
    T::Signal: Send,
    T::SignalList: Send,
    T::StructPrototype: Send,
    T::ExpressionList: Send,
    T::Builtin: HandlerActorBuiltin + GraphQlParserBuiltin + Into<reflex_wasm::stdlib::Stdlib>,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    TReconnect: ReconnectTimeout + Send + Clone + 'static,
    TGrpcConfig: GrpcConfig + Send + 'static,
    TTransformHttp: HttpGraphQlServerQueryTransform,
    TTransformWs: WebSocketGraphQlServerQueryTransform,
    TGraphQlQueryLabel: GraphQlServerQueryLabel,
    THttpMetricLabels: HttpGraphQlServerQueryMetricLabels,
    TConnectionMetricLabels: WebSocketGraphQlServerConnectionMetricLabels,
    TOperationMetricLabels: GraphQlServerOperationMetricLabels,
    TWorkerMetricLabels: BytecodeInterpreterMetricLabels,
    TTracer: Tracer,
    TTracer::Span: Send + Sync + 'static,
{
    fn from(value: GrpcHandlerConnectionTaskFactory) -> Self {
        Self::ServerTask(ServerTaskFactory::from(value))
    }
}
