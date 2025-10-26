# Activity 定义

## 📋 文档概述

本文档详细阐述基于Temporal的Activity定义，包括：

- Temporal Activity核心概念
- Rust 1.90实现
- Golang实现对比
- 心跳机制
- 取消处理
- 重试策略
- 最佳实践

---

## 🎯 Temporal Activity 概念

### 什么是Activity?

**Activity**是Temporal中执行**实际业务逻辑**的组件。与Workflow不同，Activity可以：

1. **执行非确定性操作**: HTTP请求、数据库访问、文件I/O等
2. **失败和重试**: 自动重试失败的操作
3. **超时保护**: 自动处理长时间运行的任务
4. **心跳机制**: 报告进度，防止假死
5. **优雅取消**: 响应取消请求

### Workflow vs Activity

```text
┌─────────────────────────────────────────────────────────────┐
│                  Workflow vs Activity                        │
└─────────────────────────────────────────────────────────────┘

Workflow (协调器)
├─ 必须是确定性的
├─ 不能直接访问外部系统
├─ 可以运行数月甚至数年
├─ 编排Activities和其他Workflows
└─ 状态自动持久化

Activity (执行器)
├─ 可以是非确定性的
├─ 可以访问外部系统 (数据库、API、文件系统)
├─ 通常运行几秒到几分钟
├─ 执行具体的业务逻辑
└─ 支持心跳和取消
```

### Activity 生命周期

```text
┌─────────────────────────────────────────────────────────────┐
│                    Activity 生命周期                         │
└─────────────────────────────────────────────────────────────┘

Workflow
    │
    ├─ ExecuteActivity() ──────┐
    │                           │
    │                           ▼
    │                   ┌────────────────┐
    │                   │  Temporal      │
    │                   │  Service       │
    │                   └────────────────┘
    │                           │
    │                    Schedule Task
    │                           │
    │                           ▼
    │                   ┌────────────────┐
    │                   │  Worker        │
    │                   │                │
    │                   │  ┌──────────┐  │
    │                   │  │ Activity │  │
    │                   │  │ Executor │  │
    │                   │  └──────────┘  │
    │                   │       │        │
    │                   │       ├─ Start
    │                   │       ├─ Heartbeat (定期)
    │                   │       ├─ Complete / Fail
    │                   │       └─ Cancel (可选)
    │                   └────────────────┘
    │                           │
    │                     返回结果
    │                           │
    ◀───────────────────────────┘
```

---

## 🦀 Rust实现

### Activity Trait定义

```rust
/// Activity trait - 定义Activity接口
pub trait Activity: Send + Sync + 'static {
    /// 输入类型
    type Input: DeserializeOwned + Send + 'static;
    
    /// 输出类型
    type Output: Serialize + Send + 'static;
    
    /// Activity名称
    fn name() -> &'static str;
    
    /// 执行Activity
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send;
}
```

### ActivityContext

```rust
/// Activity上下文 - 提供Activity执行环境
pub struct ActivityContext {
    /// Activity ID
    pub(crate) activity_id: ActivityId,
    
    /// 工作流执行信息
    pub(crate) workflow_execution: WorkflowExecution,
    
    /// Activity类型
    pub(crate) activity_type: String,
    
    /// 心跳句柄
    pub(crate) heartbeat: Arc<HeartbeatHandle>,
    
    /// 取消令牌
    pub(crate) cancellation: CancellationToken,
    
    /// Activity信息
    pub(crate) info: ActivityInfo,
    
    /// 日志记录器
    pub(crate) logger: Logger,
}

impl ActivityContext {
    /// 获取Activity ID
    pub fn activity_id(&self) -> &ActivityId {
        &self.activity_id
    }
    
    /// 获取工作流执行信息
    pub fn workflow_execution(&self) -> &WorkflowExecution {
        &self.workflow_execution
    }
    
    /// 发送心跳
    pub async fn heartbeat(&self) -> Result<(), ActivityError> {
        self.heartbeat.record().await
    }
    
    /// 发送带详情的心跳
    pub async fn heartbeat_with_details<T: Serialize>(
        &self,
        details: T,
    ) -> Result<(), ActivityError> {
        self.heartbeat.record_with_details(details).await
    }
    
    /// 检查是否被取消
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }
    
    /// 等待取消
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await
    }
    
    /// 获取Activity信息
    pub fn get_info(&self) -> &ActivityInfo {
        &self.info
    }
    
    /// 获取日志记录器
    pub fn logger(&self) -> &Logger {
        &self.logger
    }
}
```

### 基础Activity示例

#### 1. 简单计算Activity

```rust
use serde::{Deserialize, Serialize};
use crate::activity::{Activity, ActivityContext, ActivityError};

#[derive(Debug, Deserialize)]
pub struct CalculationInput {
    pub a: i32,
    pub b: i32,
    pub operation: String,
}

#[derive(Debug, Serialize)]
pub struct CalculationOutput {
    pub result: i32,
}

pub struct CalculationActivity;

impl Activity for CalculationActivity {
    type Input = CalculationInput;
    type Output = CalculationOutput;
    
    fn name() -> &'static str {
        "Calculation"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            let result = match input.operation.as_str() {
                "add" => input.a + input.b,
                "subtract" => input.a - input.b,
                "multiply" => input.a * input.b,
                "divide" => {
                    if input.b == 0 {
                        return Err(ActivityError::ExecutionFailed(
                            "Division by zero".into()
                        ));
                    }
                    input.a / input.b
                }
                _ => {
                    return Err(ActivityError::InvalidInput(
                        format!("Unknown operation: {}", input.operation)
                    ));
                }
            };
            
            Ok(CalculationOutput { result })
        }
    }
}
```

#### 2. HTTP请求Activity

```rust
use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct HttpRequestInput {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct HttpResponseOutput {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub struct HttpRequestActivity;

impl Activity for HttpRequestActivity {
    type Input = HttpRequestInput;
    type Output = HttpResponseOutput;
    
    fn name() -> &'static str {
        "HttpRequest"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 创建HTTP客户端
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| ActivityError::ExecutionFailed(e.to_string()))?;
            
            // 构建请求
            let mut request = match input.method.as_str() {
                "GET" => client.get(&input.url),
                "POST" => client.post(&input.url),
                "PUT" => client.put(&input.url),
                "DELETE" => client.delete(&input.url),
                _ => {
                    return Err(ActivityError::InvalidInput(
                        format!("Unsupported method: {}", input.method)
                    ));
                }
            };
            
            // 添加headers
            for (key, value) in input.headers {
                request = request.header(key, value);
            }
            
            // 添加body
            if let Some(body) = input.body {
                request = request.body(body);
            }
            
            // 发送心跳
            ctx.heartbeat().await?;
            
            // 执行请求
            let response = request
                .send()
                .await
                .map_err(|e| ActivityError::ExecutionFailed(e.to_string()))?;
            
            // 解析响应
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let body = response
                .text()
                .await
                .map_err(|e| ActivityError::ExecutionFailed(e.to_string()))?;
            
            Ok(HttpResponseOutput {
                status,
                body,
                headers,
            })
        }
    }
}
```

#### 3. 数据库操作Activity

```rust
use sqlx::{PgPool, FromRow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
    pub age: i32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserOutput {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct CreateUserActivity {
    pool: PgPool,
}

impl CreateUserActivity {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Activity for CreateUserActivity {
    type Input = CreateUserInput;
    type Output = UserOutput;
    
    fn name() -> &'static str {
        "CreateUser"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 发送心跳
            ctx.heartbeat().await?;
            
            // 检查取消
            if ctx.is_cancelled() {
                return Err(ActivityError::Cancelled);
            }
            
            // 执行数据库插入
            let user = sqlx::query_as::<_, UserOutput>(
                r#"
                INSERT INTO users (name, email, age)
                VALUES ($1, $2, $3)
                RETURNING id, name, email, age, created_at
                "#
            )
            .bind(&input.name)
            .bind(&input.email)
            .bind(input.age)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ActivityError::ExecutionFailed(e.to_string()))?;
            
            ctx.logger().info("User created", o!(
                "user_id" => user.id,
                "name" => &user.name
            ));
            
            Ok(user)
        }
    }
}
```

### 带心跳的长时间运行Activity

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
pub struct LongRunningInput {
    pub total_items: usize,
}

#[derive(Debug, Serialize)]
pub struct LongRunningOutput {
    pub processed: usize,
}

#[derive(Debug, Serialize)]
pub struct ProcessProgress {
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
}

pub struct LongRunningActivity;

impl Activity for LongRunningActivity {
    type Input = LongRunningInput;
    type Output = LongRunningOutput;
    
    fn name() -> &'static str {
        "LongRunning"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            let mut processed = 0;
            
            for i in 0..input.total_items {
                // 检查是否被取消
                if ctx.is_cancelled() {
                    ctx.logger().warn("Activity cancelled", o!(
                        "processed" => processed,
                        "total" => input.total_items
                    ));
                    return Err(ActivityError::Cancelled);
                }
                
                // 处理单个项目
                process_item(i).await?;
                processed += 1;
                
                // 每处理10个项目发送一次心跳
                if processed % 10 == 0 {
                    let progress = ProcessProgress {
                        current: processed,
                        total: input.total_items,
                        percentage: (processed as f64 / input.total_items as f64) * 100.0,
                    };
                    
                    ctx.heartbeat_with_details(progress).await?;
                }
                
                // 模拟处理时间
                sleep(Duration::from_millis(100)).await;
            }
            
            ctx.logger().info("Processing completed", o!(
                "processed" => processed
            ));
            
            Ok(LongRunningOutput { processed })
        }
    }
}

async fn process_item(index: usize) -> Result<(), ActivityError> {
    // 实际的处理逻辑
    Ok(())
}
```

### 支持取消的Activity

```rust
use tokio::select;
use tokio::time::{sleep, Duration};

pub struct CancellableActivity;

impl Activity for CancellableActivity {
    type Input = WorkInput;
    type Output = WorkOutput;
    
    fn name() -> &'static str {
        "CancellableWork"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            let work_future = async {
                // 执行实际工作
                for i in 0..100 {
                    heavy_computation(i).await;
                    ctx.heartbeat().await?;
                    sleep(Duration::from_millis(100)).await;
                }
                Ok::<_, ActivityError>(WorkOutput { completed: true })
            };
            
            // 同时等待工作完成或取消信号
            select! {
                result = work_future => result,
                _ = ctx.cancelled() => {
                    // 执行清理工作
                    cleanup().await;
                    Err(ActivityError::Cancelled)
                }
            }
        }
    }
}
```

---

## 🐹 Golang实现对比

### Activity定义 - Golang

```go
package activities

import (
    "context"
    
    "go.temporal.io/sdk/activity"
)

// 输入类型
type CalculationInput struct {
    A         int
    B         int
    Operation string
}

// 输出类型
type CalculationOutput struct {
    Result int
}

// Activity函数
func CalculationActivity(ctx context.Context, input CalculationInput) (CalculationOutput, error) {
    var result int
    
    switch input.Operation {
    case "add":
        result = input.A + input.B
    case "subtract":
        result = input.A - input.B
    case "multiply":
        result = input.A * input.B
    case "divide":
        if input.B == 0 {
            return CalculationOutput{}, errors.New("division by zero")
        }
        result = input.A / input.B
    default:
        return CalculationOutput{}, fmt.Errorf("unknown operation: %s", input.Operation)
    }
    
    return CalculationOutput{Result: result}, nil
}
```

### HTTP请求Activity - Golang

```go
type HttpRequestInput struct {
    URL     string
    Method  string
    Body    string
    Headers map[string]string
}

type HttpResponseOutput struct {
    Status  int
    Body    string
    Headers map[string]string
}

func HttpRequestActivity(ctx context.Context, input HttpRequestInput) (HttpResponseOutput, error) {
    // 创建HTTP客户端
    client := &http.Client{
        Timeout: 30 * time.Second,
    }
    
    // 构建请求
    var req *http.Request
    var err error
    
    if input.Body != "" {
        req, err = http.NewRequestWithContext(ctx, input.Method, input.URL, strings.NewReader(input.Body))
    } else {
        req, err = http.NewRequestWithContext(ctx, input.Method, input.URL, nil)
    }
    
    if err != nil {
        return HttpResponseOutput{}, err
    }
    
    // 添加headers
    for k, v := range input.Headers {
        req.Header.Set(k, v)
    }
    
    // 发送心跳
    activity.RecordHeartbeat(ctx, "sending request")
    
    // 执行请求
    resp, err := client.Do(req)
    if err != nil {
        return HttpResponseOutput{}, err
    }
    defer resp.Body.Close()
    
    // 读取响应
    body, err := io.ReadAll(resp.Body)
    if err != nil {
        return HttpResponseOutput{}, err
    }
    
    // 构建响应
    headers := make(map[string]string)
    for k, v := range resp.Header {
        if len(v) > 0 {
            headers[k] = v[0]
        }
    }
    
    return HttpResponseOutput{
        Status:  resp.StatusCode,
        Body:    string(body),
        Headers: headers,
    }, nil
}
```

### 长时间运行Activity - Golang

```go
type LongRunningInput struct {
    TotalItems int
}

type LongRunningOutput struct {
    Processed int
}

type ProcessProgress struct {
    Current    int
    Total      int
    Percentage float64
}

func LongRunningActivity(ctx context.Context, input LongRunningInput) (LongRunningOutput, error) {
    logger := activity.GetLogger(ctx)
    processed := 0
    
    for i := 0; i < input.TotalItems; i++ {
        // 检查是否被取消
        select {
        case <-ctx.Done():
            logger.Warn("Activity cancelled", "processed", processed, "total", input.TotalItems)
            return LongRunningOutput{}, ctx.Err()
        default:
            // 继续处理
        }
        
        // 处理单个项目
        if err := processItem(i); err != nil {
            return LongRunningOutput{}, err
        }
        processed++
        
        // 每处理10个项目发送一次心跳
        if processed%10 == 0 {
            progress := ProcessProgress{
                Current:    processed,
                Total:      input.TotalItems,
                Percentage: float64(processed) / float64(input.TotalItems) * 100.0,
            }
            activity.RecordHeartbeat(ctx, progress)
        }
        
        // 模拟处理时间
        time.Sleep(100 * time.Millisecond)
    }
    
    logger.Info("Processing completed", "processed", processed)
    
    return LongRunningOutput{Processed: processed}, nil
}
```

---

## 🔄 Rust vs Golang 详细对比

### 对比表

| 特性 | Rust | Golang |
|------|------|--------|
| **Activity定义** | Trait实现 | 普通函数 |
| **上下文传递** | ActivityContext结构体 | context.Context |
| **心跳机制** | `ctx.heartbeat()` | `activity.RecordHeartbeat()` |
| **取消处理** | `ctx.is_cancelled()` / `ctx.cancelled()` | `ctx.Done()` channel |
| **错误处理** | `Result<T, ActivityError>` | `(T, error)` |
| **类型安全** | 编译时完全检查 | 运行时部分检查 |
| **并发模型** | async/await | goroutine |

### 心跳机制对比

**Rust**: 显式API

```rust
// 简单心跳
ctx.heartbeat().await?;

// 带进度的心跳
ctx.heartbeat_with_details(ProcessProgress {
    current: 50,
    total: 100,
    percentage: 50.0,
}).await?;
```

**Golang**: 全局函数

```go
// 简单心跳
activity.RecordHeartbeat(ctx, "processing")

// 带进度的心跳
activity.RecordHeartbeat(ctx, ProcessProgress{
    Current:    50,
    Total:      100,
    Percentage: 50.0,
})
```

### 取消处理对比

**Rust**: 使用CancellationToken

```rust
// 检查是否被取消
if ctx.is_cancelled() {
    return Err(ActivityError::Cancelled);
}

// 等待取消信号
select! {
    result = work() => result,
    _ = ctx.cancelled() => {
        cleanup().await;
        Err(ActivityError::Cancelled)
    }
}
```

**Golang**: 使用context.Context

```go
// 检查是否被取消
select {
case <-ctx.Done():
    return Output{}, ctx.Err()
default:
    // 继续工作
}

// 等待取消信号
select {
case result := <-workChan:
    return result, nil
case <-ctx.Done():
    cleanup()
    return Output{}, ctx.Err()
}
```

---

## ⚙️ Activity选项配置

### Rust实现

```rust
#[derive(Debug, Clone)]
pub struct ActivityOptions {
    /// Activity ID
    pub activity_id: Option<ActivityId>,
    
    /// 任务队列
    pub task_queue: Option<String>,
    
    /// 调度到开始超时
    pub schedule_to_start_timeout: Option<Duration>,
    
    /// 开始到关闭超时
    pub start_to_close_timeout: Option<Duration>,
    
    /// 调度到关闭超时
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

### 重试策略

```rust
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_attempts: u32,
    
    /// 初始重试间隔
    pub initial_interval: Duration,
    
    /// 最大重试间隔
    pub max_interval: Duration,
    
    /// 退避系数
    pub backoff_coefficient: f64,
    
    /// 不重试的错误类型
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
```

### 使用示例

```rust
// 在Workflow中执行Activity
let result = ctx
    .execute_activity::<MyActivity>(
        input,
        ActivityOptions {
            start_to_close_timeout: Some(Duration::from_secs(30)),
            heartbeat_timeout: Some(Duration::from_secs(10)),
            retry_policy: Some(RetryPolicy {
                max_attempts: 5,
                initial_interval: Duration::from_secs(1),
                max_interval: Duration::from_secs(60),
                backoff_coefficient: 2.0,
                non_retryable_error_types: vec![
                    "ValidationError".to_string(),
                    "AuthenticationError".to_string(),
                ],
            }),
            ..Default::default()
        },
    )
    .await?;
```

---

## 🎯 最佳实践

### 1. 幂等性

```rust
// ✅ 好: 幂等Activity
impl Activity for CreateUserActivity {
    fn execute(ctx: ActivityContext, input: Self::Input) -> impl Future<...> {
        async move {
            // 检查用户是否已存在
            let existing_user = db.find_user_by_email(&input.email).await?;
            if let Some(user) = existing_user {
                // 用户已存在，返回现有用户
                return Ok(user);
            }
            
            // 创建新用户
            let user = db.create_user(input).await?;
            Ok(user)
        }
    }
}

// ❌ 差: 非幂等Activity
impl Activity for CreateUserActivity {
    fn execute(ctx: ActivityContext, input: Self::Input) -> impl Future<...> {
        async move {
            // 直接创建，可能导致重复
            let user = db.create_user(input).await?;
            Ok(user)
        }
    }
}
```

### 2. 适当的超时设置

```rust
// ✅ 好: 根据实际情况设置超时
ActivityOptions {
    // 快速操作: 短超时
    start_to_close_timeout: Some(Duration::from_secs(10)),
    heartbeat_timeout: Some(Duration::from_secs(3)),
    ..Default::default()
}

// 长时间运行: 长超时 + 心跳
ActivityOptions {
    start_to_close_timeout: Some(Duration::from_secs(3600)), // 1小时
    heartbeat_timeout: Some(Duration::from_secs(60)), // 1分钟心跳
    ..Default::default()
}
```

### 3. 错误分类

```rust
#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    /// 可重试的错误
    #[error("Temporary failure: {0}")]
    TemporaryFailure(String),
    
    /// 不可重试的错误
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    /// 执行失败
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    /// 取消
    #[error("Activity cancelled")]
    Cancelled,
}

// 使用
impl Activity for MyActivity {
    fn execute(ctx: ActivityContext, input: Self::Input) -> impl Future<...> {
        async move {
            // 验证错误 - 不重试
            if input.value < 0 {
                return Err(ActivityError::ValidationFailed(
                    "Value must be positive".into()
                ));
            }
            
            // 网络错误 - 可重试
            match network_call().await {
                Ok(result) => Ok(result),
                Err(e) if e.is_temporary() => {
                    Err(ActivityError::TemporaryFailure(e.to_string()))
                }
                Err(e) => {
                    Err(ActivityError::ExecutionFailed(e.to_string()))
                }
            }
        }
    }
}
```

### 4. 心跳策略

```rust
// ✅ 好: 定期发送心跳
for i in 0..total_items {
    process_item(i).await?;
    
    // 每处理N个项目发送心跳
    if i % 10 == 0 {
        ctx.heartbeat_with_details(Progress {
            current: i,
            total: total_items,
        }).await?;
    }
}

// ❌ 差: 心跳太频繁
for i in 0..total_items {
    process_item(i).await?;
    ctx.heartbeat().await?;  // 每次迭代都心跳，开销大
}
```

### 5. 资源管理

```rust
// ✅ 好: 使用RAII管理资源
impl Activity for DatabaseActivity {
    fn execute(ctx: ActivityContext, input: Self::Input) -> impl Future<...> {
        async move {
            // 获取连接（自动释放）
            let conn = pool.get().await?;
            
            // 使用连接
            let result = conn.query(...).await?;
            
            // conn在作用域结束时自动返回池
            Ok(result)
        }
    }
}
```

---

## 📚 总结

### Activity设计要点

1. **幂等性**: Activity应该是幂等的，可以安全重试
2. **超时**: 合理设置各种超时参数
3. **心跳**: 长时间运行的Activity需要定期发送心跳
4. **取消**: 支持优雅取消
5. **错误分类**: 区分可重试和不可重试的错误
6. **资源管理**: 正确管理外部资源

### Rust vs Golang

- **Rust优势**: 类型安全、零成本抽象、明确的错误处理
- **Golang优势**: 简单直观、成熟的SDK、丰富的示例

---

## 📚 下一步

- **Signal与Query**: [工作流交互](./06_signals_and_queries.md)
- **错误处理**: [错误类型详解](./error_handling.md)
- **实战示例**: [完整案例](./18_basic_examples.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
