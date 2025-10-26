# 微服务编排

## 📋 文档概述

本文档提供微服务编排的示例，包括：

- 服务间协调
- 分布式事务
- Saga模式应用
- 服务降级和熔断
- Rust + Golang并列对比

---

## 🔄 微服务编排示例

### 场景：电商订单完整流程

涉及多个微服务：订单服务、支付服务、库存服务、物流服务。

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// ========================================
// 微服务编排工作流
// ========================================

#[derive(Serialize, Deserialize)]
pub struct OrderOrchestrationInput {
    pub order_id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub shipping_address: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}

#[derive(Serialize, Deserialize)]
pub struct OrderOrchestrationOutput {
    pub order_id: String,
    pub status: String,
    pub transaction_id: Option<String>,
    pub shipment_id: Option<String>,
}

pub struct OrderOrchestrationWorkflow;

impl Workflow for OrderOrchestrationWorkflow {
    type Input = OrderOrchestrationInput;
    type Output = OrderOrchestrationOutput;
    
    fn name() -> &'static str {
        "OrderOrchestration"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            order_id = %input.order_id,
            "Starting order orchestration"
        );
        
        // 1. 验证订单
        let validation = ctx.execute_activity::<ValidateOrderActivity>(
            ValidateOrderInput {
                order_id: input.order_id.clone(),
                items: input.items.clone(),
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(10)),
                ..Default::default()
            },
        ).await?;
        
        if !validation.valid {
            return Ok(OrderOrchestrationOutput {
                order_id: input.order_id,
                status: "validation_failed".to_string(),
                transaction_id: None,
                shipment_id: None,
            });
        }
        
        // 2. 预留库存
        let reservation = ctx.execute_activity::<ReserveInventoryActivity>(
            ReserveInventoryInput {
                order_id: input.order_id.clone(),
                items: input.items.clone(),
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(30)),
                retry_policy: Some(RetryPolicy {
                    max_attempts: Some(3),
                    initial_interval: Duration::from_secs(1),
                    max_interval: Duration::from_secs(10),
                    backoff_coefficient: 2.0,
                    non_retryable_error_types: vec!["InsufficientStock"],
                }),
                ..Default::default()
            },
        ).await;
        
        let reservation_id = match reservation {
            Ok(res) => res.reservation_id,
            Err(e) => {
                tracing::error!(error = ?e, "Inventory reservation failed");
                return Ok(OrderOrchestrationOutput {
                    order_id: input.order_id,
                    status: "out_of_stock".to_string(),
                    transaction_id: None,
                    shipment_id: None,
                });
            }
        };
        
        // 3. 处理支付
        let payment = ctx.execute_activity::<ProcessPaymentActivity>(
            PaymentInput {
                order_id: input.order_id.clone(),
                user_id: input.user_id.clone(),
                amount: input.total_amount,
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(60)),
                retry_policy: Some(RetryPolicy {
                    max_attempts: Some(3),
                    initial_interval: Duration::from_secs(2),
                    max_interval: Duration::from_secs(10),
                    backoff_coefficient: 2.0,
                    non_retryable_error_types: vec!["InsufficientFunds", "InvalidPaymentMethod"],
                }),
                ..Default::default()
            },
        ).await;
        
        let transaction_id = match payment {
            Ok(pay) => pay.transaction_id,
            Err(e) => {
                tracing::error!(error = ?e, "Payment failed, releasing inventory");
                
                // 补偿：释放库存
                let _ = ctx.execute_activity::<ReleaseInventoryActivity>(
                    ReleaseInventoryInput { reservation_id },
                    ActivityOptions::default(),
                ).await;
                
                return Ok(OrderOrchestrationOutput {
                    order_id: input.order_id,
                    status: "payment_failed".to_string(),
                    transaction_id: None,
                    shipment_id: None,
                });
            }
        };
        
        // 4. 创建发货单
        let shipment = ctx.execute_activity::<CreateShipmentActivity>(
            ShipmentInput {
                order_id: input.order_id.clone(),
                items: input.items.clone(),
                address: input.shipping_address.clone(),
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(30)),
                ..Default::default()
            },
        ).await;
        
        let shipment_id = match shipment {
            Ok(ship) => ship.shipment_id,
            Err(e) => {
                tracing::error!(error = ?e, "Shipment creation failed, compensating");
                
                // 补偿：退款
                let _ = ctx.execute_activity::<RefundPaymentActivity>(
                    RefundInput {
                        transaction_id: transaction_id.clone(),
                    },
                    ActivityOptions::default(),
                ).await;
                
                // 补偿：释放库存
                let _ = ctx.execute_activity::<ReleaseInventoryActivity>(
                    ReleaseInventoryInput { reservation_id },
                    ActivityOptions::default(),
                ).await;
                
                return Ok(OrderOrchestrationOutput {
                    order_id: input.order_id,
                    status: "shipment_failed".to_string(),
                    transaction_id: Some(transaction_id),
                    shipment_id: None,
                });
            }
        };
        
        // 5. 发送通知
        let _ = ctx.execute_activity::<SendNotificationActivity>(
            NotificationInput {
                user_id: input.user_id,
                order_id: input.order_id.clone(),
                message: format!("Order {} confirmed", input.order_id),
            },
            ActivityOptions::default(),
        ).await;
        
        Ok(OrderOrchestrationOutput {
            order_id: input.order_id,
            status: "completed".to_string(),
            transaction_id: Some(transaction_id),
            shipment_id: Some(shipment_id),
        })
    }
}

// Activity定义省略（参考之前的示例）
```

#### Golang对比

```go
package workflows

import (
    "go.temporal.io/sdk/workflow"
)

func OrderOrchestrationWorkflow(ctx workflow.Context, input OrderOrchestrationInput) (OrderOrchestrationOutput, error) {
    logger := workflow.GetLogger(ctx)
    logger.Info("Starting order orchestration", "order_id", input.OrderID)
    
    // 1. 验证订单
    var validation ValidationResult
    err := workflow.ExecuteActivity(ctx, ValidateOrderActivity, input).Get(ctx, &validation)
    if err != nil || !validation.Valid {
        return OrderOrchestrationOutput{
            OrderID: input.OrderID,
            Status:  "validation_failed",
        }, nil
    }
    
    // 2. 预留库存
    var reservationID string
    err = workflow.ExecuteActivity(ctx, ReserveInventoryActivity, input).Get(ctx, &reservationID)
    if err != nil {
        return OrderOrchestrationOutput{
            OrderID: input.OrderID,
            Status:  "out_of_stock",
        }, nil
    }
    
    // 3. 处理支付
    var transactionID string
    err = workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input).Get(ctx, &transactionID)
    if err != nil {
        // 补偿：释放库存
        workflow.ExecuteActivity(ctx, ReleaseInventoryActivity, reservationID).Get(ctx, nil)
        
        return OrderOrchestrationOutput{
            OrderID: input.OrderID,
            Status:  "payment_failed",
        }, nil
    }
    
    // 4. 创建发货单
    var shipmentID string
    err = workflow.ExecuteActivity(ctx, CreateShipmentActivity, input).Get(ctx, &shipmentID)
    if err != nil {
        // 补偿：退款 + 释放库存
        workflow.ExecuteActivity(ctx, RefundPaymentActivity, transactionID).Get(ctx, nil)
        workflow.ExecuteActivity(ctx, ReleaseInventoryActivity, reservationID).Get(ctx, nil)
        
        return OrderOrchestrationOutput{
            OrderID:       input.OrderID,
            Status:        "shipment_failed",
            TransactionID: &transactionID,
        }, nil
    }
    
    // 5. 发送通知
    workflow.ExecuteActivity(ctx, SendNotificationActivity, input).Get(ctx, nil)
    
    return OrderOrchestrationOutput{
        OrderID:       input.OrderID,
        Status:        "completed",
        TransactionID: &transactionID,
        ShipmentID:    &shipmentID,
    }, nil
}
```

---

## 🔄 服务降级示例

### 场景：支付服务降级

当支付服务不可用时，使用备用支付渠道。

#### Rust实现

```rust
pub struct PaymentWithFallbackWorkflow;

impl Workflow for PaymentWithFallbackWorkflow {
    type Input = PaymentInput;
    type Output = PaymentOutput;
    
    fn name() -> &'static str {
        "PaymentWithFallback"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        // 尝试主支付渠道
        let primary_result = ctx.execute_activity::<PrimaryPaymentActivity>(
            input.clone(),
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(30)),
                retry_policy: Some(RetryPolicy {
                    max_attempts: Some(2),
                    initial_interval: Duration::from_secs(1),
                    max_interval: Duration::from_secs(5),
                    backoff_coefficient: 2.0,
                    non_retryable_error_types: vec![],
                }),
                ..Default::default()
            },
        ).await;
        
        match primary_result {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::warn!(
                    error = ?e,
                    "Primary payment failed, trying fallback"
                );
                
                // 尝试备用支付渠道
                ctx.execute_activity::<FallbackPaymentActivity>(
                    input,
                    ActivityOptions {
                        start_to_close_timeout: Some(Duration::from_secs(30)),
                        ..Default::default()
                    },
                ).await
            }
        }
    }
}
```

---

## 🔁 熔断器模式

### 场景：调用外部服务时的熔断保护

#### Rust实现

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CircuitBreakerState {
    pub failures: u32,
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
    pub state: CircuitState,
}

#[derive(Clone, PartialEq)]
pub enum CircuitState {
    Closed,    // 正常状态
    Open,      // 熔断状态
    HalfOpen,  // 半开状态
}

pub struct CircuitBreakerWorkflow {
    circuit_state: Arc<RwLock<CircuitBreakerState>>,
}

impl Workflow for CircuitBreakerWorkflow {
    type Input = ServiceCallInput;
    type Output = ServiceCallOutput;
    
    fn name() -> &'static str {
        "CircuitBreaker"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let circuit_state = Arc::new(RwLock::new(CircuitBreakerState {
            failures: 0,
            last_failure_time: None,
            state: CircuitState::Closed,
        }));
        
        const MAX_FAILURES: u32 = 3;
        const RESET_TIMEOUT: Duration = Duration::from_secs(60);
        
        loop {
            let state = circuit_state.read().await.state.clone();
            
            match state {
                CircuitState::Closed => {
                    // 正常调用服务
                    match ctx.execute_activity::<CallExternalServiceActivity>(
                        input.clone(),
                        ActivityOptions::default(),
                    ).await {
                        Ok(result) => {
                            // 成功，重置失败计数
                            circuit_state.write().await.failures = 0;
                            return Ok(result);
                        }
                        Err(e) => {
                            let mut state = circuit_state.write().await;
                            state.failures += 1;
                            state.last_failure_time = Some(ctx.now());
                            
                            if state.failures >= MAX_FAILURES {
                                state.state = CircuitState::Open;
                                tracing::warn!("Circuit breaker opened");
                            }
                            
                            return Err(e);
                        }
                    }
                }
                
                CircuitState::Open => {
                    // 检查是否应该进入半开状态
                    let last_failure = circuit_state.read().await.last_failure_time;
                    
                    if let Some(last_time) = last_failure {
                        let elapsed = ctx.now().signed_duration_since(last_time);
                        
                        if elapsed > RESET_TIMEOUT {
                            circuit_state.write().await.state = CircuitState::HalfOpen;
                            tracing::info!("Circuit breaker half-open");
                            continue;
                        }
                    }
                    
                    // 熔断状态，直接返回错误
                    return Err(WorkflowError::internal("Circuit breaker is open"));
                }
                
                CircuitState::HalfOpen => {
                    // 尝试一次调用
                    match ctx.execute_activity::<CallExternalServiceActivity>(
                        input.clone(),
                        ActivityOptions::default(),
                    ).await {
                        Ok(result) => {
                            // 成功，关闭熔断器
                            let mut state = circuit_state.write().await;
                            state.state = CircuitState::Closed;
                            state.failures = 0;
                            tracing::info!("Circuit breaker closed");
                            return Ok(result);
                        }
                        Err(e) => {
                            // 失败，重新打开熔断器
                            let mut state = circuit_state.write().await;
                            state.state = CircuitState::Open;
                            state.last_failure_time = Some(ctx.now());
                            tracing::warn!("Circuit breaker re-opened");
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
}
```

---

## 📚 总结

### 微服务编排优势

1. **统一协调**: 中心化的流程控制
2. **容错性**: 自动重试和补偿
3. **可观测性**: 完整的执行历史
4. **灵活性**: 易于修改和扩展

### Saga模式 vs 两阶段提交

| 特性 | Saga模式 | 两阶段提交 |
|------|----------|------------|
| **性能** | 高 | 低 |
| **一致性** | 最终一致性 | 强一致性 |
| **可用性** | 高 | 低 |
| **复杂度** | 中等 | 高 |

---

## 📚 下一步

- **定时任务**: [调度管理](./23_scheduled_tasks.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

