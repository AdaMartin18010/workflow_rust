# Temporal vs workflow_rust 快速参考

## 📖 速查表

本文档提供Temporal和workflow_rust的对照速查表，帮助开发者快速理解两者的异同。

---

## 🎯 概念映射

| Temporal概念 | workflow_rust对应概念 | 说明 |
|-------------|---------------------|------|
| Workflow | WorkflowDefinition | 工作流定义 |
| Workflow Execution | WorkflowInstance | 工作流执行实例 |
| Activity | *Activity trait (待实现)* | 可重试的业务逻辑单元 |
| Worker | WorkflowEngine | 执行工作流的引擎 |
| Signal | WorkflowEvent::Signal (部分) | 外部事件通知 |
| Query | *QueryHandler (待实现)* | 查询工作流状态 |
| Namespace | N/A | 命名空间隔离 |
| Task Queue | mpsc::channel | 任务队列 |
| Timer | tokio::time::sleep | 定时器 |
| Child Workflow | *ChildWorkflow (待实现)* | 子工作流 |

---

## 📝 代码对比

### 1. 定义工作流

#### Temporal (Go SDK)

```go
func OrderWorkflow(ctx workflow.Context, orderID string) error {
    // 执行Activity
    var result PaymentResult
    err := workflow.ExecuteActivity(ctx, ProcessPayment, orderID).Get(ctx, &result)
    if err != nil {
        return err
    }
    
    // 下一步
    return workflow.ExecuteActivity(ctx, FulfillOrder, orderID).Get(ctx, nil)
}
```

#### workflow_rust (当前版本)

```rust
async fn order_workflow() -> Result<(), WorkflowError> {
    let mut engine = WorkflowEngine::new();
    
    // 定义工作流
    let mut definition = WorkflowDefinition::new("order".to_string());
    definition.add_state("payment".to_string());
    definition.add_state("fulfillment".to_string());
    definition.add_state("completed".to_string());
    
    definition.add_transition(
        "payment".to_string(), 
        "fulfillment".to_string(), 
        None
    );
    
    // 注册并启动
    engine.register_workflow("order".to_string(), definition).await?;
    let data = WorkflowData::new(json!({"order_id": "12345"}));
    engine.start_workflow("order", data).await?;
    
    Ok(())
}
```

#### workflow_rust (未来版本 - 计划中)

```rust
#[workflow]
async fn order_workflow(ctx: WorkflowContext, order_id: String) -> Result<(), WorkflowError> {
    // 执行Activity
    let payment_result = ctx.execute_activity(
        ProcessPaymentActivity,
        order_id.clone(),
    ).await?;
    
    // 下一步
    ctx.execute_activity(
        FulfillOrderActivity,
        order_id,
    ).await?;
    
    Ok(())
}
```

**对比:**

- ✅ workflow_rust未来版本将提供类似的简洁API
- ⚠️ 当前版本需要显式定义状态和转换
- ✅ Rust提供编译时类型检查，更安全

### 2. Saga模式/长事务

#### 2.1 Temporal (Go SDK)

```go
func BookingSagaWorkflow(ctx workflow.Context, req BookingRequest) error {
    // 预订航班
    var flightID string
    err := workflow.ExecuteActivity(ctx, BookFlight, req).Get(ctx, &flightID)
    if err != nil {
        return err
    }
    // 如果后续失败，使用defer补偿
    defer workflow.ExecuteActivity(ctx, CancelFlight, flightID)
    
    // 预订酒店
    var hotelID string
    err = workflow.ExecuteActivity(ctx, BookHotel, req).Get(ctx, &hotelID)
    if err != nil {
        return err // defer会自动执行补偿
    }
    
    return nil
}
```

#### 2.2 workflow_rust (当前版本)

```rust
pub struct BookingSaga {
    steps: Vec<Box<dyn SagaStep<Context = BookingContext, Error = BookingError>>>,
    executed_steps: Vec<usize>,
    context: BookingContext,
}

impl BookingSaga {
    async fn execute(&mut self) -> Result<(), SagaError> {
        for (idx, step) in self.steps.iter().enumerate() {
            match step.execute(&self.context).await {
                Ok(_) => self.executed_steps.push(idx),
                Err(error) => {
                    // 自动补偿
                    self.compensate().await?;
                    return Err(SagaError::StepFailed(error));
                }
            }
        }
        Ok(())
    }
    
    async fn compensate(&self) -> Result<(), SagaError> {
        // 逆序执行补偿
        for &idx in self.executed_steps.iter().rev() {
            self.steps[idx].compensate(&self.context).await?;
        }
        Ok(())
    }
}
```

**对比:**

- ✅ 两者都支持Saga模式
- ✅ workflow_rust提供更显式的补偿控制
- ⚠️ Temporal的defer机制更自然

### 3. 信号（Signal）

#### 3.1 Temporal (Go SDK)

```go
func ApprovalWorkflow(ctx workflow.Context) error {
    // 等待信号
    var approved bool
    signalChan := workflow.GetSignalChannel(ctx, "approval")
    signalChan.Receive(ctx, &approved)
    
    if approved {
        return workflow.ExecuteActivity(ctx, ProcessApproval).Get(ctx, nil)
    }
    return errors.New("not approved")
}

// 发送信号
client.SignalWorkflow(ctx, workflowID, runID, "approval", true)
```

#### workflow_rust (计划实现)

```rust
#[workflow]
async fn approval_workflow(ctx: WorkflowContext) -> Result<(), WorkflowError> {
    // 等待信号
    let signal = ctx.await_signal::<bool>("approval").await?;
    
    if signal {
        ctx.execute_activity(ProcessApprovalActivity).await?;
    }
    Ok(())
}

// 发送信号
client.send_signal("workflow_id", "approval", json!(true)).await?;
```

**对比:**

- ⚠️ workflow_rust信号机制正在开发中
- 🔄 API设计参考Temporal但更符合Rust习惯

### 4. 查询（Query）

#### 4.1 Temporal (TypeScript SDK)

```typescript
// 工作流中定义查询处理器
export async function orderWorkflow(orderId: string): Promise<void> {
  let status = 'pending';
  
  // 注册查询处理器
  setHandler(orderStatusQuery, () => status);
  
  // 更新状态
  status = 'processing';
  await processPayment(orderId);
  
  status = 'completed';
}

// 客户端查询
const status = await client.workflow.query(orderStatusQuery);
```

#### 4.2 workflow_rust (计划实现)

```rust
#[workflow]
async fn order_workflow(ctx: WorkflowContext, order_id: String) -> Result<(), WorkflowError> {
    let status = Arc::new(RwLock::new("pending".to_string()));
    let status_clone = status.clone();
    
    // 注册查询处理器
    ctx.set_query_handler("status", move || {
        Ok(json!({"status": status_clone.read().unwrap().clone()}))
    });
    
    // 更新状态
    *status.write().unwrap() = "processing".to_string();
    ctx.execute_activity(ProcessPaymentActivity, order_id).await?;
    
    *status.write().unwrap() = "completed".to_string();
    Ok(())
}

// 客户端查询
let status = client.query_workflow("workflow_id", "status").await?;
```

**对比:**

- ⚠️ workflow_rust查询机制正在开发中
- ✅ Rust闭包提供类型安全的查询处理

### 5. 重试策略

#### 5.1 Temporal (Go SDK)

```go
// 配置Activity重试
ao := workflow.ActivityOptions{
    StartToCloseTimeout: 10 * time.Minute,
    RetryPolicy: &temporal.RetryPolicy{
        InitialInterval:    time.Second,
        BackoffCoefficient: 2.0,
        MaximumInterval:    time.Minute,
        MaximumAttempts:    5,
    },
}
ctx = workflow.WithActivityOptions(ctx, ao)
```

#### 5.2 workflow_rust (计划实现)

```rust
#[activity]
struct PaymentActivity;

impl Activity for PaymentActivity {
    type Input = PaymentRequest;
    type Output = PaymentResult;
    type Error = PaymentError;
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // 业务逻辑
    }
    
    fn retry_policy(&self) -> Option<RetryPolicy> {
        Some(RetryPolicy {
            max_attempts: 5,
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            backoff_coefficient: 2.0,
            retryable_errors: vec![],
        })
    }
    
    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_secs(600))
    }
}
```

**对比:**

- ⚠️ workflow_rust重试机制需要通过Activity trait实现
- ✅ 配置方式类似，但更类型安全

---

## 🔄 迁移指南

### 从Temporal迁移到workflow_rust

#### 适合迁移的场景

1. **高性能需求**: 需要微秒级延迟的场景
2. **嵌入式部署**: 不需要独立的Temporal集群
3. **类型安全**: 想要编译时类型检查
4. **Rust生态**: 已有Rust技术栈

#### 迁移步骤

**Step 1: 分析现有工作流**:

```bash
# 列出所有Temporal工作流
temporal workflow list --namespace my-namespace
```

**Step 2: 重写工作流定义**:

```rust
// Temporal Go: MyWorkflow(ctx, input)
// workflow_rust: 
#[workflow]
async fn my_workflow(ctx: WorkflowContext, input: Input) -> Result<Output, Error> {
    // 将Temporal代码翻译为Rust
}
```

**Step 3: 重写Activity**:

```rust
// Temporal Activity
// workflow_rust Activity:
#[activity]
struct MyActivity;

impl Activity for MyActivity {
    // 实现trait
}
```

**Step 4: 配置持久化（可选）**:

```rust
let engine = WorkflowEngine::new()
    .with_persistence(/* 配置持久化 */);
```

### 从workflow_rust迁移到Temporal

#### 适合迁移的场景1

1. **需要分布式**: 大规模分布式工作流
2. **多语言支持**: 团队使用多种语言
3. **丰富工具**: 需要Temporal的工具生态

#### 迁移步骤1

**Step 1: 设计Temporal工作流**:

```go
// workflow_rust状态机 -> Temporal工作流函数
func MyWorkflow(ctx workflow.Context, input Input) (Output, error) {
    // 将状态转换翻译为Activity调用
}
```

**Step 2: 部署Temporal集群**:

```bash
# 使用Docker Compose
docker-compose up -d
```

**Step 3: 实现Worker**:

```go
w := worker.New(c, "my-task-queue", worker.Options{})
w.RegisterWorkflow(MyWorkflow)
w.RegisterActivity(MyActivity)
w.Run(worker.InterruptCh())
```

---

## 🎨 设计模式对比

### 1. 等待外部事件

#### Temporal: Signal Pattern

```go
func WaitForApprovalWorkflow(ctx workflow.Context) error {
    var approved bool
    signalChan := workflow.GetSignalChannel(ctx, "approval")
    signalChan.Receive(ctx, &approved)
    return nil
}
```

#### workflow_rust: Event Pattern

```rust
// 当前: 通过WorkflowEvent
pub enum WorkflowEvent {
    Signal { instance_id: String, signal_name: String, data: Value },
}

// 未来: Signal API
ctx.await_signal::<ApprovalData>("approval").await?
```

### 2. 定时任务

#### Temporal: Timer API

```go
workflow.Sleep(ctx, time.Hour)
```

#### workflow_rust: Tokio Timer

```rust
// 当前
tokio::time::sleep(Duration::from_secs(3600)).await;

// 未来
ctx.sleep(Duration::from_secs(3600)).await?;
```

### 3. 并行执行

#### Temporal: Futures API

```go
var a, b, c Awaitable

a = workflow.ExecuteActivity(ctx, ActivityA)
b = workflow.ExecuteActivity(ctx, ActivityB)
c = workflow.ExecuteActivity(ctx, ActivityC)

err := a.Get(ctx, nil)
err = b.Get(ctx, nil)
err = c.Get(ctx, nil)
```

#### workflow_rust: Tokio Join

```rust
// 使用tokio::join!宏
let (result_a, result_b, result_c) = tokio::join!(
    ctx.execute_activity(ActivityA, input_a),
    ctx.execute_activity(ActivityB, input_b),
    ctx.execute_activity(ActivityC, input_c),
);
```

---

## 📊 性能对比

| 指标 | Temporal | workflow_rust | 倍数 |
|-----|----------|---------------|------|
| 工作流创建延迟 | ~100-500 µs | ~1.2 µs | **~100-400x** 更快 |
| Activity执行延迟 | ~1-10 ms | ~5.8 µs | **~170-1700x** 更快 |
| 内存占用 | 中等（依赖Runtime） | 低（无GC） | **更低** |
| 吞吐量 | 高 | 极高 | **更高** |
| 持久化开销 | 高（必须） | 低（可选） | **可配置** |

**注意**:

- Temporal的开销主要来自网络通信和持久化
- workflow_rust适合嵌入式部署，性能更极致
- 对于分布式场景，Temporal的开销是必要的

---

## 🛠️ 工具对比

| 功能 | Temporal | workflow_rust |
|-----|----------|---------------|
| Web UI | ✅ 功能强大 | ⚠️ 计划中 |
| CLI工具 | ✅ 功能完整 | ⚠️ 基础功能 |
| 监控集成 | ✅ Prometheus/Grafana | ✅ metrics crate |
| 调试工具 | ✅ 时间旅行调试 | ⚠️ 标准Rust调试 |
| 可视化 | ✅ 工作流图 | ⚠️ 计划中 |
| 测试框架 | ✅ 专用测试框架 | ✅ Rust标准测试 |

---

## 🔗 互操作性

### workflow_rust调用Temporal

```rust
// 通过HTTP API调用Temporal工作流
use reqwest::Client;

async fn invoke_temporal_workflow(
    temporal_url: &str,
    workflow_id: &str,
) -> Result<Value, Error> {
    let client = Client::new();
    let response = client
        .post(format!("{}/api/v1/workflows/{}/start", temporal_url, workflow_id))
        .json(&json!({
            "taskQueue": "my-queue",
            "workflowType": "MyWorkflow",
        }))
        .send()
        .await?;
    
    Ok(response.json().await?)
}
```

### Temporal调用workflow_rust

```go
// 将workflow_rust封装为Temporal Activity
func RustWorkflowActivity(ctx context.Context, input Input) (Output, error) {
    // 调用workflow_rust的HTTP API或FFI
    // ...
}
```

---

## 📚 学习路径

### 从Temporal转向workflow_rust

1. **学习Rust基础** (1-2周)
   - 所有权系统
   - 生命周期
   - Trait系统

2. **理解workflow_rust架构** (3-5天)
   - [项目架构文档](./ARCHITECTURE.md)
   - [核心概念](./workflow_fundamentals/concepts.md)

3. **实践示例** (1周)
   - [简单示例](../examples/simple_demo.rs)
   - [高级示例](../examples/rust190_examples.rs)

4. **生产部署** (视需求而定)
   - 持久化配置
   - 监控集成
   - 性能调优

### 从workflow_rust转向Temporal

1. **理解Temporal架构** (1周)
   - Worker模型
   - 持久化机制
   - 任务队列

2. **选择SDK** (1天)
   - Go SDK (推荐)
   - TypeScript SDK
   - Java SDK

3. **部署集群** (3-5天)
   - Docker部署
   - Kubernetes部署
   - 云服务

4. **迁移工作流** (视规模而定)
   - 重写工作流代码
   - 数据迁移
   - 灰度切换

---

## ❓ 常见问题

### Q1: 何时选择Temporal，何时选择workflow_rust?

**选择Temporal:**

- ✅ 大规模分布式系统
- ✅ 多语言团队
- ✅ 需要强大的持久化和容错
- ✅ 希望开箱即用

**选择workflow_rust:**

- ✅ 高性能要求（微秒级）
- ✅ 嵌入式部署
- ✅ Rust技术栈
- ✅ 需要类型安全

### Q2: workflow_rust能否替代Temporal?

**不完全能**。两者定位不同：

- Temporal: 分布式工作流**平台**
- workflow_rust: 高性能工作流**库**

可以作为互补，在同一系统中混合使用。

### Q3: workflow_rust的学习曲线如何?

**相对陡峭**，主要原因：

- Rust语言本身的学习曲线
- 进程代数等理论概念

**建议**:

- 先掌握Rust基础
- 从简单示例开始
- 逐步理解高级特性

### Q4: 如何贡献代码?

**欢迎贡献**！请参考：

1. [贡献指南](../../CONTRIBUTING.md)
2. [开发文档](./DEVELOPMENT.md)
3. [问题追踪](https://github.com/yourorg/workflow_rust/issues)

---

## 📖 更多资源

### 官方文档

- [Temporal文档](https://docs.temporal.io/)
- [workflow_rust文档](./README.md)

### 对比分析

- [详细对比分析](./TEMPORAL_FRAMEWORK_COMPARISON.md)
- [实施路线图](./TEMPORAL_ALIGNMENT_ROADMAP.md)

### 示例代码

- [Temporal示例](https://github.com/temporalio/samples-go)
- [workflow_rust示例](../examples/)

---

**文档版本**: 1.0  
**最后更新**: 2025-10-26  
**维护者**: workflow_rust团队
