//! Workflow client for starting workflows and sending signals

use super::{WorkflowId, WorkflowExecution};

/// Workflow client
pub struct WorkflowClient {
    // Client implementation will be added later
}

impl WorkflowClient {
    /// Create a new workflow client
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WorkflowClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Start workflow options
#[derive(Debug, Clone)]
pub struct StartWorkflowOptions {
    /// Workflow ID (if None, will be generated)
    pub workflow_id: Option<WorkflowId>,
    
    /// Task queue
    pub task_queue: String,
    
    /// Workflow execution timeout
    pub workflow_execution_timeout: Option<std::time::Duration>,
    
    /// Workflow run timeout
    pub workflow_run_timeout: Option<std::time::Duration>,
    
    /// Workflow task timeout
    pub workflow_task_timeout: Option<std::time::Duration>,
}

impl Default for StartWorkflowOptions {
    fn default() -> Self {
        Self {
            workflow_id: None,
            task_queue: "default".to_string(),
            workflow_execution_timeout: None,
            workflow_run_timeout: None,
            workflow_task_timeout: Some(std::time::Duration::from_secs(10)),
        }
    }
}

/// Workflow handle
pub struct WorkflowHandle<O> {
    execution: WorkflowExecution,
    _phantom: std::marker::PhantomData<O>,
}

impl<O> WorkflowHandle<O> {
    /// Create a new workflow handle
    pub fn new(execution: WorkflowExecution) -> Self {
        Self {
            execution,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Get workflow execution
    pub fn execution(&self) -> &WorkflowExecution {
        &self.execution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = WorkflowClient::new();
    }

    #[test]
    fn test_start_workflow_options_default() {
        let options = StartWorkflowOptions::default();
        assert_eq!(options.task_queue, "default");
    }
}

