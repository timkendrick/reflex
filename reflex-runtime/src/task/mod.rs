// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
use reflex_dispatcher::{Action, TaskFactory};
use reflex_macros::{blanket_trait, task_factory_enum, Matcher};

use crate::task::evaluate_handler::{
    EffectThrottleTaskFactory, EvaluateHandlerTask, EvaluateHandlerTaskAction,
};

pub mod bytecode_worker;
pub mod evaluate_handler;

blanket_trait!(
    pub trait RuntimeTaskAction: EvaluateHandlerTaskAction {}
);

blanket_trait!(
    pub trait RuntimeTask: EvaluateHandlerTask {}
);

// TODO: Implement Serialize/Deserialize traits for RuntimeTaskFactory
task_factory_enum!({
    #[derive(Matcher, Clone)]
    pub enum RuntimeTaskFactory {
        EvaluateHandler(EffectThrottleTaskFactory),
    }
    impl<TAction, TTask> TaskFactory<TAction, TTask> for RuntimeTaskFactory
    where
        TAction: Action + RuntimeTaskAction + Send + 'static,
        TTask: TaskFactory<TAction, TTask>,
    {
    }
});
