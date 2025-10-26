# Rust vs Golang 并列对比示例

## 📋 文档概述

本文档提供Rust和Golang实现Temporal工作流的并列对比示例，帮助开发者理解两种语言的差异和相似之处。

---

## 🎯 示例1: 简单订单工作流

### Rust 实现

```rust
// workflow/src/temporal/examples/order_workflow.rs

use serde::{Deserialize, Serialize};
use crate::temporal::{
    Workflow, WorkflowContext, WorkflowError,
    Activity, ActivityContext, ActivityError, ActivityOptions,
};

// ============ 数据类型 ============

#[derive(Debug, Deserialize)]
pub struct OrderInput {
    pub order_id: String,
    pub customer_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct OrderOutput {
    pub order_id: String,
    pub status: OrderStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderStatus {
    Completed,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct PaymentResult {
    pub transaction_id: String,
    pub success: bool,
}

// ============ Activity定义 ============

pub struct ProcessPaymentActivity;

impl Activity for ProcessPaymentActivity {
    type Input = PaymentInput;
    type Output = PaymentResult;
    
    fn name() -> &'static str {
        "ProcessPayment"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 发送心跳
            ctx.heartbeat().await?;
            
            // 处理支付
            let transaction_id = format!("txn-{}", uuid::Uuid::new_v4());
            
            ctx.heartbeat().await?;
            
            Ok(PaymentResult {
                transaction_id,
                success: true,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaymentInput {
    pub order_id: String,
    pub amount: f64,
}

// ============ Workflow定义 ============

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
            // 执行支付Activity
            let payment_result = ctx
                .execute_activity::<ProcessPaymentActivity>(
                    PaymentInput {
                        order_id: input.order_id.clone(),
                        amount: input.amount,
                    },
                    ActivityOptions::default(),
                )
                .await?;
            
            let status = if payment_result.success {
                OrderStatus::Completed
            } else {
                OrderStatus::Failed
            };
            
            Ok(OrderOutput {
                order_id: input.order_id,
                status,
            })
        }
    }
}

// ============ 客户端使用 ============

pub async fn start_order_workflow(
    client: &WorkflowClient,
    order_id: String,
    amount: f64,
) -> Result<WorkflowHandle<OrderOutput>, ClientError> {
    let handle = client
        .start_workflow::<OrderWorkflow>(
            WorkflowId::new(&order_id),
            "order-queue".to_string(),
            OrderInput {
                order_id,
                customer_id: "customer-123".to_string(),
                amount,
            },
            StartWorkflowOptions::default(),
        )
        .await?;
    
    Ok(handle)
}
```

### Golang 实现

```go
// workflows/order_workflow.go

package workflows

import (
    "fmt"
    "time"
    
    "go.temporal.io/sdk/workflow"
    "github.com/google/uuid"
)

// ============ 数据类型 ============

type OrderInput struct {
    OrderID    string
    CustomerID string
    Amount     float64
}

type OrderOutput struct {
    OrderID string
    Status  OrderStatus
}

type OrderStatus string

const (
    OrderStatusCompleted OrderStatus = "Completed"
    OrderStatusFailed    OrderStatus = "Failed"
)

type PaymentResult struct {
    TransactionID string
    Success       bool
}

type PaymentInput struct {
    OrderID string
    Amount  float64
}

// ============ Activity定义 ============

func ProcessPaymentActivity(ctx context.Context, input PaymentInput) (PaymentResult, error) {
    // 发送心跳
    activity.RecordHeartbeat(ctx, "processing")
    
    // 处理支付
    transactionID := fmt.Sprintf("txn-%s", uuid.New().String())
    
    activity.RecordHeartbeat(ctx, "completed")
    
    return PaymentResult{
        TransactionID: transactionID,
        Success:       true,
    }, nil
}

// ============ Workflow定义 ============

func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    // 配置Activity选项
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 30 * time.Second,
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)
    
    // 执行支付Activity
    var paymentResult PaymentResult
    err := workflow.ExecuteActivity(
        ctx,
        ProcessPaymentActivity,
        PaymentInput{
            OrderID: input.OrderID,
            Amount:  input.Amount,
        },
    ).Get(ctx, &paymentResult)
    
    if err != nil {
        return OrderOutput{}, err
    }
    
    status := OrderStatusFailed
    if paymentResult.Success {
        status = OrderStatusCompleted
    }
    
    return OrderOutput{
        OrderID: input.OrderID,
        Status:  status,
    }, nil
}

// ============ 客户端使用 ============

func StartOrderWorkflow(c client.Client, orderID string, amount float64) (client.WorkflowRun, error) {
    workflowOptions := client.StartWorkflowOptions{
        ID:        orderID,
        TaskQueue: "order-queue",
    }
    
    run, err := c.ExecuteWorkflow(
        context.Background(),
        workflowOptions,
        OrderWorkflow,
        OrderInput{
            OrderID:    orderID,
            CustomerID: "customer-123",
            Amount:     amount,
        },
    )
    
    return run, err
}
```

### 对比说明

| 特性 | Rust | Golang |
|------|------|--------|
| **类型定义** | 使用struct + derive宏 | 使用struct + tags |
| **Workflow定义** | Trait实现 | 普通函数 |
| **Activity执行** | 泛型方法 | 反射 + Get() |
| **错误处理** | Result<T, E> + ? | (T, error) |
| **异步** | async/await | workflow.Context |

---

## 🎯 示例2: 带重试的长时间运行工作流

### Rust 实现

```rust
use std::time::Duration;

pub struct DataProcessingWorkflow;

impl Workflow for DataProcessingWorkflow {
    type Input = ProcessingInput;
    type Output = ProcessingOutput;
    
    fn name() -> &'static str {
        "DataProcessingWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            let mut processed = 0;
            let mut failed = 0;
            
            for item in input.items {
                // 执行Activity with retry
                match ctx
                    .execute_activity::<ProcessItemActivity>(
                        item,
                        ActivityOptions {
                            start_to_close_timeout: Some(Duration::from_secs(60)),
                            retry_policy: Some(RetryPolicy {
                                max_attempts: 3,
                                initial_interval: Duration::from_secs(1),
                                max_interval: Duration::from_secs(10),
                                backoff_coefficient: 2.0,
                                non_retryable_error_types: vec![
                                    "ValidationError".to_string(),
                                ],
                            }),
                            ..Default::default()
                        },
                    )
                    .await
                {
                    Ok(_) => processed += 1,
                    Err(e) => {
                        eprintln!("Failed to process item: {}", e);
                        failed += 1;
                    }
                }
                
                // 每处理10个项目休息一会儿
                if processed % 10 == 0 {
                    ctx.sleep(Duration::from_secs(1)).await;
                }
            }
            
            Ok(ProcessingOutput {
                total: input.items.len(),
                processed,
                failed,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessingInput {
    pub items: Vec<DataItem>,
}

#[derive(Debug, Serialize)]
pub struct ProcessingOutput {
    pub total: usize,
    pub processed: usize,
    pub failed: usize,
}

#[derive(Debug, Deserialize)]
pub struct DataItem {
    pub id: String,
    pub data: String,
}
```

### Golang 实现

```go
func DataProcessingWorkflow(ctx workflow.Context, input ProcessingInput) (ProcessingOutput, error) {
    logger := workflow.GetLogger(ctx)
    
    processed := 0
    failed := 0
    
    // 配置Activity选项
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 60 * time.Second,
        RetryPolicy: &temporal.RetryPolicy{
            MaximumAttempts:    3,
            InitialInterval:    time.Second,
            MaximumInterval:    10 * time.Second,
            BackoffCoefficient: 2.0,
            NonRetryableErrorTypes: []string{
                "ValidationError",
            },
        },
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)
    
    for _, item := range input.Items {
        var result ProcessItemResult
        err := workflow.ExecuteActivity(ctx, ProcessItemActivity, item).Get(ctx, &result)
        
        if err != nil {
            logger.Error("Failed to process item", "error", err)
            failed++
        } else {
            processed++
        }
        
        // 每处理10个项目休息一会儿
        if processed%10 == 0 {
            workflow.Sleep(ctx, time.Second)
        }
    }
    
    return ProcessingOutput{
        Total:     len(input.Items),
        Processed: processed,
        Failed:    failed,
    }, nil
}

type ProcessingInput struct {
    Items []DataItem
}

type ProcessingOutput struct {
    Total     int
    Processed int
    Failed    int
}

type DataItem struct {
    ID   string
    Data string
}
```

---

## 🎯 示例3: Signal与Query

### Rust 实现

```rust
use tokio::sync::mpsc;
use tokio::select;
use std::sync::Arc;
use parking_lot::RwLock;

// ============ Signal定义 ============

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

// ============ Query定义 ============

pub struct StatusQuery;

impl Query for StatusQuery {
    fn name() -> &'static str {
        "status"
    }
    
    type Result = WorkflowStatus;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowStatus {
    pub current_step: String,
    pub is_approved: bool,
}

// ============ Workflow定义 ============

pub struct ApprovalWorkflow;

impl Workflow for ApprovalWorkflow {
    type Input = ApprovalInput;
    type Output = ApprovalOutput;
    
    fn name() -> &'static str {
        "ApprovalWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 创建共享状态
            let status = Arc::new(RwLock::new(WorkflowStatus {
                current_step: "waiting_approval".to_string(),
                is_approved: false,
            }));
            
            // 注册Query处理器
            let status_clone = status.clone();
            ctx.set_query_handler::<StatusQuery, _, _>(move || {
                let status = status_clone.clone();
                async move {
                    Ok(status.read().clone())
                }
            });
            
            // 创建Signal通道
            let (signal_tx, mut signal_rx) = mpsc::channel::<ApprovalSignal>(10);
            ctx.register_signal_handler::<ApprovalSignal>(signal_tx);
            
            // 等待审批
            let approval = select! {
                signal = signal_rx.recv() => {
                    signal.ok_or(WorkflowError::SignalChannelClosed)?
                }
                _ = ctx.sleep(Duration::from_secs(3600)) => {
                    return Ok(ApprovalOutput {
                        approved: false,
                        reason: "Timeout".to_string(),
                    });
                }
            };
            
            // 更新状态
            {
                let mut s = status.write();
                s.current_step = "approved".to_string();
                s.is_approved = approval.approved;
            }
            
            Ok(ApprovalOutput {
                approved: approval.approved,
                reason: format!("Approved by {}", approval.approver),
            })
        }
    }
}

// ============ 客户端使用 ============

// 发送Signal
pub async fn send_approval(
    client: &WorkflowClient,
    workflow_id: &WorkflowId,
    approved: bool,
) -> Result<(), ClientError> {
    client
        .signal_workflow::<ApprovalSignal>(
            workflow_id,
            ApprovalSignal {
                approved,
                approver: "John Doe".to_string(),
            },
        )
        .await
}

// 执行Query
pub async fn query_status(
    client: &WorkflowClient,
    workflow_id: &WorkflowId,
) -> Result<WorkflowStatus, ClientError> {
    client
        .query_workflow::<StatusQuery>(workflow_id)
        .await
}
```

### Golang 实现

```go
// ============ Signal和Query数据类型 ============

type ApprovalSignal struct {
    Approved bool
    Approver string
}

type WorkflowStatus struct {
    CurrentStep string
    IsApproved  bool
}

// ============ Workflow定义 ============

func ApprovalWorkflow(ctx workflow.Context, input ApprovalInput) (ApprovalOutput, error) {
    logger := workflow.GetLogger(ctx)
    
    // 创建状态变量
    status := WorkflowStatus{
        CurrentStep: "waiting_approval",
        IsApproved:  false,
    }
    
    // 注册Query处理器
    err := workflow.SetQueryHandler(ctx, "status", func() (WorkflowStatus, error) {
        return status, nil
    })
    if err != nil {
        return ApprovalOutput{}, err
    }
    
    // 获取Signal channel
    approvalChan := workflow.GetSignalChannel(ctx, "approval")
    
    // 等待审批
    selector := workflow.NewSelector(ctx)
    var approval ApprovalSignal
    var timedOut bool
    
    selector.AddReceive(approvalChan, func(c workflow.ReceiveChannel, more bool) {
        c.Receive(ctx, &approval)
    })
    
    selector.AddFuture(workflow.NewTimer(ctx, time.Hour), func(f workflow.Future) {
        timedOut = true
    })
    
    selector.Select(ctx)
    
    if timedOut {
        return ApprovalOutput{
            Approved: false,
            Reason:   "Timeout",
        }, nil
    }
    
    // 更新状态
    status.CurrentStep = "approved"
    status.IsApproved = approval.Approved
    
    return ApprovalOutput{
        Approved: approval.Approved,
        Reason:   fmt.Sprintf("Approved by %s", approval.Approver),
    }, nil
}

// ============ 客户端使用 ============

// 发送Signal
func SendApproval(c client.Client, workflowID string, approved bool) error {
    return c.SignalWorkflow(
        context.Background(),
        workflowID,
        "",
        "approval",
        ApprovalSignal{
            Approved: approved,
            Approver: "John Doe",
        },
    )
}

// 执行Query
func QueryStatus(c client.Client, workflowID string) (WorkflowStatus, error) {
    resp, err := c.QueryWorkflow(
        context.Background(),
        workflowID,
        "",
        "status",
    )
    if err != nil {
        return WorkflowStatus{}, err
    }
    
    var status WorkflowStatus
    err = resp.Get(&status)
    return status, err
}
```

---

## 📊 综合对比

### 代码量对比

| 示例 | Rust 行数 | Golang 行数 | 差异 |
|------|-----------|-------------|------|
| 简单订单工作流 | ~150 | ~130 | Rust稍多(类型系统) |
| 长时间运行工作流 | ~80 | ~75 | 基本相同 |
| Signal与Query | ~120 | ~110 | Rust稍多(Arc+RwLock) |

### 特性对比

| 特性 | Rust | Golang | 说明 |
|------|------|--------|------|
| **类型安全** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Rust编译时完全检查 |
| **性能** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Rust无GC开销 |
| **学习曲线** | ⭐⭐ | ⭐⭐⭐⭐⭐ | Golang更简单 |
| **生态成熟度** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Golang官方SDK |
| **开发速度** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Golang编译更快 |
| **错误处理** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Rust强制处理 |

---

## 🎯 选择建议

### 选择Rust的理由

1. **性能关键**: 系统对性能有极高要求
2. **类型安全**: 需要编译时保证正确性
3. **嵌入式**: 需要嵌入到其他系统
4. **无GC**: 不能容忍GC暂停
5. **团队熟悉Rust**: 团队已掌握Rust

### 选择Golang的理由

1. **快速开发**: 需要快速迭代和原型
2. **成熟生态**: 依赖Temporal官方支持
3. **团队经验**: 团队熟悉Go
4. **丰富文档**: 需要参考大量示例
5. **简单部署**: 单一二进制文件

---

## 📚 更多示例

- [基础示例](../18_basic_examples.md)
- [高级模式](../19_advanced_patterns.md)
- [性能优化](../performance_tuning.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

