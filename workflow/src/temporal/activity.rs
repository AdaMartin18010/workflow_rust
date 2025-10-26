//! Activity definitions and execution context

use std::future::Future;
use std::time::Duration;
use serde::{Serialize, de::DeserializeOwned};
use super::{ActivityId, WorkflowExecution, ActivityError};

/// Activity trait - defines the activity interface
pub trait Activity: Send + Sync + 'static {
    /// Input type
    type Input: DeserializeOwned + Send + 'static;
    
    /// Output type
    type Output: Serialize + Send + 'static;
    
    /// Activity name
    fn name() -> &'static str;
    
    /// Execute the activity
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send;
}

/// Activity context - provides activity execution environment
#[derive(Clone)]
pub struct ActivityContext {
    activity_id: ActivityId,
    workflow_execution: WorkflowExecution,
    // Additional fields will be added as implementation progresses
}

impl ActivityContext {
    /// Create a new activity context
    pub fn new(activity_id: ActivityId, workflow_execution: WorkflowExecution) -> Self {
        Self {
            activity_id,
            workflow_execution,
        }
    }
    
    /// Get activity ID
    pub fn activity_id(&self) -> &ActivityId {
        &self.activity_id
    }
    
    /// Get workflow execution
    pub fn workflow_execution(&self) -> &WorkflowExecution {
        &self.workflow_execution
    }
    
    /// Record heartbeat
    pub async fn heartbeat(&self) -> Result<(), ActivityError> {
        // Placeholder implementation
        Ok(())
    }
    
    /// Record heartbeat with details
    pub async fn heartbeat_with_details<T: Serialize>(
        &self,
        _details: T,
    ) -> Result<(), ActivityError> {
        // Placeholder implementation
        Ok(())
    }
    
    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        // Placeholder implementation
        false
    }
}

/// Activity options
#[derive(Debug, Clone)]
pub struct ActivityOptions {
    /// Activity ID
    pub activity_id: Option<ActivityId>,
    
    /// Task queue
    pub task_queue: Option<String>,
    
    /// Schedule to start timeout
    pub schedule_to_start_timeout: Option<Duration>,
    
    /// Start to close timeout
    pub start_to_close_timeout: Option<Duration>,
    
    /// Schedule to close timeout
    pub schedule_to_close_timeout: Option<Duration>,
    
    /// Heartbeat timeout
    pub heartbeat_timeout: Option<Duration>,
    
    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
}

impl Default for ActivityOptions {
    fn default() -> Self {
        Self {
            activity_id: None,
            task_queue: None,
            schedule_to_start_timeout: Some(Duration::from_secs(60)),
            start_to_close_timeout: Some(Duration::from_secs(300)),
            schedule_to_close_timeout: None,
            heartbeat_timeout: Some(Duration::from_secs(30)),
            retry_policy: Some(RetryPolicy::default()),
        }
    }
}

/// Retry policy
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts
    pub max_attempts: u32,
    
    /// Initial retry interval
    pub initial_interval: Duration,
    
    /// Maximum retry interval
    pub max_interval: Duration,
    
    /// Backoff coefficient
    pub backoff_coefficient: f64,
    
    /// Non-retryable error types
    pub non_retryable_error_types: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(100),
            backoff_coefficient: 2.0,
            non_retryable_error_types: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::{WorkflowId, RunId};

    #[test]
    fn test_activity_context_creation() {
        let activity_id = ActivityId::new("test-activity");
        let workflow_id = WorkflowId::new("test-workflow");
        let run_id = RunId::generate();
        let execution = WorkflowExecution::with_run_id(workflow_id, run_id);
        
        let ctx = ActivityContext::new(activity_id.clone(), execution);
        
        assert_eq!(ctx.activity_id(), &activity_id);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff_coefficient, 2.0);
    }
}

