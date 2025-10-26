//! Event sourcing and history

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use super::{EventId, ActivityId};

/// Event history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHistory {
    events: Vec<WorkflowEvent>,
}

impl EventHistory {
    /// Create a new empty event history
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    
    /// Add an event to the history
    pub fn add_event(&mut self, event: WorkflowEvent) {
        self.events.push(event);
    }
    
    /// Get all events
    pub fn events(&self) -> &[WorkflowEvent] {
        &self.events
    }
    
    /// Get the number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Event ID
    pub event_id: EventId,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event type
    pub event_type: EventType,
}

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Workflow execution started
    WorkflowExecutionStarted {
        workflow_type: String,
        input: serde_json::Value,
    },
    
    /// Workflow execution completed
    WorkflowExecutionCompleted {
        result: serde_json::Value,
    },
    
    /// Workflow execution failed
    WorkflowExecutionFailed {
        failure: String,
    },
    
    /// Activity task scheduled
    ActivityTaskScheduled {
        activity_id: ActivityId,
        activity_type: String,
        input: serde_json::Value,
    },
    
    /// Activity task started
    ActivityTaskStarted {
        activity_id: ActivityId,
    },
    
    /// Activity task completed
    ActivityTaskCompleted {
        activity_id: ActivityId,
        result: serde_json::Value,
    },
    
    /// Activity task failed
    ActivityTaskFailed {
        activity_id: ActivityId,
        failure: String,
    },
    
    /// Timer started
    TimerStarted {
        timer_id: String,
        duration_ms: u64,
    },
    
    /// Timer fired
    TimerFired {
        timer_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_history() {
        let mut history = EventHistory::new();
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
        
        let event = WorkflowEvent {
            event_id: EventId::zero(),
            timestamp: Utc::now(),
            event_type: EventType::WorkflowExecutionStarted {
                workflow_type: "TestWorkflow".to_string(),
                input: serde_json::json!({}),
            },
        };
        
        history.add_event(event);
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());
    }
}

