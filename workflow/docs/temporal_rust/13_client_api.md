# 客户端 API

## 📋 文档概述

本文档详细阐述Temporal客户端API的设计和实现，包括：

- 客户端架构
- Workflow启动和管理
- Signal发送
- Query执行
- Rust 1.90实现
- Golang实现对比
- 最佳实践

---

## 🎯 客户端核心概念

### 客户端是什么？

**WorkflowClient**是应用程序与Temporal Service交互的主要接口，负责：

1. **启动工作流**: 创建新的工作流实例
2. **发送Signal**: 向运行中的工作流发送信号
3. **执行Query**: 查询工作流状态
4. **管理工作流**: 取消、终止工作流
5. **等待结果**: 同步等待工作流完成

```text
┌─────────────────────────────────────────────────────────────┐
│                     客户端架构                               │
└─────────────────────────────────────────────────────────────┘

Application Code
    │
    ├─ Start Workflow ────────┐
    ├─ Send Signal ───────────┼────────┐
    ├─ Execute Query ─────────┼────────┼────┐
    │                          │        │    │
    │                          ▼        ▼    ▼
    │                    ┌───────────────────────┐
    │                    │   WorkflowClient      │
    │                    │                       │
    │                    │  ┌─────────────────┐  │
    │                    │  │  gRPC Client    │  │
    │                    │  └─────────────────┘  │
    │                    │                       │
    │                    │  ┌─────────────────┐  │
    │                    │  │  Serialization  │  │
    │                    │  └─────────────────┘  │
    │                    │                       │
    │                    │  ┌─────────────────┐  │
    │                    │  │  Retry Logic    │  │
    │                    │  └─────────────────┘  │
    │                    └───────────────────────┘
    │                              │
    │                              ▼
    │                    Temporal Service
    │                              │
    ◀──── Results ────────────────┘
```

---

## 🦀 Rust实现

### 客户端配置

```rust
use std::time::Duration;
use url::Url;

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Temporal Service地址
    pub target_url: Url,
    
    /// 命名空间
    pub namespace: String,
    
    /// 身份标识
    pub identity: String,
    
    /// 连接超时
    pub connect_timeout: Duration,
    
    /// 请求超时
    pub request_timeout: Duration,
    
    /// 启用TLS
    pub enable_tls: bool,
    
    /// TLS配置
    pub tls_config: Option<TlsConfig>,
    
    /// 重试策略
    pub retry_config: RetryConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            target_url: Url::parse("http://localhost:7233").unwrap(),
            namespace: "default".to_string(),
            identity: format!("client-{}", uuid::Uuid::new_v4()),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            enable_tls: false,
            tls_config: None,
            retry_config: RetryConfig::default(),
        }
    }
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }
}

/// 客户端配置Builder
#[derive(Default)]
pub struct ClientConfigBuilder {
    config: ClientConfig,
}

impl ClientConfigBuilder {
    pub fn target_url(mut self, url: impl Into<String>) -> Self {
        self.config.target_url = Url::parse(&url.into()).unwrap();
        self
    }
    
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.config.namespace = namespace.into();
        self
    }
    
    pub fn identity(mut self, identity: impl Into<String>) -> Self {
        self.config.identity = identity.into();
        self
    }
    
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }
    
    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.config.enable_tls = enable;
        self
    }
    
    pub fn build(self) -> ClientConfig {
        self.config
    }
}

/// TLS配置
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub ca_cert: Vec<u8>,
    pub client_cert: Option<Vec<u8>>,
    pub client_key: Option<Vec<u8>>,
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_interval: Duration,
    pub max_interval: Duration,
    pub backoff_coefficient: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(10),
            backoff_coefficient: 2.0,
        }
    }
}
```

### 客户端实现

```rust
use tonic::transport::Channel;
use std::sync::Arc;

/// Workflow客户端
pub struct WorkflowClient {
    config: ClientConfig,
    channel: Channel,
    // gRPC客户端（实际应该是Temporal的proto生成的客户端）
    // grpc_client: temporal_proto::WorkflowServiceClient<Channel>,
}

impl WorkflowClient {
    /// 创建新客户端
    pub async fn new(config: ClientConfig) -> Result<Self, ClientError> {
        // 建立gRPC连接
        let channel = Self::create_channel(&config).await?;
        
        Ok(Self {
            config,
            channel,
        })
    }
    
    /// 创建gRPC通道
    async fn create_channel(config: &ClientConfig) -> Result<Channel, ClientError> {
        let endpoint = Channel::from_shared(config.target_url.to_string())
            .map_err(|e| ClientError::ConnectionError(e.to_string()))?
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);
        
        // 如果启用TLS
        let endpoint = if config.enable_tls {
            if let Some(tls_config) = &config.tls_config {
                let tls = Self::build_tls_config(tls_config)?;
                endpoint.tls_config(tls)
                    .map_err(|e| ClientError::TlsError(e.to_string()))?
            } else {
                endpoint
            }
        } else {
            endpoint
        };
        
        endpoint
            .connect()
            .await
            .map_err(|e| ClientError::ConnectionError(e.to_string()))
    }
    
    fn build_tls_config(config: &TlsConfig) -> Result<tonic::transport::ClientTlsConfig, ClientError> {
        // 构建TLS配置
        unimplemented!("TLS configuration")
    }
    
    /// 启动工作流
    pub async fn start_workflow<W: Workflow>(
        &self,
        input: W::Input,
        options: StartWorkflowOptions,
    ) -> Result<WorkflowHandle<W>, ClientError> {
        tracing::info!("Starting workflow: {}", W::name());
        
        // 生成WorkflowId
        let workflow_id = options.workflow_id
            .unwrap_or_else(|| WorkflowId::new(format!("wf-{}", uuid::Uuid::new_v4())));
        
        let run_id = RunId::new();
        
        // 序列化输入
        let input_bytes = serde_json::to_vec(&input)
            .map_err(|e| ClientError::SerializationError(e.to_string()))?;
        
        // 发送gRPC请求到Temporal Service
        // 实际应该调用:
        // let response = self.grpc_client.start_workflow_execution(request).await?;
        
        tracing::info!("Workflow started: {} (run: {})", workflow_id.as_str(), run_id);
        
        Ok(WorkflowHandle {
            client: self.clone(),
            execution: WorkflowExecution::with_run_id(workflow_id, run_id),
            _phantom: std::marker::PhantomData,
        })
    }
    
    /// 获取工作流句柄
    pub fn get_workflow_handle<W: Workflow>(
        &self,
        workflow_id: WorkflowId,
    ) -> WorkflowHandle<W> {
        let run_id = RunId::new(); // 实际应该从Service查询
        
        WorkflowHandle {
            client: self.clone(),
            execution: WorkflowExecution::with_run_id(workflow_id, run_id),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 列出工作流
    pub async fn list_workflows(
        &self,
        query: ListWorkflowsQuery,
    ) -> Result<Vec<WorkflowExecutionInfo>, ClientError> {
        tracing::debug!("Listing workflows with query: {:?}", query);
        
        // 实际应该调用gRPC
        Ok(vec![])
    }
}

impl Clone for WorkflowClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            channel: self.channel.clone(),
        }
    }
}

/// 启动工作流选项
#[derive(Debug, Default)]
pub struct StartWorkflowOptions {
    /// 工作流ID（可选，不指定则自动生成）
    pub workflow_id: Option<WorkflowId>,
    
    /// 任务队列
    pub task_queue: String,
    
    /// 工作流执行超时
    pub workflow_execution_timeout: Option<Duration>,
    
    /// 工作流运行超时
    pub workflow_run_timeout: Option<Duration>,
    
    /// 工作流任务超时
    pub workflow_task_timeout: Option<Duration>,
    
    /// ID重用策略
    pub workflow_id_reuse_policy: WorkflowIdReusePolicy,
    
    /// 重试策略
    pub retry_policy: Option<RetryPolicy>,
    
    /// Cron表达式
    pub cron_schedule: Option<String>,
    
    /// 元数据
    pub memo: Option<HashMap<String, serde_json::Value>>,
    
    /// 搜索属性
    pub search_attributes: Option<HashMap<String, serde_json::Value>>,
}

/// WorkflowId重用策略
#[derive(Debug, Clone, Copy)]
pub enum WorkflowIdReusePolicy {
    /// 允许重复（如果没有运行中的）
    AllowDuplicate,
    
    /// 允许重复失败的
    AllowDuplicateFailedOnly,
    
    /// 拒绝重复
    RejectDuplicate,
    
    /// 终止现有的
    TerminateIfRunning,
}

impl Default for WorkflowIdReusePolicy {
    fn default() -> Self {
        Self::AllowDuplicate
    }
}

/// 工作流句柄
pub struct WorkflowHandle<W: Workflow> {
    client: WorkflowClient,
    execution: WorkflowExecution,
    _phantom: std::marker::PhantomData<W>,
}

impl<W: Workflow> WorkflowHandle<W> {
    /// 获取WorkflowId
    pub fn workflow_id(&self) -> &WorkflowId {
        &self.execution.workflow_id
    }
    
    /// 获取RunId
    pub fn run_id(&self) -> &RunId {
        &self.execution.run_id
    }
    
    /// 发送Signal
    pub async fn signal<S: Signal>(
        &self,
        signal: S,
    ) -> Result<(), ClientError> {
        tracing::info!(
            "Sending signal {} to workflow {}",
            S::name(),
            self.workflow_id().as_str()
        );
        
        // 序列化Signal
        let signal_bytes = serde_json::to_vec(&signal)
            .map_err(|e| ClientError::SerializationError(e.to_string()))?;
        
        // 发送gRPC请求
        // self.client.grpc_client.signal_workflow_execution(...).await?;
        
        Ok(())
    }
    
    /// 执行Query
    pub async fn query<Q: Query>(
        &self,
    ) -> Result<Q::Result, ClientError> {
        tracing::info!(
            "Executing query {} on workflow {}",
            Q::name(),
            self.workflow_id().as_str()
        );
        
        // 发送gRPC请求
        // let response = self.client.grpc_client.query_workflow(...).await?;
        
        // 反序列化结果
        // let result: Q::Result = serde_json::from_slice(&response.result)?;
        
        unimplemented!("Query execution")
    }
    
    /// 取消工作流
    pub async fn cancel(&self) -> Result<(), ClientError> {
        tracing::info!("Cancelling workflow {}", self.workflow_id().as_str());
        
        // 发送取消请求
        // self.client.grpc_client.request_cancel_workflow_execution(...).await?;
        
        Ok(())
    }
    
    /// 终止工作流
    pub async fn terminate(
        &self,
        reason: impl Into<String>,
    ) -> Result<(), ClientError> {
        let reason = reason.into();
        tracing::info!(
            "Terminating workflow {}: {}",
            self.workflow_id().as_str(),
            reason
        );
        
        // 发送终止请求
        // self.client.grpc_client.terminate_workflow_execution(...).await?;
        
        Ok(())
    }
    
    /// 等待工作流完成
    pub async fn get_result(self) -> Result<W::Output, ClientError> {
        tracing::info!("Waiting for workflow {} to complete", self.workflow_id().as_str());
        
        // 轮询或订阅工作流状态
        loop {
            // 查询工作流状态
            // let status = self.client.grpc_client.describe_workflow_execution(...).await?;
            
            // 如果完成，返回结果
            // if status.is_completed() {
            //     let result: W::Output = serde_json::from_slice(&status.result)?;
            //     return Ok(result);
            // }
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    
    /// 获取工作流历史
    pub async fn get_history(&self) -> Result<Vec<WorkflowEvent>, ClientError> {
        tracing::debug!("Fetching history for workflow {}", self.workflow_id().as_str());
        
        // 获取事件历史
        // let response = self.client.grpc_client.get_workflow_execution_history(...).await?;
        
        Ok(vec![])
    }
}

/// 列出工作流查询
#[derive(Debug, Clone)]
pub struct ListWorkflowsQuery {
    pub namespace: String,
    pub query: Option<String>,
    pub page_size: i32,
    pub next_page_token: Option<Vec<u8>>,
}

/// 工作流执行信息
#[derive(Debug, Clone)]
pub struct WorkflowExecutionInfo {
    pub execution: WorkflowExecution,
    pub workflow_type: String,
    pub start_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub status: WorkflowExecutionStatus,
}

/// 工作流执行状态
#[derive(Debug, Clone, Copy)]
pub enum WorkflowExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    ContinuedAsNew,
    TimedOut,
}

/// 客户端错误
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("Workflow already exists: {0}")]
    WorkflowAlreadyExists(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}
```

### 使用示例

```rust
use temporal_rust::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = WorkflowClient::new(
        ClientConfig::builder()
            .target_url("http://localhost:7233")
            .namespace("default")
            .build()
    ).await?;
    
    // 准备输入
    let order = Order {
        order_id: "ORD-12345".to_string(),
        user_id: "user-123".to_string(),
        total_amount: 99.99,
        // ...
    };
    
    // 启动工作流
    let handle = client.start_workflow::<OrderProcessingWorkflow>(
        order,
        StartWorkflowOptions {
            task_queue: "order-processing".to_string(),
            workflow_execution_timeout: Some(Duration::from_secs(300)),
            ..Default::default()
        }
    ).await?;
    
    println!("Workflow started: {}", handle.workflow_id().as_str());
    
    // 发送Signal
    tokio::time::sleep(Duration::from_secs(2)).await;
    handle.signal(ApprovalSignal {
        approved: true,
        approver: "manager-001".to_string(),
    }).await?;
    
    // 执行Query
    let status = handle.query::<OrderStatusQuery>().await?;
    println!("Current status: {:?}", status);
    
    // 等待结果
    let result = handle.get_result().await?;
    println!("Workflow completed: {:?}", result);
    
    Ok(())
}
```

---

## 🐹 Golang实现对比

### 客户端配置 - Golang

```go
package main

import (
    "context"
    "time"

    "go.temporal.io/sdk/client"
)

func main() {
    // 创建客户端
    c, err := client.Dial(client.Options{
        HostPort:  "localhost:7233",
        Namespace: "default",
        Identity:  "my-client",
        
        ConnectionOptions: client.ConnectionOptions{
            TLS: nil, // TLS配置（可选）
        },
    })
    if err != nil {
        log.Fatal(err)
    }
    defer c.Close()
    
    // 准备输入
    order := Order{
        OrderID:     "ORD-12345",
        UserID:      "user-123",
        TotalAmount: 99.99,
    }
    
    // 启动工作流
    workflowOptions := client.StartWorkflowOptions{
        ID:                    "order-" + order.OrderID,
        TaskQueue:             "order-processing",
        WorkflowExecutionTimeout: 5 * time.Minute,
    }
    
    we, err := c.ExecuteWorkflow(
        context.Background(),
        workflowOptions,
        OrderProcessingWorkflow,
        order,
    )
    if err != nil {
        log.Fatal(err)
    }
    
    log.Printf("Workflow started: WorkflowID=%s, RunID=%s",
        we.GetID(), we.GetRunID())
    
    // 发送Signal
    time.Sleep(2 * time.Second)
    err = c.SignalWorkflow(
        context.Background(),
        we.GetID(),
        we.GetRunID(),
        "approval",
        ApprovalSignal{
            Approved: true,
            Approver: "manager-001",
        },
    )
    if err != nil {
        log.Fatal(err)
    }
    
    // 执行Query
    var status OrderStatus
    result, err := c.QueryWorkflow(
        context.Background(),
        we.GetID(),
        we.GetRunID(),
        "order_status",
    )
    if err != nil {
        log.Fatal(err)
    }
    err = result.Get(&status)
    if err != nil {
        log.Fatal(err)
    }
    log.Printf("Current status: %+v", status)
    
    // 等待结果
    var orderResult OrderResult
    err = we.Get(context.Background(), &orderResult)
    if err != nil {
        log.Fatal(err)
    }
    
    log.Printf("Workflow completed: %+v", orderResult)
}
```

---

## 🎯 最佳实践

### 1. 工作流ID管理

```rust
// ✅ 好: 使用业务相关的WorkflowId
let workflow_id = WorkflowId::new(format!("order-{}", order_id));

// ✅ 好: 幂等性 - 相同订单ID不会重复启动
let options = StartWorkflowOptions {
    workflow_id: Some(workflow_id),
    workflow_id_reuse_policy: WorkflowIdReusePolicy::RejectDuplicate,
    ..Default::default()
};

// ❌ 不好: 使用随机ID，无法追踪
let workflow_id = WorkflowId::new(uuid::Uuid::new_v4().to_string());
```

### 2. 超时配置

```rust
// ✅ 好: 合理的超时配置
let options = StartWorkflowOptions {
    task_queue: "order-processing".to_string(),
    
    // 整个工作流的最长执行时间
    workflow_execution_timeout: Some(Duration::from_secs(3600)), // 1小时
    
    // 单次运行的最长时间（支持Continue As New）
    workflow_run_timeout: Some(Duration::from_secs(600)), // 10分钟
    
    // 单个Workflow Task的处理时间
    workflow_task_timeout: Some(Duration::from_secs(10)),
    
    ..Default::default()
};
```

### 3. 错误处理

```rust
// ✅ 好: 完整的错误处理
match client.start_workflow::<OrderWorkflow>(order, options).await {
    Ok(handle) => {
        tracing::info!("Workflow started: {}", handle.workflow_id().as_str());
        
        // 等待结果，处理各种情况
        match handle.get_result().await {
            Ok(result) => {
                println!("Success: {:?}", result);
            }
            Err(ClientError::WorkflowNotFound(_)) => {
                eprintln!("Workflow was deleted");
            }
            Err(ClientError::Timeout(_)) => {
                eprintln!("Workflow timed out");
            }
            Err(e) => {
                eprintln!("Workflow failed: {:?}", e);
            }
        }
    }
    Err(ClientError::WorkflowAlreadyExists(id)) => {
        tracing::warn!("Workflow already running: {}", id);
        // 获取现有句柄
        let handle = client.get_workflow_handle::<OrderWorkflow>(
            WorkflowId::new(id)
        );
    }
    Err(e) => {
        tracing::error!("Failed to start workflow: {:?}", e);
        return Err(e.into());
    }
}
```

### 4. Signal和Query

```rust
// ✅ 好: 使用类型安全的Signal和Query
#[derive(Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
    pub approver: String,
    pub comment: String,
}

impl Signal for ApprovalSignal {
    fn name() -> &'static str {
        "approval"
    }
}

// 发送Signal
handle.signal(ApprovalSignal {
    approved: true,
    approver: "manager-001".to_string(),
    comment: "Looks good".to_string(),
}).await?;

// 执行Query
pub struct OrderStatusQuery;

impl Query for OrderStatusQuery {
    fn name() -> &'static str {
        "order_status"
    }
    
    type Result = OrderStatus;
}

let status = handle.query::<OrderStatusQuery>().await?;
```

### 5. 连接池管理

```rust
// ✅ 好: 复用客户端连接
pub struct AppState {
    temporal_client: Arc<WorkflowClient>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = WorkflowClient::new(ClientConfig::default()).await?;
        
        Ok(Self {
            temporal_client: Arc::new(client),
        })
    }
}

// 在多个请求中复用
async fn handle_order(
    State(app): State<Arc<AppState>>,
    Json(order): Json<Order>,
) -> Result<Json<OrderResult>, StatusCode> {
    let handle = app.temporal_client
        .start_workflow::<OrderWorkflow>(order, options)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let result = handle.get_result().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(result))
}
```

---

## 📊 API对比

### Rust vs Golang

| 操作 | Rust | Golang |
|------|------|--------|
| **创建客户端** | `WorkflowClient::new(config)` | `client.Dial(options)` |
| **启动工作流** | `client.start_workflow::<W>(input, opts)` | `c.ExecuteWorkflow(ctx, opts, fn, input)` |
| **获取句柄** | `client.get_workflow_handle::<W>(id)` | `c.GetWorkflow(ctx, workflowID, runID)` |
| **发送Signal** | `handle.signal(signal)` | `c.SignalWorkflow(ctx, id, run, name, data)` |
| **执行Query** | `handle.query::<Q>()` | `c.QueryWorkflow(ctx, id, run, name)` |
| **等待结果** | `handle.get_result()` | `we.Get(ctx, &result)` |
| **取消** | `handle.cancel()` | `c.CancelWorkflow(ctx, id, run)` |
| **终止** | `handle.terminate(reason)` | `c.TerminateWorkflow(ctx, id, run, reason)` |

---

## 📚 总结

### 客户端核心功能

1. **启动工作流**: 创建新工作流实例
2. **Signal发送**: 向运行中的工作流发送信号
3. **Query执行**: 查询工作流状态
4. **工作流管理**: 取消、终止工作流
5. **结果等待**: 同步或异步获取结果

### Rust vs Golang 1

- **Rust**: 类型安全的泛型API，编译时检查
- **Golang**: 基于interface{}的动态API，运行时检查

### 关键设计点

- **类型安全**: 使用泛型确保Workflow/Signal/Query类型匹配
- **异步设计**: 完全异步的API
- **连接复用**: 客户端可安全共享
- **错误处理**: 详细的错误类型

---

## 📚 下一步

- **可观测性**: [监控与追踪](./14_observability.md)
- **部署指南**: [生产部署](./15_deployment.md)
- **最佳实践**: [设计原则](./16_best_practices.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
