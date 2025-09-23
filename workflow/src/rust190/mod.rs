//! # Rust 1.90 特性模块 / Rust 1.90 Features Module
//!
//! 本模块集成了 Rust 1.90 版本的新特性和改进，包括：
//! This module integrates new features and improvements from Rust 1.90, including:
//!
//! - **JIT 编译器改进** - 更高效的迭代器操作和内存分配
//! - **JIT Compiler Improvements** - More efficient iterator operations and memory allocation
//! - **const 特性增强** - 在 const 上下文中引用非静态变量
//! - **Enhanced const Features** - Reference non-static variables in const contexts
//! - **稳定 API** - BufRead::skip_while, ControlFlow, DebugList::finish_non_exhaustive
//! - **Stable APIs** - BufRead::skip_while, ControlFlow, DebugList::finish_non_exhaustive
//! - **异步迭代器改进** - 更高效的异步流处理
//! - **Async Iterator Improvements** - More efficient async stream processing
//! - **类型检查器优化** - 减少大型代码库的编译时间
//! - **Type Checker Optimizations** - Reduced compilation time for large codebases

pub mod features;
pub mod async_features;
pub mod const_features;
pub mod stable_apis;
pub mod performance;
pub mod session_types;

// 重新导出主要特性 / Re-export main features
// 注意：避免使用 glob 重新导出以防止类型名称冲突
// Note: Avoid glob re-exports to prevent type name conflicts

// 解决类型冲突，使用明确的类型别名 / Resolve type conflicts with explicit type aliases
pub use features::{
    JITOptimizedProcessor, SmallObjectManager, TypeCheckerOptimized,
    Rust190WorkflowEngine, WorkflowResult, ObjectStats, CompilationStats,
    ModuleInfo, SmallObject,
};

pub use async_features::{
    AsyncData, AsyncStreamProcessor, HighPerformanceStreamProcessor,
    AsyncWorkflowEngine as AsyncWorkflowEngine190,
    WorkflowDefinition as AsyncWorkflowDefinition,
    WorkflowStep as AsyncWorkflowStep,
};

pub use performance::{
    PerformanceMonitor, PerformanceMetrics, OverallPerformanceStats,
    HighPerformanceWorkflowEngine as HighPerformanceWorkflowEngine190,
    WorkflowDefinition as PerformanceWorkflowDefinition,
    WorkflowStep as PerformanceWorkflowStep,
    ExecutionStatus as PerformanceExecutionStatus,
    PerformanceBenchmark, BenchmarkData, BenchmarkResult,
};

pub use stable_apis::{
    BufReadProcessor, ControlFlowProcessor, DebugListProcessor,
    StableAPIWorkflowEngine as StableAPIWorkflowEngine190,
    WorkflowDefinition as StableWorkflowDefinition,
    WorkflowStep as StableWorkflowStep,
    WorkflowConfig as StableWorkflowConfig,
};

pub use const_features::{
    ConstContextProcessor, ConstWorkflowEngine, ConstWorkflowStep,
    WorkflowConfig as ConstWorkflowConfig,
    ExecutionStatus as ConstExecutionStatus,
};

#[cfg(feature = "session_types")]
pub use session_types::{
    SessionTypesWorkflowEngine, WorkflowSession, Participant, ParticipantRole,
    SessionProtocol, SessionState, SessionManager, SessionMessage, MessageContent,
    SessionTypesWorkflow, WorkflowProtocol, WorkflowStep, SessionTypesMonitor, SessionMetrics,
};

/// Rust 1.90 特性版本信息 / Rust 1.90 Features Version Info
pub const RUST_190_VERSION: &str = "1.90.0";

/// 初始化 Rust 1.90 特性模块 / Initialize Rust 1.90 Features Module
pub fn init() -> Result<(), crate::error::WorkflowError> {
    println!("Rust 1.90 特性模块已初始化 / Rust 1.90 Features Module Initialized");
    Ok(())
}
