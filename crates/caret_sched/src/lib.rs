// Caret Sched - Scheduling and execution for Caret pipelines
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod executor;
mod node;
mod policy;
mod port;
mod runtime;
mod scheduler;

pub use executor::{Executor, ExecutorConfig, NodeInstance, NodeState, TickResult};
pub use node::{NodeId, NodeProcessor, PassthroughNode, ProcessingContext, ProcessingResult};
pub use policy::{ExecutionPolicy, NodePriority, SchedulingDecision, SchedulingRequest};
pub use port::{InputPort, OutputPort, PortConnection, PortSet};
pub use runtime::{RunResult, Runtime, RuntimeHandle, RuntimeState};
pub use scheduler::{
    Scheduler, SchedulingChoice, PriorityScheduler, FairScheduler, DeadlineScheduler,
    WorkStealingScheduler, WorkStealingConfig, SchedulerConfig,
};
