# 观测性接入指南（Prometheus + Tracing）

## 指标（Prometheus）

- 默认在 `0.0.0.0:9090` 暴露 Prometheus 指标（见 `workflow/src/main.rs` 的 `init_metrics`）。
- 已内置 HTTP 指标：
  - 计数：`http_requests_total{method, path}`、`http_responses_total{path, status}`
  - 直方图：`http_request_duration_seconds{method, path, status}`
- 已内置引擎指标：
  - 计数：`workflow_register_total`、`workflow_start_total{workflow}`、`workflow_events_processed_total`、`workflow_events_sent_total{kind}`、`workflow_instances_completed_total`、`workflow_errors_total`、`workflow_timeouts_total`
  - 直方图：`workflow_op_duration_seconds{op}`、`workflow_event_duration_seconds`
  - 仪表：`workflow_instances_current`

Prometheus 抓取示例：

```yaml
scrape_configs:
  - job_name: "workflow"
    metrics_path: "/metrics"
    static_configs:
      - targets: ["workflow:9090"]
```

## 日志与 Trace（本地）

- 通过 `RUST_LOG` 控制日志级别，例如：`RUST_LOG=info`。
- HTTP TraceLayer 已添加方法、路径、User-Agent 等 span 字段。

## OTLP/Jaeger（0.30 API）

- 目前代码保持稳定可编译，OTLP 初始化暂缓。
- 若要启用：请提供 Collector 端点（或允许运行构建确认 0.30 API 符号），我将接入 `opentelemetry-otlp 0.30` 的 gRPC 导出器与 Provider。

环境变量说明：

- `OTEL_EXPORTER_OTLP_ENDPOINT`：Collector/Jaeger gRPC 端点，默认建议 `http://jaeger:4317`。

## Docker Compose/ K8s 提示

- Compose：确保 Prometheus 服务能访问 `workflow:9090`。
- K8s：为 Pod 添加 ServiceMonitor 或 PodMonitor，指向 9090 端口。
