//! # 简单示例 / Simple Example
//!
//! 本模块提供了一个简单的工作流示例，展示基本功能
//! This module provides a simple workflow example demonstrating basic functionality

use crate::rust190::{
    JITOptimizedProcessor, AsyncStreamProcessor, AsyncData, 
    ConstContextProcessor, PerformanceMonitor, PerformanceMetrics
};
use std::time::Duration;

/// 运行简单示例 / Run simple example
pub async fn run_simple_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 简单工作流示例 / Simple Workflow Example");
    println!("==========================================");

    // 1. JIT 优化处理器示例 / JIT Optimized Processor Example
    println!("\n1. JIT 优化处理器示例 / JIT Optimized Processor Example");
    let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]);
    let result = processor.process_data();
    println!("   处理结果 / Processing result: {:?}", result);
    println!("   处理数量 / Processed count: {}", result.len());
    println!("   处理时间 / Processing time: <not available>");

    // 2. 异步流处理器示例 / Async Stream Processor Example
    println!("\n2. 异步流处理器示例 / Async Stream Processor Example");
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(10));
    
    // 添加一些测试数据 / Add some test data
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

    // 3. 性能监控器示例 / Performance Monitor Example
    println!("\n3. 性能监控器示例 / Performance Monitor Example");
    let monitor = PerformanceMonitor::new();
    
    let metrics = PerformanceMetrics {
        operation_name: "data_processing".to_string(),
        execution_time: Duration::from_millis(50),
        memory_usage: 2048,
        cpu_usage: 30.5,
        throughput: 1000.0,
        error_count: 0,
    };
    
    monitor.record_metrics(metrics).await;
    let stats = monitor.get_overall_stats().await;
    println!("   性能统计 / Performance stats: {} operations recorded", stats.total_operations);

    // 4. const 特性示例 / const Features Example
    println!("\n4. const 特性示例 / const Features Example");
    let _const_processor = ConstContextProcessor::new();
    let config = ConstContextProcessor::create_config();
    println!("   const 配置 / const config: {:?}", config);
    
    let data = [1, 2, 3, 4, 5];
    let sum = ConstContextProcessor::process_const_data(&data);
    println!("   const 数据处理结果 / const data processing result: {}", sum);

    println!("\n✅ 简单示例运行完成 / Simple example completed successfully");
    Ok(())
}
