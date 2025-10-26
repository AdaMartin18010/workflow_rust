# 工作流生命周期管理

## 📋 文档概述

本文档详细阐述Temporal工作流的生命周期管理，包括：

- 工作流生命周期各个阶段
- 生命周期事件
- Rust 1.90实现
- Golang实现对比
- 最佳实践

---

## 🎯 工作流生命周期概述

### 生命周期阶段

```text
┌─────────────────────────────────────────────────────────────┐
│                    工作流生命周期                            │
└─────────────────────────────────────────────────────────────┘

1. 创建 (Created)
   │
   ├─ Client.StartWorkflow()
   │
   ▼
2. 已调度 (Scheduled)
   │
   ├─ Worker轮询任务
   │
   ▼
3. 运行中 (Running)
   │
   ├─ 执行工作流逻辑
   ├─ 调度Activities
   ├─ 接收Signals
   ├─ 处理Queries
   │
   ├─── 正常完成 ────▶ 4a. 完成 (Completed)
   │
   ├─── 失败 ────────▶ 4b. 失败 (Failed)
   │
   ├─── 取消 ────────▶ 4c. 取消 (Cancelled)
   │
   ├─── 超时 ────────▶ 4d. 超时 (Timed Out)
   │
   └─── 终止 ────────▶ 4e. 终止 (Terminated)

5. 归档 (Archived)
   │
   └─ 事件历史保存
```

### 生命周期状态

| 状态 | 说明 | 可恢复 |
|------|------|--------|
| **Scheduled** | 已调度，等待Worker执行 | N/A |
| **Running** | 正在执行 | ✅ 是 |
| **Completed** | 成功完成 | ❌ 否 |
| **Failed** | 执行失败 | ❌ 否 |
| **Cancelled** | 被取消 | ❌ 否 |
| **Timed Out** | 超时 | ❌ 否 |
| **Terminated** | 被强制终止 | ❌ 否 |
| **ContinuedAsNew** | 继续为新实例 | ✅ 是 |

---

## 🦀 Rust实现

### 生命周期事件定义

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 工作流生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowLifecycleState {
    /// 已调度
    Scheduled,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 取消
    Cancelled,
    /// 超时
    TimedOut,
    /// 终止
    Terminated,
    /// 继续为新
    ContinuedAsNew,
}

/// 生命周期事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// 工作流开始
    WorkflowStarted {
        workflow_id: WorkflowId,
        run_id: RunId,
        workflow_type: String,
        input: serde_json::Value,
        started_at: DateTime<Utc>,
    },
    
    /// 工作流完成
    WorkflowCompleted {
        workflow_id: WorkflowId,
        run_id: RunId,
        output: serde_json::Value,
        completed_at: DateTime<Utc>,
    },
    
    /// 工作流失败
    WorkflowFailed {
        workflow_id: WorkflowId,
        run_id: RunId,
        error: String,
        failed_at: DateTime<Utc>,
    },
    
    /// 工作流取消
    WorkflowCancelled {
        workflow_id: WorkflowId,
        run_id: RunId,
        reason: String,
        cancelled_at: DateTime<Utc>,
    },
    
    /// 工作流超时
    WorkflowTimedOut {
        workflow_id: WorkflowId,
        run_id: RunId,
        timeout_type: TimeoutType,
        timed_out_at: DateTime<Utc>,
    },
    
    /// 工作流终止
    WorkflowTerminated {
        workflow_id: WorkflowId,
        run_id: RunId,
        reason: String,
        terminated_at: DateTime<Utc>,
    },
    
    /// 继续为新
    WorkflowContinuedAsNew {
        old_run_id: RunId,
        new_run_id: RunId,
        new_input: serde_json::Value,
        continued_at: DateTime<Utc>,
    },
}

/// 超时类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeoutType {
    /// 工作流执行超时
    WorkflowExecution,
    /// 工作流运行超时
    WorkflowRun,
    /// 工作流任务超时
    WorkflowTask,
}
```

### 生命周期管理器

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工作流生命周期管理器
pub struct WorkflowLifecycleManager {
    /// 当前状态
    state: Arc<RwLock<WorkflowLifecycleState>>,
    
    /// 生命周期事件历史
    events: Arc<RwLock<Vec<LifecycleEvent>>>,
    
    /// 工作流执行信息
    execution: WorkflowExecution,
}

impl WorkflowLifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new(execution: WorkflowExecution) -> Self {
        Self {
            state: Arc::new(RwLock::new(WorkflowLifecycleState::Scheduled)),
            events: Arc::new(RwLock::new(Vec::new())),
            execution,
        }
    }
    
    /// 获取当前状态
    pub async fn get_state(&self) -> WorkflowLifecycleState {
        *self.state.read().await
    }
    
    /// 记录生命周期事件
    pub async fn record_event(&self, event: LifecycleEvent) {
        self.events.write().await.push(event);
    }
    
    /// 转换状态
    pub async fn transition_to(&self, new_state: WorkflowLifecycleState) -> Result<(), String> {
        let mut state = self.state.write().await;
        
        // 验证状态转换是否合法
        if !self.is_valid_transition(*state, new_state) {
            return Err(format!(
                "Invalid state transition from {:?} to {:?}",
                *state, new_state
            ));
        }
        
        *state = new_state;
        Ok(())
    }
    
    /// 验证状态转换是否合法
    fn is_valid_transition(
        &self,
        from: WorkflowLifecycleState,
        to: WorkflowLifecycleState,
    ) -> bool {
        use WorkflowLifecycleState::*;
        
        matches!(
            (from, to),
            (Scheduled, Running)
                | (Running, Completed)
                | (Running, Failed)
                | (Running, Cancelled)
                | (Running, TimedOut)
                | (Running, Terminated)
                | (Running, ContinuedAsNew)
        )
    }
    
    /// 获取所有事件
    pub async fn get_events(&self) -> Vec<LifecycleEvent> {
        self.events.read().await.clone()
    }
}
```

### 工作流启动

```rust
/// 启动工作流
pub async fn start_workflow<W: Workflow>(
    client: &WorkflowClient,
    workflow_id: WorkflowId,
    task_queue: String,
    input: W::Input,
    options: StartWorkflowOptions,
) -> Result<WorkflowHandle<W::Output>, ClientError> {
    // 创建运行ID
    let run_id = RunId::generate();
    let execution = WorkflowExecution::with_run_id(workflow_id.clone(), run_id);
    
    // 创建生命周期管理器
    let lifecycle = WorkflowLifecycleManager::new(execution.clone());
    
    // 记录开始事件
    lifecycle
        .record_event(LifecycleEvent::WorkflowStarted {
            workflow_id: workflow_id.clone(),
            run_id,
            workflow_type: W::name().to_string(),
            input: serde_json::to_value(&input)?,
            started_at: Utc::now(),
        })
        .await;
    
    // 转换到运行状态
    lifecycle.transition_to(WorkflowLifecycleState::Running).await?;
    
    // 创建工作流句柄
    let handle = WorkflowHandle::new(execution);
    
    Ok(handle)
}
```

### 工作流完成

```rust
/// 工作流完成处理
pub async fn complete_workflow<O: Serialize>(
    lifecycle: &WorkflowLifecycleManager,
    output: O,
) -> Result<(), WorkflowError> {
    // 记录完成事件
    lifecycle
        .record_event(LifecycleEvent::WorkflowCompleted {
            workflow_id: lifecycle.execution.workflow_id.clone(),
            run_id: lifecycle.execution.run_id,
            output: serde_json::to_value(&output)?,
            completed_at: Utc::now(),
        })
        .await;
    
    // 转换状态
    lifecycle.transition_to(WorkflowLifecycleState::Completed).await?;
    
    Ok(())
}
```

### 工作流取消

```rust
/// 取消工作流
pub async fn cancel_workflow(
    client: &WorkflowClient,
    workflow_id: &WorkflowId,
    reason: String,
) -> Result<(), ClientError> {
    // 发送取消信号到工作流
    client
        .signal_workflow::<CancelSignal>(
            workflow_id,
            CancelSignal { reason: reason.clone() },
        )
        .await?;
    
    Ok(())
}

/// 在工作流中处理取消
impl Workflow for CancellableWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 创建取消通道
            let (cancel_tx, mut cancel_rx) = mpsc::channel::<CancelSignal>(1);
            ctx.register_signal_handler::<CancelSignal>(cancel_tx);
            
            // 执行工作
            let work = async {
                // 工作流逻辑
                ctx.execute_activity::<MyActivity>(input, options).await
            };
            
            // 等待工作完成或取消
            select! {
                result = work => result,
                Some(cancel) = cancel_rx.recv() => {
                    // 记录取消事件
                    ctx.lifecycle().record_event(LifecycleEvent::WorkflowCancelled {
                        workflow_id: ctx.execution().workflow_id.clone(),
                        run_id: ctx.execution().run_id,
                        reason: cancel.reason,
                        cancelled_at: Utc::now(),
                    }).await;
                    
                    Err(WorkflowError::Cancelled)
                }
            }
        }
    }
}
```

### Continue As New

```rust
/// Continue As New - 继续为新工作流
pub async fn continue_as_new<W: Workflow>(
    ctx: &WorkflowContext,
    new_input: W::Input,
) -> Result<(), WorkflowError> {
    let old_run_id = ctx.execution().run_id;
    let new_run_id = RunId::generate();
    
    // 记录ContinueAsNew事件
    ctx.lifecycle()
        .record_event(LifecycleEvent::WorkflowContinuedAsNew {
            old_run_id,
            new_run_id,
            new_input: serde_json::to_value(&new_input)?,
            continued_at: Utc::now(),
        })
        .await;
    
    // 实际的ContinueAsNew实现会重启工作流
    // 这里简化处理
    Ok(())
}

/// 使用示例：循环工作流
impl Workflow for LoopingWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            let mut iteration = input.iteration;
            let max_iterations = input.max_iterations;
            
            // 处理当前迭代
            ctx.execute_activity::<ProcessIterationActivity>(
                IterationInput { iteration },
                ActivityOptions::default(),
            )
            .await?;
            
            iteration += 1;
            
            // 如果还有更多迭代，使用ContinueAsNew
            if iteration < max_iterations {
                continue_as_new::<Self>(
                    &ctx,
                    Self::Input {
                        iteration,
                        max_iterations,
                    },
                )
                .await?;
                
                // ContinueAsNew之后的代码不会执行
                unreachable!()
            }
            
            // 所有迭代完成
            Ok(Self::Output {
                total_iterations: iteration,
            })
        }
    }
}
```

---

## 🐹 Golang实现对比

### 生命周期状态 - Golang

```go
type WorkflowLifecycleState int

const (
    Scheduled WorkflowLifecycleState = iota
    Running
    Completed
    Failed
    Cancelled
    TimedOut
    Terminated
    ContinuedAsNew
)

// 生命周期事件
type LifecycleEvent struct {
    EventType string
    Timestamp time.Time
    Details   map[string]interface{}
}
```

### 工作流取消 - Golang

```go
func CancellableWorkflow(ctx workflow.Context, input WorkflowInput) (WorkflowOutput, error) {
    logger := workflow.GetLogger(ctx)
    
    // 创建取消channel
    cancelChan := workflow.GetSignalChannel(ctx, "cancel")
    
    // 执行工作
    activityCtx := workflow.WithActivityOptions(ctx, workflow.ActivityOptions{
        StartToCloseTimeout: 30 * time.Second,
    })
    
    selector := workflow.NewSelector(ctx)
    var activityResult ActivityResult
    var cancelled bool
    
    // 添加Activity执行
    activityFuture := workflow.ExecuteActivity(activityCtx, MyActivity, input)
    selector.AddFuture(activityFuture, func(f workflow.Future) {
        err := f.Get(ctx, &activityResult)
        if err != nil {
            logger.Error("Activity failed", "error", err)
        }
    })
    
    // 添加取消信号
    selector.AddReceive(cancelChan, func(c workflow.ReceiveChannel, more bool) {
        var cancel CancelSignal
        c.Receive(ctx, &cancel)
        logger.Info("Workflow cancelled", "reason", cancel.Reason)
        cancelled = true
    })
    
    selector.Select(ctx)
    
    if cancelled {
        return WorkflowOutput{}, workflow.NewCanceledError("Workflow cancelled")
    }
    
    return WorkflowOutput{Result: activityResult}, nil
}
```

### Continue As New - Golang

```go
func LoopingWorkflow(ctx workflow.Context, input LoopingInput) (LoopingOutput, error) {
    logger := workflow.GetLogger(ctx)
    iteration := input.Iteration
    maxIterations := input.MaxIterations
    
    // 处理当前迭代
    var iterationResult IterationResult
    err := workflow.ExecuteActivity(ctx, ProcessIterationActivity, IterationInput{
        Iteration: iteration,
    }).Get(ctx, &iterationResult)
    
    if err != nil {
        return LoopingOutput{}, err
    }
    
    iteration++
    
    // 如果还有更多迭代，使用ContinueAsNew
    if iteration < maxIterations {
        logger.Info("Continuing as new", "iteration", iteration)
        return LoopingOutput{}, workflow.NewContinueAsNewError(
            ctx,
            LoopingWorkflow,
            LoopingInput{
                Iteration:     iteration,
                MaxIterations: maxIterations,
            },
        )
    }
    
    // 所有迭代完成
    logger.Info("All iterations completed", "totalIterations", iteration)
    return LoopingOutput{TotalIterations: iteration}, nil
}
```

---

## 🎯 最佳实践

### 1. 优雅关闭

```rust
impl Workflow for GracefulShutdownWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 注册关闭信号
            let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<ShutdownSignal>(1);
            ctx.register_signal_handler::<ShutdownSignal>(shutdown_tx);
            
            // 工作循环
            loop {
                select! {
                    // 正常工作
                    _ = ctx.execute_activity::<WorkActivity>(input, options) => {
                        // 继续下一轮
                    }
                    
                    // 接收关闭信号
                    Some(shutdown) = shutdown_rx.recv() => {
                        // 执行清理
                        ctx.execute_activity::<CleanupActivity>(
                            CleanupInput { reason: shutdown.reason },
                            options,
                        ).await?;
                        
                        return Ok(Self::Output {
                            status: "Gracefully shut down".to_string(),
                        });
                    }
                }
            }
        }
    }
}
```

### 2. 合理使用 ContinueAsNew

```rust
// ✅ 好: 防止事件历史过大
impl Workflow for DataProcessingWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            let processed = input.processed_count;
            let batch_size = 1000;
            
            // 处理一批数据
            for i in 0..batch_size {
                process_item(i).await?;
            }
            
            let total_processed = processed + batch_size;
            
            // 每处理10000个项目，ContinueAsNew
            if total_processed % 10000 == 0 {
                continue_as_new::<Self>(
                    &ctx,
                    Self::Input {
                        processed_count: total_processed,
                        ..input
                    },
                )
                .await?;
            }
            
            Ok(Self::Output { total_processed })
        }
    }
}

// ❌ 差: 不使用ContinueAsNew导致事件历史过大
impl Workflow for LongRunningWorkflow {
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 无限循环，事件历史会无限增长
            loop {
                process_item().await?;
            }
        }
    }
}
```

### 3. 超时配置

```rust
// ✅ 好: 明确的超时配置
let options = StartWorkflowOptions {
    workflow_id: Some(WorkflowId::new("my-workflow")),
    task_queue: "my-queue".to_string(),
    // 整个工作流执行的最大时间
    workflow_execution_timeout: Some(Duration::from_secs(86400)), // 24小时
    // 单次运行的最大时间（用于ContinueAsNew场景）
    workflow_run_timeout: Some(Duration::from_secs(3600)), // 1小时
    // 单个工作流任务的最大时间
    workflow_task_timeout: Some(Duration::from_secs(10)),
};
```

---

## 📚 总结

### 关键概念

1. **生命周期状态**: 从Scheduled到终态（Completed/Failed/Cancelled等）
2. **生命周期事件**: 记录所有重要的状态转换
3. **优雅关闭**: 支持通过Signal优雅地关闭工作流
4. **ContinueAsNew**: 防止事件历史过大的关键机制
5. **超时管理**: 多层次的超时保护

### Rust vs Golang

- **Rust**: 更强的类型安全，显式的状态管理
- **Golang**: 更简单的错误处理（ContinueAsNewError）

---

## 📚 下一步

- **重试与超时**: [重试策略详解](./08_retry_and_timeout.md)
- **版本管理**: [工作流版本控制](./09_versioning.md)
- **测试策略**: [测试最佳实践](./10_testing.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
