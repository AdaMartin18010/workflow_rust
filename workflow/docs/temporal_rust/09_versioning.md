# 工作流版本管理

## 📋 文档概述

本文档详细阐述Temporal的版本管理机制，包括：

- 版本管理原理
- 兼容性策略
- Rust 1.90实现
- Golang实现对比
- 版本迁移最佳实践

---

## 🎯 为什么需要版本管理？

### 问题场景

```text
场景：正在运行的工作流遇到代码更新

旧代码:                          新代码:
┌────────────────┐              ┌────────────────┐
│ Workflow V1    │              │ Workflow V2    │
│                │              │                │
│ Step 1 ────▶   │              │ Step 1 ────▶   │
│ Step 2 ────▶   │              │ Step 1.5 ──▶   │ (新增)
│ Step 3 ────▶   │              │ Step 2 ────▶   │
└────────────────┘              │ Step 4 ────▶   │ (重命名)
                                └────────────────┘

问题：
1. 正在运行的工作流实例如何处理？
2. 如何保证事件历史的一致性？
3. 如何安全地部署新版本？
```

### Temporal的解决方案

Temporal通过**确定性重放**机制，允许安全地更新工作流代码：

1. **事件历史不变**: 已发生的事件不可更改
2. **代码向后兼容**: 新代码必须能够处理旧版本的事件历史
3. **版本标记**: 使用GetVersion API标记代码变更点

---

## 🦀 Rust实现

### 版本API

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 版本管理器
pub struct VersionManager {
    /// 已记录的版本
    versions: Arc<RwLock<HashMap<String, i32>>>,
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 获取或设置版本
    pub async fn get_version(
        &self,
        change_id: &str,
        min_supported: i32,
        max_supported: i32,
    ) -> Result<i32, VersionError> {
        let mut versions = self.versions.write().await;
        
        if let Some(&version) = versions.get(change_id) {
            // 已有版本记录，必须在支持范围内
            if version < min_supported || version > max_supported {
                return Err(VersionError::UnsupportedVersion {
                    change_id: change_id.to_string(),
                    version,
                    min_supported,
                    max_supported,
                });
            }
            Ok(version)
        } else {
            // 首次调用，使用最大支持版本
            versions.insert(change_id.to_string(), max_supported);
            Ok(max_supported)
        }
    }
}

/// 版本错误
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Unsupported version for {change_id}: {version} (supported: {min_supported}..{max_supported})")]
    UnsupportedVersion {
        change_id: String,
        version: i32,
        min_supported: i32,
        max_supported: i32,
    },
}

/// WorkflowContext扩展版本API
impl WorkflowContext {
    /// 获取版本
    pub async fn get_version(
        &self,
        change_id: &str,
        min_supported: i32,
        max_supported: i32,
    ) -> Result<i32, VersionError> {
        self.version_manager()
            .get_version(change_id, min_supported, max_supported)
            .await
    }
}
```

### 版本迁移示例

#### 场景1: 添加新步骤

```rust
// === V1: 原始版本 ===
impl Workflow for OrderWorkflowV1 {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 步骤1: 处理支付
            let payment = ctx
                .execute_activity::<ProcessPaymentActivity>(input.payment, options)
                .await?;
            
            // 步骤2: 发货
            let shipment = ctx
                .execute_activity::<ShipOrderActivity>(input.shipping, options)
                .await?;
            
            Ok(Self::Output { payment, shipment })
        }
    }
}

// === V2: 添加库存检查 ===
impl Workflow for OrderWorkflowV2 {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 步骤1: 处理支付（不变）
            let payment = ctx
                .execute_activity::<ProcessPaymentActivity>(input.payment, options)
                .await?;
            
            // 使用版本检查添加新步骤
            let version = ctx.get_version("add-inventory-check", 1, 2).await?;
            
            let inventory = if version >= 2 {
                // V2: 检查库存
                Some(ctx
                    .execute_activity::<CheckInventoryActivity>(input.items, options)
                    .await?)
            } else {
                // V1: 跳过库存检查
                None
            };
            
            // 步骤2: 发货（不变）
            let shipment = ctx
                .execute_activity::<ShipOrderActivity>(input.shipping, options)
                .await?;
            
            Ok(Self::Output { payment, inventory, shipment })
        }
    }
}
```

#### 场景2: 修改Activity输入

```rust
// === V1: 原始版本 ===
#[derive(Deserialize)]
struct PaymentInputV1 {
    amount: f64,
    currency: String,
}

// === V2: 添加新字段 ===
#[derive(Deserialize)]
struct PaymentInputV2 {
    amount: f64,
    currency: String,
    #[serde(default)]
    payment_method: Option<String>,  // 新增字段
}

impl Workflow for OrderWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            let version = ctx.get_version("payment-input-change", 1, 2).await?;
            
            let payment_result = if version >= 2 {
                // V2: 使用新输入格式
                ctx.execute_activity::<ProcessPaymentActivity>(
                    PaymentInputV2 {
                        amount: input.amount,
                        currency: input.currency.clone(),
                        payment_method: input.payment_method.clone(),
                    },
                    options,
                ).await?
            } else {
                // V1: 使用旧输入格式
                ctx.execute_activity::<ProcessPaymentActivity>(
                    PaymentInputV1 {
                        amount: input.amount,
                        currency: input.currency.clone(),
                    },
                    options,
                ).await?
            };
            
            Ok(Self::Output { payment: payment_result })
        }
    }
}
```

#### 场景3: 改变工作流逻辑

```rust
impl Workflow for OrderWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            let version = ctx.get_version("approval-logic-change", 1, 2).await?;
            
            if version >= 2 {
                // V2: 新逻辑 - 金额超过1000需要审批
                if input.amount > 1000.0 {
                    let approval = ctx.wait_for_signal::<ApprovalSignal>().await?;
                    if !approval.approved {
                        return Ok(Self::Output {
                            status: OrderStatus::Rejected,
                        });
                    }
                }
            } else {
                // V1: 旧逻辑 - 所有订单都需要审批
                let approval = ctx.wait_for_signal::<ApprovalSignal>().await?;
                if !approval.approved {
                    return Ok(Self::Output {
                        status: OrderStatus::Rejected,
                    });
                }
            }
            
            // 继续处理订单
            let payment = ctx
                .execute_activity::<ProcessPaymentActivity>(input.payment, options)
                .await?;
            
            Ok(Self::Output {
                status: OrderStatus::Completed,
            })
        }
    }
}
```

### 版本清理

```rust
/// 清理旧版本支持
/// 
/// 在所有旧版本工作流都完成后，可以清理版本检查代码
impl Workflow for OrderWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // === 阶段1: 支持V1和V2 ===
            // let version = ctx.get_version("add-inventory-check", 1, 2).await?;
            // if version >= 2 {
            //     check_inventory().await?;
            // }
            
            // === 阶段2: 只支持V2 ===
            // let version = ctx.get_version("add-inventory-check", 2, 2).await?;
            // check_inventory().await?;
            
            // === 阶段3: 移除版本检查（所有旧工作流都已完成） ===
            check_inventory().await?;
            
            Ok(Self::Output { /* ... */ })
        }
    }
}
```

---

## 🐹 Golang实现对比

### 版本API - Golang

```go
func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    // 获取版本
    version := workflow.GetVersion(ctx, "add-inventory-check", workflow.DefaultVersion, 2)
    
    // 处理支付
    var payment PaymentResult
    err := workflow.ExecuteActivity(ctx, ProcessPaymentActivity, input.Payment).Get(ctx, &payment)
    if err != nil {
        return OrderOutput{}, err
    }
    
    // 根据版本决定是否检查库存
    var inventory *InventoryResult
    if version >= 2 {
        var inv InventoryResult
        err := workflow.ExecuteActivity(ctx, CheckInventoryActivity, input.Items).Get(ctx, &inv)
        if err != nil {
            return OrderOutput{}, err
        }
        inventory = &inv
    }
    
    // 发货
    var shipment ShipmentResult
    err = workflow.ExecuteActivity(ctx, ShipOrderActivity, input.Shipping).Get(ctx, &shipment)
    if err != nil {
        return OrderOutput{}, err
    }
    
    return OrderOutput{
        Payment:   payment,
        Inventory: inventory,
        Shipment:  shipment,
    }, nil
}
```

### 版本常量 - Golang

```go
const (
    // DefaultVersion 表示首次执行时的版本
    DefaultVersion = workflow.DefaultVersion  // -1
    
    // 自定义版本号
    VersionAddInventoryCheck = 2
    VersionUpdatePaymentInput = 3
)

func OrderWorkflow(ctx workflow.Context, input OrderInput) (OrderOutput, error) {
    version := workflow.GetVersion(
        ctx,
        "workflow-changes",
        workflow.DefaultVersion,
        VersionUpdatePaymentInput,
    )
    
    if version >= VersionAddInventoryCheck {
        // 包含库存检查的逻辑
    }
    
    if version >= VersionUpdatePaymentInput {
        // 使用新的支付输入格式
    }
    
    return OrderOutput{}, nil
}
```

---

## 🔄 Rust vs Golang 对比

| 特性 | Rust | Golang |
|------|------|--------|
| **版本API** | `ctx.get_version()` | `workflow.GetVersion()` |
| **默认版本** | `1` | `workflow.DefaultVersion (-1)` |
| **版本范围** | `(min, max)` | `(min, max)` |
| **错误处理** | `Result<i32, VersionError>` | `int` (panic on error) |
| **类型安全** | 编译时检查 | 运行时检查 |

---

## 🎯 最佳实践

### 1. 版本命名

```rust
// ✅ 好: 使用描述性的change_id
ctx.get_version("add-inventory-check", 1, 2).await?;
ctx.get_version("update-payment-input", 1, 2).await?;
ctx.get_version("change-approval-logic", 1, 2).await?;

// ❌ 差: 使用模糊的change_id
ctx.get_version("v2", 1, 2).await?;
ctx.get_version("update", 1, 2).await?;
ctx.get_version("change1", 1, 2).await?;
```

### 2. 版本递增策略

```rust
// ✅ 好: 渐进式版本递增
// V1 → V2: 添加库存检查
let version = ctx.get_version("add-inventory-check", 1, 2).await?;

// V2 → V3: 修改支付逻辑
let version = ctx.get_version("update-payment", 1, 3).await?;

// ❌ 差: 跳跃式版本递增
let version = ctx.get_version("changes", 1, 10).await?;  // 跳过太多版本
```

### 3. 版本清理流程

```rust
// 阶段1: 部署支持多版本的代码
let version = ctx.get_version("feature-x", 1, 2).await?;
if version >= 2 {
    new_logic().await?;
} else {
    old_logic().await?;
}

// 阶段2: 等待所有V1工作流完成（监控）
// 检查：SELECT COUNT(*) FROM workflows WHERE version < 2

// 阶段3: 移除V1支持
let version = ctx.get_version("feature-x", 2, 2).await?;
new_logic().await?;

// 阶段4: 最终移除版本检查
new_logic().await?;
```

### 4. 不兼容变更的处理

```rust
// 场景：需要完全重写工作流逻辑

// ❌ 错误做法：试图在同一个工作流中支持
// 这会导致代码复杂度急剧增加

// ✅ 正确做法：创建新的工作流类型
pub struct OrderWorkflowV1;
pub struct OrderWorkflowV2;  // 全新实现

// 在客户端选择使用哪个版本
if use_new_version {
    client.start_workflow::<OrderWorkflowV2>(...)
} else {
    client.start_workflow::<OrderWorkflowV1>(...)
}
```

### 5. 版本测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_v1_compatibility() {
        // 创建V1的事件历史
        let history = create_v1_event_history();
        
        // 使用V2代码重放
        let result = replay_workflow::<OrderWorkflowV2>(history).await;
        
        // 验证结果
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_workflow_v2_new_execution() {
        // 测试V2的全新执行
        let result = execute_workflow::<OrderWorkflowV2>(input).await;
        
        // 验证新功能
        assert!(result.inventory.is_some());
    }
}
```

---

## 📊 版本迁移决策树

```text
┌─────────────────────────────────────────────────────────────┐
│                 需要修改工作流代码？                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ├─ 是
                            │
                            ▼
          ┌─────────────────────────────────────┐
          │  变更是否向后兼容？                  │
          │  (旧工作流能继续运行)                │
          └─────────────────────────────────────┘
                │                    │
                ├─ 是                ├─ 否
                │                    │
                ▼                    ▼
    ┌─────────────────────┐   ┌─────────────────────┐
    │ 使用 GetVersion API │   │   创建新工作流类型   │
    │                     │   │                     │
    │ 1. 添加版本检查     │   │ OrderWorkflowV2     │
    │ 2. 支持新旧逻辑     │   │                     │
    │ 3. 逐步清理旧版本   │   │ 客户端选择版本      │
    └─────────────────────┘   └─────────────────────┘
```

---

## 📚 常见场景和解决方案

### 场景1: 添加新Activity

```rust
// ✅ 使用GetVersion
let version = ctx.get_version("add-notification", 1, 2).await?;
if version >= 2 {
    ctx.execute_activity::<SendNotificationActivity>(...).await?;
}
```

### 场景2: 修改Activity参数

```rust
// ✅ 使用版本化的输入类型
let version = ctx.get_version("update-input", 1, 2).await?;
let input = if version >= 2 {
    InputV2 { /* new fields */ }
} else {
    InputV1 { /* old fields */ }.into()
};
```

### 场景3: 改变控制流

```rust
// ✅ 使用版本控制分支
let version = ctx.get_version("change-flow", 1, 2).await?;
if version >= 2 {
    // 新流程
    step_a().await?;
    step_b().await?;
} else {
    // 旧流程
    step_x().await?;
    step_y().await?;
}
```

### 场景4: 完全重写

```rust
// ✅ 创建新工作流
pub struct OrderWorkflowV2;  // 全新实现

// 部署策略：
// 1. 同时部署V1和V2
// 2. 新工作流使用V2
// 3. 等待V1工作流完成
// 4. 下线V1代码
```

---

## 📚 总结

### 版本管理原则

1. **向后兼容**: 新代码必须能处理旧事件历史
2. **渐进式变更**: 一次只改一个地方
3. **充分测试**: 测试新旧版本的兼容性
4. **监控迁移**: 跟踪旧版本工作流的完成情况
5. **及时清理**: 完成迁移后清理版本检查代码

### Rust vs Golang

- **Rust**: 更强的类型安全，显式的错误处理
- **Golang**: 更简单的API，隐式的错误处理

---

## 📚 下一步

- **测试策略**: [工作流测试](./10_testing.md)
- **部署策略**: [安全部署](./deployment.md)
- **监控告警**: [版本监控](./monitoring.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
