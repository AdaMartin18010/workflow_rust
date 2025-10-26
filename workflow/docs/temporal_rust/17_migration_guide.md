# 迁移指南

## 📋 文档概述

本文档提供从其他Temporal SDK迁移到Temporal-Rust的完整指南，包括：

- 从Temporal Go SDK迁移
- 从Temporal Java SDK迁移
- 概念对照表
- 代码迁移示例
- 常见问题

---

## 🐹 从Temporal Go SDK迁移

### 概念映射

| Go SDK | Temporal-Rust | 说明 |
|--------|---------------|------|
| `workflow.Context` | `WorkflowContext` | 工作流上下文 |
| `activity.Context` | `ActivityContext` | Activity上下文 |
| `client.Client` | `WorkflowClient` | 客户端接口 |
| `worker.Worker` | `WorkflowWorker` | Worker实例 |
| `workflow.ExecuteActivity()` | `ctx.execute_activity()` | 执行Activity |
| `workflow.Sleep()` | `ctx.sleep()` | 延时等待 |
| `workflow.Now()` | `ctx.now()` | 获取当前时间 |
| `workflow.GetLogger()` | `tracing::info!()` | 日志记录 |

### 工作流定义迁移

#### Go SDK

```go
package workflows

import (
    "time"
    "go.temporal.io/sdk/workflow"
)

type OrderInput struct {
    OrderID string
    Amount  float64
}

type OrderOutput struct {
    Status string
}

func OrderProcessingWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    logger := workflow.GetLogger(ctx)
    logger.Info("Starting order processing", "order_id", input.OrderID)
    
    // 执行Activity
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 5 * time.Minute,
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)
    
    var result string
    err := workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input.OrderID).Get(ctx, &result)
    if err != nil {
        return OrderOutput{}, err
    }
    
    return OrderOutput{Status: result}, nil
}
```

#### Temporal-Rust

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OrderInput {
    pub order_id: String,
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct OrderOutput {
    pub status: String,
}

pub struct OrderProcessingWorkflow;

impl Workflow for OrderProcessingWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderProcessing"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            order_id = %input.order_id,
            "Starting order processing"
        );
        
        // 执行Activity
        let result = ctx.execute_activity::<ProcessPaymentActivity>(
            PaymentInput { order_id: input.order_id.clone() },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(300)),
                ..Default::default()
            },
        ).await?;
        
        Ok(OrderOutput {
            status: result.status,
        })
    }
}
```

### Activity定义迁移

#### Go SDK

```go
package activities

import (
    "context"
    "go.temporal.io/sdk/activity"
)

func ProcessPaymentActivity(ctx context.Context, orderID string) (string, error) {
    logger := activity.GetLogger(ctx)
    logger.Info("Processing payment", "order_id", orderID)
    
    // 发送心跳
    activity.RecordHeartbeat(ctx, "processing")
    
    // 处理逻辑
    result := "success"
    
    return result, nil
}
```

#### Temporal-Rust

```rust
use temporal_rust::*;

pub struct ProcessPaymentActivity;

#[derive(Serialize, Deserialize)]
pub struct PaymentInput {
    pub order_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PaymentOutput {
    pub status: String,
}

impl Activity for ProcessPaymentActivity {
    type Input = PaymentInput;
    type Output = PaymentOutput;
    
    fn name() -> &'static str {
        "ProcessPayment"
    }
    
    async fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(
            order_id = %input.order_id,
            "Processing payment"
        );
        
        // 发送心跳
        ctx.record_heartbeat(serde_json::json!({
            "status": "processing"
        })).await;
        
        // 处理逻辑
        Ok(PaymentOutput {
            status: "success".to_string(),
        })
    }
}
```

### 客户端使用迁移

#### Go SDK

```go
package main

import (
    "context"
    "go.temporal.io/sdk/client"
)

func main() {
    // 创建客户端
    c, err := client.Dial(client.Options{
        HostPort: "localhost:7233",
    })
    if err != nil {
        panic(err)
    }
    defer c.Close()
    
    // 启动工作流
    workflowOptions := client.StartWorkflowOptions{
        ID:        "order-123",
        TaskQueue: "order-queue",
    }
    
    we, err := c.ExecuteWorkflow(
        context.Background(),
        workflowOptions,
        OrderProcessingWorkflow,
        OrderInput{OrderID: "123", Amount: 99.99},
    )
    if err != nil {
        panic(err)
    }
    
    // 等待结果
    var result OrderOutput
    err = we.Get(context.Background(), &result)
    if err != nil {
        panic(err)
    }
}
```

#### Temporal-Rust

```rust
use temporal_rust::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = WorkflowClient::new(
        ClientConfig::builder()
            .target_url("http://localhost:7233")
            .build()
    ).await?;
    
    // 启动工作流
    let handle = client.start_workflow::<OrderProcessingWorkflow>(
        OrderInput {
            order_id: "123".to_string(),
            amount: 99.99,
        },
        StartWorkflowOptions {
            workflow_id: Some(WorkflowId::new("order-123")),
            task_queue: "order-queue".to_string(),
            ..Default::default()
        },
    ).await?;
    
    // 等待结果
    let result = handle.get_result().await?;
    
    Ok(())
}
```

---

## ☕ 从Temporal Java SDK迁移

### 概念映射

| Java SDK | Temporal-Rust | 说明 |
|----------|---------------|------|
| `@WorkflowInterface` | `impl Workflow` | 工作流定义 |
| `@WorkflowMethod` | `fn execute()` | 工作流方法 |
| `@ActivityInterface` | `impl Activity` | Activity定义 |
| `@ActivityMethod` | `fn execute()` | Activity方法 |
| `@SignalMethod` | `impl Signal` | Signal定义 |
| `@QueryMethod` | `impl Query` | Query定义 |
| `Workflow.sleep()` | `ctx.sleep()` | 延时 |
| `Workflow.getLogger()` | `tracing::info!()` | 日志 |

### 工作流定义迁移

#### Java SDK

```java
package com.example.workflows;

import io.temporal.workflow.WorkflowInterface;
import io.temporal.workflow.WorkflowMethod;

@WorkflowInterface
public interface OrderWorkflow {
    @WorkflowMethod
    OrderOutput process(OrderInput input);
}

public class OrderWorkflowImpl implements OrderWorkflow {
    @Override
    public OrderOutput process(OrderInput input) {
        // 获取Logger
        Logger logger = Workflow.getLogger(OrderWorkflowImpl.class);
        logger.info("Processing order: " + input.getOrderId());
        
        // 创建Activity stub
        ActivityOptions options = ActivityOptions.newBuilder()
            .setStartToCloseTimeout(Duration.ofMinutes(5))
            .build();
        
        ProcessPaymentActivity activity = Workflow.newActivityStub(
            ProcessPaymentActivity.class,
            options
        );
        
        // 执行Activity
        String result = activity.processPayment(input.getOrderId());
        
        return new OrderOutput(result);
    }
}
```

#### Temporal-Rust

```rust
pub struct OrderWorkflow;

impl Workflow for OrderWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            order_id = %input.order_id,
            "Processing order"
        );
        
        // 执行Activity
        let result = ctx.execute_activity::<ProcessPaymentActivity>(
            PaymentInput { order_id: input.order_id },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(300)),
                ..Default::default()
            },
        ).await?;
        
        Ok(OrderOutput { status: result })
    }
}
```

---

## 🔄 迁移步骤

### 第1步：项目初始化

```bash
# 创建新的Rust项目
cargo new my-temporal-project
cd my-temporal-project

# 添加依赖
cat >> Cargo.toml << EOF
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
EOF
```

### 第2步：定义数据模型

```rust
// src/models.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInput {
    pub order_id: String,
    pub amount: f64,
    pub customer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderOutput {
    pub order_id: String,
    pub status: String,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}
```

### 第3步：迁移Activity

**原则**:

- 每个Go/Java Activity对应一个Rust struct
- 实现`Activity` trait
- 使用强类型的Input/Output

```rust
// src/activities/payment.rs
pub struct ProcessPaymentActivity;

impl Activity for ProcessPaymentActivity {
    type Input = PaymentInput;
    type Output = PaymentOutput;
    
    fn name() -> &'static str {
        "ProcessPayment"
    }
    
    async fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        // 迁移原有逻辑
        Ok(output)
    }
}
```

### 第4步：迁移Workflow

**注意事项**:

- 保持确定性
- 使用`ctx`提供的时间和随机数
- 异步执行改为`await`

```rust
// src/workflows/order.rs
pub struct OrderWorkflow;

impl Workflow for OrderWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        // 迁移原有工作流逻辑
        Ok(output)
    }
}
```

### 第5步：更新Worker

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建Worker
    let config = WorkerConfig::builder()
        .task_queue("order-queue")
        .build();
    
    let worker = WorkflowWorker::new(config);
    
    // 注册Workflow和Activity
    worker.register_workflow::<OrderWorkflow>().await;
    worker.register_activity::<ProcessPaymentActivity>().await;
    
    // 运行Worker
    worker.run().await?;
    
    Ok(())
}
```

---

## 📋 迁移检查清单

### 数据模型

- [ ] 定义所有Input/Output结构体
- [ ] 添加`Serialize`和`Deserialize` derive
- [ ] 验证字段类型兼容性

### Activity迁移

- [ ] 为每个Activity创建struct
- [ ] 实现`Activity` trait
- [ ] 迁移业务逻辑
- [ ] 添加错误处理
- [ ] 测试幂等性

### Workflow迁移

- [ ] 为每个Workflow创建struct
- [ ] 实现`Workflow` trait
- [ ] 确保确定性执行
- [ ] 迁移Signal和Query
- [ ] 添加重试策略

### 客户端迁移

- [ ] 更新客户端创建代码
- [ ] 迁移工作流启动逻辑
- [ ] 更新Signal发送代码
- [ ] 更新Query执行代码

### 测试

- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 验证与原系统兼容性
- [ ] 性能测试

---

## ⚠️ 常见问题

### 1. 时间处理差异

**问题**: Go/Java中直接使用`time.Now()`

**解决**: 使用`ctx.now()`

```rust
// ❌ 不要这样
let now = Utc::now();

// ✅ 应该这样
let now = ctx.now();
```

### 2. 随机数生成

**问题**: 直接使用`rand::random()`

**解决**: 使用`ctx.new_uuid()`

```rust
// ❌ 不要这样
let id = uuid::Uuid::new_v4();

// ✅ 应该这样
let id = ctx.new_uuid();
```

### 3. 错误处理

**问题**: Go的`error`类型

**解决**: 使用Rust的`Result`类型

```rust
// Go: func process() error
// Rust:
async fn process() -> Result<(), ActivityError> {
    Ok(())
}
```

### 4. 空值处理

**问题**: Go/Java的`nil`/`null`

**解决**: 使用Rust的`Option`

```rust
// Go: var result *string
// Rust:
let result: Option<String> = None;
```

---

## 🔍 性能对比

| 指标 | Go SDK | Java SDK | Temporal-Rust |
|------|--------|----------|---------------|
| **内存使用** | 中等 | 较高 | 低 |
| **启动时间** | 快速 | 较慢 | 快速 |
| **执行性能** | 优秀 | 良好 | 优秀 |
| **类型安全** | 良好 | 良好 | 优秀 |
| **并发能力** | 优秀 | 良好 | 优秀 |

---

## 📚 总结

### 迁移优势

1. **类型安全**: 编译时捕获错误
2. **性能提升**: 零成本抽象
3. **内存安全**: 无GC，所有权系统
4. **现代工具链**: Cargo生态系统

### 迁移挑战

1. **学习曲线**: Rust语法和概念
2. **生态差异**: 部分库可能需要替代
3. **异步模型**: async/await范式

### 推荐策略

1. **增量迁移**: 逐个服务迁移
2. **充分测试**: 确保功能一致性
3. **性能验证**: 对比迁移前后指标

---

## 📚 下一步

- **基础示例**: [Hello World](./18_basic_examples.md)
- **实战示例**: [完整案例](./19_advanced_examples.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
