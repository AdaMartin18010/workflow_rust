//! # 性能基准测试 / Performance Benchmarks
//!
//! 本模块包含了工作流系统的性能基准测试
//! This module contains performance benchmarks for the workflow system

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use workflow::rust190::*;
use std::time::Duration;

fn benchmark_jit_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_processor");
    
    for size in [10, 100, 1000, 10000].iter() {
        let data: Vec<i32> = (1..=*size).collect();
        group.bench_with_input(BenchmarkId::new("process_data", size), &data, |b, data| {
            b.iter(|| {
                let mut processor = JITOptimizedProcessor::new(data.clone());
                black_box(processor.process_data())
            })
        });
    }
    
    group.finish();
}

fn benchmark_async_stream_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_stream_processor");
    
    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("process_stream", count), count, |b, &count| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                rt.block_on(async {
                    let mut processor = AsyncStreamProcessor::new(Duration::from_millis(1));
                    
                    for i in 0..count {
                        processor.add_data(AsyncData {
                            id: i as u64,
                            content: format!("task_{}", i),
                            timestamp: chrono::Utc::now(),
                            priority: if i % 2 == 0 { 1 } else { 0 },
                        });
                    }
                    
                    black_box(processor.create_stream().await)
                })
            })
        });
    }
    
    group.finish();
}

fn benchmark_performance_monitor(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_monitor");
    
    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("record_metrics", count), count, |b, &count| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                rt.block_on(async {
                    let monitor = PerformanceMonitor::new();
                    
                    for i in 0..count {
                        let metrics = PerformanceMetrics {
                            operation_name: format!("operation_{}", i),
                            execution_time: Duration::from_millis(10),
                            memory_usage: 1024,
                            cpu_usage: 25.0,
                            throughput: 1000.0,
                            error_count: 0,
                        };
                        
                        monitor.record_metrics(metrics).await;
                    }
                    
                    black_box(monitor.get_overall_stats().await)
                })
            })
        });
    }
    
    group.finish();
}

fn benchmark_const_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("const_processor");
    
    for size in [10, 100, 1000, 10000].iter() {
        let data: Vec<i32> = (1..=*size).collect();
        group.bench_with_input(BenchmarkId::new("process_const_data", size), &data, |b, data| {
            b.iter(|| {
                black_box(ConstContextProcessor::process_const_data(data))
            })
        });
    }
    
    group.finish();
}

fn benchmark_workflow_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_integration");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("full_workflow", size), size, |b, &size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                rt.block_on(async {
                    // JIT 处理
                    let data: Vec<i32> = (1..=size).collect();
                    let mut jit_processor = JITOptimizedProcessor::new(data);
                    let jit_result = jit_processor.process_data();
                    
                    // 异步流处理
                    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(1));
                    for i in 0..size {
                        stream_processor.add_data(AsyncData {
                            id: i as u64,
                            content: format!("task_{}", i),
                            timestamp: chrono::Utc::now(),
                            priority: 1,
                        });
                    }
                    let stream_result = stream_processor.create_stream().await;
                    
                    // 性能监控
                    let monitor = PerformanceMonitor::new();
                    let metrics = PerformanceMetrics {
                        operation_name: "integration_benchmark".to_string(),
                        execution_time: Duration::from_millis(50),
                        memory_usage: 2048,
                        cpu_usage: 30.0,
                        throughput: 2000.0,
                        error_count: 0,
                    };
                    monitor.record_metrics(metrics).await;
                    let stats = monitor.get_overall_stats().await;
                    
                    black_box((jit_result, stream_result, stats))
                })
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_jit_processor,
    benchmark_async_stream_processor,
    benchmark_performance_monitor,
    benchmark_const_processor,
    benchmark_workflow_integration
);

criterion_main!(benches);
