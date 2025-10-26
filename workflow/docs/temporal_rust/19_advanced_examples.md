# 高级示例

## 📋 文档概述

本文档提供Temporal-Rust的高级示例，包括：

- 子工作流（Child Workflow）
- 复杂业务流程
- 高级错误处理
- 动态工作流
- 工作流版本管理
- Rust + Golang并列对比

---

## 🔄 子工作流示例

### 场景：订单处理系统

父工作流负责整体订单流程，子工作流处理独立的子任务。

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// ========================================
// 子工作流：支付处理
// ========================================

#[derive(Serialize, Deserialize)]
pub struct PaymentInput {
    pub order_id: String,
    pub amount: f64,
    pub payment_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct PaymentOutput {
    pub transaction_id: String,
    pub status: String,
}

pub struct PaymentWorkflow;

impl Workflow for PaymentWorkflow {
    type Input = PaymentInput;
    type Output = PaymentOutput;
    
    fn name() -> &'static str {
        "PaymentWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            order_id = %input.order_id,
            amount = input.amount,
            "Starting payment workflow"
        );
        
        // 1. 验证支付方法
        ctx.execute_activity::<ValidatePaymentMethodActivity>(
            ValidatePaymentInput {
                payment_method: input.payment_method.clone(),
            },
            ActivityOptions::default(),
        ).await?;
        
        // 2. 处理支付
        let payment = ctx.execute_activity::<ProcessPaymentActivity>(
            ProcessPaymentInput {
                order_id: input.order_id.clone(),
                amount: input.amount,
                payment_method: input.payment_method,
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(30)),
                retry_policy: Some(RetryPolicy {
                    max_attempts: Some(3),
                    initial_interval: Duration::from_secs(1),
                    max_interval: Duration::from_secs(10),
                    backoff_coefficient: 2.0,
                    non_retryable_error_types: vec!["InvalidPaymentMethod"],
                }),
                ..Default::default()
            },
        ).await?;
        
        // 3. 等待支付确认
        ctx.sleep(Duration::from_secs(5)).await;
        
        Ok(PaymentOutput {
            transaction_id: payment.transaction_id,
            status: "completed".to_string(),
        })
    }
}

// ========================================
// 子工作流：发货处理
// ========================================

#[derive(Serialize, Deserialize)]
pub struct ShipmentInput {
    pub order_id: String,
    pub address: String,
    pub items: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ShipmentOutput {
    pub tracking_number: String,
    pub carrier: String,
}

pub struct ShipmentWorkflow;

impl Workflow for ShipmentWorkflow {
    type Input = ShipmentInput;
    type Output = ShipmentOutput;
    
    fn name() -> &'static str {
        "ShipmentWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            order_id = %input.order_id,
            "Starting shipment workflow"
        );
        
        // 1. 选择承运商
        let carrier = ctx.execute_activity::<SelectCarrierActivity>(
            SelectCarrierInput {
                address: input.address.clone(),
            },
            ActivityOptions::default(),
        ).await?;
        
        // 2. 创建发货单
        let shipment = ctx.execute_activity::<CreateShipmentActivity>(
            CreateShipmentInput {
                order_id: input.order_id,
                carrier: carrier.carrier_name.clone(),
                items: input.items,
            },
            ActivityOptions::default(),
        ).await?;
        
        Ok(ShipmentOutput {
            tracking_number: shipment.tracking_number,
            carrier: carrier.carrier_name,
        })
    }
}

// ========================================
// 父工作流：订单处理
// ========================================

#[derive(Serialize, Deserialize)]
pub struct OrderInput {
    pub order_id: String,
    pub customer_id: String,
    pub items: Vec<String>,
    pub total_amount: f64,
    pub shipping_address: String,
    pub payment_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct OrderOutput {
    pub order_id: String,
    pub status: String,
    pub transaction_id: String,
    pub tracking_number: String,
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
            "Starting order processing workflow"
        );
        
        // 1. 启动子工作流：支付处理
        let payment_result = ctx.execute_child_workflow::<PaymentWorkflow>(
            PaymentInput {
                order_id: input.order_id.clone(),
                amount: input.total_amount,
                payment_method: input.payment_method,
            },
            ChildWorkflowOptions {
                workflow_id: Some(WorkflowId::new(
                    format!("payment-{}", input.order_id)
                )),
                ..Default::default()
            },
        ).await?;
        
        // 2. 启动子工作流：发货处理
        let shipment_result = ctx.execute_child_workflow::<ShipmentWorkflow>(
            ShipmentInput {
                order_id: input.order_id.clone(),
                address: input.shipping_address,
                items: input.items,
            },
            ChildWorkflowOptions {
                workflow_id: Some(WorkflowId::new(
                    format!("shipment-{}", input.order_id)
                )),
                ..Default::default()
            },
        ).await?;
        
        // 3. 发送通知
        ctx.execute_activity::<SendNotificationActivity>(
            NotificationInput {
                customer_id: input.customer_id,
                order_id: input.order_id.clone(),
                tracking_number: shipment_result.tracking_number.clone(),
            },
            ActivityOptions::default(),
        ).await?;
        
        Ok(OrderOutput {
            order_id: input.order_id,
            status: "completed".to_string(),
            transaction_id: payment_result.transaction_id,
            tracking_number: shipment_result.tracking_number,
        })
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

// 子工作流：支付处理
func PaymentWorkflow(ctx workflow.Context, input PaymentInput) (PaymentOutput, error) {
    logger := workflow.GetLogger(ctx)
    logger.Info("Starting payment workflow", "order_id", input.OrderID)
    
    // 验证支付方法
    err := workflow.ExecuteActivity(ctx, ValidatePaymentMethod, input.PaymentMethod).Get(ctx, nil)
    if err != nil {
        return PaymentOutput{}, err
    }
    
    // 处理支付
    var transactionID string
    err = workflow.ExecuteActivity(ctx, ProcessPayment, input).Get(ctx, &transactionID)
    if err != nil {
        return PaymentOutput{}, err
    }
    
    // 等待确认
    workflow.Sleep(ctx, 5*time.Second)
    
    return PaymentOutput{
        TransactionID: transactionID,
        Status:        "completed",
    }, nil
}

// 子工作流：发货处理
func ShipmentWorkflow(ctx workflow.Context, input ShipmentInput) (ShipmentOutput, error) {
    logger := workflow.GetLogger(ctx)
    logger.Info("Starting shipment workflow", "order_id", input.OrderID)
    
    // 选择承运商
    var carrier string
    err := workflow.ExecuteActivity(ctx, SelectCarrier, input.Address).Get(ctx, &carrier)
    if err != nil {
        return ShipmentOutput{}, err
    }
    
    // 创建发货单
    var trackingNumber string
    err = workflow.ExecuteActivity(ctx, CreateShipment, input).Get(ctx, &trackingNumber)
    if err != nil {
        return ShipmentOutput{}, err
    }
    
    return ShipmentOutput{
        TrackingNumber: trackingNumber,
        Carrier:        carrier,
    }, nil
}

// 父工作流：订单处理
func OrderProcessingWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    logger := workflow.GetLogger(ctx)
    logger.Info("Starting order processing workflow", "order_id", input.OrderID)
    
    // 启动子工作流：支付处理
    paymentCtx := workflow.WithChildOptions(ctx, workflow.ChildWorkflowOptions{
        WorkflowID: "payment-" + input.OrderID,
    })
    
    var paymentResult PaymentOutput
    err := workflow.ExecuteChildWorkflow(paymentCtx, PaymentWorkflow, PaymentInput{
        OrderID:       input.OrderID,
        Amount:        input.TotalAmount,
        PaymentMethod: input.PaymentMethod,
    }).Get(ctx, &paymentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 启动子工作流：发货处理
    shipmentCtx := workflow.WithChildOptions(ctx, workflow.ChildWorkflowOptions{
        WorkflowID: "shipment-" + input.OrderID,
    })
    
    var shipmentResult ShipmentOutput
    err = workflow.ExecuteChildWorkflow(shipmentCtx, ShipmentWorkflow, ShipmentInput{
        OrderID: input.OrderID,
        Address: input.ShippingAddress,
        Items:   input.Items,
    }).Get(ctx, &shipmentResult)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 发送通知
    err = workflow.ExecuteActivity(ctx, SendNotification, input.CustomerID, input.OrderID).Get(ctx, nil)
    if err != nil {
        return OrderOutput{}, err
    }
    
    return OrderOutput{
        OrderID:        input.OrderID,
        Status:         "completed",
        TransactionID:  paymentResult.TransactionID,
        TrackingNumber: shipmentResult.TrackingNumber,
    }, nil
}
```

---

## 🔀 动态工作流示例

### 场景：动态审批流程

根据订单金额动态决定审批流程。

#### Rust实现1

```rust
use temporal_rust::*;

#[derive(Serialize, Deserialize)]
pub struct ApprovalInput {
    pub order_id: String,
    pub amount: f64,
    pub requester: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApprovalOutput {
    pub approved: bool,
    pub approvers: Vec<String>,
}

pub struct DynamicApprovalWorkflow;

impl Workflow for DynamicApprovalWorkflow {
    type Input = ApprovalInput;
    type Output = ApprovalOutput;
    
    fn name() -> &'static str {
        "DynamicApproval"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let mut approvers = Vec::new();
        
        // 根据金额决定审批流程
        let approval_levels = if input.amount < 1000.0 {
            vec!["manager"]
        } else if input.amount < 10000.0 {
            vec!["manager", "director"]
        } else {
            vec!["manager", "director", "ceo"]
        };
        
        tracing::info!(
            order_id = %input.order_id,
            amount = input.amount,
            levels = approval_levels.len(),
            "Starting dynamic approval workflow"
        );
        
        // 逐级审批
        for level in approval_levels {
            // 发送审批请求
            ctx.execute_activity::<SendApprovalRequestActivity>(
                ApprovalRequestInput {
                    order_id: input.order_id.clone(),
                    level: level.to_string(),
                },
                ActivityOptions::default(),
            ).await?;
            
            // 等待审批Signal
            let approval = ctx.wait_for_signal::<ApprovalSignal>(
                Duration::from_secs(3600) // 1小时超时
            ).await?;
            
            if !approval.approved {
                return Ok(ApprovalOutput {
                    approved: false,
                    approvers,
                });
            }
            
            approvers.push(approval.approver);
        }
        
        Ok(ApprovalOutput {
            approved: true,
            approvers,
        })
    }
}

// 审批Signal
#[derive(Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
    pub approver: String,
    pub comment: String,
}

impl Signal for ApprovalSignal {
    fn name() -> &'static str {
        "approval"
    }
}
```

#### Golang对比1

```go
func DynamicApprovalWorkflow(ctx workflow.Context, input ApprovalInput) (ApprovalOutput, error) {
    var approvers []string
    
    // 根据金额决定审批流程
    var approvalLevels []string
    if input.Amount < 1000 {
        approvalLevels = []string{"manager"}
    } else if input.Amount < 10000 {
        approvalLevels = []string{"manager", "director"}
    } else {
        approvalLevels = []string{"manager", "director", "ceo"}
    }
    
    // 逐级审批
    for _, level := range approvalLevels {
        // 发送审批请求
        err := workflow.ExecuteActivity(ctx, SendApprovalRequest, input.OrderID, level).Get(ctx, nil)
        if err != nil {
            return ApprovalOutput{}, err
        }
        
        // 等待审批Signal
        var approval ApprovalSignal
        signalCh := workflow.GetSignalChannel(ctx, "approval")
        
        selector := workflow.NewSelector(ctx)
        selector.AddReceive(signalCh, func(c workflow.ReceiveChannel, more bool) {
            c.Receive(ctx, &approval)
        })
        selector.AddFuture(workflow.NewTimer(ctx, time.Hour), func(f workflow.Future) {
            // 超时处理
        })
        selector.Select(ctx)
        
        if !approval.Approved {
            return ApprovalOutput{
                Approved:  false,
                Approvers: approvers,
            }, nil
        }
        
        approvers = append(approvers, approval.Approver)
    }
    
    return ApprovalOutput{
        Approved:  true,
        Approvers: approvers,
    }, nil
}
```

---

## 📊 复杂状态机示例

### 场景：保险理赔流程

涉及多个状态和条件转换。

#### Rust实现2

```rust
use temporal_rust::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimStatus {
    Submitted,
    UnderReview,
    RequiresDocuments,
    Approved,
    Rejected,
    Paid,
}

#[derive(Serialize, Deserialize)]
pub struct ClaimInput {
    pub claim_id: String,
    pub policy_id: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClaimOutput {
    pub claim_id: String,
    pub status: ClaimStatus,
    pub payout_amount: Option<f64>,
}

pub struct InsuranceClaimWorkflow;

impl Workflow for InsuranceClaimWorkflow {
    type Input = ClaimInput;
    type Output = ClaimOutput;
    
    fn name() -> &'static str {
        "InsuranceClaim"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let mut status = ClaimStatus::Submitted;
        let mut payout_amount: Option<f64> = None;
        
        tracing::info!(
            claim_id = %input.claim_id,
            "Starting insurance claim workflow"
        );
        
        // 状态机循环
        loop {
            match status {
                ClaimStatus::Submitted => {
                    // 初步审核
                    let review = ctx.execute_activity::<InitialReviewActivity>(
                        ReviewInput {
                            claim_id: input.claim_id.clone(),
                            policy_id: input.policy_id.clone(),
                        },
                        ActivityOptions::default(),
                    ).await?;
                    
                    status = if review.valid {
                        ClaimStatus::UnderReview
                    } else {
                        ClaimStatus::Rejected
                    };
                }
                
                ClaimStatus::UnderReview => {
                    // 详细审核
                    let assessment = ctx.execute_activity::<DetailedAssessmentActivity>(
                        AssessmentInput {
                            claim_id: input.claim_id.clone(),
                            amount: input.amount,
                        },
                        ActivityOptions::default(),
                    ).await?;
                    
                    if assessment.requires_documents {
                        status = ClaimStatus::RequiresDocuments;
                    } else if assessment.approved {
                        status = ClaimStatus::Approved;
                        payout_amount = Some(assessment.payout_amount);
                    } else {
                        status = ClaimStatus::Rejected;
                    }
                }
                
                ClaimStatus::RequiresDocuments => {
                    // 等待文档上传（Signal）
                    ctx.execute_activity::<RequestDocumentsActivity>(
                        input.claim_id.clone(),
                        ActivityOptions::default(),
                    ).await?;
                    
                    let documents = ctx.wait_for_signal::<DocumentsSubmittedSignal>(
                        Duration::from_secs(7 * 24 * 3600) // 7天
                    ).await?;
                    
                    // 验证文档
                    let validation = ctx.execute_activity::<ValidateDocumentsActivity>(
                        documents,
                        ActivityOptions::default(),
                    ).await?;
                    
                    status = if validation.valid {
                        ClaimStatus::UnderReview
                    } else {
                        ClaimStatus::Rejected
                    };
                }
                
                ClaimStatus::Approved => {
                    // 处理付款
                    ctx.execute_activity::<ProcessPayoutActivity>(
                        PayoutInput {
                            claim_id: input.claim_id.clone(),
                            amount: payout_amount.unwrap(),
                        },
                        ActivityOptions::default(),
                    ).await?;
                    
                    status = ClaimStatus::Paid;
                }
                
                ClaimStatus::Rejected | ClaimStatus::Paid => {
                    // 终态
                    break;
                }
            }
        }
        
        Ok(ClaimOutput {
            claim_id: input.claim_id,
            status,
            payout_amount,
        })
    }
}
```

---

## 🔄 工作流版本管理示例

### 场景：升级工作流逻辑

在不影响运行中工作流的情况下升级逻辑。

#### Rust实现3

```rust
use temporal_rust::*;

pub struct OrderWorkflowV1;

impl Workflow for OrderWorkflowV1 {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderWorkflow"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let version = ctx.get_version("order-workflow", 1, 2).await;
        
        match version {
            1 => {
                // V1逻辑：简单流程
                let payment = ctx.execute_activity::<ProcessPaymentV1Activity>(
                    input.clone(),
                    ActivityOptions::default(),
                ).await?;
                
                Ok(OrderOutput {
                    order_id: input.order_id,
                    status: "completed".to_string(),
                })
            }
            2 => {
                // V2逻辑：增强流程
                // 1. 风险检查（新增）
                let risk_check = ctx.execute_activity::<RiskCheckActivity>(
                    RiskInput {
                        order_id: input.order_id.clone(),
                        amount: input.amount,
                    },
                    ActivityOptions::default(),
                ).await?;
                
                if !risk_check.passed {
                    return Err(WorkflowError::business("Risk check failed"));
                }
                
                // 2. 处理支付
                let payment = ctx.execute_activity::<ProcessPaymentV2Activity>(
                    input.clone(),
                    ActivityOptions::default(),
                ).await?;
                
                // 3. 发送通知（新增）
                ctx.execute_activity::<SendNotificationActivity>(
                    NotificationInput {
                        order_id: input.order_id.clone(),
                    },
                    ActivityOptions::default(),
                ).await?;
                
                Ok(OrderOutput {
                    order_id: input.order_id,
                    status: "completed".to_string(),
                })
            }
            _ => Err(WorkflowError::internal("Unsupported version")),
        }
    }
}
```

---

## 📚 总结

### 高级模式

1. **子工作流**: 模块化、可复用、独立管理
2. **动态工作流**: 根据输入动态决定执行路径
3. **状态机**: 复杂业务流程的标准实现
4. **版本管理**: 平滑升级运行中的工作流

### Rust优势

- ✅ 类型安全的状态机
- ✅ 编译时检查版本兼容性
- ✅ 零成本的子工作流抽象

---

## 📚 下一步

- **数据管道**: [ETL处理](./20_data_pipeline.md)
- **批量任务**: [并行处理](./21_batch_processing.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
