// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use reflex_dispatcher::{Action, TaskFactory};
use reflex_grpc::task::{GrpcHandlerConnectionTaskFactory, GrpcHandlerTaskAction};
use reflex_handlers::task::fetch::FetchHandlerTaskFactory;
use reflex_handlers::task::graphql::{
    GraphQlHandlerHttpFetchTaskFactory, GraphQlHandlerWebSocketConnectionTaskFactory,
};
use reflex_handlers::task::timeout::TimeoutHandlerTaskFactory;
use reflex_handlers::task::timestamp::TimestampHandlerTaskFactory;
use reflex_handlers::task::{
    DefaultHandlersTask, DefaultHandlersTaskAction, DefaultHandlersTaskFactory,
};
use reflex_macros::{blanket_trait, task_factory_enum, Matcher};
use reflex_runtime::task::{RuntimeTask, RuntimeTaskAction, RuntimeTaskFactory};

use crate::server::task::websocket_graphql_server::{
    WebSocketGraphQlServerTask, WebSocketGraphQlServerTaskAction,
    WebSocketGraphQlServerTaskFactory, WebSocketGraphQlServerThrottleTimeoutTaskFactory,
};

blanket_trait!(
    pub trait ServerTaskAction:
        RuntimeTaskAction
        + DefaultHandlersTaskAction
        + GrpcHandlerTaskAction
        + WebSocketGraphQlServerTaskAction
    {
    }
);

blanket_trait!(
    pub trait ServerTask<TConnect>:
        RuntimeTask + DefaultHandlersTask<TConnect> + WebSocketGraphQlServerTask
    where
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
    }
);

task_factory_enum!({
    #[derive(Matcher, Clone)]
    pub enum ServerTaskFactory<TConnect>
    where
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        Runtime(RuntimeTaskFactory),
        DefaultHandlers(DefaultHandlersTaskFactory<TConnect>),
        GrpcHandler(GrpcHandlerConnectionTaskFactory),
        WebSocketGraphQlServer(WebSocketGraphQlServerTaskFactory),
    }

    impl<TConnect, TAction, TTask> TaskFactory<TAction, TTask> for ServerTaskFactory<TConnect>
    where
        TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
        TAction: Action + ServerTaskAction + Send + 'static,
        TTask: TaskFactory<TAction, TTask> + ServerTask<TConnect>,
    {
    }
});

impl<TConnect> From<FetchHandlerTaskFactory<TConnect>> for ServerTaskFactory<TConnect>
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn from(value: FetchHandlerTaskFactory<TConnect>) -> Self {
        DefaultHandlersTaskFactory::from(value).into()
    }
}

impl<TConnect> From<GraphQlHandlerHttpFetchTaskFactory<TConnect>> for ServerTaskFactory<TConnect>
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn from(value: GraphQlHandlerHttpFetchTaskFactory<TConnect>) -> Self {
        DefaultHandlersTaskFactory::from(value).into()
    }
}

impl<TConnect> From<GraphQlHandlerWebSocketConnectionTaskFactory> for ServerTaskFactory<TConnect>
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn from(value: GraphQlHandlerWebSocketConnectionTaskFactory) -> Self {
        DefaultHandlersTaskFactory::from(value).into()
    }
}

impl<TConnect> From<TimeoutHandlerTaskFactory> for ServerTaskFactory<TConnect>
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn from(value: TimeoutHandlerTaskFactory) -> Self {
        DefaultHandlersTaskFactory::from(value).into()
    }
}

impl<TConnect> From<TimestampHandlerTaskFactory> for ServerTaskFactory<TConnect>
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn from(value: TimestampHandlerTaskFactory) -> Self {
        DefaultHandlersTaskFactory::from(value).into()
    }
}

impl<TConnect> From<WebSocketGraphQlServerThrottleTimeoutTaskFactory>
    for ServerTaskFactory<TConnect>
where
    TConnect: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn from(value: WebSocketGraphQlServerThrottleTimeoutTaskFactory) -> Self {
        WebSocketGraphQlServerTaskFactory::from(value).into()
    }
}
