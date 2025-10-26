# Worker 配置与管理

## 📋 文档概述

本文档详细阐述Temporal Worker的配置和管理，包括：

- Worker架构
- 配置选项
- 任务轮询
- Rust 1.90实现
- Golang实现对比
- 生产环境最佳实践

---

## 🎯 Worker核心概念

### Worker是什么？

**Worker**是Temporal架构中的执行引擎，负责：

1. **轮询任务**: 从Temporal Service获取任务
2. **执行工作流**: 执行Workflow代码
3. **执行Activity**: 执行Activity代码
4. **发送心跳**: 报告Activity进度
5. **处理Signal/Query**: 响应外部交互

```text
┌─────────────────────────────────────────────────────────────┐
│                      Worker 架构                             │
└─────────────────────────────────────────────────────────────┘

Temporal Service
    │
    ├─ Workflow Tasks ────────┐
    │                          │
    ├─ Activity Tasks ────────┼───────┐
    │                          │       │
    │                          ▼       ▼
    │                    ┌──────────────────┐
    │                    │   Worker Process │
    │                    │                  │
    │                    │  ┌────────────┐  │
    │                    │  │ Workflow   │  │
    │                    │  │ Executor   │  │
    │                    │  └────────────┘  │
    │                    │                  │
    │                    │  ┌────────────┐  │
    │                    │  │ Activity   │  │
    │                    │  │ Executor   │  │
    │                    │  └────────────┘  │
    │                    │                  │
    │                    │  ┌────────────┐  │
    │                    │  │ Task       │  │
    │                    │  │ Poller     │  │
    │                    │  └────────────┘  │
    │                    └──────────────────┘
    │                          │       │
    ◀──── Results ─────────────┴───────┘
```

---

## 🦀 Rust实现

### Worker配置

```rust
use std::time::Duration;
use std::collections::HashMap;

/// Worker配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// 任务队列名称
    pub task_queue: String,
    
    /// 最大并发Workflow任务数
    pub max_concurrent_workflow_tasks: usize,
    
    /// 最大并发Activity任务数
    pub max_concurrent_activity_tasks: usize,
    
    /// Workflow任务轮询超时
    pub workflow_poll_timeout: Duration,
    
    /// Activity任务轮询超时
    pub activity_poll_timeout: Duration,
    
    /// 优雅关闭超时
    pub graceful_shutdown_timeout: Duration,
    
    /// 启用指标收集
    pub enable_metrics: bool,
    
    /// 启用分布式追踪
    pub enable_tracing: bool,
    
    /// 自定义标签
    pub tags: HashMap<String, String>,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            task_queue: "default".to_string(),
            max_concurrent_workflow_tasks: 100,
            max_concurrent_activity_tasks: 100,
            workflow_poll_timeout: Duration::from_secs(60),
            activity_poll_timeout: Duration::from_secs(60),
            graceful_shutdown_timeout: Duration::from_secs(30),
            enable_metrics: true,
            enable_tracing: true,
            tags: HashMap::new(),
        }
    }
}

impl WorkerConfig {
    /// 创建Builder
    pub fn builder() -> WorkerConfigBuilder {
        WorkerConfigBuilder::default()
    }
}

/// Worker配置Builder
#[derive(Default)]
pub struct WorkerConfigBuilder {
    config: WorkerConfig,
}

impl WorkerConfigBuilder {
    pub fn task_queue(mut self, queue: impl Into<String>) -> Self {
        self.config.task_queue = queue.into();
        self
    }
    
    pub fn max_concurrent_workflow_tasks(mut self, max: usize) -> Self {
        self.config.max_concurrent_workflow_tasks = max;
        self
    }
    
    pub fn max_concurrent_activity_tasks(mut self, max: usize) -> Self {
        self.config.max_concurrent_activity_tasks = max;
        self
    }
    
    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.tags.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> WorkerConfig {
        self.config
    }
}
```

### Worker实现

```rust
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use std::collections::HashMap;

/// Worker主结构
pub struct WorkflowWorker {
    /// 配置
    config: WorkerConfig,
    
    /// 注册的Workflow类型
    workflows: Arc<RwLock<HashMap<String, WorkflowFactory>>>,
    
    /// 注册的Activity类型
    activities: Arc<RwLock<HashMap<String, ActivityFactory>>>,
    
    /// 关闭信号
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Workflow工厂
type WorkflowFactory = Arc<dyn Fn(WorkflowContext, serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, WorkflowError>> + Send + Sync>;

/// Activity工厂
type ActivityFactory = Arc<dyn Fn(ActivityContext, serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ActivityError>> + Send + Sync>;

impl WorkflowWorker {
    /// 创建新Worker
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            config,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            activities: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }
    
    /// 注册Workflow
    pub async fn register_workflow<W: Workflow>(&self) {
        let factory: WorkflowFactory = Arc::new(move |ctx, input| {
            Box::pin(async move {
                let input: W::Input = serde_json::from_value(input)
                    .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;
                
                let output = W::execute(ctx, input).await?;
                
                serde_json::to_value(output)
                    .map_err(|e| WorkflowError::SerializationError(e.to_string()))
            })
        });
        
        self.workflows
            .write()
            .await
            .insert(W::name().to_string(), factory);
        
        tracing::info!("Registered workflow: {}", W::name());
    }
    
    /// 注册Activity
    pub async fn register_activity<A: Activity>(&self) {
        let factory: ActivityFactory = Arc::new(move |ctx, input| {
            Box::pin(async move {
                let input: A::Input = serde_json::from_value(input)
                    .map_err(|e| ActivityError::InvalidInput(e.to_string()))?;
                
                let output = A::execute(ctx, input).await?;
                
                serde_json::to_value(output)
                    .map_err(|e| ActivityError::ExecutionFailed(e.to_string()))
            })
        });
        
        self.activities
            .write()
            .await
            .insert(A::name().to_string(), factory);
        
        tracing::info!("Registered activity: {}", A::name());
    }
    
    /// 运行Worker
    pub async fn run(mut self) -> Result<(), WorkerError> {
        tracing::info!("Starting worker on task queue: {}", self.config.task_queue);
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // 启动Workflow任务轮询器
        let workflow_poller = self.start_workflow_poller();
        
        // 启动Activity任务轮询器
        let activity_poller = self.start_activity_poller();
        
        // 等待关闭信号
        shutdown_rx.recv().await;
        
        tracing::info!("Shutting down worker...");
        
        // 优雅关闭
        tokio::select! {
            _ = workflow_poller => {},
            _ = activity_poller => {},
            _ = tokio::time::sleep(self.config.graceful_shutdown_timeout) => {
                tracing::warn!("Graceful shutdown timeout exceeded");
            }
        }
        
        tracing::info!("Worker stopped");
        Ok(())
    }
    
    /// 启动Workflow任务轮询器
    fn start_workflow_poller(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let workflows = self.workflows.clone();
        
        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(
                config.max_concurrent_workflow_tasks
            ));
            
            loop {
                // 获取许可证
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                
                // 轮询任务
                match poll_workflow_task(&config).await {
                    Ok(Some(task)) => {
                        let workflows = workflows.clone();
                        
                        tokio::spawn(async move {
                            let _permit = permit;  // 持有许可证直到任务完成
                            
                            if let Err(e) = execute_workflow_task(task, workflows).await {
                                tracing::error!("Workflow task execution failed: {:?}", e);
                            }
                        });
                    }
                    Ok(None) => {
                        // 没有任务，释放许可证
                        drop(permit);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to poll workflow task: {:?}", e);
                        drop(permit);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        })
    }
    
    /// 启动Activity任务轮询器
    fn start_activity_poller(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let activities = self.activities.clone();
        
        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(
                config.max_concurrent_activity_tasks
            ));
            
            loop {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                
                match poll_activity_task(&config).await {
                    Ok(Some(task)) => {
                        let activities = activities.clone();
                        
                        tokio::spawn(async move {
                            let _permit = permit;
                            
                            if let Err(e) = execute_activity_task(task, activities).await {
                                tracing::error!("Activity task execution failed: {:?}", e);
                            }
                        });
                    }
                    Ok(None) => {
                        drop(permit);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to poll activity task: {:?}", e);
                        drop(permit);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        })
    }
    
    /// 停止Worker
    pub async fn stop(&self) -> Result<(), WorkerError> {
        if let Some(tx) = &self.shutdown_tx {
            tx.send(()).await
                .map_err(|_| WorkerError::ShutdownFailed)?;
        }
        Ok(())
    }
}

/// Worker错误
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Failed to start worker: {0}")]
    StartFailed(String),
    
    #[error("Failed to shutdown worker")]
    ShutdownFailed,
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

### 使用示例

```rust
use temporal_rust::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建Worker配置
    let config = WorkerConfig::builder()
        .task_queue("my-task-queue")
        .max_concurrent_workflow_tasks(50)
        .max_concurrent_activity_tasks(100)
        .tag("environment", "production")
        .tag("version", "1.0.0")
        .build();
    
    // 创建Worker
    let worker = WorkflowWorker::new(config);
    
    // 注册Workflow
    worker.register_workflow::<OrderWorkflow>().await;
    worker.register_workflow::<PaymentWorkflow>().await;
    
    // 注册Activity
    worker.register_activity::<ProcessPaymentActivity>().await;
    worker.register_activity::<SendEmailActivity>().await;
    worker.register_activity::<UpdateInventoryActivity>().await;
    
    // 运行Worker
    tracing::info!("Worker started successfully");
    worker.run().await?;
    
    Ok(())
}
```

---

## 🐹 Golang实现对比

### Worker配置 - Golang

```go
package main

import (
    "go.temporal.io/sdk/client"
    "go.temporal.io/sdk/worker"
)

func main() {
    // 创建客户端
    c, err := client.Dial(client.Options{})
    if err != nil {
        log.Fatal(err)
    }
    defer c.Close()
    
    // 创建Worker
    w := worker.New(c, "my-task-queue", worker.Options{
        MaxConcurrentWorkflowTaskExecutionSize: 50,
        MaxConcurrentActivityExecutionSize:     100,
        WorkerStopTimeout:                      30 * time.Second,
        EnableLoggingInReplay:                  false,
    })
    
    // 注册Workflow
    w.RegisterWorkflow(OrderWorkflow)
    w.RegisterWorkflow(PaymentWorkflow)
    
    // 注册Activity
    w.RegisterActivity(ProcessPaymentActivity)
    w.RegisterActivity(SendEmailActivity)
    w.RegisterActivity(UpdateInventoryActivity)
    
    // 运行Worker
    err = w.Run(worker.InterruptCh())
    if err != nil {
        log.Fatal(err)
    }
}
```

---

## 🎯 最佳实践

### 1. 并发配置

```rust
// ✅ 好: 根据资源情况调整并发数

// CPU密集型
let cpu_intensive = WorkerConfig::builder()
    .max_concurrent_workflow_tasks(num_cpus::get())
    .max_concurrent_activity_tasks(num_cpus::get() * 2)
    .build();

// I/O密集型
let io_intensive = WorkerConfig::builder()
    .max_concurrent_workflow_tasks(50)
    .max_concurrent_activity_tasks(200)
    .build();

// 混合型
let mixed = WorkerConfig::builder()
    .max_concurrent_workflow_tasks(20)
    .max_concurrent_activity_tasks(80)
    .build();
```

### 2. 优雅关闭

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let worker = WorkflowWorker::new(config);
    
    // 注册Workflow和Activity...
    
    // 启动Worker（非阻塞）
    let worker_handle = tokio::spawn(async move {
        worker.run().await
    });
    
    // 等待关闭信号
    signal::ctrl_c().await?;
    
    tracing::info!("Received shutdown signal");
    
    // 停止Worker
    worker.stop().await?;
    
    // 等待Worker完全停止
    worker_handle.await??;
    
    tracing::info!("Shutdown complete");
    Ok(())
}
```

### 3. 健康检查

```rust
impl WorkflowWorker {
    /// 健康检查
    pub async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            is_healthy: true,
            workflow_tasks_in_progress: self.get_workflow_task_count().await,
            activity_tasks_in_progress: self.get_activity_task_count().await,
            last_poll_time: self.get_last_poll_time().await,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub workflow_tasks_in_progress: usize,
    pub activity_tasks_in_progress: usize,
    pub last_poll_time: DateTime<Utc>,
}

// HTTP健康检查端点
async fn health_handler(
    Extension(worker): Extension<Arc<WorkflowWorker>>,
) -> Json<HealthStatus> {
    Json(worker.health_check().await)
}
```

### 4. 指标收集

```rust
use prometheus::{Registry, Counter, Gauge, Histogram};

pub struct WorkerMetrics {
    /// 已处理的Workflow任务数
    pub workflows_processed: Counter,
    
    /// 已处理的Activity任务数
    pub activities_processed: Counter,
    
    /// 当前运行的Workflow任务数
    pub workflows_in_progress: Gauge,
    
    /// 当前运行的Activity任务数
    pub activities_in_progress: Gauge,
    
    /// Workflow执行时间
    pub workflow_duration: Histogram,
    
    /// Activity执行时间
    pub activity_duration: Histogram,
}

impl WorkerMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let workflows_processed = Counter::new(
            "worker_workflows_processed_total",
            "Total number of workflows processed"
        )?;
        registry.register(Box::new(workflows_processed.clone()))?;
        
        // 注册其他指标...
        
        Ok(Self {
            workflows_processed,
            // ...
        })
    }
}
```

### 5. 多Worker部署

```rust
// Worker 1: 专门处理订单工作流
let order_worker = WorkerConfig::builder()
    .task_queue("order-queue")
    .max_concurrent_workflow_tasks(50)
    .tag("type", "order")
    .build();

// Worker 2: 专门处理支付工作流
let payment_worker = WorkerConfig::builder()
    .task_queue("payment-queue")
    .max_concurrent_workflow_tasks(100)
    .tag("type", "payment")
    .build();

// Worker 3: 通用Activity处理
let activity_worker = WorkerConfig::builder()
    .task_queue("activity-queue")
    .max_concurrent_workflow_tasks(0)  // 不处理工作流
    .max_concurrent_activity_tasks(200)
    .tag("type", "activity")
    .build();
```

---

## 📊 Worker监控

### 关键指标

| 指标 | 类型 | 说明 | 告警阈值 |
|------|------|------|---------|
| **workflows_processed_total** | Counter | 已处理工作流数 | - |
| **activities_processed_total** | Counter | 已处理Activity数 | - |
| **workflows_in_progress** | Gauge | 运行中的工作流数 | > max_concurrent |
| **activities_in_progress** | Gauge | 运行中的Activity数 | > max_concurrent |
| **workflow_duration_seconds** | Histogram | 工作流执行时间 | p99 > 300s |
| **activity_duration_seconds** | Histogram | Activity执行时间 | p99 > 60s |
| **poll_errors_total** | Counter | 轮询错误数 | rate > 10/min |
| **execution_errors_total** | Counter | 执行错误数 | rate > 5/min |

---

## 📚 总结

### Worker核心职责

1. **任务轮询**: 从Temporal Service获取任务
2. **并发控制**: 限制同时执行的任务数
3. **执行管理**: 执行Workflow和Activity代码
4. **错误处理**: 处理执行中的错误
5. **优雅关闭**: 安全地停止Worker

### Rust vs Golang

- **Rust**: 更细粒度的并发控制，使用Semaphore
- **Golang**: 更简单的配置，Temporal官方支持

---

## 📚 下一步

- **持久化**: [持久化实现](./12_persistence.md)
- **部署**: [生产部署](./deployment.md)
- **监控**: [可观测性](./monitoring.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

