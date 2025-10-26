//! Temporal-based workflow system
//!
//! This module implements a Temporal-compatible workflow engine in Rust 1.90.
//!
//! ## Architecture
//!
//! The module is organized into several sub-modules:
//! - `types`: Core type definitions (WorkflowId, RunId, etc.)
//! - `workflow`: Workflow trait and execution context
//! - `activity`: Activity trait and execution context
//! - `signal`: Signal definitions and handling
//! - `query`: Query definitions and handling
//! - `client`: Client for starting workflows and sending signals
//! - `worker`: Worker for processing workflow and activity tasks
//! - `storage`: Persistence layer abstraction
//! - `event`: Event sourcing and history
//! - `error`: Error types

pub mod types;
pub mod workflow;
pub mod activity;
pub mod signal;
pub mod query;
pub mod client;
pub mod worker;
pub mod storage;
pub mod event;
pub mod error;

// Re-export commonly used items
pub use self::types::*;
pub use self::workflow::{Workflow, WorkflowContext};
pub use self::activity::{Activity, ActivityContext, ActivityOptions};
pub use self::signal::Signal;
pub use self::query::Query;
pub use self::client::WorkflowClient;
pub use self::worker::WorkflowWorker;
pub use self::error::{WorkflowError, ActivityError};

