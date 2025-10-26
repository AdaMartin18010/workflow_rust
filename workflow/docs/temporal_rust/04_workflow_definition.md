# 工作流定义

## 📋 文档概述

本文档详细阐述基于Temporal的工作流定义，包括：

- Temporal工作流核心概念
- Rust 1.90实现
- Golang实现对比
- 最佳实践
- 完整示例

---

## 🎯 Temporal工作流概念

### 什么是Workflow？

在Temporal中，**Workflow**是一个**可靠的**、**持久化的**函数执行。它具有以下特性：

1. **持久性 (Durable)**: 工作流的状态自动持久化，进程崩溃后可恢复
2. **确定性 (Deterministic)**: 相同的输入产生相同的结果
3. **长期运行 (Long-running)**: 可以运行数天、数月甚至数年
4. **可观察 (Observable)**: 通过Query可以查询工作流状态
5. **可交互 (Interactive)**: 通过Signal可以与运行中的工作流交互

### 工作流执行模型

```text
┌─────────────────────────────────────────────────────────────┐
│                    Temporal 工作流执行模型                   │
└─────────────────────────────────────────────────────────────┘

客户端 (Client)
    │
    ├─ StartWorkflow() ────────────────┐
    │                                   │
    │                                   ▼
    │                          ┌────────────────┐
    │                          │  Temporal      │
    │                          │  Service       │
    │                          └────────────────┘
    │                                   │
    │                          分发任务 │
    │                                   ▼
    │                          ┌────────────────┐
    │                          │  Worker        │
    │                          │                │
    │                          │  ┌──────────┐  │
    │                          │  │ Workflow │  │
    │                          │  │ Function │  │
    │                          │  └──────────┘  │
    │                          │       │        │
    │                          │       ├─ Activity 1
    │                          │       ├─ Activity 2
    │                          │       ├─ Timer
    │                          │       └─ Child Workflow
    │                          └────────────────┘
    │                                   │
    │                          生成事件 │
    │                                   │
    │                                   ▼
    │                          ┌────────────────┐
    │                          │  Event History │
    │                          │                │
    │                          │  [Event 1]     │
    │                          │  [Event 2]     │
    │                          │  [Event 3]     │
    │                          │  [...]         │
    │                          └────────────────┘
    │
    ├─ Signal() ───────────────────────┘
    │
    └─ Query() ────────────────────────┘
```

---

## 🦀 Rust实现

### 基础工作流定义

#### Workflow Trait

```rust
/// Workflow trait - 定义工作流接口
pub trait Workflow: Send + Sync + 'static {
    /// 输入类型
    type Input: DeserializeOwned + Send + 'static;
    
    /// 输出类型
    type Output: Serialize + Send + 'static;
    
    /// 工作流名称
    fn name() -> &'static str;
    
    /// 执行工作流
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send;
}
```

#### 简单示例：问候工作流

```rust
use serde::{Deserialize, Serialize};
use crate::workflow::{Workflow, WorkflowContext, WorkflowError};

// 定义输入类型
#[derive(Debug, Deserialize)]
pub struct GreetingInput {
    pub name: String,
}

// 定义输出类型
#[derive(Debug, Serialize)]
pub struct GreetingOutput {
    pub message: String,
}

// 定义工作流
pub struct GreetingWorkflow;

impl Workflow for GreetingWorkflow {
    type Input = GreetingInput;
    type Output = GreetingOutput;
    
    fn name() -> &'static str {
        "GreetingWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 简单的工作流逻辑
            let message = format!("Hello, {}!", input.name);
            
            Ok(GreetingOutput { message })
        }
    }
}
```

### 带Activity的工作流

```rust
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::workflow::{Workflow, WorkflowContext, WorkflowError};
use crate::activity::{Activity, ActivityContext, ActivityError, ActivityOptions};

// ============ Activity 定义 ============

#[derive(Debug, Deserialize)]
pub struct EmailData {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct EmailResult {
    pub success: bool,
    pub message_id: String,
}

pub struct SendEmailActivity;

impl Activity for SendEmailActivity {
    type Input = EmailData;
    type Output = EmailResult;
    
    fn name() -> &'static str {
        "SendEmail"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 发送心跳
            ctx.heartbeat().await?;
            
            // 实际的邮件发送逻辑
            let result = email_service::send(
                &input.to,
                &input.subject,
                &input.body,
            ).await?;
            
            Ok(EmailResult {
                success: true,
                message_id: result.id,
            })
        }
    }
}

// ============ Workflow 定义 ============

#[derive(Debug, Deserialize)]
pub struct NotificationInput {
    pub user_email: String,
    pub user_name: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationOutput {
    pub sent: bool,
    pub message_id: String,
}

pub struct NotificationWorkflow;

impl Workflow for NotificationWorkflow {
    type Input = NotificationInput;
    type Output = NotificationOutput;
    
    fn name() -> &'static str {
        "NotificationWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 执行Activity
            let email_result = ctx
                .execute_activity::<SendEmailActivity>(
                    EmailData {
                        to: input.user_email,
                        subject: "Welcome!".to_string(),
                        body: format!("Hello {}, welcome to our service!", input.user_name),
                    },
                    ActivityOptions {
                        start_to_close_timeout: Some(Duration::from_secs(30)),
                        retry_policy: Some(RetryPolicy::default()),
                        ..Default::default()
                    },
                )
                .await?;
            
            Ok(NotificationOutput {
                sent: email_result.success,
                message_id: email_result.message_id,
            })
        }
    }
}
```

### 复杂工作流：订单处理

```rust
use std::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============ 数据类型 ============

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderInput {
    pub order_id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub payment_info: PaymentInfo,
    pub shipping_address: Address,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentInfo {
    pub method: String,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug, Serialize)]
pub struct OrderOutput {
    pub order_id: String,
    pub status: OrderStatus,
    pub tracking_number: Option<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderStatus {
    Created,
    PaymentProcessed,
    InventoryReserved,
    Shipped,
    Completed,
    Cancelled,
}

// ============ Activities ============

pub struct ProcessPaymentActivity;

impl Activity for ProcessPaymentActivity {
    type Input = PaymentInfo;
    type Output = PaymentResult;
    
    fn name() -> &'static str {
        "ProcessPayment"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            ctx.heartbeat().await?;
            
            // 调用支付网关
            let result = payment_gateway::charge(&input).await?;
            
            Ok(PaymentResult {
                transaction_id: result.id,
                success: result.success,
                amount: result.amount,
            })
        }
    }
}

pub struct ReserveInventoryActivity;

impl Activity for ReserveInventoryActivity {
    type Input = Vec<OrderItem>;
    type Output = ReservationResult;
    
    fn name() -> &'static str {
        "ReserveInventory"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            ctx.heartbeat().await?;
            
            let reservation_id = inventory_service::reserve(&input).await?;
            
            Ok(ReservationResult {
                reservation_id,
                reserved: true,
            })
        }
    }
}

pub struct ShipOrderActivity;

impl Activity for ShipOrderActivity {
    type Input = ShipmentRequest;
    type Output = ShipmentResult;
    
    fn name() -> &'static str {
        "ShipOrder"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            ctx.heartbeat().await?;
            
            let shipment = shipping_service::create_shipment(&input).await?;
            
            Ok(ShipmentResult {
                tracking_number: shipment.tracking_number,
                carrier: shipment.carrier,
            })
        }
    }
}

// ============ 工作流实现 ============

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
            let order_id = input.order_id.clone();
            
            // 步骤1: 处理支付
            let payment_result = ctx
                .execute_activity::<ProcessPaymentActivity>(
                    input.payment_info.clone(),
                    ActivityOptions {
                        start_to_close_timeout: Some(Duration::from_secs(30)),
                        retry_policy: Some(RetryPolicy {
                            max_attempts: 3,
                            initial_interval: Duration::from_secs(1),
                            max_interval: Duration::from_secs(10),
                            backoff_coefficient: 2.0,
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            
            if !payment_result.success {
                return Ok(OrderOutput {
                    order_id,
                    status: OrderStatus::Cancelled,
                    tracking_number: None,
                    completed_at: Utc::now(),
                });
            }
            
            // 步骤2: 预留库存
            let reservation_result = ctx
                .execute_activity::<ReserveInventoryActivity>(
                    input.items.clone(),
                    ActivityOptions::default(),
                )
                .await?;
            
            // 步骤3: 等待一段时间（模拟处理时间）
            ctx.sleep(Duration::from_secs(10)).await;
            
            // 步骤4: 发货
            let shipment_result = ctx
                .execute_activity::<ShipOrderActivity>(
                    ShipmentRequest {
                        order_id: order_id.clone(),
                        items: input.items,
                        address: input.shipping_address,
                    },
                    ActivityOptions::default(),
                )
                .await?;
            
            Ok(OrderOutput {
                order_id,
                status: OrderStatus::Completed,
                tracking_number: Some(shipment_result.tracking_number),
                completed_at: Utc::now(),
            })
        }
    }
}
```

---

## 🐹 Golang实现对比

### Temporal Go SDK - 基础工作流

```go
package workflows

import (
    "time"
    
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
func GreetingWorkflow(ctx workflow.Context, input GreetingInput) (GreetingOutput, error) {
    message := "Hello, " + input.Name + "!"
    
    return GreetingOutput{Message: message}, nil
}
```

### 带Activity的工作流 - Golang

```go
package workflows

import (
    "time"
    
    "go.temporal.io/sdk/workflow"
)

// ============ Activity 定义 ============

type EmailData struct {
    To      string
    Subject string
    Body    string
}

type EmailResult struct {
    Success   bool
    MessageID string
}

// Activity函数
func SendEmailActivity(ctx context.Context, data EmailData) (EmailResult, error) {
    // 发送心跳
    activity.RecordHeartbeat(ctx, "processing")
    
    // 实际邮件发送逻辑
    result, err := emailService.Send(data.To, data.Subject, data.Body)
    if err != nil {
        return EmailResult{}, err
    }
    
    return EmailResult{
        Success:   true,
        MessageID: result.ID,
    }, nil
}

// ============ Workflow 定义 ============

type NotificationInput struct {
    UserEmail string
    UserName  string
}

type NotificationOutput struct {
    Sent      bool
    MessageID string
}

func NotificationWorkflow(ctx workflow.Context, input NotificationInput) (NotificationOutput, error) {
    // 配置Activity选项
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 30 * time.Second,
        RetryPolicy: &temporal.RetryPolicy{
            MaximumAttempts: 3,
        },
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)
    
    // 执行Activity
    var emailResult EmailResult
    err := workflow.ExecuteActivity(
        ctx,
        SendEmailActivity,
        EmailData{
            To:      input.UserEmail,
            Subject: "Welcome!",
            Body:    "Hello " + input.UserName + ", welcome to our service!",
        },
    ).Get(ctx, &emailResult)
    
    if err != nil {
        return NotificationOutput{}, err
    }
    
    return NotificationOutput{
        Sent:      emailResult.Success,
        MessageID: emailResult.MessageID,
    }, nil
}
```

### 复杂工作流：订单处理 - Golang

```go
package workflows

import (
    "time"
    
    "go.temporal.io/sdk/workflow"
)

// ============ 数据类型 ============

type OrderInput struct {
    OrderID         string
    UserID          string
    Items           []OrderItem
    PaymentInfo     PaymentInfo
    ShippingAddress Address
}

type OrderItem struct {
    ProductID string
    Quantity  int
    Price     float64
}

type PaymentInfo struct {
    Method string
    Token  string
}

type Address struct {
    Street     string
    City       string
    PostalCode string
    Country    string
}

type OrderOutput struct {
    OrderID        string
    Status         string
    TrackingNumber string
    CompletedAt    time.Time
}

// ============ Activities ============

func ProcessPaymentActivity(ctx context.Context, info PaymentInfo) (PaymentResult, error) {
    activity.RecordHeartbeat(ctx, "processing payment")
    
    result, err := paymentGateway.Charge(info)
    if err != nil {
        return PaymentResult{}, err
    }
    
    return PaymentResult{
        TransactionID: result.ID,
        Success:       result.Success,
        Amount:        result.Amount,
    }, nil
}

func ReserveInventoryActivity(ctx context.Context, items []OrderItem) (ReservationResult, error) {
    activity.RecordHeartbeat(ctx, "reserving inventory")
    
    reservationID, err := inventoryService.Reserve(items)
    if err != nil {
        return ReservationResult{}, err
    }
    
    return ReservationResult{
        ReservationID: reservationID,
        Reserved:      true,
    }, nil
}

func ShipOrderActivity(ctx context.Context, req ShipmentRequest) (ShipmentResult, error) {
    activity.RecordHeartbeat(ctx, "creating shipment")
    
    shipment, err := shippingService.CreateShipment(req)
    if err != nil {
        return ShipmentResult{}, err
    }
    
    return ShipmentResult{
        TrackingNumber: shipment.TrackingNumber,
        Carrier:        shipment.Carrier,
    }, nil
}

// ============ 工作流实现 ============

func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    logger := workflow.GetLogger(ctx)
    orderID := input.OrderID
    
    // 配置Activity选项
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 30 * time.Second,
        RetryPolicy: &temporal.RetryPolicy{
            InitialInterval:    time.Second,
            BackoffCoefficient: 2.0,
            MaximumInterval:    10 * time.Second,
            MaximumAttempts:    3,
        },
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)
    
    // 步骤1: 处理支付
    var paymentResult PaymentResult
    err := workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input.PaymentInfo).Get(ctx, &paymentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    if !paymentResult.Success {
        return OrderOutput{
            OrderID:     orderID,
            Status:      "Cancelled",
            CompletedAt: time.Now(),
        }, nil
    }
    
    // 步骤2: 预留库存
    var reservationResult ReservationResult
    err = workflow.ExecuteActivity(ctx, ReserveInventoryActivity, input.Items).Get(ctx, &reservationResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 步骤3: 等待一段时间
    err = workflow.Sleep(ctx, 10*time.Second)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 步骤4: 发货
    var shipmentResult ShipmentResult
    err = workflow.ExecuteActivity(ctx, ShipOrderActivity, ShipmentRequest{
        OrderID: orderID,
        Items:   input.Items,
        Address: input.ShippingAddress,
    }).Get(ctx, &shipmentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    logger.Info("Order completed", "orderID", orderID, "trackingNumber", shipmentResult.TrackingNumber)
    
    return OrderOutput{
        OrderID:        orderID,
        Status:         "Completed",
        TrackingNumber: shipmentResult.TrackingNumber,
        CompletedAt:    time.Now(),
    }, nil
}
```

---

## 🔄 Rust vs Golang 对比分析

### 语法对比表

| 特性 | Rust | Golang |
|------|------|--------|
| **工作流定义** | Trait + impl | 普通函数 |
| **类型系统** | 强类型，关联类型 | 强类型，泛型参数 |
| **异步支持** | async/await (原生) | goroutine + channel |
| **错误处理** | Result<T, E> | (T, error) |
| **上下文传递** | 显式WorkflowContext | workflow.Context |
| **Activity执行** | `ctx.execute_activity::<T>()` | `workflow.ExecuteActivity()` |
| **类型安全** | 编译时完全检查 | 运行时部分检查 |
| **零成本抽象** | ✅ | ❌ |

### 详细对比

#### 1. 工作流定义方式

**Rust**: 使用Trait定义接口

```rust
pub trait Workflow: Send + Sync + 'static {
    type Input: DeserializeOwned + Send + 'static;
    type Output: Serialize + Send + 'static;
    
    fn name() -> &'static str;
    fn execute(ctx: WorkflowContext, input: Self::Input) 
        -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send;
}

impl Workflow for OrderWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    // ...
}
```

**优点**:

- ✅ 类型安全: 输入输出类型在编译时严格检查
- ✅ 可扩展: 可以为trait添加默认实现
- ✅ 组合性: 可以通过trait bounds组合行为

**Golang**: 普通函数

```go
func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    // ...
}
```

**优点**:

- ✅ 简单直观: 学习曲线平缓
- ✅ 灵活: 不受接口约束

#### 2. Activity执行

**Rust**: 类型参数化

```rust
let result = ctx
    .execute_activity::<ProcessPaymentActivity>(
        payment_info,
        options,
    )
    .await?;
// result 类型自动推断为 PaymentResult
```

**优点**:

- ✅ 类型推断: 返回类型自动推断
- ✅ 编译时检查: 输入类型必须匹配

**Golang**: 反射 + Get()

```go
var paymentResult PaymentResult
err := workflow.ExecuteActivity(ctx, ProcessPaymentActivity, paymentInfo).Get(ctx, &paymentResult)
```

**缺点**:

- ⚠️ 需要手动声明结果变量
- ⚠️ Get()在运行时解析类型

#### 3. 错误处理

**Rust**: Result类型

```rust
async move {
    let result = ctx.execute_activity::<T>(...).await?;
    // ? 运算符自动传播错误
    Ok(output)
}
```

**优点**:

- ✅ 强制错误处理
- ✅ 类型安全
- ✅ 组合性好 (?, and_then, map_err)

**Golang**: 错误返回值

```go
result, err := doSomething()
if err != nil {
    return Output{}, err
}
```

**缺点**:

- ⚠️ 容易忘记检查错误
- ⚠️ 冗长的if err != nil模式

#### 4. 并发模型

**Rust**: async/await

```rust
async move {
    // 并发执行多个Activity
    let (result1, result2) = tokio::join!(
        ctx.execute_activity::<Activity1>(...),
        ctx.execute_activity::<Activity2>(...),
    );
}
```

**Golang**: goroutine + Selector

```go
selector := workflow.NewSelector(ctx)

f1 := workflow.ExecuteActivity(ctx, Activity1, ...)
selector.AddFuture(f1, func(f workflow.Future) {
    // 处理结果
})

f2 := workflow.ExecuteActivity(ctx, Activity2, ...)
selector.AddFuture(f2, func(f workflow.Future) {
    // 处理结果
})

selector.Select(ctx)
```

#### 5. 性能对比

| 指标 | Rust | Golang |
|------|------|--------|
| **内存占用** | 更低（零成本抽象） | 中等（GC开销） |
| **CPU效率** | 更高（无GC暂停） | 高（偶尔GC暂停） |
| **编译时间** | 较慢 | 快 |
| **运行时性能** | 最优 | 优秀 |
| **启动时间** | 快（无VM） | 快 |

---

## 🏗️ 高级工作流模式

### 1. Saga模式 (补偿事务)

#### Rust实现

```rust
pub struct BookingSagaWorkflow;

impl Workflow for BookingSagaWorkflow {
    type Input = BookingInput;
    type Output = BookingOutput;
    
    fn name() -> &'static str {
        "BookingSaga"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            let mut compensations = Vec::new();
            
            // 步骤1: 预订酒店
            match ctx.execute_activity::<ReserveHotelActivity>(input.hotel, options).await {
                Ok(hotel_result) => {
                    compensations.push(Compensation::CancelHotel(hotel_result.reservation_id));
                }
                Err(e) => {
                    return Err(e);
                }
            }
            
            // 步骤2: 预订航班
            match ctx.execute_activity::<ReserveFlightActivity>(input.flight, options).await {
                Ok(flight_result) => {
                    compensations.push(Compensation::CancelFlight(flight_result.booking_id));
                }
                Err(e) => {
                    // 失败: 执行补偿
                    execute_compensations(&ctx, compensations).await?;
                    return Err(e);
                }
            }
            
            // 步骤3: 支付
            match ctx.execute_activity::<ProcessPaymentActivity>(input.payment, options).await {
                Ok(payment_result) => {
                    Ok(BookingOutput {
                        booking_id: generate_id(),
                        status: BookingStatus::Confirmed,
                    })
                }
                Err(e) => {
                    // 失败: 执行所有补偿
                    execute_compensations(&ctx, compensations).await?;
                    Err(e)
                }
            }
        }
    }
}

async fn execute_compensations(
    ctx: &WorkflowContext,
    compensations: Vec<Compensation>,
) -> Result<(), WorkflowError> {
    for comp in compensations.into_iter().rev() {
        match comp {
            Compensation::CancelHotel(id) => {
                ctx.execute_activity::<CancelHotelActivity>(id, options).await?;
            }
            Compensation::CancelFlight(id) => {
                ctx.execute_activity::<CancelFlightActivity>(id, options).await?;
            }
        }
    }
    Ok(())
}
```

#### Golang实现

```go
func BookingSagaWorkflow(ctx workflow.Context, input BookingInput) (BookingOutput, error) {
    var compensations []func() error
    
    // 步骤1: 预订酒店
    var hotelResult HotelReservation
    err := workflow.ExecuteActivity(ctx, ReserveHotelActivity, input.Hotel).Get(ctx, &hotelResult)
    if err != nil {
        return BookingOutput{}, err
    }
    compensations = append(compensations, func() error {
        return workflow.ExecuteActivity(ctx, CancelHotelActivity, hotelResult.ReservationID).Get(ctx, nil)
    })
    
    // 步骤2: 预订航班
    var flightResult FlightBooking
    err = workflow.ExecuteActivity(ctx, ReserveFlightActivity, input.Flight).Get(ctx, &flightResult)
    if err != nil {
        executeCompensations(ctx, compensations)
        return BookingOutput{}, err
    }
    compensations = append(compensations, func() error {
        return workflow.ExecuteActivity(ctx, CancelFlightActivity, flightResult.BookingID).Get(ctx, nil)
    })
    
    // 步骤3: 支付
    var paymentResult PaymentResult
    err = workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input.Payment).Get(ctx, &paymentResult)
    if err != nil {
        executeCompensations(ctx, compensations)
        return BookingOutput{}, err
    }
    
    return BookingOutput{
        BookingID: generateID(),
        Status:    "Confirmed",
    }, nil
}

func executeCompensations(ctx workflow.Context, compensations []func() error) {
    for i := len(compensations) - 1; i >= 0; i-- {
        compensations[i]()
    }
}
```

### 2. 子工作流模式

#### 2.1 Rust实现

```rust
pub struct ParentWorkflow;

impl Workflow for ParentWorkflow {
    type Input = ParentInput;
    type Output = ParentOutput;
    
    fn name() -> &'static str {
        "ParentWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 启动多个子工作流
            let mut handles = Vec::new();
            
            for item in input.items {
                let handle = ctx
                    .start_child_workflow::<ChildWorkflow>(
                        item,
                        ChildWorkflowOptions {
                            workflow_id: Some(WorkflowId::generate()),
                            ..Default::default()
                        },
                    )
                    .await?;
                
                handles.push(handle);
            }
            
            // 等待所有子工作流完成
            let mut results = Vec::new();
            for handle in handles {
                let result = handle.get_result().await?;
                results.push(result);
            }
            
            Ok(ParentOutput { results })
        }
    }
}
```

#### 2.2 Golang实现

```go
func ParentWorkflow(ctx workflow.Context, input ParentInput) (ParentOutput, error) {
    var futures []workflow.ChildWorkflowFuture
    
    // 启动多个子工作流
    for _, item := range input.Items {
        childCtx := workflow.WithChildOptions(ctx, workflow.ChildWorkflowOptions{
            WorkflowID: generateID(),
        })
        
        future := workflow.ExecuteChildWorkflow(childCtx, ChildWorkflow, item)
        futures = append(futures, future)
    }
    
    // 等待所有子工作流完成
    var results []ChildOutput
    for _, future := range futures {
        var result ChildOutput
        err := future.Get(ctx, &result)
        if err != nil {
            return ParentOutput{}, err
        }
        results = append(results, result)
    }
    
    return ParentOutput{Results: results}, nil
}
```

---

## 🎯 最佳实践

### 1. 工作流设计原则

#### ✅ DO: 确定性

```rust
// ✅ 好: 确定性实现
async fn execute(ctx: WorkflowContext, input: Input) -> Result<Output, WorkflowError> {
    let result = ctx.execute_activity::<MyActivity>(input, options).await?;
    Ok(Output { data: result })
}

// ❌ 差: 非确定性 (使用随机数)
async fn execute(ctx: WorkflowContext, input: Input) -> Result<Output, WorkflowError> {
    let random = rand::random::<u32>();  // 不确定!
    // ...
}
```

#### ✅ DO: 使用Activity执行外部操作

```rust
// ✅ 好: 通过Activity执行HTTP请求
let result = ctx
    .execute_activity::<HttpRequestActivity>(request, options)
    .await?;

// ❌ 差: 直接在工作流中执行HTTP请求
let response = reqwest::get("https://api.example.com").await?;  // 不确定!
```

#### ✅ DO: 合理的超时设置

```rust
// ✅ 好: 明确的超时配置
let options = ActivityOptions {
    start_to_close_timeout: Some(Duration::from_secs(30)),
    schedule_to_close_timeout: Some(Duration::from_secs(60)),
    retry_policy: Some(RetryPolicy {
        max_attempts: 3,
        initial_interval: Duration::from_secs(1),
        max_interval: Duration::from_secs(10),
        backoff_coefficient: 2.0,
    }),
    ..Default::default()
};
```

### 2. 错误处理策略

```rust
async fn execute(ctx: WorkflowContext, input: Input) -> Result<Output, WorkflowError> {
    // 可重试的错误: 使用RetryPolicy
    let result = ctx
        .execute_activity::<NetworkActivity>(data, ActivityOptions {
            retry_policy: Some(RetryPolicy::default()),
            ..Default::default()
        })
        .await?;
    
    // 不可重试的错误: 立即失败
    if !result.is_valid() {
        return Err(WorkflowError::ValidationFailed("Invalid result".into()));
    }
    
    Ok(Output { result })
}
```

### 3. 状态管理

```rust
// 使用可序列化的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowState {
    step: u32,
    processed_items: Vec<String>,
    metadata: HashMap<String, String>,
}

async fn execute(ctx: WorkflowContext, input: Input) -> Result<Output, WorkflowError> {
    let mut state = WorkflowState::default();
    
    // 步骤1
    state.step = 1;
    let result1 = ctx.execute_activity::<Step1Activity>(data, options).await?;
    state.processed_items.push(result1.id);
    
    // 步骤2
    state.step = 2;
    let result2 = ctx.execute_activity::<Step2Activity>(data, options).await?;
    state.processed_items.push(result2.id);
    
    Ok(Output { state })
}
```

---

## 📚 总结

### Rust实现的优势

1. **类型安全**: 编译时保证类型正确性
2. **零成本抽象**: 性能优异
3. **内存安全**: 无GC，确定性性能
4. **现代语法**: async/await原生支持

### Golang实现的优势

1. **简单直观**: 学习曲线平缓
2. **成熟生态**: Temporal官方SDK
3. **快速开发**: 编译快，迭代快
4. **社区支持**: 丰富的文档和示例

### 选择建议

- **选择Rust**: 性能关键、类型安全、嵌入式系统
- **选择Golang**: 快速原型、团队熟悉、成熟生态

---

## 📚 下一步

- **Activity定义**: [Activity详解](./05_activity_definition.md)
- **Signal与Query**: [交互机制](./06_signals_and_queries.md)
- **完整示例**: [实战案例](./18_basic_examples.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
