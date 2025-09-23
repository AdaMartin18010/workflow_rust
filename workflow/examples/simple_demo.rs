//! # 简单演示 / Simple Demo
//!
//! 本示例展示了 Rust 1.90 工作流系统的基本功能
//! This example demonstrates basic functionality of the Rust 1.90 workflow system

use workflow::rust190::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rust 1.90 工作流系统演示 / Rust 1.90 Workflow System Demo");
    println!("========================================================");

    // 1. JIT 优化处理器演示 / JIT Optimized Processor Demo
    println!("\n1. JIT 优化处理器演示 / JIT Optimized Processor Demo");
    let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]);
    let result = processor.process_data();
    println!("   处理结果 / Processing result: {:?}", result);
    println!("   处理数量 / Processed count: {}", result.len());

    // 2. 异步流处理器演示 / Async Stream Processor Demo
    println!("\n2. 异步流处理器演示 / Async Stream Processor Demo");
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(10));
    
    // 添加测试数据 / Add test data
    stream_processor.add_data(AsyncData {
        id: 1,
        content: "task_a".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 1,
    });
    
    stream_processor.add_data(AsyncData {
        id: 2,
        content: "task_b".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 2,
    });

    println!("   异步流处理开始... / Async stream processing started...");
    let results = stream_processor.create_stream().await;
    println!("   异步流处理结果 / Async stream processing results: {} items", results.len());

    // 3. 性能监控演示 / Performance Monitor Demo
    println!("\n3. 性能监控演示 / Performance Monitor Demo");
    let monitor = PerformanceMonitor::new();
    
    let metrics = PerformanceMetrics {
        operation_name: "demo_operation".to_string(),
        execution_time: Duration::from_millis(50),
        memory_usage: 2048,
        cpu_usage: 30.5,
        throughput: 1000.0,
        error_count: 0,
    };
    
    monitor.record_metrics(metrics).await;
    let stats = monitor.get_overall_stats().await;
    println!("   性能统计 / Performance stats: {} operations recorded", stats.total_operations);

    // 4. const 特性演示 / const Features Demo
    println!("\n4. const 特性演示 / const Features Demo");
    let config = ConstContextProcessor::create_config();
    println!("   const 配置 / const config: {:?}", config);
    
    let data = [1, 2, 3, 4, 5];
    let sum = ConstContextProcessor::process_const_data(&data);
    println!("   const 数据处理结果 / const data processing result: {}", sum);

    println!("\n✅ 演示完成 / Demo completed successfully");
    println!("🎉 Rust 1.90 工作流系统运行正常！/ Rust 1.90 Workflow System is working properly!");
    
    Ok(())
}
