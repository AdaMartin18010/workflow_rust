# 重试与超时

## 📋 文档概述

本文档详细阐述Temporal的重试和超时机制，包括：

- 重试策略原理
- 超时类型
- Rust 1.90实现
- Golang实现对比
- 最佳实践

---

## 🎯 核心概念

### 重试策略

Temporal提供自动重试机制，当Activity或Workflow失败时，系统会根据配置的重试策略自动重试。

```text
┌─────────────────────────────────────────────────────────────┐
│                      重试机制流程                            │
└─────────────────────────────────────────────────────────────┘

Activity执行
    │
    ├─ 成功 ──────────▶ 返回结果
    │
    ├─ 失败 (可重试)
    │   │
    │   ├─ 检查重试策略
    │   │   ├─ 尝试次数 < MaxAttempts?
    │   │   ├─ 错误类型可重试?
    │   │   └─ 计算退避时间
    │   │
    │   ├─ 等待退避时间
    │   │   BackoffTime = InitialInterval × (BackoffCoefficient ^ attempt)
    │   │   BackoffTime = min(BackoffTime, MaxInterval)
    │   │
    │   └─ 重试执行 ──┐
    │                 │
    │   ◀─────────────┘
    │
    └─ 失败 (不可重试) ──▶ 返回错误
```

### 超时类型

Temporal支持多种超时类型，提供细粒度的控制：

| 超时类型 | 作用域 | 说明 |
|---------|--------|------|
| **ScheduleToStart** | Activity | 从调度到开始执行的最大时间 |
| **StartToClose** | Activity | 从开始到完成的最大时间 |
| **ScheduleToClose** | Activity | 从调度到完成的总时间 |
| **HeartbeatTimeout** | Activity | 心跳间隔的最大时间 |
| **WorkflowExecutionTimeout** | Workflow | 工作流执行的总时间 |
| **WorkflowRunTimeout** | Workflow | 单次运行的最大时间 |
| **WorkflowTaskTimeout** | Workflow | 单个任务的最大时间 |

---

## 🦀 Rust实现

### 重试策略定义

```rust
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大尝试次数 (包括初始尝试)
    pub max_attempts: u32,
    
    /// 初始重试间隔
    pub initial_interval: Duration,
    
    /// 最大重试间隔
    pub max_interval: Duration,
    
    /// 退避系数 (每次重试间隔乘以此系数)
    pub backoff_coefficient: f64,
    
    /// 不可重试的错误类型
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

impl RetryPolicy {
    /// 计算下一次重试的间隔时间
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_interval;
        }
        
        // BackoffTime = InitialInterval × (BackoffCoefficient ^ attempt)
        let backoff_ms = self.initial_interval.as_millis() as f64
            * self.backoff_coefficient.powi(attempt as i32);
        
        let backoff = Duration::from_millis(backoff_ms as u64);
        
        // 不超过最大间隔
        std::cmp::min(backoff, self.max_interval)
    }
    
    /// 检查错误是否可重试
    pub fn is_retryable(&self, error: &ActivityError) -> bool {
        match error {
            ActivityError::ValidationFailed(_) => {
                !self.non_retryable_error_types.contains(&"ValidationFailed".to_string())
            }
            ActivityError::TemporaryFailure(_) => true,
            ActivityError::Cancelled => false,
            ActivityError::Timeout => false,
            _ => true,
        }
    }
    
    /// 检查是否还可以重试
    pub fn can_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}
```

### Activity超时配置

```rust
/// Activity选项（包含所有超时配置）
#[derive(Debug, Clone)]
pub struct ActivityOptions {
    /// Activity ID
    pub activity_id: Option<ActivityId>,
    
    /// 任务队列
    pub task_queue: Option<String>,
    
    /// ScheduleToStart超时：从调度到开始执行
    pub schedule_to_start_timeout: Option<Duration>,
    
    /// StartToClose超时：从开始到完成
    pub start_to_close_timeout: Option<Duration>,
    
    /// ScheduleToClose超时：从调度到完成
    pub schedule_to_close_timeout: Option<Duration>,
    
    /// 心跳超时
    pub heartbeat_timeout: Option<Duration>,
    
    /// 重试策略
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
```

### 重试执行器

```rust
use tokio::time::{sleep, timeout};

/// Activity重试执行器
pub struct ActivityRetryExecutor<A: Activity> {
    options: ActivityOptions,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Activity> ActivityRetryExecutor<A> {
    pub fn new(options: ActivityOptions) -> Self {
        Self {
            options,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 执行Activity with retry
    pub async fn execute(
        &self,
        ctx: ActivityContext,
        input: A::Input,
    ) -> Result<A::Output, ActivityError> {
        let retry_policy = self.options.retry_policy.as_ref()
            .ok_or(ActivityError::Custom("No retry policy".to_string()))?;
        
        let mut attempt = 0;
        let mut last_error = None;
        
        while attempt < retry_policy.max_attempts {
            // 如果不是第一次尝试，等待退避时间
            if attempt > 0 {
                let backoff = retry_policy.calculate_backoff(attempt - 1);
                tracing::info!(
                    "Retrying activity after {} ms (attempt {}/{})",
                    backoff.as_millis(),
                    attempt + 1,
                    retry_policy.max_attempts
                );
                sleep(backoff).await;
            }
            
            // 执行Activity
            let result = self.execute_with_timeout(ctx.clone(), input.clone()).await;
            
            match result {
                Ok(output) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Activity succeeded after {} attempts",
                            attempt + 1
                        );
                    }
                    return Ok(output);
                }
                Err(error) => {
                    // 检查错误是否可重试
                    if !retry_policy.is_retryable(&error) {
                        tracing::warn!(
                            "Activity failed with non-retryable error: {:?}",
                            error
                        );
                        return Err(error);
                    }
                    
                    last_error = Some(error);
                    attempt += 1;
                }
            }
        }
        
        // 所有重试都失败
        tracing::error!(
            "Activity failed after {} attempts",
            retry_policy.max_attempts
        );
        
        Err(last_error.unwrap_or(ActivityError::Custom(
            "Max attempts reached".to_string()
        )))
    }
    
    /// 执行Activity with timeout
    async fn execute_with_timeout(
        &self,
        ctx: ActivityContext,
        input: A::Input,
    ) -> Result<A::Output, ActivityError> {
        let execution_timeout = self.options.start_to_close_timeout
            .unwrap_or(Duration::from_secs(300));
        
        match timeout(execution_timeout, A::execute(ctx, input)).await {
            Ok(result) => result,
            Err(_) => Err(ActivityError::Timeout),
        }
    }
}
```

### 使用示例

```rust
/// 带重试的HTTP请求Activity
pub struct HttpRequestActivity;

impl Activity for HttpRequestActivity {
    type Input = HttpRequest;
    type Output = HttpResponse;
    
    fn name() -> &'static str {
        "HttpRequest"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 发送心跳
            ctx.heartbeat().await?;
            
            // 执行HTTP请求
            let response = reqwest::get(&input.url)
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        ActivityError::Timeout
                    } else if e.is_connect() {
                        // 连接错误，可重试
                        ActivityError::TemporaryFailure(e.to_string())
                    } else {
                        ActivityError::ExecutionFailed(e.to_string())
                    }
                })?;
            
            // 检查状态码
            if response.status().is_client_error() {
                // 4xx错误，不可重试
                return Err(ActivityError::ValidationFailed(
                    format!("Client error: {}", response.status())
                ));
            }
            
            if response.status().is_server_error() {
                // 5xx错误，可重试
                return Err(ActivityError::TemporaryFailure(
                    format!("Server error: {}", response.status())
                ));
            }
            
            let body = response.text().await
                .map_err(|e| ActivityError::ExecutionFailed(e.to_string()))?;
            
            Ok(HttpResponse { body })
        }
    }
}

// 在Workflow中使用
impl Workflow for MyWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 配置重试策略
            let options = ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(30)),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 5,
                    initial_interval: Duration::from_secs(1),
                    max_interval: Duration::from_secs(60),
                    backoff_coefficient: 2.0,
                    non_retryable_error_types: vec![
                        "ValidationFailed".to_string(),
                    ],
                }),
                ..Default::default()
            };
            
            // 执行Activity（会自动重试）
            let response = ctx
                .execute_activity::<HttpRequestActivity>(
                    HttpRequest {
                        url: "https://api.example.com".to_string(),
                    },
                    options,
                )
                .await?;
            
            Ok(Self::Output { response })
        }
    }
}
```

### Workflow超时配置

```rust
/// Workflow启动选项
#[derive(Debug, Clone)]
pub struct StartWorkflowOptions {
    /// Workflow ID
    pub workflow_id: Option<WorkflowId>,
    
    /// 任务队列
    pub task_queue: String,
    
    /// WorkflowExecutionTimeout: 整个工作流执行的最大时间
    pub workflow_execution_timeout: Option<Duration>,
    
    /// WorkflowRunTimeout: 单次运行的最大时间（用于ContinueAsNew）
    pub workflow_run_timeout: Option<Duration>,
    
    /// WorkflowTaskTimeout: 单个工作流任务的最大时间
    pub workflow_task_timeout: Option<Duration>,
}

impl Default for StartWorkflowOptions {
    fn default() -> Self {
        Self {
            workflow_id: None,
            task_queue: "default".to_string(),
            workflow_execution_timeout: None,  // 无限制
            workflow_run_timeout: None,        // 无限制
            workflow_task_timeout: Some(Duration::from_secs(10)),
        }
    }
}
```

---

## 🐹 Golang实现对比

### 重试策略 - Golang

```go
type RetryPolicy struct {
    InitialInterval    time.Duration
    BackoffCoefficient float64
    MaximumInterval    time.Duration
    MaximumAttempts    int32
    NonRetryableErrorTypes []string
}

// 使用示例
func MyWorkflow(ctx workflow.Context, input WorkflowInput) (WorkflowOutput, error) {
    // 配置重试策略
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 30 * time.Second,
        RetryPolicy: &temporal.RetryPolicy{
            InitialInterval:    time.Second,
            BackoffCoefficient: 2.0,
            MaximumInterval:    60 * time.Second,
            MaximumAttempts:    5,
            NonRetryableErrorTypes: []string{
                "ValidationError",
            },
        },
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)
    
    var result ActivityResult
    err := workflow.ExecuteActivity(ctx, MyActivity, input).Get(ctx, &result)
    if err != nil {
        return WorkflowOutput{}, err
    }
    
    return WorkflowOutput{Result: result}, nil
}
```

### 超时配置 - Golang

```go
// Activity超时
activityOptions := workflow.ActivityOptions{
    ScheduleToStartTimeout: 60 * time.Second,
    StartToCloseTimeout:    300 * time.Second,
    HeartbeatTimeout:       30 * time.Second,
}

// Workflow超时
workflowOptions := client.StartWorkflowOptions{
    WorkflowExecutionTimeout: 24 * time.Hour,
    WorkflowRunTimeout:       time.Hour,
    WorkflowTaskTimeout:      10 * time.Second,
}
```

---

## 🎯 最佳实践

### 1. 重试策略设计

```rust
// ✅ 好: 根据场景定制重试策略

// 网络请求：激进重试
let network_retry = RetryPolicy {
    max_attempts: 5,
    initial_interval: Duration::from_secs(1),
    max_interval: Duration::from_secs(60),
    backoff_coefficient: 2.0,
    non_retryable_error_types: vec![
        "ValidationFailed".to_string(),
        "AuthenticationFailed".to_string(),
    ],
};

// 数据库操作：保守重试
let database_retry = RetryPolicy {
    max_attempts: 3,
    initial_interval: Duration::from_secs(5),
    max_interval: Duration::from_secs(30),
    backoff_coefficient: 1.5,
    non_retryable_error_types: vec![
        "ConstraintViolation".to_string(),
    ],
};

// 关键操作：不重试
let critical_retry = RetryPolicy {
    max_attempts: 1,
    ..Default::default()
};
```

### 2. 超时设置

```rust
// ✅ 好: 根据实际情况设置超时

// 快速操作
let quick_options = ActivityOptions {
    start_to_close_timeout: Some(Duration::from_secs(10)),
    heartbeat_timeout: Some(Duration::from_secs(3)),
    ..Default::default()
};

// 中等操作
let medium_options = ActivityOptions {
    start_to_close_timeout: Some(Duration::from_secs(300)),
    heartbeat_timeout: Some(Duration::from_secs(30)),
    ..Default::default()
};

// 长时间操作
let long_options = ActivityOptions {
    start_to_close_timeout: Some(Duration::from_secs(3600)),
    heartbeat_timeout: Some(Duration::from_secs(60)),
    ..Default::default()
};
```

### 3. 错误分类

```rust
// ✅ 好: 明确区分可重试和不可重试的错误

impl Activity for MyActivity {
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 验证输入 - 不可重试
            if !input.is_valid() {
                return Err(ActivityError::ValidationFailed(
                    "Invalid input".to_string()
                ));
            }
            
            // 网络请求 - 可重试
            match make_network_call(&input).await {
                Ok(result) => Ok(result),
                Err(e) if e.is_timeout() => {
                    Err(ActivityError::TemporaryFailure(
                        "Network timeout".to_string()
                    ))
                }
                Err(e) => {
                    Err(ActivityError::ExecutionFailed(e.to_string()))
                }
            }
        }
    }
}
```

### 4. 心跳与超时

```rust
// ✅ 好: 合理使用心跳防止超时

pub struct LongRunningActivity;

impl Activity for LongRunningActivity {
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            let total_items = input.items.len();
            
            for (i, item) in input.items.iter().enumerate() {
                // 处理项目
                process_item(item).await?;
                
                // 每10个项目发送心跳
                if i % 10 == 0 {
                    ctx.heartbeat_with_details(ProcessProgress {
                        current: i,
                        total: total_items,
                        percentage: (i as f64 / total_items as f64) * 100.0,
                    }).await?;
                }
            }
            
            Ok(ProcessResult {
                total_processed: total_items,
            })
        }
    }
}
```

---

## 📊 重试和超时对比

### Rust vs Golang

| 特性 | Rust | Golang |
|------|------|--------|
| **重试策略配置** | RetryPolicy结构体 | temporal.RetryPolicy |
| **超时配置** | Duration类型 | time.Duration |
| **错误分类** | 自定义Error枚举 | 字符串匹配 |
| **类型安全** | 编译时检查 | 运行时检查 |
| **默认值** | trait Default | 零值 |

---

## 📚 总结

### 重试策略

1. **指数退避**: 使用BackoffCoefficient实现指数退避
2. **最大尝试次数**: 限制重试次数避免无限重试
3. **错误分类**: 区分可重试和不可重试的错误
4. **间隔限制**: 使用MaxInterval限制最大间隔

### 超时管理

1. **多层次超时**: 从调度、执行到完成的多个超时点
2. **心跳机制**: 防止长时间运行的Activity超时
3. **合理配置**: 根据实际情况设置超时时间
4. **超时处理**: 优雅处理超时情况

---

## 📚 下一步

- **版本管理**: [工作流版本控制](./09_versioning.md)
- **测试策略**: [测试最佳实践](./10_testing.md)
- **监控告警**: [可观测性](./monitoring.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

