# 电商订单处理 - Golang实现对比

## 📋 概述

本文档展示相同的电商订单处理工作流在Golang（使用Temporal Go SDK）中的实现，用于对比学习。

---

## 🐹 Golang实现

### 数据模型

```go
package workflows

import (
    "time"
)

// 订单信息
type Order struct {
    OrderID         string         `json:"order_id"`
    UserID          string         `json:"user_id"`
    Items           []OrderItem    `json:"items"`
    TotalAmount     float64        `json:"total_amount"`
    PaymentMethod   PaymentMethod  `json:"payment_method"`
    ShippingAddress Address        `json:"shipping_address"`
}

type OrderItem struct {
    ProductID   string  `json:"product_id"`
    ProductName string  `json:"product_name"`
    Quantity    int     `json:"quantity"`
    UnitPrice   float64 `json:"unit_price"`
}

type PaymentMethod struct {
    Type  string `json:"type"`  // "credit_card", "paypal", "alipay"
    Token string `json:"token"`
}

type Address struct {
    Recipient  string `json:"recipient"`
    Phone      string `json:"phone"`
    Street     string `json:"street"`
    City       string `json:"city"`
    Province   string `json:"province"`
    PostalCode string `json:"postal_code"`
    Country    string `json:"country"`
}

// 订单状态
type OrderStatus string

const (
    OrderStatusPending           OrderStatus = "PENDING"
    OrderStatusPaymentProcessing OrderStatus = "PAYMENT_PROCESSING"
    OrderStatusPaymentCompleted  OrderStatus = "PAYMENT_COMPLETED"
    OrderStatusInventoryReserved OrderStatus = "INVENTORY_RESERVED"
    OrderStatusShipping          OrderStatus = "SHIPPING"
    OrderStatusCompleted         OrderStatus = "COMPLETED"
    OrderStatusCancelled         OrderStatus = "CANCELLED"
    OrderStatusFailed            OrderStatus = "FAILED"
)

// 订单处理结果
type OrderResult struct {
    OrderID        string       `json:"order_id"`
    Status         OrderStatus  `json:"status"`
    PaymentID      *string      `json:"payment_id,omitempty"`
    TrackingNumber *string      `json:"tracking_number,omitempty"`
    CompletedAt    *time.Time   `json:"completed_at,omitempty"`
    FailureReason  string       `json:"failure_reason,omitempty"`
}
```

### Activity定义

```go
package activities

import (
    "context"
    "fmt"
    "time"

    "go.temporal.io/sdk/activity"
    "github.com/google/uuid"
)

// ValidateOrderActivity 验证订单
type ValidateOrderInput struct {
    Order Order
}

type ValidateOrderOutput struct {
    IsValid          bool     `json:"is_valid"`
    ValidationErrors []string `json:"validation_errors"`
}

func ValidateOrder(ctx context.Context, input ValidateOrderInput) (ValidateOrderOutput, error) {
    logger := activity.GetLogger(ctx)
    logger.Info("Validating order", "order_id", input.Order.OrderID)

    var errors []string

    // 验证订单项
    if len(input.Order.Items) == 0 {
        errors = append(errors, "Order must contain at least one item")
    }

    // 验证金额
    if input.Order.TotalAmount <= 0 {
        errors = append(errors, "Order total must be positive")
    }

    // 验证地址
    if input.Order.ShippingAddress.Recipient == "" {
        errors = append(errors, "Recipient name is required")
    }

    return ValidateOrderOutput{
        IsValid:          len(errors) == 0,
        ValidationErrors: errors,
    }, nil
}

// ReserveInventoryActivity 预留库存
type ReserveInventoryInput struct {
    OrderID string
    Items   []OrderItem
}

type ReserveInventoryOutput struct {
    ReservationID string   `json:"reservation_id"`
    ReservedItems []string `json:"reserved_items"`
}

func ReserveInventory(ctx context.Context, input ReserveInventoryInput) (ReserveInventoryOutput, error) {
    logger := activity.GetLogger(ctx)
    logger.Info("Reserving inventory", "order_id", input.OrderID)

    // 模拟库存检查
    time.Sleep(500 * time.Millisecond)

    // 发送心跳
    activity.RecordHeartbeat(ctx, "checking_inventory")

    // 检查每个商品的库存
    var reservedItems []string
    for _, item := range input.Items {
        logger.Debug("Checking inventory",
            "product_id", item.ProductID,
            "quantity", item.Quantity)
        reservedItems = append(reservedItems, item.ProductID)
    }

    reservationID := fmt.Sprintf("RES-%s", uuid.New().String())

    return ReserveInventoryOutput{
        ReservationID: reservationID,
        ReservedItems: reservedItems,
    }, nil
}

// ProcessPaymentActivity 处理支付
type ProcessPaymentInput struct {
    OrderID       string
    Amount        float64
    PaymentMethod PaymentMethod
}

type ProcessPaymentOutput struct {
    PaymentID     string `json:"payment_id"`
    TransactionID string `json:"transaction_id"`
    Status        string `json:"status"`
}

func ProcessPayment(ctx context.Context, input ProcessPaymentInput) (ProcessPaymentOutput, error) {
    logger := activity.GetLogger(ctx)
    logger.Info("Processing payment",
        "order_id", input.OrderID,
        "amount", input.Amount)

    // 模拟支付处理
    time.Sleep(2 * time.Second)

    // 发送心跳
    activity.RecordHeartbeat(ctx, "contacting_payment_gateway")

    paymentID := fmt.Sprintf("PAY-%s", uuid.New().String())
    transactionID := fmt.Sprintf("TXN-%s", uuid.New().String())

    return ProcessPaymentOutput{
        PaymentID:     paymentID,
        TransactionID: transactionID,
        Status:        "SUCCESS",
    }, nil
}

// CreateShipmentActivity 创建发货单
type CreateShipmentInput struct {
    OrderID string
    Items   []OrderItem
    Address Address
}

type CreateShipmentOutput struct {
    ShipmentID        string    `json:"shipment_id"`
    TrackingNumber    string    `json:"tracking_number"`
    EstimatedDelivery time.Time `json:"estimated_delivery"`
}

func CreateShipment(ctx context.Context, input CreateShipmentInput) (CreateShipmentOutput, error) {
    logger := activity.GetLogger(ctx)
    logger.Info("Creating shipment", "order_id", input.OrderID)

    // 模拟物流系统调用
    time.Sleep(300 * time.Millisecond)

    shipmentID := fmt.Sprintf("SHIP-%s", uuid.New().String())
    trackingNumber := fmt.Sprintf("TRK-%s", uuid.New().String())
    estimatedDelivery := time.Now().Add(72 * time.Hour)

    return CreateShipmentOutput{
        ShipmentID:        shipmentID,
        TrackingNumber:    trackingNumber,
        EstimatedDelivery: estimatedDelivery,
    }, nil
}

// SendNotificationActivity 发送通知
type SendNotificationInput struct {
    UserID           string
    NotificationType string
    Message          string
}

func SendNotification(ctx context.Context, input SendNotificationInput) error {
    logger := activity.GetLogger(ctx)
    logger.Info("Sending notification",
        "type", input.NotificationType,
        "user_id", input.UserID)

    fmt.Printf("📧 Notification: %s\n", input.Message)

    return nil
}

// 补偿Activities

func ReleaseInventory(ctx context.Context, reservationID string) error {
    logger := activity.GetLogger(ctx)
    logger.Info("Releasing inventory reservation", "reservation_id", reservationID)

    time.Sleep(200 * time.Millisecond)

    return nil
}

func RefundPayment(ctx context.Context, paymentID string, amount float64) error {
    logger := activity.GetLogger(ctx)
    logger.Info("Refunding payment",
        "payment_id", paymentID,
        "amount", amount)

    time.Sleep(1 * time.Second)

    return nil
}
```

### Workflow定义

```go
package workflows

import (
    "fmt"
    "time"

    "go.temporal.io/sdk/temporal"
    "go.temporal.io/sdk/workflow"
)

// OrderProcessingWorkflow 订单处理工作流
func OrderProcessingWorkflow(ctx workflow.Context, order Order) (OrderResult, error) {
    logger := workflow.GetLogger(ctx)
    logger.Info("Starting order processing workflow", "order_id", order.OrderID)

    result := OrderResult{
        OrderID: order.OrderID,
        Status:  OrderStatusPending,
    }

    // Activity选项
    activityOptions := workflow.ActivityOptions{
        StartToCloseTimeout: 30 * time.Second,
        RetryPolicy: &temporal.RetryPolicy{
            MaximumAttempts: 3,
        },
    }
    ctx = workflow.WithActivityOptions(ctx, activityOptions)

    // 1. 验证订单
    logger.Info("Step 1: Validating order")
    var validation ValidateOrderOutput
    err := workflow.ExecuteActivity(ctx, ValidateOrder, ValidateOrderInput{
        Order: order,
    }).Get(ctx, &validation)

    if err != nil {
        result.Status = OrderStatusFailed
        result.FailureReason = err.Error()
        return result, nil
    }

    if !validation.IsValid {
        result.Status = OrderStatusFailed
        result.FailureReason = fmt.Sprintf("Validation failed: %v", validation.ValidationErrors)
        return result, nil
    }

    // 2. 预留库存
    logger.Info("Step 2: Reserving inventory")
    var reservation ReserveInventoryOutput
    err = workflow.ExecuteActivity(ctx, ReserveInventory, ReserveInventoryInput{
        OrderID: order.OrderID,
        Items:   order.Items,
    }).Get(ctx, &reservation)

    if err != nil {
        result.Status = OrderStatusFailed
        result.FailureReason = fmt.Sprintf("Inventory reservation failed: %v", err)
        return result, nil
    }

    result.Status = OrderStatusInventoryReserved

    // 3. 处理支付
    logger.Info("Step 3: Processing payment")
    var payment ProcessPaymentOutput
    err = workflow.ExecuteActivity(ctx, ProcessPayment, ProcessPaymentInput{
        OrderID:       order.OrderID,
        Amount:        order.TotalAmount,
        PaymentMethod: order.PaymentMethod,
    }).Get(ctx, &payment)

    if err != nil {
        // 支付失败，补偿：释放库存
        logger.Warn("Payment failed, initiating compensation")

        _ = workflow.ExecuteActivity(ctx, ReleaseInventory, reservation.ReservationID).Get(ctx, nil)

        result.Status = OrderStatusFailed
        result.FailureReason = fmt.Sprintf("Payment failed: %v", err)
        return result, nil
    }

    result.PaymentID = &payment.PaymentID
    result.Status = OrderStatusPaymentCompleted

    // 4. 创建发货单
    logger.Info("Step 4: Creating shipment")
    var shipment CreateShipmentOutput
    err = workflow.ExecuteActivity(ctx, CreateShipment, CreateShipmentInput{
        OrderID: order.OrderID,
        Items:   order.Items,
        Address: order.ShippingAddress,
    }).Get(ctx, &shipment)

    if err != nil {
        // 发货失败，补偿：退款 + 释放库存
        logger.Warn("Shipment creation failed, initiating compensation")

        _ = workflow.ExecuteActivity(ctx, RefundPayment, payment.PaymentID, order.TotalAmount).Get(ctx, nil)
        _ = workflow.ExecuteActivity(ctx, ReleaseInventory, reservation.ReservationID).Get(ctx, nil)

        result.Status = OrderStatusFailed
        result.FailureReason = fmt.Sprintf("Shipment creation failed: %v", err)
        return result, nil
    }

    result.TrackingNumber = &shipment.TrackingNumber
    result.Status = OrderStatusShipping

    // 5. 发送通知
    logger.Info("Step 5: Sending notification")
    _ = workflow.ExecuteActivity(ctx, SendNotification, SendNotificationInput{
        UserID:           order.UserID,
        NotificationType: "ORDER_SHIPPED",
        Message: fmt.Sprintf(
            "Your order %s has been shipped. Tracking: %s",
            order.OrderID,
            shipment.TrackingNumber,
        ),
    }).Get(ctx, nil)

    // 6. 完成订单
    now := time.Now()
    result.Status = OrderStatusCompleted
    result.CompletedAt = &now

    logger.Info("Order processing completed successfully", "order_id", order.OrderID)
    return result, nil
}
```

### Worker和客户端

```go
package main

import (
    "context"
    "log"

    "go.temporal.io/sdk/client"
    "go.temporal.io/sdk/worker"
    "github.com/google/uuid"
)

func main() {
    // 创建客户端
    c, err := client.Dial(client.Options{})
    if err != nil {
        log.Fatal(err)
    }
    defer c.Close()

    // 启动Worker（在goroutine中）
    go func() {
        w := worker.New(c, "order-processing", worker.Options{})

        // 注册Workflow
        w.RegisterWorkflow(OrderProcessingWorkflow)

        // 注册Activities
        w.RegisterActivity(ValidateOrder)
        w.RegisterActivity(ReserveInventory)
        w.RegisterActivity(ProcessPayment)
        w.RegisterActivity(CreateShipment)
        w.RegisterActivity(SendNotification)
        w.RegisterActivity(ReleaseInventory)
        w.RegisterActivity(RefundPayment)

        err = w.Run(worker.InterruptCh())
        if err != nil {
            log.Fatal(err)
        }
    }()

    // 创建测试订单
    testOrder := Order{
        OrderID: fmt.Sprintf("ORD-%s", uuid.New().String()),
        UserID:  "user-123",
        Items: []OrderItem{
            {
                ProductID:   "PROD-001",
                ProductName: "Rust Programming Book",
                Quantity:    1,
                UnitPrice:   49.99,
            },
            {
                ProductID:   "PROD-002",
                ProductName: "Mechanical Keyboard",
                Quantity:    1,
                UnitPrice:   129.99,
            },
        },
        TotalAmount: 179.98,
        PaymentMethod: PaymentMethod{
            Type:  "credit_card",
            Token: "tok_visa_4242",
        },
        ShippingAddress: Address{
            Recipient:  "Zhang San",
            Phone:      "+86 138-0000-0000",
            Street:     "123 Tech Street",
            City:       "Shanghai",
            Province:   "Shanghai",
            PostalCode: "200000",
            Country:    "China",
        },
    }

    // 启动工作流
    workflowOptions := client.StartWorkflowOptions{
        ID:        testOrder.OrderID,
        TaskQueue: "order-processing",
    }

    we, err := c.ExecuteWorkflow(context.Background(), workflowOptions, OrderProcessingWorkflow, testOrder)
    if err != nil {
        log.Fatal(err)
    }

    log.Printf("Started workflow: WorkflowID=%s, RunID=%s", we.GetID(), we.GetRunID())

    // 等待结果
    var result OrderResult
    err = we.Get(context.Background(), &result)
    if err != nil {
        log.Fatal(err)
    }

    log.Printf("Workflow result: %+v", result)
}
```

---

## 📊 Rust vs Golang 对比

### 相同点

1. **概念模型**: 完全相同的Workflow/Activity概念
2. **执行流程**: 相同的执行步骤和补偿逻辑
3. **错误处理**: 相同的重试和补偿策略

### 差异点

| 方面 | Rust | Golang |
|------|------|--------|
| **类型系统** | 强类型，泛型，Trait约束 | 接口，类型断言 |
| **异步模型** | async/await (Tokio) | goroutine + channel |
| **错误处理** | Result<T, E> | error interface |
| **内存管理** | 所有权系统，零成本抽象 | GC |
| **Activity定义** | impl Activity trait | 函数签名 |
| **Workflow定义** | impl Workflow trait | 函数签名 |
| **依赖注入** | 通过Context | 通过Context |

---

## 🎯 总结

这个电商订单处理示例展示了：

1. **完整的业务流程**: 从订单验证到发货完成
2. **Saga模式**: 补偿机制处理失败场景
3. **Activity心跳**: 长时间运行任务的进度报告
4. **重试策略**: 自动重试失败的操作
5. **Rust vs Golang**: 相同概念的不同语言实现

两种实现在概念上完全一致，只是语言特性不同。

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26

