//! # 工作流中间件模块 / Workflow Middleware Module
//!
//! 本模块提供了工作流中间件系统，包括认证、授权、日志、监控等功能
//! This module provides a workflow middleware system including authentication, authorization, logging, monitoring, etc.

pub mod core;
pub mod extensions;
pub mod plugins;

// 重新导出主要类型 / Re-export main types
pub use core::*;
pub use extensions::*;
pub use plugins::*;

/// 中间件管理器 / Middleware Manager
pub struct WorkflowMiddlewareManager {
    middlewares: Vec<std::sync::Arc<dyn WorkflowMiddleware>>,
}

/// 中间件优先级 / Middleware Priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MiddlewarePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

impl Default for MiddlewarePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 工作流中间件 trait / Workflow Middleware Trait
#[async_trait::async_trait]
pub trait WorkflowMiddleware: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn priority(&self) -> MiddlewarePriority;
    
    async fn before_request(&self, context: &mut MiddlewareContext) -> Result<(), String>;
    async fn after_request(&self, context: &mut MiddlewareContext) -> Result<(), String>;
    async fn handle_error(&self, context: &mut MiddlewareContext, error: &str) -> Result<(), String>;
}

/// 中间件上下文 / Middleware Context
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    pub request_id: String,
    pub workflow_id: String,
    pub data: serde_json::Value,
    pub start_time: std::time::Instant,
    pub headers: std::collections::HashMap<String, String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl MiddlewareContext {
    pub fn new(request_id: String, workflow_id: String, data: serde_json::Value) -> Self {
        Self {
            request_id,
            workflow_id,
            data,
            start_time: std::time::Instant::now(),
            headers: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
    
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// 中间件错误 / Middleware Error
#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("认证失败 / Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("授权失败 / Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("中间件处理错误 / Middleware processing error: {0}")]
    ProcessingError(String),
}

impl WorkflowMiddlewareManager {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }
    
    pub fn register_middleware(&mut self, middleware: Box<dyn WorkflowMiddleware>) {
        self.middlewares.push(std::sync::Arc::from(middleware));
    }
    
    pub async fn create_chain(&self, context: MiddlewareContext) -> Result<MiddlewareChain, MiddlewareError> {
        // 按优先级排序中间件 / Sort middlewares by priority
        let mut sorted_middlewares = self.middlewares.clone();
        sorted_middlewares.sort_by(|a, b| a.priority().cmp(&b.priority()));
        
        Ok(MiddlewareChain {
            middlewares: sorted_middlewares,
            context,
        })
    }
}

/// 中间件链 / Middleware Chain
pub struct MiddlewareChain {
    middlewares: Vec<std::sync::Arc<dyn WorkflowMiddleware>>,
    context: MiddlewareContext,
}

impl MiddlewareChain {
    pub async fn execute(&mut self) -> Result<MiddlewareContext, MiddlewareError> {
        // 执行 before_request 阶段 / Execute before_request phase
        for middleware in &self.middlewares {
            if let Err(e) = middleware.before_request(&mut self.context).await {
                // 处理错误 / Handle error
                for error_middleware in &self.middlewares {
                    let _ = error_middleware.handle_error(&mut self.context, &e).await;
                }
                return Err(MiddlewareError::ProcessingError(e));
            }
        }
        
        // 执行 after_request 阶段 / Execute after_request phase
        for middleware in &self.middlewares {
            if let Err(e) = middleware.after_request(&mut self.context).await {
                // 处理错误 / Handle error
                for error_middleware in &self.middlewares {
                    let _ = error_middleware.handle_error(&mut self.context, &e).await;
                }
                return Err(MiddlewareError::ProcessingError(e));
            }
        }
        
        Ok(self.context.clone())
    }
}