//! Worker for processing workflow and activity tasks

/// Workflow worker
pub struct WorkflowWorker {
    // Worker implementation will be added later
}

impl WorkflowWorker {
    /// Create a new workflow worker
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WorkflowWorker {
    fn default() -> Self {
        Self::new()
    }
}

/// Worker config
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Task queue
    pub task_queue: String,
    
    /// Maximum concurrent workflow tasks
    pub max_concurrent_workflow_tasks: usize,
    
    /// Maximum concurrent activity tasks
    pub max_concurrent_activity_tasks: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            task_queue: "default".to_string(),
            max_concurrent_workflow_tasks: 100,
            max_concurrent_activity_tasks: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let _worker = WorkflowWorker::new();
    }

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert_eq!(config.task_queue, "default");
        assert_eq!(config.max_concurrent_workflow_tasks, 100);
    }
}

