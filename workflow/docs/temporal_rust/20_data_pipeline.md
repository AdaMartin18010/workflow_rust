# 数据管道示例

## 📋 文档概述

本文档提供数据管道相关的示例，包括：

- ETL（Extract, Transform, Load）流程
- 流式数据处理
- 数据同步
- 批量导入
- 数据验证和清洗
- Rust + Golang并列对比

---

## 🔄 ETL管道示例

### 场景：从多个数据源提取、转换并加载数据

#### Rust实现

```rust
use temporal_rust::*;
use serde::{Serialize, Deserialize};

// ========================================
// ETL管道工作流
// ========================================

#[derive(Serialize, Deserialize)]
pub struct ETLInput {
    pub pipeline_id: String,
    pub sources: Vec<DataSource>,
    pub target: DataTarget,
    pub batch_size: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataSource {
    pub source_id: String,
    pub source_type: String, // "database", "api", "file"
    pub connection_string: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataTarget {
    pub target_type: String,
    pub connection_string: String,
}

#[derive(Serialize, Deserialize)]
pub struct ETLOutput {
    pub pipeline_id: String,
    pub total_records: usize,
    pub successful_records: usize,
    pub failed_records: usize,
    pub duration_seconds: u64,
}

pub struct ETLPipelineWorkflow;

impl Workflow for ETLPipelineWorkflow {
    type Input = ETLInput;
    type Output = ETLOutput;
    
    fn name() -> &'static str {
        "ETLPipeline"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        let start_time = ctx.now();
        
        tracing::info!(
            pipeline_id = %input.pipeline_id,
            sources = input.sources.len(),
            "Starting ETL pipeline"
        );
        
        let mut total_records = 0;
        let mut successful_records = 0;
        let mut failed_records = 0;
        
        // 1. Extract阶段 - 并行从多个源提取数据
        let extract_futures: Vec<_> = input.sources.iter().map(|source| {
            ctx.execute_activity::<ExtractDataActivity>(
                ExtractInput {
                    source: source.clone(),
                    batch_size: input.batch_size,
                },
                ActivityOptions {
                    start_to_close_timeout: Some(Duration::from_secs(600)),
                    ..Default::default()
                },
            )
        }).collect();
        
        let extract_results = futures::future::join_all(extract_futures).await;
        
        // 收集所有提取的数据
        let mut all_records = Vec::new();
        for result in extract_results {
            match result {
                Ok(data) => {
                    total_records += data.records.len();
                    all_records.extend(data.records);
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Extract failed");
                    return Err(e);
                }
            }
        }
        
        tracing::info!(
            total_records = total_records,
            "Extraction completed"
        );
        
        // 2. Transform阶段 - 分批转换数据
        let mut transformed_records = Vec::new();
        
        for chunk in all_records.chunks(input.batch_size) {
            let transform_result = ctx.execute_activity::<TransformDataActivity>(
                TransformInput {
                    records: chunk.to_vec(),
                },
                ActivityOptions {
                    start_to_close_timeout: Some(Duration::from_secs(300)),
                    retry_policy: Some(RetryPolicy {
                        max_attempts: Some(3),
                        initial_interval: Duration::from_secs(1),
                        max_interval: Duration::from_secs(10),
                        backoff_coefficient: 2.0,
                        non_retryable_error_types: vec![],
                    }),
                    ..Default::default()
                },
            ).await?;
            
            transformed_records.extend(transform_result.records);
            
            // 发送心跳（长时间运行）
            ctx.record_heartbeat(serde_json::json!({
                "phase": "transform",
                "processed": transformed_records.len(),
                "total": total_records
            })).await;
        }
        
        tracing::info!(
            transformed_records = transformed_records.len(),
            "Transformation completed"
        );
        
        // 3. Load阶段 - 分批加载到目标
        for chunk in transformed_records.chunks(input.batch_size) {
            match ctx.execute_activity::<LoadDataActivity>(
                LoadInput {
                    target: input.target.clone(),
                    records: chunk.to_vec(),
                },
                ActivityOptions {
                    start_to_close_timeout: Some(Duration::from_secs(300)),
                    retry_policy: Some(RetryPolicy {
                        max_attempts: Some(5),
                        initial_interval: Duration::from_secs(2),
                        max_interval: Duration::from_secs(30),
                        backoff_coefficient: 2.0,
                        non_retryable_error_types: vec!["ValidationError"],
                    }),
                    ..Default::default()
                },
            ).await {
                Ok(result) => {
                    successful_records += result.loaded_count;
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Load failed for batch");
                    failed_records += chunk.len();
                    // 继续处理其他批次
                }
            }
            
            // 发送心跳
            ctx.record_heartbeat(serde_json::json!({
                "phase": "load",
                "loaded": successful_records,
                "total": total_records
            })).await;
        }
        
        let end_time = ctx.now();
        let duration = end_time.signed_duration_since(start_time);
        
        tracing::info!(
            successful = successful_records,
            failed = failed_records,
            duration_secs = duration.num_seconds(),
            "ETL pipeline completed"
        );
        
        Ok(ETLOutput {
            pipeline_id: input.pipeline_id,
            total_records,
            successful_records,
            failed_records,
            duration_seconds: duration.num_seconds() as u64,
        })
    }
}

// ========================================
// Activity定义
// ========================================

pub struct ExtractDataActivity;

#[derive(Serialize, Deserialize)]
pub struct ExtractInput {
    pub source: DataSource,
    pub batch_size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ExtractOutput {
    pub records: Vec<RawRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RawRecord {
    pub id: String,
    pub data: serde_json::Value,
}

impl Activity for ExtractDataActivity {
    type Input = ExtractInput;
    type Output = ExtractOutput;
    
    fn name() -> &'static str {
        "ExtractData"
    }
    
    async fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(
            source_id = %input.source.source_id,
            source_type = %input.source.source_type,
            "Extracting data"
        );
        
        // 根据源类型提取数据
        let records = match input.source.source_type.as_str() {
            "database" => extract_from_database(&input.source, input.batch_size).await?,
            "api" => extract_from_api(&input.source, input.batch_size).await?,
            "file" => extract_from_file(&input.source, input.batch_size).await?,
            _ => return Err(ActivityError::invalid_input("Unsupported source type")),
        };
        
        // 定期发送心跳
        ctx.record_heartbeat(serde_json::json!({
            "extracted": records.len()
        })).await;
        
        Ok(ExtractOutput { records })
    }
}

pub struct TransformDataActivity;

#[derive(Serialize, Deserialize)]
pub struct TransformInput {
    pub records: Vec<RawRecord>,
}

#[derive(Serialize, Deserialize)]
pub struct TransformOutput {
    pub records: Vec<TransformedRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransformedRecord {
    pub id: String,
    pub data: serde_json::Value,
    pub metadata: RecordMetadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordMetadata {
    pub processed_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

impl Activity for TransformDataActivity {
    type Input = TransformInput;
    type Output = TransformOutput;
    
    fn name() -> &'static str {
        "TransformData"
    }
    
    async fn execute(
        _ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(
            records = input.records.len(),
            "Transforming data"
        );
        
        let mut transformed_records = Vec::new();
        
        for record in input.records {
            // 数据转换逻辑
            let transformed = transform_record(record)?;
            transformed_records.push(transformed);
        }
        
        Ok(TransformOutput {
            records: transformed_records,
        })
    }
}

pub struct LoadDataActivity;

#[derive(Serialize, Deserialize)]
pub struct LoadInput {
    pub target: DataTarget,
    pub records: Vec<TransformedRecord>,
}

#[derive(Serialize, Deserialize)]
pub struct LoadOutput {
    pub loaded_count: usize,
}

impl Activity for LoadDataActivity {
    type Input = LoadInput;
    type Output = LoadOutput;
    
    fn name() -> &'static str {
        "LoadData"
    }
    
    async fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output, ActivityError> {
        tracing::info!(
            target_type = %input.target.target_type,
            records = input.records.len(),
            "Loading data"
        );
        
        // 批量写入目标
        let loaded_count = match input.target.target_type.as_str() {
            "database" => load_to_database(&input.target, &input.records).await?,
            "warehouse" => load_to_warehouse(&input.target, &input.records).await?,
            "file" => load_to_file(&input.target, &input.records).await?,
            _ => return Err(ActivityError::invalid_input("Unsupported target type")),
        };
        
        ctx.record_heartbeat(serde_json::json!({
            "loaded": loaded_count
        })).await;
        
        Ok(LoadOutput { loaded_count })
    }
}

// 辅助函数
async fn extract_from_database(
    source: &DataSource,
    batch_size: usize,
) -> Result<Vec<RawRecord>, ActivityError> {
    // 实际实现：连接数据库并查询
    Ok(vec![])
}

async fn extract_from_api(
    source: &DataSource,
    batch_size: usize,
) -> Result<Vec<RawRecord>, ActivityError> {
    // 实际实现：调用API并获取数据
    Ok(vec![])
}

async fn extract_from_file(
    source: &DataSource,
    batch_size: usize,
) -> Result<Vec<RawRecord>, ActivityError> {
    // 实际实现：读取文件
    Ok(vec![])
}

fn transform_record(record: RawRecord) -> Result<TransformedRecord, ActivityError> {
    // 实际实现：数据转换逻辑
    Ok(TransformedRecord {
        id: record.id,
        data: record.data,
        metadata: RecordMetadata {
            processed_at: chrono::Utc::now(),
            version: "1.0".to_string(),
        },
    })
}

async fn load_to_database(
    target: &DataTarget,
    records: &[TransformedRecord],
) -> Result<usize, ActivityError> {
    // 实际实现：批量插入数据库
    Ok(records.len())
}

async fn load_to_warehouse(
    target: &DataTarget,
    records: &[TransformedRecord],
) -> Result<usize, ActivityError> {
    // 实际实现：加载到数据仓库
    Ok(records.len())
}

async fn load_to_file(
    target: &DataTarget,
    records: &[TransformedRecord],
) -> Result<usize, ActivityError> {
    // 实际实现：写入文件
    Ok(records.len())
}
```

#### Golang对比

```go
package workflows

import (
    "go.temporal.io/sdk/workflow"
)

type ETLInput struct {
    PipelineID string
    Sources    []DataSource
    Target     DataTarget
    BatchSize  int
}

type ETLOutput struct {
    PipelineID        string
    TotalRecords      int
    SuccessfulRecords int
    FailedRecords     int
    DurationSeconds   int64
}

func ETLPipelineWorkflow(ctx workflow.Context, input ETLInput) (ETLOutput, error) {
    startTime := workflow.Now(ctx)
    logger := workflow.GetLogger(ctx)
    
    logger.Info("Starting ETL pipeline", "pipeline_id", input.PipelineID)
    
    var totalRecords, successfulRecords, failedRecords int
    
    // 1. Extract阶段 - 并行提取
    var extractFutures []workflow.Future
    for _, source := range input.Sources {
        future := workflow.ExecuteActivity(ctx, ExtractDataActivity, source, input.BatchSize)
        extractFutures = append(extractFutures, future)
    }
    
    var allRecords []RawRecord
    for _, future := range extractFutures {
        var extractOutput ExtractOutput
        err := future.Get(ctx, &extractOutput)
        if err != nil {
            return ETLOutput{}, err
        }
        totalRecords += len(extractOutput.Records)
        allRecords = append(allRecords, extractOutput.Records...)
    }
    
    logger.Info("Extraction completed", "total_records", totalRecords)
    
    // 2. Transform阶段
    var transformedRecords []TransformedRecord
    for i := 0; i < len(allRecords); i += input.BatchSize {
        end := i + input.BatchSize
        if end > len(allRecords) {
            end = len(allRecords)
        }
        chunk := allRecords[i:end]
        
        var transformOutput TransformOutput
        err := workflow.ExecuteActivity(ctx, TransformDataActivity, chunk).Get(ctx, &transformOutput)
        if err != nil {
            return ETLOutput{}, err
        }
        
        transformedRecords = append(transformedRecords, transformOutput.Records...)
        
        // 记录心跳
        workflow.RecordHeartbeat(ctx, map[string]interface{}{
            "phase":     "transform",
            "processed": len(transformedRecords),
            "total":     totalRecords,
        })
    }
    
    // 3. Load阶段
    for i := 0; i < len(transformedRecords); i += input.BatchSize {
        end := i + input.BatchSize
        if end > len(transformedRecords) {
            end = len(transformedRecords)
        }
        chunk := transformedRecords[i:end]
        
        var loadOutput LoadOutput
        err := workflow.ExecuteActivity(ctx, LoadDataActivity, input.Target, chunk).Get(ctx, &loadOutput)
        if err != nil {
            logger.Error("Load failed for batch", "error", err)
            failedRecords += len(chunk)
            continue
        }
        
        successfulRecords += loadOutput.LoadedCount
    }
    
    duration := workflow.Now(ctx).Sub(startTime)
    
    return ETLOutput{
        PipelineID:        input.PipelineID,
        TotalRecords:      totalRecords,
        SuccessfulRecords: successfulRecords,
        FailedRecords:     failedRecords,
        DurationSeconds:   int64(duration.Seconds()),
    }, nil
}
```

---

## 🌊 流式数据处理示例

### 场景：实时数据流处理

#### Rust实现1

```rust
use temporal_rust::*;

#[derive(Serialize, Deserialize)]
pub struct StreamProcessingInput {
    pub stream_id: String,
    pub source_topic: String,
    pub target_topic: String,
    pub window_size_seconds: u64,
}

#[derive(Serialize, Deserialize)]
pub struct StreamProcessingOutput {
    pub stream_id: String,
    pub total_events: usize,
    pub processed_windows: usize,
}

pub struct StreamProcessingWorkflow;

impl Workflow for StreamProcessingWorkflow {
    type Input = StreamProcessingInput;
    type Output = StreamProcessingOutput;
    
    fn name() -> &'static str {
        "StreamProcessing"
    }
    
    async fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkflowError> {
        tracing::info!(
            stream_id = %input.stream_id,
            "Starting stream processing workflow"
        );
        
        let mut total_events = 0;
        let mut processed_windows = 0;
        let window_duration = Duration::from_secs(input.window_size_seconds);
        
        loop {
            // 1. 读取时间窗口内的事件
            let events = ctx.execute_activity::<ReadStreamEventsActivity>(
                ReadStreamInput {
                    topic: input.source_topic.clone(),
                    window_size: window_duration,
                },
                ActivityOptions {
                    start_to_close_timeout: Some(Duration::from_secs(60)),
                    ..Default::default()
                },
            ).await?;
            
            if events.events.is_empty() {
                // 没有更多事件，等待
                ctx.sleep(window_duration).await;
                continue;
            }
            
            total_events += events.events.len();
            
            // 2. 处理事件
            let processed = ctx.execute_activity::<ProcessStreamEventsActivity>(
                ProcessStreamInput {
                    events: events.events,
                },
                ActivityOptions::default(),
            ).await?;
            
            // 3. 写入结果
            ctx.execute_activity::<WriteStreamResultsActivity>(
                WriteStreamInput {
                    topic: input.target_topic.clone(),
                    results: processed.results,
                },
                ActivityOptions::default(),
            ).await?;
            
            processed_windows += 1;
            
            // 检查是否应该继续（通过Signal控制）
            if let Some(stop_signal) = ctx.try_receive_signal::<StopStreamingSignal>().await {
                tracing::info!("Received stop signal, terminating stream");
                break;
            }
            
            // Continue As New以避免历史过大
            if processed_windows >= 1000 {
                return ctx.continue_as_new(StreamProcessingInput {
                    stream_id: input.stream_id,
                    source_topic: input.source_topic,
                    target_topic: input.target_topic,
                    window_size_seconds: input.window_size_seconds,
                });
            }
        }
        
        Ok(StreamProcessingOutput {
            stream_id: input.stream_id,
            total_events,
            processed_windows,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct StopStreamingSignal;

impl Signal for StopStreamingSignal {
    fn name() -> &'static str {
        "stop_streaming"
    }
}
```

---

## 📚 总结

### ETL管道优势

1. **可靠性**: 自动重试和错误处理
2. **可观测性**: 详细的进度追踪
3. **可扩展性**: 并行处理多个数据源
4. **容错性**: 部分失败不影响整体流程

### 流式处理优势

1. **实时性**: 持续处理数据流
2. **长时间运行**: Continue As New避免历史过大
3. **可控制**: 通过Signal动态控制
4. **时间窗口**: 支持窗口聚合

---

## 📚 下一步

- **批量任务**: [并行处理](./21_batch_processing.md)
- **微服务编排**: [服务协调](./22_microservices.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
