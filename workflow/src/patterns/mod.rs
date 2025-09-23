//! # 工作流设计模式模块 / Workflow Design Patterns Module
//!
//! 本模块提供了工作流设计模式的实现，包括创建型、结构型、行为型和并发模式
//! This module provides implementations of workflow design patterns including creational, structural, behavioral, and concurrent patterns

pub mod creational;
pub mod structural;
pub mod behavioral;
pub mod concurrent;

// 重新导出主要类型 / Re-export main types
pub use creational::*;
pub use structural::*;
pub use behavioral::*;
pub use concurrent::*;

/// 工作流模式工厂 / Workflow Pattern Factory
pub struct WorkflowPatternFactory {
    patterns: std::collections::HashMap<String, Box<dyn WorkflowPattern>>,
}

/// 工作流模式 trait / Workflow Pattern Trait
pub trait WorkflowPattern: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> PatternCategory;
    fn apply(&self, context: &WorkflowContext) -> Result<WorkflowResult, PatternError>;
    fn validate(&self, context: &WorkflowContext) -> Result<(), PatternError>;
}

/// 模式分类 / Pattern Category
#[derive(Debug, Clone, PartialEq)]
pub enum PatternCategory {
    Creational,
    Structural,
    Behavioral,
    Concurrent,
}

/// 工作流上下文 / Workflow Context
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub workflow_id: String,
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

/// 工作流结果 / Workflow Result
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
}

/// 模式错误 / Pattern Error
#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    #[error("模式应用失败 / Pattern application failed: {0}")]
    ApplicationFailed(String),
    
    #[error("上下文无效 / Invalid context: {0}")]
    InvalidContext(String),
    
    #[error("模式不支持 / Pattern not supported: {0}")]
    PatternNotSupported(String),
}

impl From<String> for PatternError {
    fn from(error: String) -> Self {
        PatternError::ApplicationFailed(error)
    }
}

impl WorkflowPatternFactory {
    pub fn new() -> Self {
        Self {
            patterns: std::collections::HashMap::new(),
        }
    }
    
    pub fn register_pattern(&mut self, name: String, pattern: Box<dyn WorkflowPattern>) {
        self.patterns.insert(name, pattern);
    }
    
    pub fn create_pattern(&self, name: &str, category: PatternCategory) -> Option<&Box<dyn WorkflowPattern>> {
        self.patterns.get(name).filter(|p| p.category() == category)
    }
    
    pub fn get_all_patterns(&self) -> Vec<PatternInfo> {
        self.patterns.values()
            .map(|p| PatternInfo {
                name: p.name().to_string(),
                description: format!("{} pattern", p.name()),
                category: p.category(),
            })
            .collect()
    }
}

/// 模式信息 / Pattern Info
#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub name: String,
    pub description: String,
    pub category: PatternCategory,
}