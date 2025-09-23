//! # 集成测试 / Integration Tests
//!
//! 本模块包含了工作流系统的集成测试
//! This module contains integration tests for the workflow system

use workflow::rust190::*;
use std::time::Duration;
use axum::{Router, body::{Body, to_bytes}, http::{Request, StatusCode}};
use tower::ServiceExt;
use workflow::http::build_router;

#[tokio::test]
async fn test_jit_optimized_processor() {
    let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]);
    let result = processor.process_data();
    
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
    assert_eq!(result.len(), 5);
}

#[tokio::test]
async fn test_async_stream_processor() {
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(1));
    
    // 添加测试数据
    stream_processor.add_data(AsyncData {
        id: 1,
        content: "test_task".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 1,
    });
    
    stream_processor.add_data(AsyncData {
        id: 2,
        content: "another_task".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 0, // 这个应该被过滤掉
    });
    
    let results = stream_processor.create_stream().await;
    
    // 只有 priority > 0 的数据应该被处理
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "TEST_TASK");
}

#[tokio::test]
async fn test_performance_monitor() {
    let monitor = PerformanceMonitor::new();
    
    let metrics = PerformanceMetrics {
        operation_name: "test_operation".to_string(),
        execution_time: Duration::from_millis(100),
        memory_usage: 1024,
        cpu_usage: 50.0,
        throughput: 500.0,
        error_count: 0,
    };
    
    monitor.record_metrics(metrics).await;
    let stats = monitor.get_overall_stats().await;
    
    assert_eq!(stats.total_operations, 1);
    assert_eq!(stats.total_memory_usage, 1024);
    assert_eq!(stats.total_errors, 0);
}

#[tokio::test]
async fn test_const_context_processor() {
    let config = ConstContextProcessor::create_config();
    
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.batch_size, 100);
    assert!(config.enable_logging);
    
    let data = [1, 2, 3, 4, 5];
    let sum = ConstContextProcessor::process_const_data(&data);
    assert_eq!(sum, 15);
}

#[tokio::test]
async fn test_workflow_integration() {
    // 测试完整的工作流集成
    let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3]);
    let result = processor.process_data();
    
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(1));
    stream_processor.add_data(AsyncData {
        id: 1,
        content: "integration_test".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 1,
    });
    
    let monitor = PerformanceMonitor::new();
    let metrics = PerformanceMetrics {
        operation_name: "integration_test".to_string(),
        execution_time: Duration::from_millis(50),
        memory_usage: 512,
        cpu_usage: 25.0,
        throughput: 1000.0,
        error_count: 0,
    };
    
    monitor.record_metrics(metrics).await;
    let stream_results = stream_processor.create_stream().await;
    let stats = monitor.get_overall_stats().await;
    
    // 验证所有组件都正常工作
    assert_eq!(result.len(), 3);
    assert_eq!(stream_results.len(), 1);
    assert_eq!(stats.total_operations, 1);
}

#[tokio::test]
async fn test_http_health_and_version() {
    let app: Router = build_router();

    // /health
    let response = app.clone().oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "OK");

    // /version
    let app2: Router = build_router();
    let response = app2.oneshot(Request::get("/version").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, workflow::VERSION);
}

#[tokio::test]
async fn test_http_stats() {
    let app: Router = workflow::http::build_router();
    let response = app.clone().oneshot(Request::get("/stats").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let s = String::from_utf8(body.to_vec()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v.get("version").and_then(|x| x.as_str()).unwrap(), workflow::VERSION);
}
