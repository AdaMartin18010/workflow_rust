//! Core type definitions
//!
//! This module defines the fundamental types used throughout the Temporal workflow system.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Identifier Types
// ============================================================================

/// Workflow ID - uniquely identifies a workflow
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowId(pub String);

impl WorkflowId {
    /// Create a new workflow ID
    pub fn new(id: impl Into<String>) -> Self {
        WorkflowId(id.into())
    }
    
    /// Generate a random workflow ID
    pub fn generate() -> Self {
        WorkflowId(format!("workflow-{}", Uuid::new_v4()))
    }
    
    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for WorkflowId {
    fn from(s: String) -> Self {
        WorkflowId(s)
    }
}

impl From<&str> for WorkflowId {
    fn from(s: &str) -> Self {
        WorkflowId(s.to_string())
    }
}

/// Run ID - identifies a specific execution of a workflow
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl RunId {
    /// Generate a new run ID
    pub fn generate() -> Self {
        RunId(Uuid::new_v4())
    }
    
    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(RunId(Uuid::parse_str(s)?))
    }
    
    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Activity ID - identifies an activity within a workflow
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityId(pub String);

impl ActivityId {
    pub fn new(id: impl Into<String>) -> Self {
        ActivityId(id.into())
    }
    
    pub fn generate() -> Self {
        ActivityId(format!("activity-{}", Uuid::new_v4()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActivityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Timer ID - identifies a timer within a workflow
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimerId(pub String);

impl TimerId {
    pub fn new(id: impl Into<String>) -> Self {
        TimerId(id.into())
    }
    
    pub fn generate() -> Self {
        TimerId(format!("timer-{}", Uuid::new_v4()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Event ID - sequence number in event history
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl EventId {
    pub fn zero() -> Self {
        EventId(0)
    }
    
    pub fn next(&self) -> Self {
        EventId(self.0 + 1)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Execution Types
// ============================================================================

/// Workflow execution - identifies a specific workflow run
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// Run ID
    pub run_id: RunId,
}

impl WorkflowExecution {
    /// Create a new execution
    pub fn new(workflow_id: WorkflowId) -> Self {
        Self {
            workflow_id,
            run_id: RunId::generate(),
        }
    }
    
    /// Create with specified run ID
    pub fn with_run_id(workflow_id: WorkflowId, run_id: RunId) -> Self {
        Self {
            workflow_id,
            run_id,
        }
    }
}

impl fmt::Display for WorkflowExecution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.workflow_id, self.run_id)
    }
}

// ============================================================================
// Info Types
// ============================================================================

/// Workflow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub workflow_type: String,
    pub workflow_execution: WorkflowExecution,
    pub task_queue: String,
}

/// Activity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityInfo {
    pub activity_id: ActivityId,
    pub activity_type: String,
    pub workflow_execution: WorkflowExecution,
    pub attempt: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_id() {
        let id = WorkflowId::new("test-workflow");
        assert_eq!(id.as_str(), "test-workflow");
        
        let generated = WorkflowId::generate();
        assert!(generated.as_str().starts_with("workflow-"));
    }

    #[test]
    fn test_run_id() {
        let run_id = RunId::generate();
        let s = run_id.to_string();
        let parsed = RunId::parse(&s).unwrap();
        assert_eq!(run_id, parsed);
    }

    #[test]
    fn test_workflow_execution() {
        let workflow_id = WorkflowId::new("test");
        let execution = WorkflowExecution::new(workflow_id.clone());
        
        assert_eq!(execution.workflow_id, workflow_id);
    }
}

