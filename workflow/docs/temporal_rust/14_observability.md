# 可观测性

## 📋 文档概述

本文档详细阐述Temporal工作流系统的可观测性实现，包括：

- 指标收集（Metrics）
- 分布式追踪（Tracing）
- 日志记录（Logging）
- Rust 1.90实现
- Golang实现对比
- 最佳实践

---

## 🎯 可观测性三大支柱

```text
┌─────────────────────────────────────────────────────────────┐
│                     可观测性架构                             │
└─────────────────────────────────────────────────────────────┘

Application
    │
    ├─ Metrics ──────────┐
    │                     │
    ├─ Traces ───────────┼────────┐
    │                     │        │
    ├─ Logs ─────────────┼────────┼────┐
    │                     │        │    │
    │                     ▼        ▼    ▼
    │              ┌──────────────────────────┐
    │              │   Observability Layer     │
    │              │                           │
    │              │  ┌─────────────────────┐  │
    │              │  │  Prometheus         │  │  ← Metrics
    │              │  │  (metrics)          │  │
    │              │  └─────────────────────┘  │
    │              │                           │
    │              │  ┌─────────────────────┐  │
    │              │  │  Jaeger/Tempo       │  │  ← Traces
    │              │  │  (traces)           │  │
    │              │  └─────────────────────┘  │
    │              │                           │
    │              │  ┌─────────────────────┐  │
    │              │  │  Loki/ELK           │  │  ← Logs
    │              │  │  (logs)             │  │
    │              │  └─────────────────────┘  │
    │              └──────────────────────────┘
    │                           │
    │                           ▼
    │                    Grafana Dashboard
```

---

## 🦀 Rust实现

### 1. 指标收集（Metrics）

#### 指标定义

```rust
use prometheus::{
    Registry, Counter, Gauge, Histogram,
    HistogramOpts, Opts, IntCounter, IntGauge,
};
use std::sync::Arc;

/// 工作流指标
pub struct WorkflowMetrics {
    /// 注册表
    registry: Registry,
    
    // Workflow指标
    pub workflows_started_total: IntCounter,
    pub workflows_completed_total: IntCounter,
    pub workflows_failed_total: IntCounter,
    pub workflows_cancelled_total: IntCounter,
    pub workflows_in_progress: IntGauge,
    pub workflow_duration_seconds: Histogram,
    
    // Activity指标
    pub activities_started_total: IntCounter,
    pub activities_completed_total: IntCounter,
    pub activities_failed_total: IntCounter,
    pub activity_duration_seconds: Histogram,
    
    // Worker指标
    pub worker_task_slots_available: IntGauge,
    pub worker_task_slots_used: IntGauge,
    pub worker_task_execution_latency: Histogram,
    
    // Client指标
    pub client_requests_total: IntCounter,
    pub client_errors_total: IntCounter,
    pub client_request_latency: Histogram,
}

impl WorkflowMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // Workflow指标
        let workflows_started_total = IntCounter::with_opts(
            Opts::new("workflows_started_total", "Total number of workflows started")
        )?;
        registry.register(Box::new(workflows_started_total.clone()))?;
        
        let workflows_completed_total = IntCounter::with_opts(
            Opts::new("workflows_completed_total", "Total number of workflows completed")
        )?;
        registry.register(Box::new(workflows_completed_total.clone()))?;
        
        let workflows_failed_total = IntCounter::with_opts(
            Opts::new("workflows_failed_total", "Total number of workflows failed")
        )?;
        registry.register(Box::new(workflows_failed_total.clone()))?;
        
        let workflows_cancelled_total = IntCounter::with_opts(
            Opts::new("workflows_cancelled_total", "Total number of workflows cancelled")
        )?;
        registry.register(Box::new(workflows_cancelled_total.clone()))?;
        
        let workflows_in_progress = IntGauge::with_opts(
            Opts::new("workflows_in_progress", "Number of workflows currently in progress")
        )?;
        registry.register(Box::new(workflows_in_progress.clone()))?;
        
        let workflow_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("workflow_duration_seconds", "Workflow execution duration")
                .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0])
        )?;
        registry.register(Box::new(workflow_duration_seconds.clone()))?;
        
        // Activity指标
        let activities_started_total = IntCounter::with_opts(
            Opts::new("activities_started_total", "Total number of activities started")
        )?;
        registry.register(Box::new(activities_started_total.clone()))?;
        
        let activities_completed_total = IntCounter::with_opts(
            Opts::new("activities_completed_total", "Total number of activities completed")
        )?;
        registry.register(Box::new(activities_completed_total.clone()))?;
        
        let activities_failed_total = IntCounter::with_opts(
            Opts::new("activities_failed_total", "Total number of activities failed")
        )?;
        registry.register(Box::new(activities_failed_total.clone()))?;
        
        let activity_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("activity_duration_seconds", "Activity execution duration")
                .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0])
        )?;
        registry.register(Box::new(activity_duration_seconds.clone()))?;
        
        // Worker指标
        let worker_task_slots_available = IntGauge::with_opts(
            Opts::new("worker_task_slots_available", "Available task execution slots")
        )?;
        registry.register(Box::new(worker_task_slots_available.clone()))?;
        
        let worker_task_slots_used = IntGauge::with_opts(
            Opts::new("worker_task_slots_used", "Used task execution slots")
        )?;
        registry.register(Box::new(worker_task_slots_used.clone()))?;
        
        let worker_task_execution_latency = Histogram::with_opts(
            HistogramOpts::new("worker_task_execution_latency", "Task execution latency")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
        )?;
        registry.register(Box::new(worker_task_execution_latency.clone()))?;
        
        // Client指标
        let client_requests_total = IntCounter::with_opts(
            Opts::new("client_requests_total", "Total client requests")
        )?;
        registry.register(Box::new(client_requests_total.clone()))?;
        
        let client_errors_total = IntCounter::with_opts(
            Opts::new("client_errors_total", "Total client errors")
        )?;
        registry.register(Box::new(client_errors_total.clone()))?;
        
        let client_request_latency = Histogram::with_opts(
            HistogramOpts::new("client_request_latency", "Client request latency")
                .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        )?;
        registry.register(Box::new(client_request_latency.clone()))?;
        
        Ok(Self {
            registry,
            workflows_started_total,
            workflows_completed_total,
            workflows_failed_total,
            workflows_cancelled_total,
            workflows_in_progress,
            workflow_duration_seconds,
            activities_started_total,
            activities_completed_total,
            activities_failed_total,
            activity_duration_seconds,
            worker_task_slots_available,
            worker_task_slots_used,
            worker_task_execution_latency,
            client_requests_total,
            client_errors_total,
            client_request_latency,
        })
    }
    
    /// 获取注册表
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// 指标HTTP端点
pub async fn metrics_handler(
    Extension(metrics): Extension<Arc<WorkflowMetrics>>,
) -> impl IntoResponse {
    use prometheus::Encoder;
    
    let encoder = prometheus::TextEncoder::new();
    let metric_families = metrics.registry().gather();
    
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        buffer,
    )
}
```

#### 指标使用

```rust
impl WorkflowWorker {
    async fn execute_workflow_with_metrics(
        &self,
        task: WorkflowTask,
    ) -> Result<(), WorkerError> {
        let metrics = &self.metrics;
        
        // 记录开始
        metrics.workflows_started_total.inc();
        metrics.workflows_in_progress.inc();
        let timer = metrics.workflow_duration_seconds.start_timer();
        
        // 执行工作流
        let result = self.execute_workflow_internal(task).await;
        
        // 记录结束
        metrics.workflows_in_progress.dec();
        timer.observe_duration();
        
        match result {
            Ok(_) => {
                metrics.workflows_completed_total.inc();
            }
            Err(WorkerError::WorkflowFailed(_)) => {
                metrics.workflows_failed_total.inc();
            }
            Err(WorkerError::WorkflowCancelled) => {
                metrics.workflows_cancelled_total.inc();
            }
            Err(_) => {
                metrics.workflows_failed_total.inc();
            }
        }
        
        result
    }
}
```

### 2. 分布式追踪（Tracing）

#### OpenTelemetry集成

```rust
use opentelemetry::{
    global,
    trace::{Tracer, SpanKind, Status},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{info, warn, error, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 初始化追踪
pub fn init_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 配置OpenTelemetry
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                ]))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    // 配置tracing订阅者
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    
    Ok(())
}

/// 关闭追踪
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}
```

#### Workflow追踪

```rust
use tracing::{instrument, Span};

impl WorkflowWorker {
    #[instrument(
        name = "execute_workflow",
        skip(self, task),
        fields(
            workflow_id = %task.workflow_id,
            workflow_type = %task.workflow_type,
            run_id = %task.run_id,
        )
    )]
    async fn execute_workflow(
        &self,
        task: WorkflowTask,
    ) -> Result<(), WorkerError> {
        let span = Span::current();
        
        span.record("workflow_id", &task.workflow_id.as_str());
        
        info!("Executing workflow");
        
        // 执行工作流逻辑
        let result = self.execute_workflow_internal(task).await;
        
        match &result {
            Ok(_) => {
                span.record("status", "completed");
                info!("Workflow completed successfully");
            }
            Err(e) => {
                span.record("status", "failed");
                span.record("error", &format!("{:?}", e));
                error!("Workflow failed: {:?}", e);
            }
        }
        
        result
    }
    
    #[instrument(
        name = "execute_activity",
        skip(self, task),
        fields(
            activity_id = %task.activity_id,
            activity_type = %task.activity_type,
        )
    )]
    async fn execute_activity(
        &self,
        task: ActivityTask,
    ) -> Result<serde_json::Value, WorkerError> {
        info!("Executing activity");
        
        let result = self.execute_activity_internal(task).await;
        
        match &result {
            Ok(_) => info!("Activity completed"),
            Err(e) => error!("Activity failed: {:?}", e),
        }
        
        result
    }
}
```

#### 客户端追踪

```rust
impl WorkflowClient {
    #[instrument(
        name = "start_workflow",
        skip(self, input),
        fields(
            workflow_type = W::name(),
            workflow_id,
        )
    )]
    pub async fn start_workflow<W: Workflow>(
        &self,
        input: W::Input,
        options: StartWorkflowOptions,
    ) -> Result<WorkflowHandle<W>, ClientError> {
        let workflow_id = options.workflow_id
            .unwrap_or_else(|| WorkflowId::new(format!("wf-{}", uuid::Uuid::new_v4())));
        
        Span::current().record("workflow_id", &workflow_id.as_str());
        
        info!("Starting workflow");
        
        // 实现启动逻辑...
        
        info!("Workflow started successfully");
        
        Ok(WorkflowHandle {
            client: self.clone(),
            execution: WorkflowExecution::with_run_id(workflow_id, RunId::new()),
            _phantom: std::marker::PhantomData,
        })
    }
}
```

### 3. 结构化日志（Logging）

#### 日志配置

```rust
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .json() // JSON格式输出
        .init();
}
```

#### 结构化日志示例

```rust
use tracing::{info, warn, error, debug};

impl OrderProcessingWorkflow {
    async fn execute(
        ctx: WorkflowContext,
        order: Order,
    ) -> Result<OrderResult, WorkflowError> {
        info!(
            order_id = %order.order_id,
            user_id = %order.user_id,
            total_amount = order.total_amount,
            "Starting order processing"
        );
        
        // 验证订单
        debug!(order_id = %order.order_id, "Validating order");
        let validation = ctx.execute_activity::<ValidateOrderActivity>(
            ValidateOrderInput { order: order.clone() },
            ActivityOptions::default(),
        ).await?;
        
        if !validation.is_valid {
            warn!(
                order_id = %order.order_id,
                errors = ?validation.validation_errors,
                "Order validation failed"
            );
            return Ok(OrderResult {
                order_id: order.order_id,
                status: OrderStatus::Failed {
                    reason: validation.validation_errors.join(", "),
                },
                ..Default::default()
            });
        }
        
        // 处理支付
        info!(order_id = %order.order_id, "Processing payment");
        match ctx.execute_activity::<ProcessPaymentActivity>(
            ProcessPaymentInput {
                order_id: order.order_id.clone(),
                amount: order.total_amount,
                payment_method: order.payment_method.clone(),
            },
            ActivityOptions::default(),
        ).await {
            Ok(payment) => {
                info!(
                    order_id = %order.order_id,
                    payment_id = %payment.payment_id,
                    "Payment processed successfully"
                );
            }
            Err(e) => {
                error!(
                    order_id = %order.order_id,
                    error = ?e,
                    "Payment processing failed"
                );
                return Err(e);
            }
        }
        
        info!(order_id = %order.order_id, "Order processing completed");
        Ok(OrderResult {
            order_id: order.order_id,
            status: OrderStatus::Completed,
            completed_at: Some(Utc::now()),
            ..Default::default()
        })
    }
}
```

---

## 🐹 Golang实现对比

### 指标收集 - Golang

```go
package metrics

import (
    "github.com/prometheus/client_golang/prometheus"
    "github.com/prometheus/client_golang/prometheus/promauto"
)

type WorkflowMetrics struct {
    WorkflowsStarted   prometheus.Counter
    WorkflowsCompleted prometheus.Counter
    WorkflowsFailed    prometheus.Counter
    WorkflowDuration   prometheus.Histogram
    
    ActivitiesStarted   prometheus.Counter
    ActivitiesCompleted prometheus.Counter
    ActivitiesFailed    prometheus.Counter
    ActivityDuration    prometheus.Histogram
}

func NewWorkflowMetrics() *WorkflowMetrics {
    return &WorkflowMetrics{
        WorkflowsStarted: promauto.NewCounter(prometheus.CounterOpts{
            Name: "workflows_started_total",
            Help: "Total number of workflows started",
        }),
        WorkflowsCompleted: promauto.NewCounter(prometheus.CounterOpts{
            Name: "workflows_completed_total",
            Help: "Total number of workflows completed",
        }),
        WorkflowsFailed: promauto.NewCounter(prometheus.CounterOpts{
            Name: "workflows_failed_total",
            Help: "Total number of workflows failed",
        }),
        WorkflowDuration: promauto.NewHistogram(prometheus.HistogramOpts{
            Name: "workflow_duration_seconds",
            Help: "Workflow execution duration",
            Buckets: []float64{0.1, 0.5, 1, 5, 10, 30, 60, 300, 600},
        }),
        // ... 其他指标
    }
}
```

### 分布式追踪 - Golang

```go
package main

import (
    "context"
    
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
    "go.opentelemetry.io/otel/sdk/trace"
    "go.temporal.io/sdk/client"
    "go.temporal.io/sdk/interceptor"
)

func main() {
    // 初始化OpenTelemetry
    ctx := context.Background()
    
    exporter, err := otlptracegrpc.New(ctx,
        otlptracegrpc.WithEndpoint("localhost:4317"),
        otlptracegrpc.WithInsecure(),
    )
    if err != nil {
        log.Fatal(err)
    }
    
    tp := trace.NewTracerProvider(
        trace.WithBatcher(exporter),
    )
    otel.SetTracerProvider(tp)
    
    // 创建Temporal客户端（带追踪）
    c, err := client.Dial(client.Options{
        Interceptors: []interceptor.ClientInterceptor{
            interceptor.NewTracingInterceptor(interceptor.TracerOptions{}),
        },
    })
    if err != nil {
        log.Fatal(err)
    }
    defer c.Close()
}
```

---

## 🎯 最佳实践

### 1. 指标命名规范

```rust
// ✅ 好: 遵循Prometheus命名约定
// 格式: <namespace>_<subsystem>_<name>_<unit>
workflows_started_total          // Counter
workflows_in_progress           // Gauge
workflow_duration_seconds       // Histogram
activity_execution_failures_total // Counter

// ❌ 不好: 不规范的命名
WorkflowsCount
workflow-duration
activity_exec_time_ms
```

### 2. 日志级别使用

```rust
// ✅ 好: 合理使用日志级别
use tracing::{error, warn, info, debug, trace};

// ERROR: 错误，需要立即处理
error!(error = ?e, "Failed to connect to database");

// WARN: 警告，需要关注但不影响主流程
warn!(order_id = %id, "Order already processed, skipping");

// INFO: 重要信息，生产环境默认级别
info!(workflow_id = %wf_id, "Workflow started");

// DEBUG: 调试信息，开发和排查问题时使用
debug!(state = ?current_state, "State transition");

// TRACE: 详细追踪信息，仅在需要详细日志时使用
trace!(input = ?data, "Processing input");
```

### 3. 追踪上下文传递

```rust
// ✅ 好: 自动传递追踪上下文
#[instrument(skip(self))]
async fn process_order(&self, order: Order) -> Result<(), Error> {
    // 追踪上下文自动传递到子函数
    self.validate_order(&order).await?;
    self.process_payment(&order).await?;
    self.create_shipment(&order).await?;
    Ok(())
}

#[instrument(skip(order))]
async fn validate_order(&self, order: &Order) -> Result<(), Error> {
    // 这个函数的span会自动成为上层span的子span
    info!("Validating order");
    Ok(())
}
```

### 4. 指标标签使用

```rust
use prometheus::{IntCounterVec, Opts, register_int_counter_vec};

// ✅ 好: 使用标签区分不同维度
let workflows_total = register_int_counter_vec!(
    Opts::new("workflows_total", "Total workflows"),
    &["workflow_type", "status", "namespace"]
)?;

// 记录指标时指定标签
workflows_total
    .with_label_values(&["OrderProcessing", "completed", "production"])
    .inc();

// ❌ 不好: 每个维度创建一个指标（维度爆炸）
let order_workflows_completed = Counter::new(...);
let payment_workflows_completed = Counter::new(...);
let shipment_workflows_completed = Counter::new(...);
```

---

## 📊 监控仪表板

### Grafana Dashboard配置

```json
{
  "dashboard": {
    "title": "Temporal Workflow Metrics",
    "panels": [
      {
        "title": "Workflow Execution Rate",
        "targets": [
          {
            "expr": "rate(workflows_started_total[5m])"
          }
        ]
      },
      {
        "title": "Workflow Success Rate",
        "targets": [
          {
            "expr": "rate(workflows_completed_total[5m]) / rate(workflows_started_total[5m])"
          }
        ]
      },
      {
        "title": "Workflow Duration p99",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(workflow_duration_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Active Workflows",
        "targets": [
          {
            "expr": "workflows_in_progress"
          }
        ]
      }
    ]
  }
}
```

---

## 📚 总结

### 可观测性关键点

1. **指标收集**: 使用Prometheus收集关键指标
2. **分布式追踪**: 使用OpenTelemetry追踪请求链路
3. **结构化日志**: 使用tracing库输出结构化日志
4. **统一规范**: 遵循命名和使用规范
5. **性能考虑**: 避免过度采样和标签爆炸

### Rust vs Golang

- **Rust**: prometheus + tracing + opentelemetry
- **Golang**: prometheus + opentelemetry (Temporal内置支持)

---

## 📚 下一步

- **部署指南**: [生产部署](./15_deployment.md)
- **最佳实践**: [设计原则](./16_best_practices.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
