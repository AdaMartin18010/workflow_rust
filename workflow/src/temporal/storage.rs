//! Storage abstraction for workflow persistence

use async_trait::async_trait;
use super::{WorkflowId, WorkflowExecution, event::EventHistory, error::StorageError};

/// Workflow storage trait
#[async_trait]
pub trait WorkflowStorage: Send + Sync {
    /// Save workflow execution
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        history: &EventHistory,
    ) -> Result<(), StorageError>;
    
    /// Load workflow execution
    async fn load_workflow_execution(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<(WorkflowExecution, EventHistory), StorageError>;
}

/// In-memory storage (for testing)
pub struct InMemoryStorage;

#[async_trait]
impl WorkflowStorage for InMemoryStorage {
    async fn save_workflow_execution(
        &self,
        _execution: &WorkflowExecution,
        _history: &EventHistory,
    ) -> Result<(), StorageError> {
        // Placeholder implementation
        Ok(())
    }
    
    async fn load_workflow_execution(
        &self,
        _workflow_id: &WorkflowId,
    ) -> Result<(WorkflowExecution, EventHistory), StorageError> {
        // Placeholder implementation
        Err(StorageError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage;
        let workflow_id = WorkflowId::new("test");
        let result = storage.load_workflow_execution(&workflow_id).await;
        
        assert!(result.is_err());
    }
}

