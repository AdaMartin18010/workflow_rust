//! # Rust 1.90 特性示例 / Rust 1.90 Features Examples
//!
//! 本模块展示了如何使用 Rust 1.90 的新特性来构建工作流系统。
//! This module demonstrates how to use new features from Rust 1.90 to build workflow systems.

use crate::rust190::{
    JITOptimizedProcessor, PerformanceBenchmark, AsyncStreamProcessor, AsyncData,
    ConstContextProcessor, StableWorkflowDefinition, StableWorkflowStep, StableWorkflowConfig,
    StableAPIWorkflowEngine190, PerformanceWorkflowDefinition, PerformanceWorkflowStep, 
    HighPerformanceWorkflowEngine190, PerformanceMonitor, PerformanceMetrics
};

#[cfg(feature = "session_types")]
use crate::rust190::{
    SessionTypesWorkflowEngine, Participant, ParticipantRole, SessionProtocol
};
use std::time::Duration;

/// 运行 Rust 1.90 特性示例 / Run Rust 1.90 features examples
pub async fn run_rust190_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rust 1.90 特性示例 / Rust 1.90 Features Examples");
    println!("==================================================");
    
    // 1. JIT 编译器改进示例 / JIT Compiler Improvements Example
    println!("\n1. JIT 编译器改进示例 / JIT Compiler Improvements Example");
    let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5, -1, -2, 6, 7, 8]);
    let result = processor.process_data();
    println!("   处理结果 / Processed result: {:?}", result);
    
    // 2. 异步流处理示例 / Async Stream Processing Example
    println!("\n2. 异步流处理示例 / Async Stream Processing Example");
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(50));
    
    // 添加测试数据 / Add test data
    for i in 1..=5 {
        let data = AsyncData {
            id: i,
            content: format!("test_data_{}", i),
            timestamp: chrono::Utc::now(),
            priority: (i % 3) as u8,
        };
        stream_processor.add_data(data);
    }
    
    let stream = stream_processor.create_stream();
    let results = stream.await;
    println!("   异步流处理结果 / Async stream results: {} 条记录", results.len());
    
    // 3. 会话类型工作流示例 / Session Types Workflow Example
    println!("\n3. 会话类型工作流示例 / Session Types Workflow Example");
    #[cfg(feature = "session_types")]
    {
        let mut engine = SessionTypesWorkflowEngine::new();
        
        let participants = vec![
            Participant {
                id: "participant1".to_string(),
                role: ParticipantRole::Initiator,
                endpoint: "endpoint1".to_string(),
            },
            Participant {
                id: "participant2".to_string(),
                role: ParticipantRole::Responder,
                endpoint: "endpoint2".to_string(),
            },
        ];
        
        let session_id = engine.create_session(SessionProtocol::RequestResponse, participants).await?;
        println!("   会话已创建 / Session created: {}", session_id);
    }
    #[cfg(not(feature = "session_types"))]
    {
        println!("   会话类型示例已跳过，因为未启用 'session_types' 特性。");
    }
    
    // 会话类型功能已通过特性标志控制
    
    // 4. 性能监控示例 / Performance Monitoring Example
    println!("\n4. 性能监控示例 / Performance Monitoring Example");
    let monitor = PerformanceMonitor::new();
    
    // 记录一些性能指标 / Record some performance metrics
    let metrics = PerformanceMetrics {
        operation_name: "data_processing".to_string(),
        execution_time: Duration::from_millis(150),
        memory_usage: 2048,
        cpu_usage: 75.5,
        throughput: 1000.0,
        error_count: 0,
    };
    monitor.record_metrics(metrics).await;
    
    let stats = monitor.get_overall_stats().await;
    println!("   总体性能统计 / Overall performance stats:");
    println!("     总操作数 / Total operations: {}", stats.total_operations);
    println!("     总执行时间 / Total execution time: {:?}", stats.total_execution_time);
    println!("     总内存使用 / Total memory usage: {} bytes", stats.total_memory_usage);
    println!("     总错误数 / Total errors: {}", stats.total_errors);
    println!("     平均吞吐量 / Average throughput: {:.2} ops/sec", stats.average_throughput);
    println!("     运行时间 / Uptime: {:?}", stats.uptime);
    
    // 5. const 特性示例 / const Features Example
    println!("\n5. const 特性示例 / const Features Example");
    let _const_processor = ConstContextProcessor::new();
    let config = ConstContextProcessor::create_config();
    println!("   const 配置 / const config: {:?}", config);
    
    let data = [1, 2, 3, 4, 5];
    let sum = ConstContextProcessor::process_const_data(&data);
    println!("   const 数据处理结果 / const data processing result: {}", sum);
    
    // 6. 稳定 API 示例 / Stable APIs Example
    println!("\n6. 稳定 API 示例 / Stable APIs Example");
    let mut stable_engine = StableAPIWorkflowEngine190::new();
    
    let workflow = StableWorkflowDefinition {
        name: "test_workflow".to_string(),
        steps: vec![
            StableWorkflowStep {
                name: "step1".to_string(),
                action: "process".to_string(),
                input: "input1".to_string(),
                output: "output1".to_string(),
            },
            StableWorkflowStep {
                name: "step2".to_string(),
                action: "complete".to_string(),
                input: "input2".to_string(),
                output: "output2".to_string(),
            },
        ],
        config: StableWorkflowConfig {
            timeout: 30,
            retries: 3,
            enable_debug: true,
        },
    };
    
    stable_engine.register_workflow("test".to_string(), workflow);
    let result = stable_engine.execute_workflow("test")?;
    println!("   稳定 API 工作流结果 / Stable API workflow result:");
    println!("     工作流名称 / Workflow name: {}", result.workflow_name);
    println!("     跳过的字符数 / Skipped characters: {}", result.skipped_chars);
    println!("     处理的步骤数 / Processed steps: {}", result.processed_steps.len());
    println!("     调试统计 / Debug stats: {:?}", result.debug_stats);
    
    // 7. 高性能工作流引擎示例 / High-Performance Workflow Engine Example
    println!("\n7. 高性能工作流引擎示例 / High-Performance Workflow Engine Example");
    let engine = HighPerformanceWorkflowEngine190::new();
    
    let workflow = PerformanceWorkflowDefinition {
        name: "high_perf_workflow".to_string(),
        steps: vec![
            PerformanceWorkflowStep {
                name: "step1".to_string(),
                action: "process".to_string(),
                timeout: Duration::from_millis(100),
                retries: 3,
            },
            PerformanceWorkflowStep {
                name: "step2".to_string(),
                action: "complete".to_string(),
                timeout: Duration::from_millis(100),
                retries: 3,
            },
        ],
        timeout: Duration::from_secs(30),
        retries: 3,
        priority: 1,
    };
    
    engine.register_workflow("high_perf".to_string(), workflow).await;
    
    let execution_id = "exec1".to_string();
    engine.start_execution("high_perf", execution_id.clone()).await?;
    println!("   高性能工作流已启动 / High-performance workflow started: {}", execution_id);
    
    engine.execute_step(&execution_id, 0).await?;
    engine.execute_step(&execution_id, 1).await?;
    
    let status = engine.get_execution_status(&execution_id).await;
    println!("   执行状态 / Execution status: {:?}", status);
    
    let perf_stats = engine.get_performance_stats().await;
    println!("   性能统计 / Performance stats:");
    println!("     总操作数 / Total operations: {}", perf_stats.total_operations);
    println!("     平均吞吐量 / Average throughput: {:.2} ops/sec", perf_stats.average_throughput);
    
    // 8. 性能基准测试示例 / Performance Benchmark Example
    println!("\n8. 性能基准测试示例 / Performance Benchmark Example");
    let mut benchmark = PerformanceBenchmark::new();
    
    benchmark.generate_test_data(1000, 512);
    println!("   生成了 1000 个测试数据项 / Generated 1000 test data items");
    
    let result = benchmark.run_benchmark("rust190_benchmark").await;
    println!("   基准测试结果 / Benchmark result:");
    println!("     测试名称 / Test name: {}", result.test_name);
    println!("     执行时间 / Execution time: {:?}", result.execution_time);
    println!("     内存使用 / Memory usage: {} bytes", result.memory_usage);
    println!("     吞吐量 / Throughput: {:.2} ops/sec", result.throughput);
    
    let average = benchmark.get_average_performance();
    if let Some(avg) = average {
        println!("   平均性能 / Average performance:");
        println!("     平均执行时间 / Average execution time: {:?}", avg.execution_time);
        println!("     平均吞吐量 / Average throughput: {:.2} ops/sec", avg.throughput);
    }
    
    println!("\n✅ 所有 Rust 1.90 特性示例执行完成！");
    println!("✅ All Rust 1.90 features examples completed!");
    
    Ok(())
}
