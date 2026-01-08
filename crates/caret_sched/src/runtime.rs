// Caret Sched - Runtime
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{executor::Executor, policy::ExecutionPolicy};
use caret_core::Result;
use parking_lot::Mutex;
use std::sync::Arc;

/// State of the runtime
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeState {
    /// Runtime is stopped
    Stopped,
    /// Runtime is running
    Running,
    /// Runtime is paused
    Paused,
    /// Runtime encountered an error
    Error,
    /// Runtime completed successfully
    Completed,
}

/// Handle to a running runtime
///
/// Allows external control and monitoring of the runtime.
#[derive(Clone)]
pub struct RuntimeHandle {
    /// Runtime state
    state: Arc<Mutex<RuntimeState>>,
}

impl RuntimeHandle {
    /// Create a new runtime handle
    pub fn new(state: Arc<Mutex<RuntimeState>>) -> Self {
        Self { state }
    }

    /// Get the current runtime state
    pub fn state(&self) -> RuntimeState {
        *self.state.lock()
    }

    /// Check if the runtime is running
    pub fn is_running(&self) -> bool {
        matches!(self.state(), RuntimeState::Running)
    }

    /// Pause the runtime
    pub fn pause(&self) -> Result<()> {
        *self.state.lock() = RuntimeState::Paused;
        Ok(())
    }

    /// Resume the runtime
    pub fn resume(&self) -> Result<()> {
        *self.state.lock() = RuntimeState::Running;
        Ok(())
    }

    /// Stop the runtime
    pub fn stop(&self) -> Result<()> {
        *self.state.lock() = RuntimeState::Stopped;
        Ok(())
    }
}

/// Caret runtime
///
/// The runtime manages the execution of Caret pipelines.
pub struct Runtime {
    /// Executor for running the graph
    executor: Executor,
    /// Runtime state
    state: Arc<Mutex<RuntimeState>>,
}

impl Runtime {
    /// Create a new runtime with the default configuration
    pub fn new() -> Self {
        Self::with_policy(ExecutionPolicy::default())
    }

    /// Create a new runtime with the specified execution policy
    pub fn with_policy(policy: ExecutionPolicy) -> Self {
        let config = crate::executor::ExecutorConfig {
            policy,
            ..Default::default()
        };

        let state = Arc::new(Mutex::new(RuntimeState::Stopped));
        let executor = Executor::new(config);

        Self { executor, state }
    }

    /// Get the executor
    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    /// Get a mutable reference to the executor
    pub fn executor_mut(&mut self) -> &mut Executor {
        &mut self.executor
    }

    /// Get the runtime state
    pub fn state(&self) -> RuntimeState {
        *self.state.lock()
    }

    /// Get a handle to the runtime
    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(self.state.clone())
    }

    /// Start the runtime
    pub fn start(&mut self) -> Result<()> {
        *self.state.lock() = RuntimeState::Running;
        self.executor.start()
    }

    /// Pause the runtime
    pub fn pause(&mut self) -> Result<()> {
        *self.state.lock() = RuntimeState::Paused;
        self.executor.pause()
    }

    /// Stop the runtime
    pub fn stop(&mut self) -> Result<()> {
        *self.state.lock() = RuntimeState::Stopped;
        self.executor.stop()
    }

    /// Run the runtime to completion
    ///
    /// This will execute ticks until all nodes complete
    /// or an error occurs.
    pub fn run(&mut self) -> Result<RunResult> {
        self.start()?;

        let mut total_ticks = 0;
        let mut total_nodes_run = 0;

        loop {
            match self.executor.tick_once()? {
                crate::executor::TickResult::Skipped => {
                    break;
                }
                crate::executor::TickResult::Executed {
                    nodes_run,
                    nodes_done,
                    nodes_error,
                } => {
                    total_ticks += 1;
                    total_nodes_run += nodes_run;

                    // Check if all nodes are done
                    let node_count = self.executor.node_ids().len();
                    if nodes_done >= node_count {
                        *self.state.lock() = RuntimeState::Completed;
                        break;
                    }

                    // Check for errors
                    if nodes_error > 0 {
                        *self.state.lock() = RuntimeState::Error;
                        return Ok(RunResult {
                            ticks: total_ticks,
                            nodes_run: total_nodes_run,
                            status: crate::executor::TickResult::Executed {
                                nodes_run,
                                nodes_done,
                                nodes_error,
                            },
                        });
                    }
                }
            }

            // Safety valve - don't run forever
            if total_ticks > 10000 {
                *self.state.lock() = RuntimeState::Completed;
                break;
            }
        }

        Ok(RunResult {
            ticks: total_ticks,
            nodes_run: total_nodes_run,
            status: crate::executor::TickResult::Executed {
                nodes_run: 0,
                nodes_done: 0,
                nodes_error: 0,
            },
        })
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a runtime execution
#[derive(Clone, Debug)]
pub struct RunResult {
    /// Total ticks executed
    pub ticks: u64,
    /// Total nodes run
    pub nodes_run: usize,
    /// Final tick status
    pub status: crate::executor::TickResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_create() {
        let runtime = Runtime::new();
        assert_eq!(runtime.state(), RuntimeState::Stopped);
    }

    #[test]
    fn test_runtime_handle() {
        let runtime = Runtime::new();
        let handle = runtime.handle();
        assert_eq!(handle.state(), RuntimeState::Stopped);
        assert!(!handle.is_running());
    }

    #[test]
    fn test_runtime_start_pause_stop() {
        let mut runtime = Runtime::new();

        runtime.start().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Running);

        runtime.pause().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Paused);

        runtime.stop().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Stopped);
    }

    #[test]
    fn test_runtime_with_empty_graph() {
        let mut runtime = Runtime::new();
        let result = runtime.run().unwrap();
        // Empty graph runs one tick (which is skipped), then completes
        assert_eq!(result.ticks, 1);
    }
}
