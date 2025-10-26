# 基础示例

## 📋 文档概述

本文档提供Temporal-Rust的基础示例，包括：

- Hello World
- 简单工作流
- Activity调用
- Signal和Query
- 错误处理
- Rust + Golang并列对比

---

## 🌟 Hello World

### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// 输入类型
#[derive(Serialize, Deserialize)]
pub struct GreetingInput {
    pub name: String,
}

// 输出类型
#[derive(Serialize, Deserialize)]
pub struct GreetingOutput {
    pub message: String,
}

// 定义工作流
pub struct HelloWorkflow;

impl Workflow for HelloWorkflow {
    type Input = GreetingInput;
    type Output = GreetingOutput;
    
    fn name() -> &'static str {
        "HelloWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let message = format!("Hello, {}!", input.name);
        
        tracing::info!(
            name = %input.name,
            message = %message,
            "Greeting generated"
        );
        
        Ok(GreetingOutput { message })
    }
}

// 主程序
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建Worker
    let config = WorkerConfig::builder()
        .task_queue("hello-queue")
        .build();
    
    let worker = WorkflowWorker::new(config);
    worker.register_workflow::<HelloWorkflow>().await;
    
    // 启动Worker（在后台）
    tokio::spawn(async move {
        worker.run().await.ok();
    });
    
    // 创建客户端并启动工作流
    let client = WorkflowClient::new(ClientConfig::default()).await?;
    
    let handle = client.start_workflow::<HelloWorkflow>(
        GreetingInput {
            name: "Temporal".to_string(),
        },
        StartWorkflowOptions {
            task_queue: "hello-queue".to_string(),
            ..Default::default()
        },
    ).await?;
    
    let result = handle.get_result().await?;
    println!("{}", result.message);
    
    Ok(())
}
```

### Golang对比

```go
package main

import (
    "context"
    "fmt"
    "log"

    "go.temporal.io/sdk/client"
    "go.temporal.io/sdk/worker"
    "go.temporal.io/sdk/workflow"
)

// 输入类型
type GreetingInput struct {
    Name string
}

// 输出类型
type GreetingOutput struct {
    Message string
}

// 工作流定义
func HelloWorkflow(ctx workflow.Context, input GreetingInput) (GreetingOutput, error) {
    message := fmt.Sprintf("Hello, %s!", input.Name)
    
    workflow.GetLogger(ctx).Info("Greeting generated", "name", input.Name, "message", message)
    
    return GreetingOutput{Message: message}, nil
}

func main() {
    // 创建客户端
    c, err := client.Dial(client.Options{})
    if err != nil {
        log.Fatal(err)
    }
    defer c.Close()
    
    // 启动Worker
    w := worker.New(c, "hello-queue", worker.Options{})
    w.RegisterWorkflow(HelloWorkflow)
    
    go func() {
        err := w.Run(worker.InterruptCh())
        if err != nil {
            log.Fatal(err)
        }
    }()
    
    // 启动工作流
    workflowOptions := client.StartWorkflowOptions{
        TaskQueue: "hello-queue",
    }
    
    we, err := c.ExecuteWorkflow(
        context.Background(),
        workflowOptions,
        HelloWorkflow,
        GreetingInput{Name: "Temporal"},
    )
    if err != nil {
        log.Fatal(err)
    }
    
    // 获取结果
    var result GreetingOutput
    err = we.Get(context.Background(), &result)
    if err != nil {
        log.Fatal(err)
    }
    
    fmt.Println(result.Message)
}
```

---

## 🔄 简单工作流

### 示例：用户注册流程

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// 输入
#[derive(Serialize, Deserialize)]
pub struct UserRegistrationInput {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

// 输出
#[derive(Serialize, Deserialize)]
pub struct UserRegistrationOutput {
    pub user_id: String,
    pub status: String,
}

// Activity: 创建用户
pub struct CreateUserActivity;

#[derive(Serialize, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserOutput {
    pub user_id: String,
}

impl Activity for CreateUserActivity {
    type Input = CreateUserInput;
    type Output = CreateUserOutput;
    
    fn name() -> &'static str {
        "CreateUser"
    }
    
    async fn execute(
        _ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        // 模拟数据库操作
        let user_id = format!("user-{}", uuid::Uuid::new_v4());
        
        tracing::info!(
            user_id = %user_id,
            email = %input.email,
            "User created"
        );
        
        Ok(CreateUserOutput { user_id })
    }
}

// Activity: 发送欢迎邮件
pub struct SendWelcomeEmailActivity;

#[derive(Serialize, Deserialize)]
pub struct SendEmailInput {
    pub user_id: String,
    pub email: String,
}

impl Activity for SendWelcomeEmailActivity {
    type Input = SendEmailInput;
    type Output = ();
    
    fn name() -> &'static str {
        "SendWelcomeEmail"
    }
    
    async fn execute(
        _ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(
            user_id = %input.user_id,
            email = %input.email,
            "Welcome email sent"
        );
        
        Ok(())
    }
}

// 工作流
pub struct UserRegistrationWorkflow;

impl Workflow for UserRegistrationWorkflow {
    type Input = UserRegistrationInput;
    type Output = UserRegistrationOutput;
    
    fn name() -> &'static str {
        "UserRegistration"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        // 1. 创建用户
        let user = ctx.execute_activity::<CreateUserActivity>(
            CreateUserInput {
                email: input.email.clone(),
                username: input.username,
                password_hash: input.password_hash,
            },
            ActivityOptions::default(),
        ).await?;
        
        // 2. 发送欢迎邮件
        ctx.execute_activity::<SendWelcomeEmailActivity>(
            SendEmailInput {
                user_id: user.user_id.clone(),
                email: input.email,
            },
            ActivityOptions::default(),
        ).await?;
        
        Ok(UserRegistrationOutput {
            user_id: user.user_id,
            status: "completed".to_string(),
        })
    }
}
```

#### Golang对比

```go
package workflows

import (
    "fmt"
    "go.temporal.io/sdk/workflow"
)

type UserRegistrationInput struct {
    Email        string
    Username     string
    PasswordHash string
}

type UserRegistrationOutput struct {
    UserID string
    Status string
}

func UserRegistrationWorkflow(ctx workflow.Context, input UserRegistrationInput) (UserRegistrationOutput, error) {
    // 1. 创建用户
    var userID string
    err := workflow.ExecuteActivity(ctx, CreateUserActivity, input).Get(ctx, &userID)
    if err != nil {
        return UserRegistrationOutput{}, err
    }
    
    // 2. 发送欢迎邮件
    err = workflow.ExecuteActivity(ctx, SendWelcomeEmailActivity, userID, input.Email).Get(ctx, nil)
    if err != nil {
        return UserRegistrationOutput{}, err
    }
    
    return UserRegistrationOutput{
        UserID: userID,
        Status: "completed",
    }, nil
}
```

---

## 📡 Signal和Query示例

### 示例：可控制的长时间运行任务

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// 工作流状态
#[derive(Clone, Serialize, Deserialize)]
pub struct ProcessingState {
    pub total_items: usize,
    pub processed_items: usize,
    pub is_paused: bool,
}

// Signal: 暂停
#[derive(Serialize, Deserialize)]
pub struct PauseSignal;

impl Signal for PauseSignal {
    fn name() -> &'static str {
        "pause"
    }
}

// Signal: 恢复
#[derive(Serialize, Deserialize)]
pub struct ResumeSignal;

impl Signal for ResumeSignal {
    fn name() -> &'static str {
        "resume"
    }
}

// Query: 获取状态
pub struct StatusQuery;

impl Query for StatusQuery {
    fn name() -> &'static str {
        "status"
    }
    
    type Result = ProcessingState;
}

// 工作流
pub struct DataProcessingWorkflow {
    state: Arc<RwLock<ProcessingState>>,
}

impl Workflow for DataProcessingWorkflow {
    type Input = usize;  // total items
    type Output = ProcessingState;
    
    fn name() -> &'static str {
        "DataProcessing"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        total_items: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let state = Arc::new(RwLock::new(ProcessingState {
            total_items,
            processed_items: 0,
            is_paused: false,
        }));
        
        // 处理Signal
        let state_clone = state.clone();
        ctx.on_signal::<PauseSignal>(move |_signal| {
            let state = state_clone.clone();
            async move {
                state.write().await.is_paused = true;
                tracing::info!("Processing paused");
            }
        });
        
        let state_clone = state.clone();
        ctx.on_signal::<ResumeSignal>(move |_signal| {
            let state = state_clone.clone();
            async move {
                state.write().await.is_paused = false;
                tracing::info!("Processing resumed");
            }
        });
        
        // 处理Query
        let state_clone = state.clone();
        ctx.on_query::<StatusQuery>(move || {
            let state = state_clone.clone();
            async move {
                Ok(state.read().await.clone())
            }
        });
        
        // 主处理循环
        for i in 0..total_items {
            // 检查暂停状态
            while state.read().await.is_paused {
                ctx.sleep(Duration::from_secs(1)).await;
            }
            
            // 处理项目
            ctx.execute_activity::<ProcessItemActivity>(
                ItemInput { index: i },
                ActivityOptions::default(),
            ).await?;
            
            // 更新状态
            state.write().await.processed_items = i + 1;
        }
        
        Ok(state.read().await.clone())
    }
}
```

#### Golang对比

```go
package workflows

import (
    "time"
    "go.temporal.io/sdk/workflow"
)

type ProcessingState struct {
    TotalItems     int
    ProcessedItems int
    IsPaused       bool
}

func DataProcessingWorkflow(ctx workflow.Context, totalItems int) (ProcessingState, error) {
    state := ProcessingState{
        TotalItems:     totalItems,
        ProcessedItems: 0,
        IsPaused:       false,
    }
    
    // Signal channel
    pauseCh := workflow.GetSignalChannel(ctx, "pause")
    resumeCh := workflow.GetSignalChannel(ctx, "resume")
    
    // Query handler
    err := workflow.SetQueryHandler(ctx, "status", func() (ProcessingState, error) {
        return state, nil
    })
    if err != nil {
        return state, err
    }
    
    // 主处理循环
    for i := 0; i < totalItems; i++ {
        // 检查暂停状态
        for state.IsPaused {
            selector := workflow.NewSelector(ctx)
            selector.AddReceive(resumeCh, func(c workflow.ReceiveChannel, more bool) {
                c.Receive(ctx, nil)
                state.IsPaused = false
            })
            selector.Select(ctx)
        }
        
        // 检查Signal
        selector := workflow.NewSelector(ctx)
        selector.AddReceive(pauseCh, func(c workflow.ReceiveChannel, more bool) {
            c.Receive(ctx, nil)
            state.IsPaused = true
        })
        selector.AddDefault(func() {})
        selector.Select(ctx)
        
        // 处理项目
        err := workflow.ExecuteActivity(ctx, ProcessItemActivity, i).Get(ctx, nil)
        if err != nil {
            return state, err
        }
        
        state.ProcessedItems = i + 1
    }
    
    return state, nil
}
```

---

## ⚠️ 错误处理示例

### 示例：带重试的API调用

#### Rust实现

```rust
use temporal_rust::*;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<ApiError> for ActivityError {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::Network(_) | ApiError::InvalidResponse(_) => {
                ActivityError::Retryable(err.to_string())
            }
            ApiError::NotFound(_) => {
                ActivityError::NonRetryable(err.to_string())
            }
        }
    }
}

pub struct CallExternalApiActivity;

#[derive(Serialize, Deserialize)]
pub struct ApiCallInput {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApiCallOutput {
    pub data: String,
}

impl Activity for CallExternalApiActivity {
    type Input = ApiCallInput;
    type Output = ApiCallOutput;
    
    fn name() -> &'static str {
        "CallExternalApi"
    }
    
    async fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(url = %input.url, "Calling external API");
        
        // 发送心跳
        ctx.record_heartbeat(serde_json::json!({
            "status": "calling_api"
        })).await;
        
        // 模拟API调用
        match call_api(&input.url).await {
            Ok(data) => Ok(ApiCallOutput { data }),
            Err(e) => Err(e.into()),
        }
    }
}

// 工作流with重试策略
pub struct ApiWorkflow;

impl Workflow for ApiWorkflow {
    type Input = String;  // URL
    type Output = String;  // Data
    
    fn name() -> &'static str {
        "ApiWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        url: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        // 配置重试策略
        let retry_policy = RetryPolicy {
            max_attempts: Some(3),
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(10),
            backoff_coefficient: 2.0,
            non_retryable_error_types: vec!["NotFound"],
        };
        
        let result = ctx.execute_activity::<CallExternalApiActivity>(
            ApiCallInput { url },
            ActivityOptions {
                retry_policy: Some(retry_policy),
                start_to_close_timeout: Some(Duration::from_secs(30)),
                ..Default::default()
            },
        ).await?;
        
        Ok(result.data)
    }
}
```

---

## 📚 总结

### 示例覆盖

- ✅ Hello World - 最简单的工作流
- ✅ 用户注册 - 多Activity协作
- ✅ Signal和Query - 运行时交互
- ✅ 错误处理 - 重试策略

### Rust vs Golang对比

| 特性 | Rust | Golang |
|------|------|--------|
| **类型安全** | 编译时检查 | 运行时检查 |
| **错误处理** | Result<T, E> | error interface |
| **并发** | async/await | goroutine + channel |
| **性能** | 零成本抽象 | GC影响 |

---

## 📚 下一步

- **实战示例**: [高级案例](./19_advanced_examples.md)
- **性能优化**: [最佳实践](./16_best_practices.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

