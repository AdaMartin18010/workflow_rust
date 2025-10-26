# Signal 与 Query

## 📋 文档概述

本文档详细阐述Temporal的Signal和Query机制，包括：

- Signal和Query核心概念
- Rust 1.90实现
- Golang实现对比
- 使用模式
- 最佳实践

---

## 🎯 核心概念

### Signal vs Query

```text
┌─────────────────────────────────────────────────────────────┐
│                    Signal vs Query                           │
└─────────────────────────────────────────────────────────────┘

Signal (信号)
├─ 异步消息
├─ 可以改变工作流状态
├─ 持久化到事件历史
├─ 不返回响应给调用者
└─ 用于触发工作流行为

Query (查询)
├─ 同步请求
├─ 不能改变工作流状态
├─ 不持久化
├─ 返回响应给调用者
└─ 用于读取工作流当前状态
```

### 使用场景

**Signal适用于**:

- 人工审批
- 外部事件通知
- 动态配置更新
- 取消请求
- 暂停/恢复工作流

**Query适用于**:

- 查询工作流进度
- 读取当前状态
- 调试信息
- 实时监控

### 交互模型

```text
┌─────────────────────────────────────────────────────────────┐
│                  Signal/Query 交互模型                       │
└─────────────────────────────────────────────────────────────┘

客户端
    │
    ├─ Signal ──────────────┐
    │                       │
    │                       ▼
    │               ┌────────────────┐
    │               │  Temporal      │
    │               │  Service       │
    │               └────────────────┘
    │                       │
    │              添加到事件历史
    │                       │
    │                       ▼
    │               ┌────────────────┐
    │               │  Worker        │
    │               │                │
    │               │  ┌──────────┐  │
    │               │  │ Workflow │  │
    │               │  │          │  │
    │               │  │  处理    │  │
    │               │  │  Signal  │  │
    │               │  └──────────┘  │
    │               └────────────────┘
    │
    │
    ├─ Query ───────────────┐
    │                       │
    │                       ▼
    │               ┌────────────────┐
    │               │  Temporal      │
    │               │  Service       │
    │               └────────────────┘
    │                       │
    │               转发到Worker
    │                       │
    │                       ▼
    │               ┌────────────────┐
    │               │  Worker        │
    │               │                │
    │               │  ┌──────────┐  │
    │               │  │ Workflow │  │
    │               │  │          │  │
    │               │  │  处理    │  │
    │               │  │  Query   │  │
    │               │  └──────────┘  │
    │               └────────────────┘
    │                       │
    │                返回结果
    │                       │
    ◀───────────────────────┘
```

---

## 🦀 Rust实现

### Signal定义

#### Signal Trait

```rust
/// Signal trait - 定义Signal接口
pub trait Signal: Serialize + DeserializeOwned + Send + 'static {
    /// Signal名称
    fn name() -> &'static str;
}
```

#### 简单Signal示例

```rust
use serde::{Deserialize, Serialize};

// 审批Signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
    pub approver: String,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl Signal for ApprovalSignal {
    fn name() -> &'static str {
        "approval"
    }
}

// 取消Signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelSignal {
    pub reason: String,
}

impl Signal for CancelSignal {
    fn name() -> &'static str {
        "cancel"
    }
}

// 更新配置Signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigSignal {
    pub key: String,
    pub value: String,
}

impl Signal for UpdateConfigSignal {
    fn name() -> &'static str {
        "update_config"
    }
}
```

### 在Workflow中接收Signal

```rust
use tokio::sync::mpsc;
use tokio::select;

pub struct OrderWorkflow;

impl Workflow for OrderWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 处理订单
            let order_id = input.order_id.clone();
            
            // 创建Signal通道
            let (signal_tx, mut signal_rx) = mpsc::channel::<ApprovalSignal>(10);
            
            // 注册Signal处理器
            ctx.register_signal_handler::<ApprovalSignal>(signal_tx);
            
            // 步骤1: 处理支付
            let payment_result = ctx
                .execute_activity::<ProcessPaymentActivity>(
                    input.payment_info,
                    ActivityOptions::default(),
                )
                .await?;
            
            // 步骤2: 等待审批Signal
            ctx.logger().info("Waiting for approval", o!("order_id" => &order_id));
            
            let approval = select! {
                signal = signal_rx.recv() => {
                    signal.ok_or(WorkflowError::SignalChannelClosed)?
                }
                _ = ctx.sleep(Duration::from_secs(3600)) => {
                    // 1小时后超时
                    return Ok(OrderOutput {
                        order_id,
                        status: OrderStatus::Timeout,
                        approved: false,
                    });
                }
            };
            
            if !approval.approved {
                ctx.logger().info("Order rejected", o!(
                    "order_id" => &order_id,
                    "approver" => &approval.approver
                ));
                
                return Ok(OrderOutput {
                    order_id,
                    status: OrderStatus::Rejected,
                    approved: false,
                });
            }
            
            ctx.logger().info("Order approved", o!(
                "order_id" => &order_id,
                "approver" => &approval.approver
            ));
            
            // 步骤3: 发货
            let shipment_result = ctx
                .execute_activity::<ShipOrderActivity>(
                    input.shipping_info,
                    ActivityOptions::default(),
                )
                .await?;
            
            Ok(OrderOutput {
                order_id,
                status: OrderStatus::Completed,
                approved: true,
            })
        }
    }
}
```

### 发送Signal (客户端)

```rust
use crate::client::WorkflowClient;

pub async fn send_approval(
    client: &WorkflowClient,
    workflow_id: &WorkflowId,
) -> Result<(), ClientError> {
    // 创建Signal
    let signal = ApprovalSignal {
        approved: true,
        approver: "John Doe".to_string(),
        comment: Some("Looks good!".to_string()),
        timestamp: Utc::now(),
    };
    
    // 发送Signal
    client
        .signal_workflow::<ApprovalSignal>(workflow_id, signal)
        .await?;
    
    Ok(())
}
```

### Query定义

#### Query Trait

```rust
/// Query trait - 定义Query接口
pub trait Query: Send + 'static {
    /// Query名称
    fn name() -> &'static str;
    
    /// 结果类型
    type Result: Serialize + DeserializeOwned + Send;
}
```

#### Query示例

```rust
// 状态查询
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
    pub started_at: DateTime<Utc>,
    pub items_processed: usize,
}

// 进度查询
pub struct ProgressQuery;

impl Query for ProgressQuery {
    fn name() -> &'static str {
        "progress"
    }
    
    type Result = ProgressInfo;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub percentage: f64,
    pub current_item: usize,
    pub total_items: usize,
    pub estimated_completion: Option<DateTime<Utc>>,
}
```

### 在Workflow中处理Query

```rust
use std::sync::Arc;
use parking_lot::RwLock;

pub struct OrderWorkflow;

impl Workflow for OrderWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 创建共享状态
            let status = Arc::new(RwLock::new(WorkflowStatus {
                current_step: "initialized".to_string(),
                progress: 0.0,
                started_at: Utc::now(),
                items_processed: 0,
            }));
            
            // 注册Query处理器
            let status_clone = status.clone();
            ctx.set_query_handler::<StatusQuery, _, _>(move || {
                let status = status_clone.clone();
                async move {
                    let status = status.read().clone();
                    Ok(status)
                }
            });
            
            // 步骤1: 处理支付
            {
                let mut s = status.write();
                s.current_step = "processing_payment".to_string();
                s.progress = 0.25;
            }
            
            let payment_result = ctx
                .execute_activity::<ProcessPaymentActivity>(
                    input.payment_info,
                    ActivityOptions::default(),
                )
                .await?;
            
            // 步骤2: 预留库存
            {
                let mut s = status.write();
                s.current_step = "reserving_inventory".to_string();
                s.progress = 0.5;
            }
            
            let reservation_result = ctx
                .execute_activity::<ReserveInventoryActivity>(
                    input.items.clone(),
                    ActivityOptions::default(),
                )
                .await?;
            
            // 步骤3: 发货
            {
                let mut s = status.write();
                s.current_step = "shipping".to_string();
                s.progress = 0.75;
            }
            
            let shipment_result = ctx
                .execute_activity::<ShipOrderActivity>(
                    input.shipping_info,
                    ActivityOptions::default(),
                )
                .await?;
            
            // 完成
            {
                let mut s = status.write();
                s.current_step = "completed".to_string();
                s.progress = 1.0;
            }
            
            Ok(OrderOutput {
                order_id: input.order_id,
                status: OrderStatus::Completed,
            })
        }
    }
}
```

### 执行Query (客户端)

```rust
pub async fn check_workflow_status(
    client: &WorkflowClient,
    workflow_id: &WorkflowId,
) -> Result<WorkflowStatus, ClientError> {
    // 执行Query
    let status = client
        .query_workflow::<StatusQuery>(workflow_id)
        .await?;
    
    println!("Workflow Status:");
    println!("  Current Step: {}", status.current_step);
    println!("  Progress: {:.1}%", status.progress * 100.0);
    println!("  Started At: {}", status.started_at);
    
    Ok(status)
}
```

---

## 🐹 Golang实现对比

### Signal定义 - Golang

```go
package workflows

import (
    "time"
    
    "go.temporal.io/sdk/workflow"
)

// Signal数据结构
type ApprovalSignal struct {
    Approved  bool
    Approver  string
    Comment   string
    Timestamp time.Time
}

type CancelSignal struct {
    Reason string
}

// 在Workflow中接收Signal
func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    logger := workflow.GetLogger(ctx)
    orderID := input.OrderID
    
    // 创建Signal channel
    approvalChan := workflow.GetSignalChannel(ctx, "approval")
    cancelChan := workflow.GetSignalChannel(ctx, "cancel")
    
    // 步骤1: 处理支付
    var paymentResult PaymentResult
    err := workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input.PaymentInfo).Get(ctx, &paymentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 步骤2: 等待审批Signal
    logger.Info("Waiting for approval", "orderID", orderID)
    
    selector := workflow.NewSelector(ctx)
    var approval ApprovalSignal
    var cancelled bool
    
    // 等待审批Signal
    selector.AddReceive(approvalChan, func(c workflow.ReceiveChannel, more bool) {
        c.Receive(ctx, &approval)
    })
    
    // 等待取消Signal
    selector.AddReceive(cancelChan, func(c workflow.ReceiveChannel, more bool) {
        var cancel CancelSignal
        c.Receive(ctx, &cancel)
        cancelled = true
    })
    
    // 等待1小时超时
    selector.AddFuture(workflow.NewTimer(ctx, time.Hour), func(f workflow.Future) {
        logger.Info("Approval timeout", "orderID", orderID)
    })
    
    selector.Select(ctx)
    
    if cancelled {
        return OrderOutput{
            OrderID: orderID,
            Status:  "Cancelled",
        }, nil
    }
    
    if !approval.Approved {
        logger.Info("Order rejected", "orderID", orderID, "approver", approval.Approver)
        return OrderOutput{
            OrderID:  orderID,
            Status:   "Rejected",
            Approved: false,
        }, nil
    }
    
    logger.Info("Order approved", "orderID", orderID, "approver", approval.Approver)
    
    // 步骤3: 发货
    var shipmentResult ShipmentResult
    err = workflow.ExecuteActivity(ctx, ShipOrderActivity, input.ShippingInfo).Get(ctx, &shipmentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    return OrderOutput{
        OrderID:  orderID,
        Status:   "Completed",
        Approved: true,
    }, nil
}
```

### 发送Signal - Golang

```go
package main

import (
    "context"
    "time"
    
    "go.temporal.io/sdk/client"
)

func sendApproval(c client.Client, workflowID string) error {
    // 创建Signal数据
    signal := ApprovalSignal{
        Approved:  true,
        Approver:  "John Doe",
        Comment:   "Looks good!",
        Timestamp: time.Now(),
    }
    
    // 发送Signal
    err := c.SignalWorkflow(
        context.Background(),
        workflowID,
        "",  // runID (empty = latest run)
        "approval",  // signal name
        signal,
    )
    
    return err
}
```

### Query定义 - Golang

```go
// Query结果类型
type WorkflowStatus struct {
    CurrentStep     string
    Progress        float64
    StartedAt       time.Time
    ItemsProcessed  int
}

type ProgressInfo struct {
    Percentage           float64
    CurrentItem          int
    TotalItems           int
    EstimatedCompletion  *time.Time
}

// 在Workflow中处理Query
func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    // 创建状态变量
    status := WorkflowStatus{
        CurrentStep:    "initialized",
        Progress:       0.0,
        StartedAt:      workflow.Now(ctx),
        ItemsProcessed: 0,
    }
    
    // 注册Query处理器
    err := workflow.SetQueryHandler(ctx, "status", func() (WorkflowStatus, error) {
        return status, nil
    })
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 步骤1: 处理支付
    status.CurrentStep = "processing_payment"
    status.Progress = 0.25
    
    var paymentResult PaymentResult
    err = workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input.PaymentInfo).Get(ctx, &paymentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 步骤2: 预留库存
    status.CurrentStep = "reserving_inventory"
    status.Progress = 0.5
    
    var reservationResult ReservationResult
    err = workflow.ExecuteActivity(ctx, ReserveInventoryActivity, input.Items).Get(ctx, &reservationResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 步骤3: 发货
    status.CurrentStep = "shipping"
    status.Progress = 0.75
    
    var shipmentResult ShipmentResult
    err = workflow.ExecuteActivity(ctx, ShipOrderActivity, input.ShippingInfo).Get(ctx, &shipmentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 完成
    status.CurrentStep = "completed"
    status.Progress = 1.0
    
    return OrderOutput{
        OrderID: input.OrderID,
        Status:  "Completed",
    }, nil
}
```

### 执行Query - Golang

```go
func checkWorkflowStatus(c client.Client, workflowID string) (WorkflowStatus, error) {
    // 执行Query
    resp, err := c.QueryWorkflow(
        context.Background(),
        workflowID,
        "",  // runID
        "status",  // query name
    )
    if err != nil {
        return WorkflowStatus{}, err
    }
    
    var status WorkflowStatus
    err = resp.Get(&status)
    if err != nil {
        return WorkflowStatus{}, err
    }
    
    fmt.Printf("Workflow Status:\n")
    fmt.Printf("  Current Step: %s\n", status.CurrentStep)
    fmt.Printf("  Progress: %.1f%%\n", status.Progress*100)
    fmt.Printf("  Started At: %s\n", status.StartedAt)
    
    return status, nil
}
```

---

## 🔄 Rust vs Golang 对比

### 对比表

| 特性 | Rust | Golang |
|------|------|--------|
| **Signal定义** | Trait实现 | 结构体 |
| **Signal接收** | mpsc channel | workflow.GetSignalChannel() |
| **Query定义** | Trait实现 | 结构体 |
| **Query处理** | `ctx.set_query_handler()` | `workflow.SetQueryHandler()` |
| **类型安全** | 编译时完全检查 | 运行时部分检查 |
| **状态管理** | `Arc<RwLock<T>>` | 普通变量 |

### Signal接收对比

**Rust**: 使用tokio channel

```rust
// 创建channel
let (signal_tx, mut signal_rx) = mpsc::channel::<ApprovalSignal>(10);

// 注册处理器
ctx.register_signal_handler::<ApprovalSignal>(signal_tx);

// 接收Signal
let approval = signal_rx.recv().await;
```

**Golang**: 使用workflow.GetSignalChannel

```go
// 获取Signal channel
approvalChan := workflow.GetSignalChannel(ctx, "approval")

// 接收Signal
var approval ApprovalSignal
approvalChan.Receive(ctx, &approval)
```

### Query处理对比

**Rust**: 使用闭包

```rust
let status = Arc::new(RwLock::new(Status::default()));
let status_clone = status.clone();

ctx.set_query_handler::<StatusQuery, _, _>(move || {
    let status = status_clone.clone();
    async move {
        Ok(status.read().clone())
    }
});
```

**Golang**: 使用函数

```go
status := WorkflowStatus{}

err := workflow.SetQueryHandler(ctx, "status", func() (WorkflowStatus, error) {
    return status, nil
})
```

---

## 📚 高级模式

### 1. 多Signal处理

#### Rust实现

```rust
pub struct MultiSignalWorkflow;

impl Workflow for MultiSignalWorkflow {
    type Input = WorkflowInput;
    type Output = WorkflowOutput;
    
    fn name() -> &'static str {
        "MultiSignalWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 创建多个Signal通道
            let (approval_tx, mut approval_rx) = mpsc::channel::<ApprovalSignal>(10);
            let (cancel_tx, mut cancel_rx) = mpsc::channel::<CancelSignal>(10);
            let (update_tx, mut update_rx) = mpsc::channel::<UpdateSignal>(10);
            
            // 注册Signal处理器
            ctx.register_signal_handler::<ApprovalSignal>(approval_tx);
            ctx.register_signal_handler::<CancelSignal>(cancel_tx);
            ctx.register_signal_handler::<UpdateSignal>(update_tx);
            
            let mut running = true;
            let mut config = Config::default();
            
            while running {
                select! {
                    // 处理审批Signal
                    Some(approval) = approval_rx.recv() => {
                        if approval.approved {
                            ctx.logger().info("Approved", o!("approver" => approval.approver));
                            running = false;
                        }
                    }
                    
                    // 处理取消Signal
                    Some(cancel) = cancel_rx.recv() => {
                        ctx.logger().info("Cancelled", o!("reason" => cancel.reason));
                        return Ok(WorkflowOutput {
                            status: Status::Cancelled,
                        });
                    }
                    
                    // 处理更新Signal
                    Some(update) = update_rx.recv() => {
                        ctx.logger().info("Config updated", o!(
                            "key" => &update.key,
                            "value" => &update.value
                        ));
                        config.update(update.key, update.value);
                    }
                    
                    // 超时
                    _ = ctx.sleep(Duration::from_secs(3600)) => {
                        return Ok(WorkflowOutput {
                            status: Status::Timeout,
                        });
                    }
                }
            }
            
            Ok(WorkflowOutput {
                status: Status::Completed,
            })
        }
    }
}
```

#### Golang实现

```go
func MultiSignalWorkflow(ctx workflow.Context, input WorkflowInput) (WorkflowOutput, error) {
    logger := workflow.GetLogger(ctx)
    
    // 获取Signal channels
    approvalChan := workflow.GetSignalChannel(ctx, "approval")
    cancelChan := workflow.GetSignalChannel(ctx, "cancel")
    updateChan := workflow.GetSignalChannel(ctx, "update")
    
    running := true
    config := Config{}
    
    for running {
        selector := workflow.NewSelector(ctx)
        
        // 处理审批Signal
        selector.AddReceive(approvalChan, func(c workflow.ReceiveChannel, more bool) {
            var approval ApprovalSignal
            c.Receive(ctx, &approval)
            if approval.Approved {
                logger.Info("Approved", "approver", approval.Approver)
                running = false
            }
        })
        
        // 处理取消Signal
        selector.AddReceive(cancelChan, func(c workflow.ReceiveChannel, more bool) {
            var cancel CancelSignal
            c.Receive(ctx, &cancel)
            logger.Info("Cancelled", "reason", cancel.Reason)
            running = false
        })
        
        // 处理更新Signal
        selector.AddReceive(updateChan, func(c workflow.ReceiveChannel, more bool) {
            var update UpdateSignal
            c.Receive(ctx, &update)
            logger.Info("Config updated", "key", update.Key, "value", update.Value)
            config.Update(update.Key, update.Value)
        })
        
        // 超时
        selector.AddFuture(workflow.NewTimer(ctx, time.Hour), func(f workflow.Future) {
            logger.Info("Timeout")
            running = false
        })
        
        selector.Select(ctx)
    }
    
    return WorkflowOutput{
        Status: "Completed",
    }, nil
}
```

### 2. Signal with Start

#### 2.1 Rust实现

```rust
// 启动工作流并发送Signal
pub async fn start_workflow_with_signal(
    client: &WorkflowClient,
) -> Result<WorkflowHandle<OrderOutput>, ClientError> {
    let workflow_id = WorkflowId::generate();
    
    // 启动工作流
    let handle = client
        .start_workflow::<OrderWorkflow>(
            workflow_id.clone(),
            "order-queue".to_string(),
            OrderInput { /* ... */ },
            StartWorkflowOptions::default(),
        )
        .await?;
    
    // 立即发送Signal
    client
        .signal_workflow::<ApprovalSignal>(
            &workflow_id,
            ApprovalSignal {
                approved: true,
                approver: "System".to_string(),
                comment: None,
                timestamp: Utc::now(),
            },
        )
        .await?;
    
    Ok(handle)
}

// 或者使用专门的API
pub async fn signal_with_start(
    client: &WorkflowClient,
) -> Result<WorkflowHandle<OrderOutput>, ClientError> {
    client
        .signal_with_start::<OrderWorkflow, ApprovalSignal>(
            WorkflowId::generate(),
            "order-queue".to_string(),
            OrderInput { /* ... */ },
            ApprovalSignal {
                approved: true,
                approver: "System".to_string(),
                comment: None,
                timestamp: Utc::now(),
            },
            SignalWithStartOptions::default(),
        )
        .await
}
```

#### 2.2 Golang实现

```go
func signalWithStart(c client.Client) (client.WorkflowRun, error) {
    workflowID := uuid.New().String()
    
    workflowOptions := client.StartWorkflowOptions{
        ID:        workflowID,
        TaskQueue: "order-queue",
    }
    
    signalName := "approval"
    signalArg := ApprovalSignal{
        Approved:  true,
        Approver:  "System",
        Timestamp: time.Now(),
    }
    
    workflowInput := OrderInput{
        // ...
    }
    
    // SignalWithStart - 原子操作
    run, err := c.SignalWithStartWorkflow(
        context.Background(),
        workflowID,
        signalName,
        signalArg,
        workflowOptions,
        OrderWorkflow,
        workflowInput,
    )
    
    return run, err
}
```

### 3. 动态Query

#### 3.1 Rust实现

```rust
pub struct StatisticsWorkflow;

impl Workflow for StatisticsWorkflow {
    type Input = StatsInput;
    type Output = StatsOutput;
    
    fn name() -> &'static str {
        "StatisticsWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 共享统计数据
            let stats = Arc::new(RwLock::new(HashMap::<String, i64>::new()));
            
            // 注册总体统计Query
            let stats_clone = stats.clone();
            ctx.set_query_handler::<AllStatsQuery, _, _>(move || {
                let stats = stats_clone.clone();
                async move {
                    Ok(stats.read().clone())
                }
            });
            
            // 注册单项统计Query
            let stats_clone = stats.clone();
            ctx.set_query_handler::<SingleStatQuery, _, _>(move || {
                let stats = stats_clone.clone();
                async move {
                    // 从Query参数中获取key
                    let key = "specific_key";  // 实际应该从Query获取
                    let value = stats.read().get(key).copied();
                    Ok(value)
                }
            });
            
            // 处理数据并更新统计
            for item in input.items {
                process_item(item, &stats).await?;
            }
            
            Ok(StatsOutput {
                total_processed: stats.read().len(),
            })
        }
    }
}
```

---

## 🎯 最佳实践

### 1. Signal设计

```rust
// ✅ 好: 包含足够的上下文信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
    pub approver: String,
    pub approver_id: String,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
}

// ❌ 差: 信息不足
#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
}
```

### 2. Query设计

```rust
// ✅ 好: Query不修改状态
ctx.set_query_handler::<StatusQuery, _, _>(move || {
    let status = status_clone.clone();
    async move {
        Ok(status.read().clone())  // 只读
    }
});

// ❌ 差: Query修改状态
ctx.set_query_handler::<StatusQuery, _, _>(move || {
    let status = status_clone.clone();
    async move {
        status.write().counter += 1;  // 不应该修改状态!
        Ok(status.read().clone())
    }
});
```

### 3. 超时处理

```rust
// ✅ 好: 总是设置合理的超时
select! {
    signal = signal_rx.recv() => {
        // 处理Signal
    }
    _ = ctx.sleep(Duration::from_secs(3600)) => {
        // 超时处理
        return Ok(Output::timeout());
    }
}

// ❌ 差: 无限等待
let signal = signal_rx.recv().await;  // 可能永远等待
```

---

## 📚 总结

### Signal

- 异步消息机制
- 可以改变工作流状态
- 持久化到事件历史
- 用于触发工作流行为

### Query

- 同步查询机制
- 不能改变工作流状态
- 不持久化
- 用于读取当前状态

### Rust vs Golang

- **Rust**: 更强的类型安全，使用tokio channel和`Arc<RwLock<T>>`
- **Golang**: 更简单直观，使用workflow.GetSignalChannel和普通变量

---

## 📚 下一步

- **工作流生命周期**: [生命周期管理](./07_lifecycle.md)
- **错误处理**: [错误处理策略](./error_handling.md)
- **实战示例**: [完整案例](./18_basic_examples.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
