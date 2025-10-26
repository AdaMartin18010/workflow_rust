//! Workflow definitions and execution context

use std::future::Future;
use serde::{Serialize, de::DeserializeOwned};
use super::{WorkflowExecution, WorkflowError, ActivityOptions, Activity, ActivityError};

/// Workflow trait - defines the workflow interface
pub trait Workflow: Send + Sync + 'static {
    /// Input type
    type Input: DeserializeOwned + Send + 'static;
    
    /// Output type
    type Output: Serialize + Send + 'static;
    
    /// Workflow name
    fn name() -> &'static str;
    
    /// Execute the workflow
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send;
}

/// Workflow context - provides workflow execution environment
#[derive(Clone)]
pub struct WorkflowContext {
    execution: WorkflowExecution,
    // Additional fields will be added as implementation progresses
}

impl WorkflowContext {
    /// Create a new workflow context
    pub fn new(execution: WorkflowExecution) -> Self {
        Self { execution }
    }
    
    /// Get workflow execution
    pub fn execution(&self) -> &WorkflowExecution {
        &self.execution
    }
    
    /// Execute an activity
    pub async fn execute_activity<A: Activity>(
        &self,
        input: A::Input,
        _options: ActivityOptions,
    ) -> Result<A::Output, WorkflowError> {
        // Placeholder implementation
        // In actual implementation, this would:
        // 1. Schedule activity task
        // 2. Wait for completion
        // 3. Return result
        todo!("Activity execution not yet implemented")
    }
    
    /// Sleep for a duration
    pub async fn sleep(&self, _duration: std::time::Duration) {
        // Placeholder implementation
        // In actual implementation, this would use a durable timer
        todo!("Sleep not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::WorkflowId;

    #[test]
    fn test_workflow_context_creation() {
        let workflow_id = WorkflowId::new("test");
        let execution = WorkflowExecution::new(workflow_id);
        let ctx = WorkflowContext::new(execution.clone());
        
        assert_eq!(ctx.execution(), &execution);
    }
}

