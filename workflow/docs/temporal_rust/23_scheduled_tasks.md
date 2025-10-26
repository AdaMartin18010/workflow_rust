# 定时任务调度

## 📋 文档概述

本文档提供定时任务调度的示例，包括：

- Cron工作流
- 周期性任务
- 动态调度
- 任务管理
- Rust + Golang并列对比

---

## ⏰ Cron工作流示例

### 场景：每日数据备份

每天凌晨2点执行数据备份任务。

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// ========================================
// 定时备份工作流
// ========================================

#[derive(Serialize, Deserialize)]
pub struct DailyBackupInput {
    pub backup_type: String,
    pub target_location: String,
}

#[derive(Serialize, Deserialize)]
pub struct DailyBackupOutput {
    pub backup_id: String,
    pub backup_size: u64,
    pub duration_seconds: u64,
}

pub struct DailyBackupWorkflow;

impl Workflow for DailyBackupWorkflow {
    type Input = DailyBackupInput;
    type Output = DailyBackupOutput;
    
    fn name() -> &'static str {
        "DailyBackup"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let start_time = ctx.now();
        
        tracing::info!(
            backup_type = %input.backup_type,
            target = %input.target_location,
            "Starting daily backup"
        );
        
        // 1. 生成备份ID
        let backup_id = format!("backup-{}", ctx.now().format("%Y%m%d%H%M%S"));
        
        // 2. 执行备份
        let backup_result = ctx.execute_activity::<PerformBackupActivity>(
            BackupInput {
                backup_id: backup_id.clone(),
                backup_type: input.backup_type,
                target_location: input.target_location.clone(),
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(3600)), // 1小时
                retry_policy: Some(RetryPolicy {
                    max_attempts: Some(3),
                    initial_interval: Duration::from_secs(60),
                    max_interval: Duration::from_secs(300),
                    backoff_coefficient: 2.0,
                    non_retryable_error_types: vec!["InvalidConfiguration"],
                }),
                ..Default::default()
            },
        ).await?;
        
        // 3. 验证备份
        let validation = ctx.execute_activity::<ValidateBackupActivity>(
            ValidationInput {
                backup_id: backup_id.clone(),
            },
            ActivityOptions::default(),
        ).await?;
        
        if !validation.valid {
            return Err(WorkflowError::internal("Backup validation failed"));
        }
        
        // 4. 清理旧备份
        ctx.execute_activity::<CleanupOldBackupsActivity>(
            CleanupInput {
                retention_days: 30,
            },
            ActivityOptions::default(),
        ).await?;
        
        // 5. 发送通知
        ctx.execute_activity::<SendBackupNotificationActivity>(
            NotificationInput {
                backup_id: backup_id.clone(),
                status: "success".to_string(),
                size: backup_result.backup_size,
            },
            ActivityOptions::default(),
        ).await?;
        
        let end_time = ctx.now();
        let duration = end_time.signed_duration_since(start_time);
        
        Ok(DailyBackupOutput {
            backup_id,
            backup_size: backup_result.backup_size,
            duration_seconds: duration.num_seconds() as u64,
        })
    }
}

// 启动定时工作流
async fn start_cron_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let client = WorkflowClient::new(ClientConfig::default()).await?;
    
    let handle = client.start_workflow::<DailyBackupWorkflow>(
        DailyBackupInput {
            backup_type: "full".to_string(),
            target_location: "s3://backups/".to_string(),
        },
        StartWorkflowOptions {
            workflow_id: Some(WorkflowId::new("daily-backup")),
            task_queue: "backup-queue".to_string(),
            // Cron表达式：每天凌晨2点
            cron_schedule: Some("0 2 * * *".to_string()),
            ..Default::default()
        },
    ).await?;
    
    println!("Cron workflow started: {}", handle.workflow_id().as_str());
    
    Ok(())
}
```

#### Golang对比

```go
package workflows

import (
    "time"
    "go.temporal.io/sdk/workflow"
)

type DailyBackupInput struct {
    BackupType     string
    TargetLocation string
}

type DailyBackupOutput struct {
    BackupID       string
    BackupSize     int64
    DurationSecs   int64
}

func DailyBackupWorkflow(ctx workflow.Context, input DailyBackupInput) (DailyBackupOutput, error) {
    startTime := workflow.Now(ctx)
    logger := workflow.GetLogger(ctx)
    
    logger.Info("Starting daily backup",
        "backup_type", input.BackupType,
        "target", input.TargetLocation)
    
    // 生成备份ID
    backupID := fmt.Sprintf("backup-%s", startTime.Format("20060102150405"))
    
    // 执行备份
    var backupResult BackupResult
    err := workflow.ExecuteActivity(ctx, PerformBackupActivity, BackupInput{
        BackupID:       backupID,
        BackupType:     input.BackupType,
        TargetLocation: input.TargetLocation,
    }).Get(ctx, &backupResult)
    if err != nil {
        return DailyBackupOutput{}, err
    }
    
    // 验证备份
    var validation ValidationResult
    err = workflow.ExecuteActivity(ctx, ValidateBackupActivity, backupID).Get(ctx, &validation)
    if err != nil || !validation.Valid {
        return DailyBackupOutput{}, fmt.Errorf("backup validation failed")
    }
    
    // 清理旧备份
    err = workflow.ExecuteActivity(ctx, CleanupOldBackupsActivity, 30).Get(ctx, nil)
    if err != nil {
        logger.Warn("Cleanup failed", "error", err)
    }
    
    // 发送通知
    workflow.ExecuteActivity(ctx, SendBackupNotificationActivity, backupID).Get(ctx, nil)
    
    duration := workflow.Now(ctx).Sub(startTime)
    
    return DailyBackupOutput{
        BackupID:     backupID,
        BackupSize:   backupResult.Size,
        DurationSecs: int64(duration.Seconds()),
    }, nil
}

// 启动Cron工作流
func StartCronWorkflow() error {
    c, err := client.Dial(client.Options{})
    if err != nil {
        return err
    }
    defer c.Close()
    
    workflowOptions := client.StartWorkflowOptions{
        ID:           "daily-backup",
        TaskQueue:    "backup-queue",
        CronSchedule: "0 2 * * *", // 每天凌晨2点
    }
    
    we, err := c.ExecuteWorkflow(
        context.Background(),
        workflowOptions,
        DailyBackupWorkflow,
        DailyBackupInput{
            BackupType:     "full",
            TargetLocation: "s3://backups/",
        },
    )
    if err != nil {
        return err
    }
    
    log.Printf("Cron workflow started: %s", we.GetID())
    return nil
}
```

---

## 🔄 周期性任务示例

### 场景：每小时数据同步

#### Rust实现1

```rust
#[derive(Serialize, Deserialize)]
pub struct HourlySyncInput {
    pub source: String,
    pub target: String,
}

pub struct HourlySyncWorkflow;

impl Workflow for HourlySyncWorkflow {
    type Input = HourlySyncInput;
    type Output = SyncResult;
    
    fn name() -> &'static str {
        "HourlySync"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            source = %input.source,
            target = %input.target,
            "Starting hourly sync"
        );
        
        // 检查上次同步时间
        let last_sync_time = ctx.execute_activity::<GetLastSyncTimeActivity>(
            input.source.clone(),
            ActivityOptions::default(),
        ).await?;
        
        // 获取增量数据
        let delta_data = ctx.execute_activity::<FetchDeltaDataActivity>(
            DeltaInput {
                source: input.source.clone(),
                since: last_sync_time.timestamp,
            },
            ActivityOptions::default(),
        ).await?;
        
        if delta_data.records.is_empty() {
            tracing::info!("No new data to sync");
            return Ok(SyncResult {
                synced_records: 0,
                success: true,
            });
        }
        
        // 同步数据
        let sync_result = ctx.execute_activity::<SyncDataActivity>(
            SyncInput {
                target: input.target,
                records: delta_data.records,
            },
            ActivityOptions {
                start_to_close_timeout: Some(Duration::from_secs(300)),
                ..Default::default()
            },
        ).await?;
        
        // 更新同步时间戳
        ctx.execute_activity::<UpdateSyncTimestampActivity>(
            UpdateTimestampInput {
                source: input.source,
                timestamp: ctx.now(),
            },
            ActivityOptions::default(),
        ).await?;
        
        Ok(sync_result)
    }
}
```

---

## 📅 动态调度示例

### 场景：根据业务需求动态调整任务执行时间

#### Rust实现2

```rust
pub struct DynamicSchedulerWorkflow;

impl Workflow for DynamicSchedulerWorkflow {
    type Input = SchedulerInput;
    type Output = SchedulerOutput;
    
    fn name() -> &'static str {
        "DynamicScheduler"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        loop {
            // 获取下次执行时间
            let next_run_time = ctx.execute_activity::<GetNextRunTimeActivity>(
                input.task_id.clone(),
                ActivityOptions::default(),
            ).await?;
            
            let now = ctx.now();
            let wait_duration = next_run_time.timestamp
                .signed_duration_since(now);
            
            if wait_duration > chrono::Duration::zero() {
                tracing::info!(
                    wait_seconds = wait_duration.num_seconds(),
                    "Waiting until next run time"
                );
                
                ctx.sleep(Duration::from_secs(wait_duration.num_seconds() as u64)).await;
            }
            
            // 执行任务
            let result = ctx.execute_activity::<ExecuteScheduledTaskActivity>(
                TaskExecutionInput {
                    task_id: input.task_id.clone(),
                },
                ActivityOptions::default(),
            ).await?;
            
            tracing::info!(
                task_id = %input.task_id,
                result = ?result,
                "Task executed"
            );
            
            // 检查是否应该继续
            if let Some(stop_signal) = ctx.try_receive_signal::<StopSchedulerSignal>().await {
                tracing::info!("Received stop signal, terminating scheduler");
                break;
            }
            
            // 检查是否需要更新调度策略
            if let Some(update_signal) = ctx.try_receive_signal::<UpdateScheduleSignal>().await {
                tracing::info!(
                    new_schedule = %update_signal.cron_expression,
                    "Schedule updated"
                );
                // 更新调度策略...
            }
        }
        
        Ok(SchedulerOutput {
            task_id: input.task_id,
            status: "stopped".to_string(),
        })
    }
}

// Signal定义
#[derive(Serialize, Deserialize)]
pub struct StopSchedulerSignal;

impl Signal for StopSchedulerSignal {
    fn name() -> &'static str {
        "stop_scheduler"
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateScheduleSignal {
    pub cron_expression: String,
}

impl Signal for UpdateScheduleSignal {
    fn name() -> &'static str {
        "update_schedule"
    }
}
```

---

## 🎯 任务管理示例

### 场景：任务队列管理

#### Rust实现3

```rust
pub struct TaskQueueManagerWorkflow;

impl Workflow for TaskQueueManagerWorkflow {
    type Input = QueueManagerInput;
    type Output = QueueManagerOutput;
    
    fn name() -> &'static str {
        "TaskQueueManager"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let mut processed_tasks = 0;
        
        loop {
            // 获取待处理任务
            let tasks = ctx.execute_activity::<FetchPendingTasksActivity>(
                FetchTasksInput {
                    queue_name: input.queue_name.clone(),
                    batch_size: 10,
                },
                ActivityOptions::default(),
            ).await?;
            
            if tasks.items.is_empty() {
                // 没有任务，等待
                ctx.sleep(Duration::from_secs(60)).await;
                continue;
            }
            
            // 并行处理任务
            let futures: Vec<_> = tasks.items.iter().map(|task| {
                ctx.execute_activity::<ProcessTaskActivity>(
                    task.clone(),
                    ActivityOptions::default(),
                )
            }).collect();
            
            let results = futures::future::join_all(futures).await;
            
            // 统计结果
            for result in results {
                if result.is_ok() {
                    processed_tasks += 1;
                }
            }
            
            // 检查是否应该停止
            if let Some(_) = ctx.try_receive_signal::<StopQueueManagerSignal>().await {
                break;
            }
        }
        
        Ok(QueueManagerOutput {
            queue_name: input.queue_name,
            processed_tasks,
        })
    }
}
```

---

## 📚 总结

### 定时任务优势

1. **可靠性**: 自动重试和错误处理
2. **可观测性**: 完整的执行历史
3. **灵活性**: 动态调整调度策略
4. **可维护性**: 集中化的任务管理

### Cron表达式示例

| 表达式 | 说明 |
|--------|------|
| `0 * * * *` | 每小时整点 |
| `0 2 * * *` | 每天凌晨2点 |
| `0 0 * * 0` | 每周日午夜 |
| `0 0 1 * *` | 每月1号午夜 |
| `*/15 * * * *` | 每15分钟 |

---

## 📚 总结1

**恭喜！** 您已完成所有23章文档的学习！

### 完整文档体系

✅ **核心概念** (1-3章)  
✅ **工作流开发** (4-6章)  
✅ **高级特性** (7-10章)  
✅ **运行时与部署** (11-15章)  
✅ **最佳实践** (16-17章)  
✅ **完整示例** (18-23章)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

🎉 **文档完结！感谢您的学习！** 🚀
