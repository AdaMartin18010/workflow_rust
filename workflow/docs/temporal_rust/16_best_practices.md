# 最佳实践

## 📋 文档概述

本文档汇总Temporal工作流系统的最佳实践，包括：

- 工作流设计原则
- 错误处理模式
- 性能优化技巧
- 安全考虑
- 测试策略
- 运维建议

---

## 🎯 工作流设计原则

### 1. 确定性执行

```rust
// ✅ 好: 确定性的工作流
impl Workflow for OrderWorkflow {
    async fn execute(ctx: WorkflowContext, order: Order) -> Result<OrderResult, WorkflowError> {
        // 使用WorkflowContext提供的时间
        let now = ctx.now();
        
        // 使用确定性的随机数
        let request_id = ctx.new_uuid();
        
        // Activity执行是确定性的
        let payment = ctx.execute_activity::<ProcessPaymentActivity>(...).await?;
        
        Ok(result)
    }
}

// ❌ 不好: 非确定性的工作流
impl Workflow for BadWorkflow {
    async fn execute(ctx: WorkflowContext, input: Input) -> Result<Output, WorkflowError> {
        // ❌ 直接使用系统时间（每次重放会不同）
        let now = Utc::now();
        
        // ❌ 使用随机数（每次重放会不同）
        let random = rand::random::<u64>();
        
        // ❌ 直接进行I/O操作
        let data = tokio::fs::read_to_string("file.txt").await?;
        
        Ok(result)
    }
}
```

### 2. 单一职责原则

```rust
// ✅ 好: 每个工作流职责单一
pub struct OrderProcessingWorkflow;  // 只处理订单流程

pub struct PaymentWorkflow;          // 只处理支付

pub struct ShipmentWorkflow;         // 只处理发货

// ❌ 不好: 一个工作流做所有事情
pub struct MegaWorkflow;  // 处理订单、支付、发货、库存、通知...
```

### 3. 活动粒度

```rust
// ✅ 好: 适当的Activity粒度
pub struct ValidateOrderActivity;      // 单一职责
pub struct ReserveInventoryActivity;   // 可重试
pub struct ProcessPaymentActivity;     // 独立事务

// ❌ 不好: Activity过大
pub struct ProcessEverythingActivity;  // 做所有事情，难以重试

// ❌ 不好: Activity过小
pub struct ValidateOrderIdActivity;
pub struct ValidateOrderItemsActivity;
pub struct ValidateOrderAmountActivity;  // 应该合并
```

### 4. 幂等性设计

```rust
// ✅ 好: 幂等的Activity
impl Activity for ProcessPaymentActivity {
    async fn execute(ctx: ActivityContext, input: PaymentInput) -> Result<PaymentOutput, ActivityError> {
        // 使用唯一的幂等性键
        let idempotency_key = format!("payment-{}", input.order_id);
        
        // 检查是否已处理
        if let Some(existing) = payment_service
            .get_by_idempotency_key(&idempotency_key)
            .await?
        {
            return Ok(existing);
        }
        
        // 处理支付，使用幂等性键
        let result = payment_service
            .process_with_key(input, idempotency_key)
            .await?;
        
        Ok(result)
    }
}
```

---

## 🛡️ 错误处理模式

### 1. 区分可重试和不可重试错误

```rust
#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    // 可重试的错误
    #[error("Temporary error: {0}")]
    Temporary(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    // 不可重试的错误
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl ActivityError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Temporary(_) | Self::Network(_) | Self::ServiceUnavailable(_)
        )
    }
}
```

### 2. 重试策略配置

```rust
// ✅ 好: 根据错误类型配置重试
let retry_policy = RetryPolicy {
    max_attempts: Some(3),
    initial_interval: Duration::from_millis(100),
    backoff_coefficient: 2.0,
    max_interval: Duration::from_secs(10),
    non_retryable_error_types: vec![
        "InvalidInput",
        "PermissionDenied",
        "NotFound",
    ],
};

ctx.execute_activity::<ProcessPaymentActivity>(
    input,
    ActivityOptions {
        retry_policy: Some(retry_policy),
        ..Default::default()
    }
).await?;
```

### 3. 补偿机制（Saga）

```rust
// ✅ 好: 完整的补偿流程
async fn execute_with_compensation(
    ctx: WorkflowContext,
    order: Order,
) -> Result<OrderResult, WorkflowError> {
    // 1. 预留库存
    let reservation = match ctx.execute_activity::<ReserveInventoryActivity>(input).await {
        Ok(res) => res,
        Err(e) => {
            return Ok(OrderResult::failed("Inventory reservation failed"));
        }
    };
    
    // 2. 处理支付
    let payment = match ctx.execute_activity::<ProcessPaymentActivity>(input).await {
        Ok(pay) => pay,
        Err(e) => {
            // 补偿：释放库存
            let _ = ctx.execute_activity::<ReleaseInventoryActivity>(
                ReleaseInput { reservation_id: reservation.id }
            ).await;
            
            return Ok(OrderResult::failed("Payment failed"));
        }
    };
    
    // 3. 创建发货单
    match ctx.execute_activity::<CreateShipmentActivity>(input).await {
        Ok(shipment) => Ok(OrderResult::success(shipment)),
        Err(e) => {
            // 补偿：退款 + 释放库存
            let _ = ctx.execute_activity::<RefundPaymentActivity>(
                RefundInput { payment_id: payment.id }
            ).await;
            
            let _ = ctx.execute_activity::<ReleaseInventoryActivity>(
                ReleaseInput { reservation_id: reservation.id }
            ).await;
            
            Ok(OrderResult::failed("Shipment creation failed"))
        }
    }
}
```

---

## ⚡ 性能优化

### 1. 并行执行Activity

```rust
// ✅ 好: 并行执行独立的Activity
let (validation, inventory_check) = tokio::join!(
    ctx.execute_activity::<ValidateOrderActivity>(validate_input),
    ctx.execute_activity::<CheckInventoryActivity>(inventory_input),
);

// ❌ 不好: 串行执行可并行的Activity
let validation = ctx.execute_activity::<ValidateOrderActivity>(validate_input).await?;
let inventory_check = ctx.execute_activity::<CheckInventoryActivity>(inventory_input).await?;
```

### 2. 批量处理

```rust
// ✅ 好: 批量处理多个项目
impl Activity for BatchProcessActivity {
    async fn execute(
        ctx: ActivityContext,
        items: Vec<Item>,
    ) -> Result<Vec<ItemResult>, ActivityError> {
        // 批量处理，减少网络往返
        let results = service.batch_process(&items).await?;
        Ok(results)
    }
}

// ❌ 不好: 逐个处理
for item in items {
    ctx.execute_activity::<ProcessSingleItemActivity>(item).await?;
}
```

### 3. Continue As New

```rust
// ✅ 好: 使用Continue As New处理长时间运行的工作流
impl Workflow for DataProcessingWorkflow {
    async fn execute(
        ctx: WorkflowContext,
        mut state: ProcessingState,
    ) -> Result<ProcessingResult, WorkflowError> {
        const MAX_ITERATIONS: usize = 1000;
        
        for i in 0..MAX_ITERATIONS {
            // 处理一批数据
            state = process_batch(&ctx, state).await?;
            
            // 检查是否需要Continue As New
            if ctx.should_continue_as_new() {
                return ctx.continue_as_new(state);
            }
        }
        
        Ok(ProcessingResult::complete(state))
    }
}
```

### 4. 缓存和复用

```rust
// ✅ 好: 复用客户端连接
pub struct AppState {
    workflow_client: Arc<WorkflowClient>,
    http_client: Arc<reqwest::Client>,
    db_pool: Arc<PgPool>,
}

// 在整个应用中复用
async fn handle_request(State(app): State<Arc<AppState>>) -> Response {
    let handle = app.workflow_client
        .start_workflow::<MyWorkflow>(input, options)
        .await?;
    // ...
}
```

---

## 🔒 安全考虑

### 1. 敏感数据处理

```rust
// ✅ 好: 不在工作流历史中存储敏感数据
#[derive(Serialize, Deserialize)]
pub struct PaymentInput {
    pub order_id: String,
    pub amount: f64,
    // ❌ 不要存储完整的信用卡号
    // pub credit_card_number: String,
    
    // ✅ 存储Token引用
    pub payment_token: String,
}

// ✅ 在Activity中处理敏感数据
impl Activity for ProcessPaymentActivity {
    async fn execute(ctx: ActivityContext, input: PaymentInput) -> Result<Output, Error> {
        // 从安全存储中获取实际的支付信息
        let payment_details = vault_service
            .get_payment_details(&input.payment_token)
            .await?;
        
        // 处理支付
        let result = payment_gateway.charge(payment_details).await?;
        
        Ok(result)
    }
}
```

### 2. 访问控制

```rust
// ✅ 好: 在Activity层实现访问控制
impl Activity for UpdateUserDataActivity {
    async fn execute(ctx: ActivityContext, input: UpdateInput) -> Result<Output, Error> {
        // 验证权限
        if !auth_service.has_permission(&input.user_id, Permission::UpdateUser).await? {
            return Err(ActivityError::PermissionDenied(
                "User does not have permission to update data".into()
            ));
        }
        
        // 执行操作
        let result = user_service.update(&input).await?;
        Ok(result)
    }
}
```

### 3. 输入验证

```rust
// ✅ 好: 严格的输入验证
#[derive(Deserialize)]
pub struct OrderInput {
    #[serde(deserialize_with = "validate_order_id")]
    pub order_id: String,
    
    #[serde(deserialize_with = "validate_positive")]
    pub amount: f64,
    
    #[serde(deserialize_with = "validate_email")]
    pub customer_email: String,
}

fn validate_order_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    
    if s.len() < 5 || s.len() > 50 {
        return Err(serde::de::Error::custom("Invalid order ID length"));
    }
    
    if !s.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(serde::de::Error::custom("Invalid order ID format"));
    }
    
    Ok(s)
}
```

---

## 🧪 测试策略

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_activity_success() {
        let ctx = ActivityContext::mock();
        let input = TestInput { value: 42 };
        
        let result = TestActivity::execute(ctx, input).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 42);
    }
    
    #[tokio::test]
    async fn test_activity_retry() {
        let ctx = ActivityContext::mock();
        let input = TestInput { value: -1 };
        
        let result = TestActivity::execute(ctx, input).await;
        
        assert!(matches!(result, Err(ActivityError::Temporary(_))));
    }
}
```

### 2. 集成测试

```rust
#[tokio::test]
async fn test_order_workflow() {
    // 创建测试客户端
    let client = WorkflowClient::new_for_testing().await;
    
    // 准备测试数据
    let order = create_test_order();
    
    // 启动工作流
    let handle = client
        .start_workflow::<OrderWorkflow>(order, options)
        .await
        .unwrap();
    
    // 验证结果
    let result = handle.get_result().await.unwrap();
    assert_eq!(result.status, OrderStatus::Completed);
}
```

### 3. 时间控制测试

```rust
#[tokio::test]
async fn test_workflow_timeout() {
    let ctx = WorkflowContext::mock();
    
    // 设置模拟时间
    ctx.set_current_time(Utc::now());
    
    // 执行工作流
    let result = TimeoutWorkflow::execute(ctx.clone(), input).await;
    
    // 推进时间
    ctx.advance_time(Duration::from_secs(3600));
    
    // 验证超时行为
    assert!(matches!(result, Err(WorkflowError::Timeout)));
}
```

---

## 📊 监控和运维

### 1. 关键指标

```rust
// ✅ 记录关键指标
impl WorkflowWorker {
    async fn execute_workflow_with_metrics(&self, task: WorkflowTask) -> Result<(), Error> {
        // 记录开始
        self.metrics.workflows_started.inc();
        let timer = self.metrics.workflow_duration.start_timer();
        
        // 执行
        let result = self.execute_workflow(task).await;
        
        // 记录结果
        match &result {
            Ok(_) => self.metrics.workflows_completed.inc(),
            Err(_) => self.metrics.workflows_failed.inc(),
        }
        
        timer.observe_duration();
        result
    }
}
```

### 2. 结构化日志

```rust
// ✅ 好: 结构化日志
tracing::info!(
    workflow_id = %workflow_id,
    workflow_type = "OrderProcessing",
    order_id = %order.id,
    amount = order.amount,
    "Processing order"
);

// ❌ 不好: 非结构化日志
println!("Processing order {} for workflow {}", order.id, workflow_id);
```

### 3. 告警规则

```yaml
# Prometheus告警规则
groups:
  - name: temporal_workflow
    rules:
    - alert: HighWorkflowFailureRate
      expr: rate(workflows_failed_total[5m]) > 0.1
      for: 5m
      annotations:
        summary: "High workflow failure rate"
        
    - alert: WorkflowDurationHigh
      expr: histogram_quantile(0.99, rate(workflow_duration_seconds_bucket[5m])) > 300
      for: 10m
      annotations:
        summary: "Workflow duration p99 > 5 minutes"
```

---

## 📚 代码组织

### 1. 项目结构

```text
src/
├── temporal/           # Temporal核心实现
│   ├── workflow.rs
│   ├── activity.rs
│   └── ...
├── workflows/          # 业务工作流
│   ├── order.rs
│   ├── payment.rs
│   └── mod.rs
├── activities/         # 业务Activity
│   ├── payment.rs
│   ├── inventory.rs
│   └── mod.rs
├── services/           # 外部服务集成
│   ├── payment_service.rs
│   └── mod.rs
└── lib.rs
```

### 2. 模块化设计

```rust
// ✅ 好: 清晰的模块边界
mod workflows {
    pub mod order;
    pub mod payment;
}

mod activities {
    pub mod payment;
    pub mod inventory;
}

// 公开API
pub use workflows::{OrderWorkflow, PaymentWorkflow};
pub use activities::{ProcessPaymentActivity, ReserveInventoryActivity};
```

---

## 📚 总结

### 核心原则

1. **确定性**: 工作流必须是确定性的
2. **幂等性**: Activity应该是幂等的
3. **单一职责**: 每个工作流/Activity职责单一
4. **错误处理**: 区分可重试和不可重试错误
5. **性能**: 并行执行、批量处理、Continue As New
6. **安全**: 不存储敏感数据，实现访问控制
7. **监控**: 完整的指标、日志、追踪

---

## 📚 下一步

- **迁移指南**: [从其他系统迁移](./17_migration_guide.md)
- **完整示例**: [实战案例](./18_basic_examples.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
