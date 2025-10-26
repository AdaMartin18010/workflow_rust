# 批量任务处理

## 📋 文档概述

本文档提供批量任务处理的示例，包括：

- 大规模并行处理
- 进度跟踪和报告
- 失败重试策略
- 动态任务分配
- Rust + Golang并列对比

---

## 🔄 批量数据处理示例

### 场景：批量导入用户数据

需要处理数百万条用户记录，每条记录需要验证、转换和存储。

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// ========================================
// 批量处理工作流
// ========================================

#[derive(Serialize, Deserialize)]
pub struct BatchProcessingInput {
    pub batch_id: String,
    pub total_items: usize,
    pub batch_size: usize,
    pub source_file: String,
}

#[derive(Serialize, Deserialize)]
pub struct BatchProcessingOutput {
    pub batch_id: String,
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_seconds: u64,
}

pub struct BatchProcessingWorkflow;

impl Workflow for BatchProcessingWorkflow {
    type Input = BatchProcessingInput;
    type Output = BatchProcessingOutput;
    
    fn name() -> &'static str {
        "BatchProcessing"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let start_time = ctx.now();
        
        tracing::info!(
            batch_id = %input.batch_id,
            total_items = input.total_items,
            "Starting batch processing"
        );
        
        let mut total_processed = 0;
        let mut successful = 0;
        let mut failed = 0;
        
        // 计算批次数量
        let num_batches = (input.total_items + input.batch_size - 1) / input.batch_size;
        
        // 并行处理多个批次
        let max_parallel = 10; // 最多10个并行任务
        
        for chunk_start in (0..num_batches).step_by(max_parallel) {
            let chunk_end = std::cmp::min(chunk_start + max_parallel, num_batches);
            
            // 创建并行任务
            let mut futures = Vec::new();
            
            for batch_idx in chunk_start..chunk_end {
                let offset = batch_idx * input.batch_size;
                let limit = std::cmp::min(input.batch_size, input.total_items - offset);
                
                let future = ctx.execute_activity::<ProcessBatchActivity>(
                    BatchInput {
                        batch_id: input.batch_id.clone(),
                        batch_index: batch_idx,
                        source_file: input.source_file.clone(),
                        offset,
                        limit,
                    },
                    ActivityOptions {
                        start_to_close_timeout: Some(Duration::from_secs(300)),
                        retry_policy: Some(RetryPolicy {
                            max_attempts: Some(3),
                            initial_interval: Duration::from_secs(1),
                            max_interval: Duration::from_secs(10),
                            backoff_coefficient: 2.0,
                            non_retryable_error_types: vec!["ValidationError"],
                        }),
                        ..Default::default()
                    },
                );
                
                futures.push(future);
            }
            
            // 等待所有并行任务完成
            let results = futures::future::join_all(futures).await;
            
            // 汇总结果
            for result in results {
                match result {
                    Ok(batch_result) => {
                        total_processed += batch_result.processed;
                        successful += batch_result.successful;
                        failed += batch_result.failed;
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Batch processing failed");
                        failed += input.batch_size;
                    }
                }
            }
            
            // 更新进度
            let progress = (total_processed as f64 / input.total_items as f64) * 100.0;
            
            ctx.record_heartbeat(serde_json::json!({
                "batch_id": input.batch_id,
                "progress": progress,
                "processed": total_processed,
                "successful": successful,
                "failed": failed,
            })).await;
            
            tracing::info!(
                processed = total_processed,
                successful = successful,
                failed = failed,
                progress = format!("{:.2}%", progress),
                "Batch progress update"
            );
        }
        
        let end_time = ctx.now();
        let duration = end_time.signed_duration_since(start_time);
        
        tracing::info!(
            total_processed = total_processed,
            successful = successful,
            failed = failed,
            duration_secs = duration.num_seconds(),
            "Batch processing completed"
        );
        
        Ok(BatchProcessingOutput {
            batch_id: input.batch_id,
            total_processed,
            successful,
            failed,
            duration_seconds: duration.num_seconds() as u64,
        })
    }
}

// ========================================
// Activity定义
// ========================================

pub struct ProcessBatchActivity;

#[derive(Serialize, Deserialize)]
pub struct BatchInput {
    pub batch_id: String,
    pub batch_index: usize,
    pub source_file: String,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BatchResult {
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
}

impl Activity for ProcessBatchActivity {
    type Input = BatchInput;
    type Output = BatchResult;
    
    fn name() -> &'static str {
        "ProcessBatch"
    }
    
    async fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(
            batch_id = %input.batch_id,
            batch_index = input.batch_index,
            offset = input.offset,
            limit = input.limit,
            "Processing batch"
        );
        
        let mut processed = 0;
        let mut successful = 0;
        let mut failed = 0;
        
        // 读取批次数据
        let items = read_batch_items(&input.source_file, input.offset, input.limit).await?;
        
        for (idx, item) in items.iter().enumerate() {
            // 发送心跳（每10条记录）
            if idx % 10 == 0 {
                ctx.record_heartbeat(serde_json::json!({
                    "batch_index": input.batch_index,
                    "processed": processed,
                })).await;
            }
            
            // 处理单条记录
            match process_single_item(item).await {
                Ok(_) => {
                    successful += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        item_id = ?item,
                        error = ?e,
                        "Failed to process item"
                    );
                    failed += 1;
                }
            }
            
            processed += 1;
        }
        
        Ok(BatchResult {
            processed,
            successful,
            failed,
        })
    }
}

// 辅助函数
async fn read_batch_items(
    source_file: &str,
    offset: usize,
    limit: usize,
) -> Result<Vec<serde_json::Value>, ActivityError> {
    // 实际实现：从文件或数据库读取批次数据
    Ok(vec![])
}

async fn process_single_item(item: &serde_json::Value) -> Result<(), ActivityError> {
    // 实际实现：验证、转换、存储单条记录
    Ok(())
}
```

#### Golang对比

```go
package workflows

import (
    "go.temporal.io/sdk/workflow"
)

type BatchProcessingInput struct {
    BatchID    string
    TotalItems int
    BatchSize  int
    SourceFile string
}

type BatchProcessingOutput struct {
    BatchID        string
    TotalProcessed int
    Successful     int
    Failed         int
    DurationSecs   int64
}

func BatchProcessingWorkflow(ctx workflow.Context, input BatchProcessingInput) (BatchProcessingOutput, error) {
    startTime := workflow.Now(ctx)
    logger := workflow.GetLogger(ctx)
    
    logger.Info("Starting batch processing",
        "batch_id", input.BatchID,
        "total_items", input.TotalItems)
    
    var totalProcessed, successful, failed int
    
    numBatches := (input.TotalItems + input.BatchSize - 1) / input.BatchSize
    maxParallel := 10
    
    for chunkStart := 0; chunkStart < numBatches; chunkStart += maxParallel {
        chunkEnd := chunkStart + maxParallel
        if chunkEnd > numBatches {
            chunkEnd = numBatches
        }
        
        // 创建并行任务
        var futures []workflow.Future
        
        for batchIdx := chunkStart; batchIdx < chunkEnd; batchIdx++ {
            offset := batchIdx * input.BatchSize
            limit := input.BatchSize
            if offset+limit > input.TotalItems {
                limit = input.TotalItems - offset
            }
            
            future := workflow.ExecuteActivity(ctx, ProcessBatchActivity, BatchInput{
                BatchID:    input.BatchID,
                BatchIndex: batchIdx,
                SourceFile: input.SourceFile,
                Offset:     offset,
                Limit:      limit,
            })
            
            futures = append(futures, future)
        }
        
        // 等待所有任务完成
        for _, future := range futures {
            var result BatchResult
            err := future.Get(ctx, &result)
            if err != nil {
                logger.Error("Batch processing failed", "error", err)
                failed += input.BatchSize
                continue
            }
            
            totalProcessed += result.Processed
            successful += result.Successful
            failed += result.Failed
        }
        
        // 更新进度
        progress := float64(totalProcessed) / float64(input.TotalItems) * 100.0
        
        workflow.RecordHeartbeat(ctx, map[string]interface{}{
            "batch_id":   input.BatchID,
            "progress":   progress,
            "processed":  totalProcessed,
            "successful": successful,
            "failed":     failed,
        })
        
        logger.Info("Batch progress update",
            "processed", totalProcessed,
            "progress", progress)
    }
    
    duration := workflow.Now(ctx).Sub(startTime)
    
    return BatchProcessingOutput{
        BatchID:        input.BatchID,
        TotalProcessed: totalProcessed,
        Successful:     successful,
        Failed:         failed,
        DurationSecs:   int64(duration.Seconds()),
    }, nil
}
```

---

## 📊 进度跟踪示例

### 场景：实时进度报告

通过Query实时查询批量任务的进度。

#### Rust实现

```rust
use temporal_rust::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProcessingProgress {
    pub total_items: usize,
    pub processed_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub current_stage: String,
    pub progress_percentage: f64,
}

pub struct BatchWithProgressWorkflow {
    progress: Arc<RwLock<ProcessingProgress>>,
}

impl Workflow for BatchWithProgressWorkflow {
    type Input = BatchProcessingInput;
    type Output = BatchProcessingOutput;
    
    fn name() -> &'static str {
        "BatchWithProgress"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let progress = Arc::new(RwLock::new(ProcessingProgress {
            total_items: input.total_items,
            processed_items: 0,
            successful_items: 0,
            failed_items: 0,
            current_stage: "Initializing".to_string(),
            progress_percentage: 0.0,
        }));
        
        // 注册进度Query
        let progress_clone = progress.clone();
        ctx.on_query::<ProgressQuery>(move || {
            let progress = progress_clone.clone();
            async move {
                Ok(progress.read().await.clone())
            }
        });
        
        // 更新阶段
        {
            let mut p = progress.write().await;
            p.current_stage = "Processing batches".to_string();
        }
        
        let num_batches = (input.total_items + input.batch_size - 1) / input.batch_size;
        
        for batch_idx in 0..num_batches {
            let offset = batch_idx * input.batch_size;
            let limit = std::cmp::min(input.batch_size, input.total_items - offset);
            
            let result = ctx.execute_activity::<ProcessBatchActivity>(
                BatchInput {
                    batch_id: input.batch_id.clone(),
                    batch_index: batch_idx,
                    source_file: input.source_file.clone(),
                    offset,
                    limit,
                },
                ActivityOptions::default(),
            ).await?;
            
            // 更新进度
            {
                let mut p = progress.write().await;
                p.processed_items += result.processed;
                p.successful_items += result.successful;
                p.failed_items += result.failed;
                p.progress_percentage = (p.processed_items as f64 / input.total_items as f64) * 100.0;
            }
        }
        
        // 完成
        {
            let mut p = progress.write().await;
            p.current_stage = "Completed".to_string();
        }
        
        let final_progress = progress.read().await.clone();
        
        Ok(BatchProcessingOutput {
            batch_id: input.batch_id,
            total_processed: final_progress.processed_items,
            successful: final_progress.successful_items,
            failed: final_progress.failed_items,
            duration_seconds: 0,
        })
    }
}

// 进度Query
pub struct ProgressQuery;

impl Query for ProgressQuery {
    fn name() -> &'static str {
        "progress"
    }
    
    type Result = ProcessingProgress;
}
```

---

## 🔄 动态任务分配示例

### 场景：根据Worker负载动态调整并发度

#### Rust实现

```rust
#[derive(Serialize, Deserialize)]
pub struct DynamicBatchInput {
    pub batch_id: String,
    pub total_items: usize,
    pub min_batch_size: usize,
    pub max_batch_size: usize,
}

pub struct DynamicBatchWorkflow;

impl Workflow for DynamicBatchWorkflow {
    type Input = DynamicBatchInput;
    type Output = BatchProcessingOutput;
    
    fn name() -> &'static str {
        "DynamicBatch"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let mut processed = 0;
        let mut successful = 0;
        let mut failed = 0;
        
        while processed < input.total_items {
            // 查询当前系统负载
            let load_info = ctx.execute_activity::<GetSystemLoadActivity>(
                (),
                ActivityOptions::default(),
            ).await?;
            
            // 根据负载动态调整批次大小
            let batch_size = if load_info.cpu_usage < 0.5 {
                input.max_batch_size
            } else if load_info.cpu_usage < 0.8 {
                (input.min_batch_size + input.max_batch_size) / 2
            } else {
                input.min_batch_size
            };
            
            let remaining = input.total_items - processed;
            let current_batch_size = std::cmp::min(batch_size, remaining);
            
            tracing::info!(
                batch_size = current_batch_size,
                cpu_usage = load_info.cpu_usage,
                "Processing batch with dynamic size"
            );
            
            let result = ctx.execute_activity::<ProcessBatchActivity>(
                BatchInput {
                    batch_id: input.batch_id.clone(),
                    batch_index: processed / input.min_batch_size,
                    source_file: "data.json".to_string(),
                    offset: processed,
                    limit: current_batch_size,
                },
                ActivityOptions::default(),
            ).await?;
            
            processed += result.processed;
            successful += result.successful;
            failed += result.failed;
        }
        
        Ok(BatchProcessingOutput {
            batch_id: input.batch_id,
            total_processed: processed,
            successful,
            failed,
            duration_seconds: 0,
        })
    }
}

pub struct GetSystemLoadActivity;

#[derive(Serialize, Deserialize)]
pub struct SystemLoadInfo {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

impl Activity for GetSystemLoadActivity {
    type Input = ();
    type Output = SystemLoadInfo;
    
    fn name() -> &'static str {
        "GetSystemLoad"
    }
    
    async fn execute(
        _ctx: ActivityContext,
        _input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        // 实际实现：获取系统负载信息
        Ok(SystemLoadInfo {
            cpu_usage: 0.6,
            memory_usage: 0.5,
        })
    }
}
```

---

## 📚 总结

### 批量处理优势

1. **并行处理**: 提高处理速度
2. **进度跟踪**: 实时了解处理状态
3. **容错性**: 单个批次失败不影响其他
4. **动态调整**: 根据系统负载自适应

### Rust vs Golang

| 特性 | Rust | Golang |
|------|------|--------|
| **并行执行** | `futures::join_all` | `workflow.Future` |
| **进度跟踪** | `Arc<RwLock<Progress>>` | Channel + Selector |
| **类型安全** | 编译时检查 | 运行时检查 |
| **性能** | 零成本抽象 | GC影响 |

---

## 📚 下一步

- **微服务编排**: [服务协调](./22_microservices.md)
- **定时任务**: [调度管理](./23_scheduled_tasks.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队

