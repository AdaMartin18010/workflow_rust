//! Error types for the Temporal workflow system

use std::fmt;
use std::error::Error;

/// Workflow error type
#[derive(Debug)]
pub enum WorkflowError {
    /// Activity execution failed
    ActivityFailed(String),
    
    /// Child workflow failed
    ChildWorkflowFailed(String),
    
    /// Timeout occurred
    Timeout(String),
    
    /// Workflow was cancelled
    Cancelled,
    
    /// Signal channel closed
    SignalChannelClosed,
    
    /// Invalid input
    InvalidInput(String),
    
    /// Storage error
    StorageError(String),
    
    /// Serialization error
    SerializationError(String),
    
    /// Custom error
    Custom(String),
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowError::ActivityFailed(msg) => write!(f, "Activity failed: {}", msg),
            WorkflowError::ChildWorkflowFailed(msg) => write!(f, "Child workflow failed: {}", msg),
            WorkflowError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            WorkflowError::Cancelled => write!(f, "Workflow cancelled"),
            WorkflowError::SignalChannelClosed => write!(f, "Signal channel closed"),
            WorkflowError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            WorkflowError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            WorkflowError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            WorkflowError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for WorkflowError {}

/// Activity error type
#[derive(Debug)]
pub enum ActivityError {
    /// Temporary failure (will be retried)
    TemporaryFailure(String),
    
    /// Validation failed (will not be retried)
    ValidationFailed(String),
    
    /// Execution failed
    ExecutionFailed(String),
    
    /// Activity was cancelled
    Cancelled,
    
    /// Timeout occurred
    Timeout,
    
    /// Heartbeat failed
    HeartbeatFailed(String),
    
    /// Invalid input
    InvalidInput(String),
    
    /// Custom error
    Custom(String),
}

impl fmt::Display for ActivityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityError::TemporaryFailure(msg) => write!(f, "Temporary failure: {}", msg),
            ActivityError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ActivityError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ActivityError::Cancelled => write!(f, "Activity cancelled"),
            ActivityError::Timeout => write!(f, "Activity timeout"),
            ActivityError::HeartbeatFailed(msg) => write!(f, "Heartbeat failed: {}", msg),
            ActivityError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ActivityError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ActivityError {}

/// Signal error type
#[derive(Debug)]
pub enum SignalError {
    /// Workflow not found
    WorkflowNotFound,
    
    /// Signal not registered
    SignalNotRegistered(String),
    
    /// Serialization error
    SerializationError(String),
    
    /// Custom error
    Custom(String),
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalError::WorkflowNotFound => write!(f, "Workflow not found"),
            SignalError::SignalNotRegistered(name) => write!(f, "Signal not registered: {}", name),
            SignalError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SignalError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for SignalError {}

/// Query error type
#[derive(Debug)]
pub enum QueryError {
    /// Workflow not found
    WorkflowNotFound,
    
    /// Query not registered
    QueryNotRegistered(String),
    
    /// Serialization error
    SerializationError(String),
    
    /// Workflow not running
    WorkflowNotRunning,
    
    /// Custom error
    Custom(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::WorkflowNotFound => write!(f, "Workflow not found"),
            QueryError::QueryNotRegistered(name) => write!(f, "Query not registered: {}", name),
            QueryError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            QueryError::WorkflowNotRunning => write!(f, "Workflow not running"),
            QueryError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for QueryError {}

/// Storage error type
#[derive(Debug)]
pub enum StorageError {
    /// Connection error
    ConnectionError(String),
    
    /// Query execution error
    QueryError(String),
    
    /// Serialization error
    SerializationError(String),
    
    /// Not found
    NotFound,
    
    /// Custom error
    Custom(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            StorageError::QueryError(msg) => write!(f, "Query error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::NotFound => write!(f, "Not found"),
            StorageError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for StorageError {}

