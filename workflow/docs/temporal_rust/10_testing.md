# 工作流测试策略

## 📋 文档概述

本文档详细阐述Temporal工作流的测试策略，包括：

- 单元测试
- 集成测试
- 端到端测试
- Rust 1.90实现
- Golang实现对比
- 测试最佳实践

---

## 🎯 测试金字塔

```text
┌─────────────────────────────────────────────────────────────┐
│                      测试金字塔                              │
└─────────────────────────────────────────────────────────────┘

                    ▲
                   ╱ ╲
                  ╱   ╲
                 ╱ E2E ╲               数量: 少
                ╱───────╲              速度: 慢
               ╱         ╲             成本: 高
              ╱ 集成测试  ╲
             ╱─────────────╲
            ╱               ╲
           ╱   单元测试      ╲         数量: 多
          ╱─────────────────╲        速度: 快
         ╱                   ╲       成本: 低
        ───────────────────────
```

---

## 🦀 Rust实现

### 1. 单元测试

#### Activity单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试Activity逻辑（不涉及Temporal）
    #[tokio::test]
    async fn test_calculate_total() {
        let input = CalculationInput {
            items: vec![
                OrderItem { price: 10.0, quantity: 2 },
                OrderItem { price: 5.0, quantity: 3 },
            ],
        };
        
        let result = calculate_total(&input);
        
        assert_eq!(result, 35.0);
    }
    
    /// 测试Activity错误处理
    #[tokio::test]
    async fn test_invalid_input() {
        let input = CalculationInput {
            items: vec![
                OrderItem { price: -10.0, quantity: 2 },  // 负价格
            ],
        };
        
        let result = validate_input(&input);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Price cannot be negative"
        );
    }
}
```

#### Workflow逻辑测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试工作流业务逻辑
    #[tokio::test]
    async fn test_order_validation() {
        let order = Order {
            amount: 1500.0,
            items: vec![],
        };
        
        // 测试是否需要审批
        assert!(requires_approval(&order));
        
        let small_order = Order {
            amount: 500.0,
            items: vec![],
        };
        
        assert!(!requires_approval(&small_order));
    }
}
```

### 2. Workflow测试

#### Mock Activity测试

```rust
use std::collections::HashMap;
use async_trait::async_trait;

/// Mock Activity执行器
struct MockActivityExecutor {
    responses: HashMap<String, serde_json::Value>,
}

impl MockActivityExecutor {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }
    
    fn with_response<A: Activity>(
        mut self,
        response: A::Output,
    ) -> Self {
        self.responses.insert(
            A::name().to_string(),
            serde_json::to_value(response).unwrap(),
        );
        self
    }
}

#[async_trait]
trait ActivityExecutor {
    async fn execute<A: Activity>(
        &self,
        input: A::Input,
    ) -> Result<A::Output, ActivityError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_order_workflow_success() {
        // 创建Mock执行器
        let executor = MockActivityExecutor::new()
            .with_response::<ProcessPaymentActivity>(PaymentResult {
                transaction_id: "txn-123".to_string(),
                success: true,
            })
            .with_response::<ShipOrderActivity>(ShipmentResult {
                tracking_number: "track-456".to_string(),
            });
        
        // 创建测试上下文
        let ctx = create_test_context(executor);
        
        // 执行工作流
        let input = OrderInput {
            order_id: "order-123".to_string(),
            amount: 100.0,
        };
        
        let result = OrderWorkflow::execute(ctx, input).await;
        
        // 验证结果
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.status, OrderStatus::Completed);
    }
    
    #[tokio::test]
    async fn test_order_workflow_payment_failed() {
        // Mock支付失败
        let executor = MockActivityExecutor::new()
            .with_response::<ProcessPaymentActivity>(PaymentResult {
                transaction_id: "".to_string(),
                success: false,
            });
        
        let ctx = create_test_context(executor);
        
        let input = OrderInput {
            order_id: "order-123".to_string(),
            amount: 100.0,
        };
        
        let result = OrderWorkflow::execute(ctx, input).await;
        
        // 验证结果
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.status, OrderStatus::Cancelled);
    }
}
```

#### 测试Signal

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_workflow_with_approval_signal() {
        let ctx = create_test_context_with_signals();
        
        // 启动工作流（异步）
        let workflow_future = OrderWorkflow::execute(ctx.clone(), input);
        
        // 模拟延迟后发送Signal
        tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            ctx.send_signal(ApprovalSignal {
                approved: true,
                approver: "test".to_string(),
            }).await.unwrap();
        });
        
        // 等待工作流完成
        let result = workflow_future.await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, OrderStatus::Completed);
    }
    
    #[tokio::test]
    async fn test_workflow_approval_timeout() {
        let ctx = create_test_context_with_timeout(Duration::from_millis(100));
        
        // 不发送Signal，测试超时
        let result = OrderWorkflow::execute(ctx, input).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, OrderStatus::Timeout);
    }
}
```

#### 测试Query

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_status_query() {
        let ctx = create_test_context();
        
        // 启动工作流
        let workflow_handle = tokio::spawn(async move {
            OrderWorkflow::execute(ctx.clone(), input).await
        });
        
        // 在工作流运行时查询状态
        sleep(Duration::from_millis(50)).await;
        
        let status = ctx.query::<StatusQuery>().await.unwrap();
        
        assert_eq!(status.current_step, "processing_payment");
        assert!(status.progress > 0.0 && status.progress < 1.0);
        
        // 等待工作流完成
        workflow_handle.await.unwrap().unwrap();
    }
}
```

### 3. 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// 集成测试：使用真实的Activity实现
    #[tokio::test]
    async fn test_order_workflow_integration() {
        // 设置测试环境
        let test_env = setup_test_environment().await;
        
        // 创建客户端
        let client = WorkflowClient::new(test_env.config());
        
        // 启动工作流
        let handle = client
            .start_workflow::<OrderWorkflow>(
                WorkflowId::generate(),
                "test-queue".to_string(),
                OrderInput {
                    order_id: "test-order".to_string(),
                    amount: 100.0,
                    items: vec![
                        OrderItem { product_id: "p1".to_string(), quantity: 2 },
                    ],
                },
                StartWorkflowOptions::default(),
            )
            .await
            .unwrap();
        
        // 等待完成
        let result = handle.get_result().await.unwrap();
        
        // 验证结果
        assert_eq!(result.status, OrderStatus::Completed);
        assert!(result.tracking_number.is_some());
        
        // 清理
        test_env.cleanup().await;
    }
    
    /// 测试工作流与真实数据库的交互
    #[tokio::test]
    async fn test_workflow_with_database() {
        let db = setup_test_database().await;
        
        // 插入测试数据
        db.insert_order(Order {
            id: "order-123".to_string(),
            status: OrderStatus::Pending,
        }).await.unwrap();
        
        // 启动工作流
        let result = execute_workflow_with_db(db.clone()).await;
        
        assert!(result.is_ok());
        
        // 验证数据库状态
        let order = db.get_order("order-123").await.unwrap();
        assert_eq!(order.status, OrderStatus::Completed);
        
        // 清理
        db.cleanup().await;
    }
}
```

### 4. 端到端测试

```rust
#[cfg(test)]
mod e2e_tests {
    use super::*;
    
    /// E2E测试：完整的工作流生命周期
    #[tokio::test]
    async fn test_complete_order_flow() {
        // 启动Temporal服务（测试容器）
        let temporal_server = start_temporal_test_server().await;
        
        // 启动Worker
        let worker = WorkflowWorker::new(WorkerConfig {
            task_queue: "e2e-test-queue".to_string(),
            ..Default::default()
        });
        
        worker.register_workflow::<OrderWorkflow>();
        worker.register_activity::<ProcessPaymentActivity>();
        worker.register_activity::<ShipOrderActivity>();
        
        let worker_handle = tokio::spawn(async move {
            worker.run().await
        });
        
        // 创建客户端
        let client = WorkflowClient::connect(&temporal_server.address()).await.unwrap();
        
        // 启动工作流
        let workflow_id = WorkflowId::generate();
        let handle = client
            .start_workflow::<OrderWorkflow>(
                workflow_id.clone(),
                "e2e-test-queue".to_string(),
                OrderInput {
                    order_id: "e2e-order".to_string(),
                    amount: 1500.0,  // 需要审批
                    items: vec![],
                },
                StartWorkflowOptions::default(),
            )
            .await
            .unwrap();
        
        // 等待工作流运行
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 查询状态
        let status = client
            .query_workflow::<StatusQuery>(&workflow_id)
            .await
            .unwrap();
        
        assert_eq!(status.current_step, "waiting_approval");
        
        // 发送审批Signal
        client
            .signal_workflow::<ApprovalSignal>(
                &workflow_id,
                ApprovalSignal {
                    approved: true,
                    approver: "e2e-test".to_string(),
                    comment: None,
                    timestamp: Utc::now(),
                },
            )
            .await
            .unwrap();
        
        // 等待完成
        let result = handle.get_result().await.unwrap();
        
        // 验证结果
        assert_eq!(result.status, OrderStatus::Completed);
        
        // 清理
        worker_handle.abort();
        temporal_server.stop().await;
    }
}
```

### 5. 测试工具和辅助函数

```rust
/// 测试工具模块
pub mod test_utils {
    use super::*;
    
    /// 创建测试用的WorkflowContext
    pub fn create_test_context() -> WorkflowContext {
        let execution = WorkflowExecution::new(WorkflowId::new("test-workflow"));
        WorkflowContext::new_for_test(execution)
    }
    
    /// 创建带Mock Activity执行器的上下文
    pub fn create_test_context_with_executor(
        executor: MockActivityExecutor,
    ) -> WorkflowContext {
        let execution = WorkflowExecution::new(WorkflowId::new("test-workflow"));
        let mut ctx = WorkflowContext::new_for_test(execution);
        ctx.set_activity_executor(Box::new(executor));
        ctx
    }
    
    /// 断言工作流状态
    pub async fn assert_workflow_status(
        ctx: &WorkflowContext,
        expected_state: WorkflowLifecycleState,
    ) {
        let actual = ctx.lifecycle().get_state().await;
        assert_eq!(actual, expected_state);
    }
    
    /// 等待条件满足
    pub async fn wait_for_condition<F>(
        mut condition: F,
        timeout: Duration,
    ) -> Result<(), String>
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        
        while !condition() {
            if start.elapsed() > timeout {
                return Err("Timeout waiting for condition".to_string());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        Ok(())
    }
}
```

---

## 🐹 Golang实现对比

### 单元测试 - Golang

```go
func TestCalculateTotal(t *testing.T) {
    items := []OrderItem{
        {Price: 10.0, Quantity: 2},
        {Price: 5.0, Quantity: 3},
    }
    
    total := calculateTotal(items)
    
    assert.Equal(t, 35.0, total)
}
```

### Workflow测试 - Golang

```go
func TestOrderWorkflow(t *testing.T) {
    // 创建测试环境
    testSuite := &testsuite.WorkflowTestSuite{}
    env := testSuite.NewTestWorkflowEnvironment()
    
    // Mock Activity
    env.OnActivity(ProcessPaymentActivity, mock.Anything, mock.Anything).
        Return(PaymentResult{
            TransactionID: "txn-123",
            Success:       true,
        }, nil)
    
    env.OnActivity(ShipOrderActivity, mock.Anything, mock.Anything).
        Return(ShipmentResult{
            TrackingNumber: "track-456",
        }, nil)
    
    // 执行工作流
    env.ExecuteWorkflow(OrderWorkflow, OrderInput{
        OrderID: "order-123",
        Amount:  100.0,
    })
    
    // 验证结果
    require.True(t, env.IsWorkflowCompleted())
    require.NoError(t, env.GetWorkflowError())
    
    var result OrderOutput
    require.NoError(t, env.GetWorkflowResult(&result))
    assert.Equal(t, OrderStatusCompleted, result.Status)
}
```

### 测试Signal - Golang

```go
func TestWorkflowWithSignal(t *testing.T) {
    testSuite := &testsuite.WorkflowTestSuite{}
    env := testSuite.NewTestWorkflowEnvironment()
    
    // 注册Signal回调
    env.RegisterDelayedCallback(func() {
        env.SignalWorkflow("approval", ApprovalSignal{
            Approved: true,
            Approver: "test",
        })
    }, 100*time.Millisecond)
    
    // 执行工作流
    env.ExecuteWorkflow(OrderWorkflow, input)
    
    require.True(t, env.IsWorkflowCompleted())
    
    var result OrderOutput
    env.GetWorkflowResult(&result)
    assert.Equal(t, OrderStatusCompleted, result.Status)
}
```

---

## 🎯 最佳实践

### 1. 测试覆盖率

```rust
// ✅ 好: 全面的测试覆盖

#[cfg(test)]
mod tests {
    // 正常路径
    #[tokio::test]
    async fn test_happy_path() { /* ... */ }
    
    // 错误处理
    #[tokio::test]
    async fn test_payment_failure() { /* ... */ }
    
    #[tokio::test]
    async fn test_shipment_failure() { /* ... */ }
    
    // 边界条件
    #[tokio::test]
    async fn test_zero_amount() { /* ... */ }
    
    #[tokio::test]
    async fn test_large_amount() { /* ... */ }
    
    // 并发场景
    #[tokio::test]
    async fn test_concurrent_signals() { /* ... */ }
}
```

### 2. 测试数据管理

```rust
// ✅ 好: 使用测试工厂
pub mod test_fixtures {
    pub fn create_test_order() -> Order {
        Order {
            id: format!("test-{}", Uuid::new_v4()),
            amount: 100.0,
            status: OrderStatus::Pending,
            ..Default::default()
        }
    }
    
    pub fn create_large_order() -> Order {
        Order {
            id: format!("test-{}", Uuid::new_v4()),
            amount: 10000.0,
            status: OrderStatus::Pending,
            ..Default::default()
        }
    }
}
```

### 3. 测试隔离

```rust
// ✅ 好: 每个测试独立
#[tokio::test]
async fn test_isolated() {
    // 使用唯一的workflow_id
    let workflow_id = WorkflowId::generate();
    
    // 使用独立的测试数据库
    let db = create_test_db().await;
    
    // 测试逻辑...
    
    // 清理
    db.cleanup().await;
}
```

### 4. 异步测试

```rust
// ✅ 好: 正确处理异步

#[tokio::test]
async fn test_async_workflow() {
    let result = execute_workflow().await;
    assert!(result.is_ok());
}

// ⚠️ 注意: 使用timeout避免永久挂起
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        execute_workflow()
    ).await;
    
    assert!(result.is_ok(), "Test timed out");
}
```

---

## 📊 测试策略矩阵

| 测试类型 | 覆盖范围 | 执行速度 | 成本 | 推荐数量 |
|---------|---------|---------|------|---------|
| **单元测试** | 单个函数/方法 | 非常快 | 低 | 多 (70%) |
| **Workflow测试** | 单个工作流 | 快 | 中 | 中等 (20%) |
| **集成测试** | 多个组件 | 中等 | 中 | 少 (7%) |
| **E2E测试** | 完整系统 | 慢 | 高 | 很少 (3%) |

---

## 📚 总结

### 测试原则

1. **金字塔原则**: 多写单元测试，少写E2E测试
2. **快速反馈**: 测试应该快速执行
3. **独立性**: 测试之间应该互不影响
4. **可重复**: 测试结果应该一致
5. **清晰性**: 测试意图应该明确

### Rust vs Golang

- **Rust**: 使用tokio::test，需要手动构建测试环境
- **Golang**: 使用Temporal官方测试框架，开箱即用

---

## 📚 下一步

- **性能测试**: [性能基准测试](./performance_testing.md)
- **生产部署**: [部署策略](./deployment.md)
- **监控告警**: [可观测性](./monitoring.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

