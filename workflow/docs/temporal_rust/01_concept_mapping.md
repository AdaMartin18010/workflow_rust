# 概念映射与思维导图

## 📋 文档概述

本文档提供 Temporal 概念到 Rust 1.90 实现的完整映射，包括：

- 核心概念思维导图
- Rust vs Golang API 详细对比
- 类型系统映射矩阵
- 概念关系属性表

---

## 🗺️ 核心概念完整思维导图

```text
═══════════════════════════════════════════════════════════════════════════
                         TEMPORAL 工作流系统
═══════════════════════════════════════════════════════════════════════════
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
            ┌───────▼────────┐          ┌──────▼───────┐
            │  编程模型       │          │  运行时架构   │
            │  (Programming) │          │  (Runtime)   │
            └───────┬────────┘          └──────┬───────┘
                    │                           │
        ┌───────────┼───────────┐       ┌──────┼──────┬──────┐
        │           │           │       │      │      │      │
    ┌───▼───┐   ┌──▼──┐    ┌──▼──┐  ┌─▼─┐  ┌─▼─┐  ┌─▼──┐  ┌─▼──┐
    │Workflow│   │Activity│  │Signal│  │Server│ │Worker│ │Client│ │Storage│
    │ 工作流  │   │  活动  │  │ 信号  │  │服务器│ │执行器│ │客户端│ │存储层 │
    └───┬───┘   └──┬──┘    └──┬──┘  └───┘  └───┘  └────┘  └────┘
        │          │           │
        │          │           │
═══════════════════════════════════════════════════════════════════════════
                          详细概念展开
═══════════════════════════════════════════════════════════════════════════

1. WORKFLOW (工作流)
   │
   ├─ 定义 (Definition)
   │  ├─ Rust: #[workflow] async fn
   │  ├─ Golang: func(ctx workflow.Context)
   │  └─ 特性: 确定性、可重放、持久化
   │
   ├─ 上下文 (Context)
   │  ├─ Rust: WorkflowContext
   │  ├─ Golang: workflow.Context
   │  └─ 能力:
   │     ├─ execute_activity() / workflow.ExecuteActivity()
   │     ├─ await_signal() / workflow.GetSignalChannel()
   │     ├─ set_query_handler() / workflow.SetQueryHandler()
   │     ├─ sleep() / workflow.Sleep()
   │     ├─ start_child_workflow() / workflow.ExecuteChildWorkflow()
   │     └─ get_info() / workflow.GetInfo()
   │
   ├─ 执行 (Execution)
   │  ├─ WorkflowExecution { workflow_id, run_id }
   │  ├─ 生命周期: Started → Running → Completed/Failed
   │  └─ 事件历史: Event Sourcing
   │
   ├─ 选项 (Options)
   │  ├─ StartWorkflowOptions
   │  │  ├─ id: WorkflowId
   │  │  ├─ task_queue: String
   │  │  ├─ execution_timeout: Duration
   │  │  ├─ run_timeout: Duration
   │  │  └─ retry_policy: Option<RetryPolicy>
   │  └─ 对应 Golang: StartWorkflowOptions
   │
   └─ 高级特性
      ├─ 子工作流 (Child Workflow)
      ├─ 继续执行 (Continue As New)
      ├─ 版本管理 (Versioning)
      └─ 并行执行 (Parallel Execution)

2. ACTIVITY (活动)
   │
   ├─ 定义 (Definition)
   │  ├─ Rust: #[activity] async fn
   │  ├─ Golang: func(ctx context.Context)
   │  └─ 特性: 可重试、有副作用、超时控制
   │
   ├─ 上下文 (Context)
   │  ├─ Rust: ActivityContext
   │  ├─ Golang: context.Context + activity.GetInfo()
   │  └─ 能力:
   │     ├─ heartbeat() / activity.RecordHeartbeat()
   │     ├─ is_cancelled() / ctx.Done()
   │     ├─ get_info() / activity.GetInfo()
   │     └─ get_heartbeat_details() / activity.GetHeartbeatDetails()
   │
   ├─ 选项 (Options)
   │  ├─ ActivityOptions
   │  │  ├─ activity_id: Option<ActivityId>
   │  │  ├─ task_queue: Option<String>
   │  │  ├─ schedule_to_close_timeout: Duration
   │  │  ├─ start_to_close_timeout: Duration
   │  │  ├─ heartbeat_timeout: Duration
   │  │  ├─ retry_policy: Option<RetryPolicy>
   │  │  └─ cancellation_type: CancellationType
   │  └─ 对应 Golang: ActivityOptions
   │
   ├─ 重试策略 (Retry Policy)
   │  ├─ RetryPolicy
   │  │  ├─ initial_interval: Duration
   │  │  ├─ backoff_coefficient: f64
   │  │  ├─ maximum_interval: Duration
   │  │  ├─ maximum_attempts: u32
   │  │  └─ non_retryable_error_types: Vec<String>
   │  └─ 对应 Golang: RetryPolicy
   │
   └─ 执行模式
      ├─ 同步执行
      ├─ 异步执行
      ├─ 本地 Activity
      └─ 心跳机制

3. SIGNAL (信号)
   │
   ├─ 定义 (Definition)
   │  ├─ Rust: Signal trait
   │  │  ├─ fn name() -> &'static str
   │  │  └─ + Serialize + DeserializeOwned
   │  └─ Golang: workflow.SignalChannel
   │
   ├─ 发送 (Send)
   │  ├─ Rust: client.signal_workflow(workflow_id, signal)
   │  └─ Golang: client.SignalWorkflow(ctx, workflowID, runID, signalName, arg)
   │
   ├─ 接收 (Receive)
   │  ├─ Rust: ctx.await_signal::<MySignal>("signal_name")
   │  └─ Golang: signalChan.Receive(ctx, &value)
   │
   └─ 特性
      ├─ 异步通知
      ├─ 外部触发
      ├─ 携带数据
      └─ 多次接收

4. QUERY (查询)
   │
   ├─ 定义 (Definition)
   │  ├─ Rust: Query trait
   │  │  ├─ fn name() -> &'static str
   │  │  ├─ type Result: Serialize + DeserializeOwned
   │  │  └─ + Send + 'static
   │  └─ Golang: workflow.SetQueryHandler()
   │
   ├─ 注册处理器 (Register Handler)
   │  ├─ Rust: ctx.set_query_handler::<MyQuery, _, _>(handler)
   │  └─ Golang: workflow.SetQueryHandler(ctx, "query_name", handler)
   │
   ├─ 执行查询 (Execute Query)
   │  ├─ Rust: client.query_workflow::<MyQuery>(workflow_id)
   │  └─ Golang: client.QueryWorkflow(ctx, workflowID, runID, queryType, args)
   │
   └─ 特性
      ├─ 同步查询
      ├─ 只读操作
      ├─ 不改变状态
      └─ 返回当前状态

5. WORKER (执行器)
   │
   ├─ 组件
   │  ├─ Rust: WorkflowWorker
   │  ├─ Golang: worker.Worker
   │  └─ 职责: 轮询任务、执行工作流/Activity
   │
   ├─ 配置 (Configuration)
   │  ├─ WorkerConfig
   │  │  ├─ task_queue: String
   │  │  ├─ max_concurrent_workflow_executions: usize
   │  │  ├─ max_concurrent_activity_executions: usize
   │  │  ├─ identity: String
   │  │  └─ max_concurrent_local_activities: usize
   │  └─ 对应 Golang: worker.Options
   │
   ├─ 注册 (Registration)
   │  ├─ Rust: worker.register_workflow() / register_activity()
   │  └─ Golang: w.RegisterWorkflow() / w.RegisterActivity()
   │
   ├─ 启动 (Start)
   │  ├─ Rust: worker.start().await
   │  └─ Golang: worker.Run()
   │
   └─ 任务处理
      ├─ 工作流任务轮询
      ├─ Activity 任务轮询
      ├─ 并发控制
      └─ 错误处理

6. CLIENT (客户端)
   │
   ├─ 初始化 (Initialization)
   │  ├─ Rust: WorkflowClient::new(config)
   │  └─ Golang: client.NewClient()
   │
   ├─ 工作流操作 (Workflow Operations)
   │  ├─ 启动: start_workflow() / ExecuteWorkflow()
   │  ├─ Signal: signal_workflow() / SignalWorkflow()
   │  ├─ Query: query_workflow() / QueryWorkflow()
   │  ├─ 取消: cancel_workflow() / CancelWorkflow()
   │  └─ 终止: terminate_workflow() / TerminateWorkflow()
   │
   ├─ 工作流句柄 (Workflow Handle)
   │  ├─ Rust: WorkflowHandle<O>
   │  │  ├─ get_result() → Future<Output = Result<O>>
   │  │  ├─ signal()
   │  │  └─ cancel()
   │  └─ Golang: WorkflowRun
   │     ├─ Get() → error
   │     ├─ GetID() → string
   │     └─ GetRunID() → string
   │
   └─ 批量操作
      ├─ 列表工作流
      ├─ 批量 Signal
      └─ 批量终止

7. STORAGE (存储层)
   │
   ├─ 抽象接口
   │  ├─ Rust: WorkflowStorage trait
   │  │  ├─ save_workflow_execution()
   │  │  ├─ load_workflow_execution()
   │  │  ├─ append_event()
   │  │  ├─ get_event_history()
   │  │  └─ save_activity_heartbeat()
   │  └─ Temporal Server: 内部实现
   │
   ├─ 实现
   │  ├─ PostgresWorkflowStorage
   │  ├─ MySQLWorkflowStorage
   │  └─ InMemoryWorkflowStorage (测试用)
   │
   ├─ 数据模型
   │  ├─ workflow_executions 表
   │  ├─ workflow_events 表
   │  ├─ activity_heartbeats 表
   │  └─ timer_info 表
   │
   └─ 事件溯源 (Event Sourcing)
      ├─ 事件追加
      ├─ 事件重放
      └─ 状态重建

8. EVENT HISTORY (事件历史)
   │
   ├─ 事件类型 (Event Types)
   │  ├─ WorkflowEvent 枚举
   │  │  ├─ WorkflowExecutionStarted
   │  │  ├─ WorkflowExecutionCompleted
   │  │  ├─ WorkflowExecutionFailed
   │  │  ├─ ActivityTaskScheduled
   │  │  ├─ ActivityTaskStarted
   │  │  ├─ ActivityTaskCompleted
   │  │  ├─ ActivityTaskFailed
   │  │  ├─ TimerStarted
   │  │  ├─ TimerFired
   │  │  ├─ WorkflowSignalReceived
   │  │  └─ ChildWorkflowExecutionStarted
   │  └─ 对应 Temporal: History Events
   │
   ├─ 事件属性
   │  ├─ event_id: EventId (递增序号)
   │  ├─ timestamp: DateTime<Utc>
   │  ├─ event_type: String
   │  └─ event_data: Value
   │
   ├─ 事件历史管理
   │  ├─ EventHistory 结构
   │  ├─ append() - 追加事件
   │  ├─ get_events() - 获取所有事件
   │  └─ replay() - 重放事件
   │
   └─ 重放机制 (Replay)
      ├─ 确定性保证
      ├─ 状态重建
      └─ 错误恢复

9. 高级模式 (Advanced Patterns)
   │
   ├─ SAGA 模式
   │  ├─ Rust: Saga<C, E>
   │  │  ├─ SagaStep trait
   │  │  │  ├─ execute()
   │  │  │  └─ compensate()
   │  │  └─ Saga::execute()
   │  ├─ Golang: 自定义实现
   │  └─ 特性:
   │     ├─ 长事务管理
   │     ├─ 补偿机制
   │     └─ 最终一致性
   │
   ├─ 子工作流 (Child Workflow)
   │  ├─ Rust: ChildWorkflowHandle<T>
   │  │  └─ wait_for_completion()
   │  ├─ Golang: workflow.ExecuteChildWorkflow()
   │  └─ 特性:
   │     ├─ 独立执行
   │     ├─ 可嵌套
   │     └─ 生命周期管理
   │
   ├─ 定时器 (Timer)
   │  ├─ Rust: WorkflowTimer
   │  │  └─ wait()
   │  ├─ Golang: workflow.NewTimer()
   │  └─ 特性:
   │     ├─ 持久化
   │     ├─ 可取消
   │     └─ 精确触发
   │
   └─ 并行执行 (Parallel Execution)
      ├─ Rust: tokio::join! / tokio::try_join!
      ├─ Golang: workflow.Go() + selector
      └─ 特性:
         ├─ 并发控制
         ├─ 结果聚合
         └─ 错误处理

═══════════════════════════════════════════════════════════════════════════
```

---

## 📊 Rust vs Golang API 详细对比矩阵

### 1. 工作流定义对比

| 功能 | Rust 1.90 | Golang | 对比说明 |
|------|----------|--------|---------|
| **工作流函数签名** | `#[workflow]` `async fn my_workflow(` `ctx: WorkflowContext,` `input: MyInput` `) -> Result<MyOutput>` | `func MyWorkflow(` `ctx workflow.Context,` `input MyInput` `) (MyOutput, error)` | Rust 使用宏和 async/await Golang 使用普通函数 |
| **类型安全** | 编译时检查 泛型支持 生命周期保证 | 运行时检查 interface{} 无生命周期 | Rust 更强的类型安全 |
| **错误处理** | `Result<T, WorkflowError>` | 返回 `error` | Rust 使用 Result 枚举 |
| **异步模型** | `async/await` (Future-based) | `workflow.Context` (特殊运行时) | Rust 使用标准异步 Golang 使用定制运行时 |

**代码对比示例**:

```rust
// Rust 1.90
#[workflow]
pub async fn order_workflow(
    ctx: WorkflowContext,
    input: OrderInput,
) -> Result<OrderOutput, WorkflowError> {
    // 执行 Activity
    let result = ctx
        .execute_activity::<ProcessPaymentActivity>(
            ActivityInput::new(input.clone()),
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(30),
                ..Default::default()
            },
        )
        .await?;
    
    Ok(OrderOutput {
        order_id: input.order_id,
        status: OrderStatus::Completed,
    })
}
```

```go
// Golang
func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    // 执行 Activity
    var result PaymentResult
    err := workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input).Get(ctx, &result)
    if err != nil {
        return OrderOutput{}, err
    }
    
    return OrderOutput{
        OrderID: input.OrderID,
        Status: OrderStatusCompleted,
    }, nil
}
```

**关键差异**:

- ✅ Rust: 类型安全更强，`ActivityInput::new()` 类型明确
- ✅ Golang: 代码更简洁，运行时更灵活
- ⚠️ Rust: 需要显式处理 `Result`
- ⚠️ Golang: 需要手动处理 `error`

### 2. Activity 定义对比

| 功能 | Rust 1.90 | Golang | 对比说明 |
|------|----------|--------|---------|
| **Activity 签名** | `#[activity]` `async fn my_activity(` `ctx: ActivityContext,` `input: MyInput` `) -> Result<MyOutput>` | `func MyActivity(` `ctx context.Context,` `input MyInput` `) (MyOutput, error)` | Rust 使用宏 Golang 使用标准 context |
| **心跳** | `ctx.heartbeat().await?` | `activity.RecordHeartbeat(ctx, details)` | Rust 使用 async Golang 使用同步调用 |
| **取消检查** | `ctx.is_cancelled()` | `ctx.Done()` channel | Rust 返回 bool Golang 使用 channel |
| **重试控制** | 在 `ActivityOptions` 中配置 | 在 `ActivityOptions` 中配置 | 基本一致 |

**代码对比示例**:

```rust
// Rust 1.90
#[activity]
pub async fn process_payment(
    ctx: ActivityContext,
    input: PaymentInput,
) -> Result<PaymentResult, ActivityError> {
    // 发送心跳
    ctx.heartbeat().await?;
    
    // 检查取消
    if ctx.is_cancelled() {
        return Err(ActivityError::Cancelled);
    }
    
    // 执行支付逻辑
    let result = payment_service::process(&input).await?;
    
    Ok(result)
}
```

```go
// Golang
func ProcessPayment(ctx context.Context, input PaymentInput) (PaymentResult, error) {
    // 发送心跳
    activity.RecordHeartbeat(ctx, "processing")
    
    // 检查取消
    select {
    case <-ctx.Done():
        return PaymentResult{}, ctx.Err()
    default:
    }
    
    // 执行支付逻辑
    result, err := paymentService.Process(&input)
    if err != nil {
        return PaymentResult{}, err
    }
    
    return result, nil
}
```

### 3. Signal 和 Query 对比

| 功能 | Rust 1.90 | Golang | 对比说明 |
|------|----------|--------|---------|
| **Signal 定义** | `Signal` trait `fn name() -> &'static str` | 字符串名称 | Rust 使用 trait Golang 使用字符串 |
| **Signal 发送** | `ctx.signal_workflow(` `workflow_id,` `signal` `)` | `client.SignalWorkflow(` `ctx, workflowID, runID,` `signalName, arg` `)` | Rust 类型安全 Golang 运行时检查 |
| **Signal 接收** | `ctx.await_signal::<MySignal>()` `.await?` | `signalChan := workflow.` `GetSignalChannel(` `ctx, "signal_name"` `)` `signalChan.Receive(ctx, &val)` | Rust 更简洁 Golang 更显式 |
| **Query 定义** | `Query` trait `type Result = MyResult` | `SetQueryHandler()` | Rust 类型关联 Golang 函数注册 |
| **Query 注册** | `ctx.set_query_handler::<Q, _, _>` `(handler)` | `workflow.SetQueryHandler(` `ctx, "query", handler` `)` | Rust 使用泛型 Golang 使用字符串 |

**代码对比示例 - Signal**:

```rust
// Rust 1.90 - Signal 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
    pub approver: String,
}

impl Signal for ApprovalSignal {
    fn name() -> &'static str {
        "approval"
    }
}

// 在工作流中等待 Signal
#[workflow]
async fn approval_workflow(ctx: WorkflowContext) -> Result<ApprovalResult> {
    let signal = ctx.await_signal::<ApprovalSignal>("approval").await?;
    
    if signal.approved {
        Ok(ApprovalResult::Approved)
    } else {
        Ok(ApprovalResult::Rejected)
    }
}

// 发送 Signal
client.signal_workflow(
    &workflow_id,
    ApprovalSignal {
        approved: true,
        approver: "admin".to_string(),
    },
).await?;
```

```go
// Golang - Signal 使用
type ApprovalSignal struct {
    Approved bool
    Approver string
}

// 在工作流中等待 Signal
func ApprovalWorkflow(ctx workflow.Context) (ApprovalResult, error) {
    var signal ApprovalSignal
    signalChan := workflow.GetSignalChannel(ctx, "approval")
    signalChan.Receive(ctx, &signal)
    
    if signal.Approved {
        return ApprovalResult{Status: "Approved"}, nil
    } else {
        return ApprovalResult{Status: "Rejected"}, nil
    }
}

// 发送 Signal
err := client.SignalWorkflow(
    ctx,
    workflowID,
    runID,
    "approval",
    ApprovalSignal{
        Approved: true,
        Approver: "admin",
    },
)
```

**代码对比示例 - Query**:

```rust
// Rust 1.90 - Query 定义
pub struct StatusQuery;

impl Query for StatusQuery {
    fn name() -> &'static str {
        "status"
    }
    type Result = WorkflowStatus;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub current_step: String,
    pub progress: f64,
}

// 在工作流中注册 Query 处理器
#[workflow]
async fn monitored_workflow(ctx: WorkflowContext) -> Result<Output> {
    let status = Arc::new(RwLock::new(WorkflowStatus {
        current_step: "initializing".to_string(),
        progress: 0.0,
    }));
    
    let status_clone = status.clone();
    ctx.set_query_handler::<StatusQuery, _, _>(move || {
        let status = status_clone.clone();
        async move {
            Ok(status.read().unwrap().clone())
        }
    });
    
    // 工作流逻辑...
    *status.write().unwrap() = WorkflowStatus {
        current_step: "processing".to_string(),
        progress: 0.5,
    };
    
    Ok(Output {})
}

// 执行 Query
let status = client
    .query_workflow::<StatusQuery>(&workflow_id)
    .await?;
```

```go
// Golang - Query 使用
type WorkflowStatus struct {
    CurrentStep string
    Progress    float64
}

func MonitoredWorkflow(ctx workflow.Context) (Output, error) {
    status := WorkflowStatus{
        CurrentStep: "initializing",
        Progress:    0.0,
    }
    
    // 注册 Query 处理器
    err := workflow.SetQueryHandler(ctx, "status", func() (WorkflowStatus, error) {
        return status, nil
    })
    if err != nil {
        return Output{}, err
    }
    
    // 工作流逻辑...
    status = WorkflowStatus{
        CurrentStep: "processing",
        Progress:    0.5,
    }
    
    return Output{}, nil
}

// 执行 Query
value, err := client.QueryWorkflow(ctx, workflowID, runID, "status")
if err != nil {
    return err
}
var status WorkflowStatus
err = value.Get(&status)
```

### 4. Worker 对比

| 功能 | Rust 1.90 | Golang | 对比说明 |
|------|----------|--------|---------|
| **Worker 创建** | `WorkflowWorker::new(` `task_queue,` `storage,` `config` `)` | `worker.New(` `client,` `taskQueue,` `options` `)` | Rust 需要显式存储 Golang 由 Server 管理 |
| **注册工作流** | `worker.register_workflow(` `"WorkflowName",` `workflow_fn` `)` | `w.RegisterWorkflow(` `WorkflowFunc` `)` | Rust 需要名称 Golang 自动推断 |
| **注册 Activity** | `worker.register_activity(` `"ActivityName",` `activity_fn` `)` | `w.RegisterActivity(` `ActivityFunc` `)` | 同上 |
| **启动 Worker** | `worker.start().await?` | `err := worker.Run(` `worker.InterruptCh()` `)` | Rust 使用 async Golang 阻塞运行 |

**代码对比示例**:

```rust
// Rust 1.90
#[tokio::main]
async fn main() -> Result<()> {
    // 创建存储
    let storage = Arc::new(
        PostgresWorkflowStorage::new("postgres://...").await?
    );
    
    // 创建 Worker
    let mut worker = WorkflowWorker::new(
        "my-task-queue".to_string(),
        storage,
        WorkerConfig {
            max_concurrent_workflow_executions: 100,
            max_concurrent_activity_executions: 1000,
            identity: "worker-1".to_string(),
        },
    );
    
    // 注册工作流和 Activities
    worker.register_workflow(
        "OrderWorkflow".to_string(),
        order_workflow,
    );
    worker.register_activity(
        "ProcessPayment".to_string(),
        process_payment,
    );
    
    // 启动 Worker
    let worker = Arc::new(worker);
    worker.start().await?;
    
    Ok(())
}
```

```go
// Golang
func main() {
    // 创建客户端
    c, err := client.NewClient(client.Options{})
    if err != nil {
        log.Fatalln("Unable to create client", err)
    }
    defer c.Close()
    
    // 创建 Worker
    w := worker.New(c, "my-task-queue", worker.Options{
        MaxConcurrentWorkflowTaskExecutionSize: 100,
        MaxConcurrentActivityExecutionSize:     1000,
        Identity:                               "worker-1",
    })
    
    // 注册工作流和 Activities
    w.RegisterWorkflow(OrderWorkflow)
    w.RegisterActivity(ProcessPayment)
    
    // 启动 Worker
    err = w.Run(worker.InterruptCh())
    if err != nil {
        log.Fatalln("Unable to start worker", err)
    }
}
```

### 5. Client 对比

| 功能 | Rust 1.90 | Golang | 对比说明 |
|------|----------|--------|---------|
| **客户端创建** | `WorkflowClient::new(config)` | `client.NewClient(options)` | 基本一致 |
| **启动工作流** | `client.start_workflow::<W, I, O>(` `workflow_id,` `task_queue,` `input,` `options` `)` | `workflowRun, err := client.` `ExecuteWorkflow(` `ctx, options, workflow, args` `)` | Rust 使用泛型 Golang 使用反射 |
| **获取结果** | `handle.get_result().await?` | `err = workflowRun.Get(ctx, &result)` | Rust 返回 Future Golang 阻塞等待 |
| **发送 Signal** | `handle.signal(signal).await?` | `client.SignalWorkflow(` `ctx, workflowID, runID,` `signalName, arg` `)` | Rust 通过 Handle Golang 直接调用 |

**代码对比示例**:

```rust
// Rust 1.90
#[tokio::main]
async fn main() -> Result<()> {
    let client = WorkflowClient::new(config)?;
    
    // 启动工作流
    let handle = client
        .start_workflow::<OrderWorkflow, _, _>(
            WorkflowId(format!("order-{}", Uuid::new_v4())),
            "my-task-queue".to_string(),
            OrderInput {
                order_id: "ORDER-123".to_string(),
                amount: 100.0,
            },
            StartWorkflowOptions::default(),
        )
        .await?;
    
    println!("Started workflow: {}", handle.workflow_id());
    
    // 发送 Signal
    handle.signal(ApprovalSignal {
        approved: true,
        approver: "admin".to_string(),
    }).await?;
    
    // 等待结果
    let result = handle.get_result().await?;
    println!("Result: {:?}", result);
    
    Ok(())
}
```

```go
// Golang
func main() {
    c, err := client.NewClient(client.Options{})
    if err != nil {
        log.Fatalln("Unable to create client", err)
    }
    defer c.Close()
    
    // 启动工作流
    workflowOptions := client.StartWorkflowOptions{
        ID:        "order-" + uuid.New().String(),
        TaskQueue: "my-task-queue",
    }
    
    workflowRun, err := c.ExecuteWorkflow(
        context.Background(),
        workflowOptions,
        OrderWorkflow,
        OrderInput{
            OrderID: "ORDER-123",
            Amount:  100.0,
        },
    )
    if err != nil {
        log.Fatalln("Unable to execute workflow", err)
    }
    
    fmt.Println("Started workflow:", workflowRun.GetID())
    
    // 发送 Signal
    err = c.SignalWorkflow(
        context.Background(),
        workflowRun.GetID(),
        workflowRun.GetRunID(),
        "approval",
        ApprovalSignal{
            Approved: true,
            Approver: "admin",
        },
    )
    if err != nil {
        log.Fatalln("Unable to signal workflow", err)
    }
    
    // 等待结果
    var result OrderOutput
    err = workflowRun.Get(context.Background(), &result)
    if err != nil {
        log.Fatalln("Unable to get workflow result", err)
    }
    
    fmt.Println("Result:", result)
}
```

---

## 🔗 概念关系图

```text
┌─────────────────────────────────────────────────────────────────┐
│                       概念关系与依赖                              │
└─────────────────────────────────────────────────────────────────┘

WorkflowClient ──uses──> WorkflowStorage
       │
       │ starts
       ▼
WorkflowExecution ──has──> WorkflowId + RunId
       │
       │ dispatches to
       ▼
WorkflowWorker ──polls──> TaskQueue
       │
       │ creates
       ▼
WorkflowContext ──provides──> WorkflowCapabilities
       │                           │
       ├───────────────────────────┼─────────────────────┐
       │                           │                     │
       │ executes                  │ awaits             │ registers
       ▼                           ▼                     ▼
ActivityContext ──has──> Signal ──notifies──> Query ──reads──>
       │                  WorkflowState         WorkflowState
       │
       │ uses
       ▼
ActivityOptions ──contains──> RetryPolicy
       │                           │
       │                           │ defines
       ▼                           ▼
ActivityExecution            RetryBehavior
       │
       │ produces
       ▼
WorkflowEvent ──appends to──> EventHistory
       │                           │
       │                           │ enables
       ▼                           ▼
WorkflowStorage ──persists──> EventSourcing ──enables──> Replay
```

---

## 📈 特性对比矩阵

### 语言特性对比

| 特性 | Rust 1.90 | Golang | 优势方 |
|------|----------|--------|--------|
| **类型安全** | 强静态类型 + 泛型 | 静态类型 + interface{} | Rust ✅ |
| **编译时检查** | 完整 | 基础 | Rust ✅ |
| **零成本抽象** | 是 | 否 | Rust ✅ |
| **内存安全** | 编译时保证 (所有权) | 运行时 (GC) | Rust ✅ |
| **并发模型** | async/await | goroutines | 各有优势 |
| **学习曲线** | 陡峭 | 平缓 | Golang ✅ |
| **开发速度** | 中等 | 快速 | Golang ✅ |
| **运行时开销** | 极低 | 低 (GC) | Rust ✅ |
| **生态成熟度** | 成长中 | 成熟 | Golang ✅ |

### 性能对比

| 指标 | Rust 1.90 | Golang | 倍数 |
|------|----------|--------|------|
| **工作流创建延迟** | ~1.2 µs | ~50-100 µs | 40-80x |
| **Activity 调用开销** | ~5-10 µs | ~100-200 µs | 10-40x |
| **内存占用** | 极低 (无 GC) | 低 (有 GC) | 更优 |
| **CPU 使用** | 优化更好 | 良好 | 更优 |
| **吞吐量** | 极高 | 高 | 更优 |

### 适用场景对比

| 场景 | Rust 1.90 | Golang | 推荐 |
|------|----------|--------|------|
| **微服务编排** | ✅ 优秀 | ✅ 优秀 | 两者皆可 |
| **高性能计算** | ✅ 最佳 | ⚠️ 良好 | Rust |
| **IoT/边缘计算** | ✅ 最佳 | ⚠️ 可行 | Rust |
| **快速原型开发** | ⚠️ 可行 | ✅ 最佳 | Golang |
| **企业级应用** | ✅ 优秀 | ✅ 优秀 | 两者皆可 |
| **系统级编程** | ✅ 最佳 | ⚠️ 可行 | Rust |
| **Web 服务** | ✅ 优秀 | ✅ 优秀 | 两者皆可 |
| **团队协作** | ⚠️ 需要培训 | ✅ 易上手 | Golang |

---

## 🎯 选择建议

### 何时选择 Rust 1.90 实现

✅ **推荐场景**:

- 需要**极致性能**和**低延迟**
- **嵌入式系统**或**IoT 设备**
- **安全关键**应用
- 需要**严格的类型安全**
- **长期运行**的服务（无 GC 停顿）
- 团队有 Rust 经验

### 何时选择 Golang 实现

✅ **推荐场景**:

- 需要**快速开发**和迭代
- 团队**缺乏 Rust 经验**
- 需要与现有 **Temporal 生态**紧密集成
- **原型开发**和 **POC**
- 性能要求**不是最严格**
- 团队协作和**代码可读性**优先

---

## 📚 下一步

- **继续学习**: [架构设计](./02_architecture.md)
- **查看示例**: [基础示例](./18_basic_examples.md)
- **技术栈详解**: [技术栈对比](./21_tech_stack_comparison.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
