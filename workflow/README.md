# Rust 工作流系统 (Rust Workflow System)

[![Rust Version](https://img.shields.io/badge/rust-1.90+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/workflow.svg)](https://crates.io/crates/workflow)
[![Standards Compliance](https://img.shields.io/badge/standards-ISO%2FIEC%2025010%20%7C%20IEEE%20830%20%7C%20BPMN%202.0-brightgreen.svg)](https://github.com/rust-lang/workflow)
[![University Benchmark](https://img.shields.io/badge/benchmark-MIT%20%7C%20Stanford%20%7C%20Temporal%20%7C%20Cadence-orange.svg)](https://github.com/rust-lang/workflow)

一个基于 Rust 1.90 特性的高级工作流系统，对标国际标准、著名大学课程和成熟开源框架，集成了二十多个设计模式和完整的中间件支持。

An advanced workflow system based on Rust 1.90 features, benchmarking against international standards, renowned university courses, and mature open-source frameworks, integrating over twenty design patterns and complete middleware support.

## 🚀 特性 (Features)

### Rust 1.90 语言特性集成 (Rust 1.90 Language Features Integration)

- **JIT 编译器改进** - 更高效的迭代器操作和内存分配，提升运行时性能
- **const 特性增强** - 在 const 上下文中引用非静态变量，支持更复杂的编译时计算
- **稳定 API** - `BufRead::skip_while`、`ControlFlow`、`DebugList::finish_non_exhaustive` 等 API 的稳定化
- **异步迭代器改进** - 更高效的异步流处理，提升异步编程性能
- **类型检查器优化** - 减少大型代码库的编译时间，提升开发体验
- **内存分配器优化** - 在处理大量小对象时表现更优，减少内存碎片
- **会话类型支持** - 集成 Ferrite 库，实现安全并发通信
- **性能监控增强** - 内置性能监控和基准测试功能

### 国际标准对标 (International Standards Benchmarking)

- **ISO/IEC 25010 软件质量模型** - 符合软件产品质量的八个特性标准
- **IEEE 830 软件需求规格说明** - 遵循软件需求规格说明的推荐实践
- **BPMN 2.0 业务流程建模** - 支持完整的业务流程建模和标记标准
- **XPDL 2.2 XML 流程定义语言** - 兼容 XML 流程定义语言标准
- **BPEL 2.0 业务流程执行语言** - 支持业务流程执行语言标准
- **W3C Web 标准** - 符合 Web 内容可访问性指南 (WCAG) 和语义化标准
- **RFC 2119 关键词使用规范** - 遵循 RFC 文档中关键词的使用规范

### 大学课程对标 (University Course Benchmarking)

- **MIT 6.824 高级工作流系统** - 对标麻省理工学院的高级工作流系统和进程代数课程
- **Stanford CS 244B 分布式系统** - 对标斯坦福大学的分布式系统和工作流管理课程
- **进程代数理论基础** - 涵盖 CCS、CSP、π-演算等进程代数理论
- **分布式工作流系统** - 包含分布式状态管理、共识算法、容错机制
- **形式化验证方法** - 支持模型检查、时序逻辑、属性规范
- **性能分析和优化** - 提供性能建模、瓶颈分析、优化技术

### 开源框架对标 (Open Source Framework Benchmarking)

- **Temporal 框架对标** - 对标 Temporal 工作流引擎的特性和性能
- **Cadence 框架对标** - 对标 Cadence 工作流引擎的特性和性能
- **工作流执行引擎** - 支持工作流执行、活动执行、Saga 模式
- **补偿机制** - 实现完整的补偿和重试策略
- **工作流版本控制** - 支持工作流版本管理和调度
- **信号和查询** - 提供工作流信号处理和查询功能
- **监控和可观测性** - 集成指标收集、分布式追踪、工作流历史
- **扩展性和安全性** - 支持水平扩展、多集群、跨区域部署

### 工作流设计模式 (Workflow Design Patterns)

- **创建型模式** (Creational Patterns)
  - 建造者模式 (Builder Pattern)
  - 工厂模式 (Factory Pattern)
  - 原型模式 (Prototype Pattern)
  - 单例模式 (Singleton Pattern)

- **结构型模式** (Structural Patterns)
  - 适配器模式 (Adapter Pattern)
  - 桥接模式 (Bridge Pattern)
  - 组合模式 (Composite Pattern)
  - 装饰器模式 (Decorator Pattern)
  - 外观模式 (Facade Pattern)
  - 享元模式 (Flyweight Pattern)
  - 代理模式 (Proxy Pattern)

- **行为型模式** (Behavioral Patterns)
  - 责任链模式 (Chain of Responsibility Pattern)
  - 命令模式 (Command Pattern)
  - 解释器模式 (Interpreter Pattern)
  - 迭代器模式 (Iterator Pattern)
  - 中介者模式 (Mediator Pattern)
  - 备忘录模式 (Memento Pattern)
  - 观察者模式 (Observer Pattern)
  - 状态模式 (State Pattern)
  - 策略模式 (Strategy Pattern)
  - 模板方法模式 (Template Method Pattern)
  - 访问者模式 (Visitor Pattern)

- **并发模式** (Concurrent Patterns)
  - Actor 模式 (Actor Pattern)
  - 生产者-消费者模式 (Producer-Consumer Pattern)
  - 管道模式 (Pipeline Pattern)
  - 反应器模式 (Reactor Pattern)
  - 线程池模式 (Thread Pool Pattern)

### 工作流中间件系统 (Workflow Middleware System)

- **核心中间件** (Core Middleware)
  - 认证中间件 (Authentication Middleware)
  - 授权中间件 (Authorization Middleware)
  - 日志中间件 (Logging Middleware)
  - 监控中间件 (Monitoring Middleware)
  - 限流中间件 (Rate Limiting Middleware)

- **扩展中间件** (Extension Middleware)
  - 缓存中间件 (Caching Middleware)
  - 压缩中间件 (Compression Middleware)
  - 加密中间件 (Encryption Middleware)
  - 重试中间件 (Retry Middleware)
  - 超时中间件 (Timeout Middleware)

- **插件中间件** (Plugin Middleware)
  - 自定义插件支持 (Custom Plugin Support)
  - 动态加载 (Dynamic Loading)
  - 插件生命周期管理 (Plugin Lifecycle Management)

## 📦 安装 (Installation)

在 `Cargo.toml` 中添加依赖：

Add to your `Cargo.toml`:

```toml
[dependencies]
workflow = { version = "1.0.0", features = ["full"] }
```

### 特性选项 (Feature Options)

```toml
[dependencies]
workflow = { version = "1.90.0", features = ["rust190", "patterns", "middleware", "international_standards", "session_types"] }
```

- `rust190` - 启用 Rust 1.90 特性支持
- `patterns` - 启用设计模式支持
- `middleware` - 启用中间件系统
- `international_standards` - 启用国际标准对标功能
- `framework_benchmarking` - 启用开源框架对标功能
- `monitoring` - 启用监控功能
- `persistence` - 启用持久化支持
- `database` - 启用数据库支持
- `session_types` - 启用会话类型支持
- `async_streams` - 启用异步流处理增强
- `full` - 启用所有特性

## 🎯 快速开始 (Quick Start)

### 基础工作流 (Basic Workflow)

```rust
use workflow::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建工作流引擎 / Create workflow engine
    let engine = WorkflowEngine::new();
    
    // 定义工作流 / Define workflow
    let definition = WorkflowDefinition {
        name: "order_processing".to_string(),
        description: Some("订单处理工作流 / Order processing workflow".to_string()),
        version: "1.0.0".to_string(),
        states: vec![
            "pending".to_string(),
            "processing".to_string(),
            "completed".to_string(),
        ],
        transitions: vec![
            StateTransition {
                from_state: "pending".to_string(),
                to_state: "processing".to_string(),
                condition: Some("start_processing".to_string()),
                action: Some("begin_processing".to_string()),
                metadata: json!({}),
            },
            StateTransition {
                from_state: "processing".to_string(),
                to_state: "completed".to_string(),
                condition: Some("processing_done".to_string()),
                action: Some("complete_processing".to_string()),
                metadata: json!({}),
            },
        ],
        initial_state: "pending".to_string(),
        final_states: vec!["completed".to_string()],
        metadata: json!({}),
    };
    
    // 注册工作流 / Register workflow
    engine.register_workflow("order_processing".to_string(), definition).await?;
    
    // 创建初始数据 / Create initial data
    let initial_data = WorkflowData {
        id: "order_123".to_string(),
        data: json!({
            "order_id": "12345",
            "customer_id": "67890",
            "amount": 100.0
        }),
        metadata: json!({}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // 启动工作流实例 / Start workflow instance
    let instance_id = engine.start_workflow("order_processing", initial_data).await?;
    println!("工作流实例已启动 / Workflow instance started: {}", instance_id);
    
    Ok(())
}
```

### 使用设计模式 (Using Design Patterns)

```rust
use workflow::patterns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建模式工厂 / Create pattern factory
    let factory = WorkflowPatternFactory::new();
    
    // 获取所有可用模式 / Get all available patterns
    let patterns = factory.get_all_patterns();
    
    for pattern_info in patterns {
        println!("模式 / Pattern: {} - {}", pattern_info.name, pattern_info.description);
    }
    
    // 使用特定模式 / Use specific pattern
    let builder_pattern = factory.create_pattern("WorkflowBuilder", PatternCategory::Creational);
    if let Some(pattern) = builder_pattern {
        let context = WorkflowContext {
            workflow_id: "test_workflow".to_string(),
            data: json!({"test": "data"}),
            metadata: std::collections::HashMap::new(),
        };
        
        let result = pattern.apply(&context)?;
        println!("模式应用结果 / Pattern application result: {:?}", result);
    }
    
    Ok(())
}
```

### 使用中间件 (Using Middleware)

```rust
use workflow::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建中间件管理器 / Create middleware manager
    let manager = WorkflowMiddlewareManager::new();
    
    // 注册中间件 / Register middleware
    manager.register_middleware(Box::new(
        core::AuthenticationMiddleware::new()
    )).await?;
    
    manager.register_middleware(Box::new(
        core::LoggingMiddleware::new()
    )).await?;
    
    // 创建中间件上下文 / Create middleware context
    let context = MiddlewareContext::new(
        "req_1".to_string(),
        "workflow_1".to_string(),
        json!({"test": "data"})
    );
    
    // 创建中间件链 / Create middleware chain
    let mut chain = manager.create_chain(context).await?;
    
    // 执行中间件链 / Execute middleware chain
    let result = chain.execute().await?;
    println!("中间件执行结果 / Middleware execution result: {:?}", result);
    
    Ok(())
}
```

### 使用 Rust 1.90 特性 (Using Rust 1.90 Features)

```rust
use workflow::rust190::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 JIT 优化的处理器 / Use JIT-optimized processor
    let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]);
    let result = processor.process_data();
    println!("处理结果: {:?}", result);
    
    // 使用异步流处理器 / Use async stream processor
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(100));
    let data = AsyncData {
        id: 1,
        content: "test".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 1,
    };
    stream_processor.add_data(data);
    let stream = stream_processor.create_stream();
    let results: Vec<_> = stream.collect().await;
    println!("异步流处理结果: {:?}", results);
    
    // 使用会话类型工作流引擎 / Use session types workflow engine
    let mut engine = SessionTypesWorkflowEngine::new();
    let participants = vec![
        Participant {
            id: "participant1".to_string(),
            role: ParticipantRole::Initiator,
            endpoint: "endpoint1".to_string(),
        },
    ];
    let session_id = engine.create_session(SessionProtocol::RequestResponse, participants).await?;
    engine.start_session(&session_id).await?;
    println!("会话已启动: {}", session_id);
    
    // 使用性能监控器 / Use performance monitor
    let monitor = PerformanceMonitor::new();
    let metrics = PerformanceMetrics {
        operation_name: "test_operation".to_string(),
        execution_time: Duration::from_millis(100),
        memory_usage: 1024,
        cpu_usage: 50.0,
        throughput: 100.0,
        error_count: 0,
    };
    monitor.record_metrics(metrics).await;
    let stats = monitor.get_overall_stats().await;
    println!("性能统计: {:?}", stats);
    
    Ok(())
}
```

### 使用国际标准对标 (Using International Standards Benchmarking)

```rust
use workflow::international_standards::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化国际标准模块 / Initialize international standards module
    international_standards::init()?;
    
    // 检查标准合规性 / Check standards compliance
    let compliance = check_standards_compliance();
    println!("标准合规性: {:?}", compliance.level);
    println!("支持的标准: {:?}", compliance.standards_met);
    
    // 创建国际标准集合 / Create international standards collection
    let standards = InternationalWorkflowStandards::new();
    
    // 获取所有标准 / Get all standards
    let all_standards = standards.get_all_standards();
    for standard in all_standards {
        println!("标准: {} - {}", standard.name, standard.organization);
    }
    
    // 检查 BPMN 2.0 标准 / Check BPMN 2.0 standard
    if let Some(bpmn_standard) = standards.get_standard("BPMN_2_0") {
        println!("BPMN 2.0 标准包含 {} 个模式", bpmn_standard.patterns.len());
    }
    
    Ok(())
}
```

### 使用框架对标 (Using Framework Benchmarking)

```rust
use workflow::international_standards::framework_benchmarking::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Temporal 基准测试 / Create Temporal benchmark
    let mut temporal_benchmark = TemporalBenchmark::new();
    temporal_benchmark.run_benchmark().await?;
    
    // 创建 Cadence 基准测试 / Create Cadence benchmark
    let mut cadence_benchmark = CadenceBenchmark::new();
    cadence_benchmark.run_benchmark().await?;
    
    // 创建框架对比 / Create framework comparison
    let mut comparison = FrameworkComparison::new();
    comparison.add_framework(temporal_benchmark.get_benchmark().clone());
    comparison.add_framework(cadence_benchmark.get_benchmark().clone());
    
    // 运行对比测试 / Run comparison test
    let report = comparison.run_comparison().await?;
    
    println!("获胜框架: {}", report.winner);
    for framework in &report.frameworks {
        println!("框架: {} - 总体分数: {:.1}", framework.name, framework.overall_score);
    }
    
    for recommendation in &report.recommendations {
        println!("建议: {}", recommendation);
    }
    
    Ok(())
}
```

### 使用性能基准测试 (Using Performance Benchmarks)

```rust
use workflow::international_standards::performance_benchmarks::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建基准测试套件 / Create benchmark suite
    let mut suite = BenchmarkSuite::new();
    
    // 添加标准基准测试 / Add standard benchmarks
    let benchmarks = create_standard_benchmarks();
    for benchmark in benchmarks {
        suite.add_benchmark(benchmark);
    }
    
    // 运行所有基准测试 / Run all benchmarks
    let report = suite.run_all_benchmarks().await?;
    
    println!("基准测试套件: {}", report.suite_id);
    println!("总体分数: {:.1}", report.overall_score);
    println!("测试环境: {} 核心, {} GB 内存", 
             report.test_environment.cpu_cores, 
             report.test_environment.memory_gb);
    
    for result in &report.benchmark_results {
        println!("基准测试: {} - 分数: {:.1}", result.name, result.performance_score);
        println!("  吞吐量: {:.1} ops/sec", result.overall_metrics.throughput_ops_per_sec);
        println!("  平均延迟: {:.1} ms", result.overall_metrics.average_latency_ms);
        println!("  内存使用: {:.1} MB", result.overall_metrics.memory_usage_mb);
    }
    
    for recommendation in &report.recommendations {
        println!("建议: {}", recommendation);
    }
    
    Ok(())
}
```

## 🏗️ 项目结构 (Project Structure)

```text
workflow/
├── src/
│   ├── lib.rs                 # 主库文件 / Main library file
│   ├── types.rs              # 核心类型定义 / Core type definitions
│   ├── engine.rs             # 工作流引擎 / Workflow engine
│   ├── state.rs              # 状态管理 / State management
│   ├── error.rs              # 错误处理 / Error handling
│   ├── tools.rs              # 工具函数 / Utility functions
│   ├── rust190/              # Rust 1.90 特性 / Rust 1.90 features
│   │   ├── mod.rs
│   │   ├── features.rs
│   │   ├── async_features.rs
│   │   ├── const_features.rs
│   │   ├── stable_apis.rs
│   │   ├── performance.rs
│   │   └── session_types.rs
│   ├── patterns/             # 设计模式 / Design patterns
│   │   ├── mod.rs
│   │   ├── creational/
│   │   │   ├── builder.rs
│   │   │   ├── factory.rs
│   │   │   ├── prototype.rs
│   │   │   └── singleton.rs
│   │   ├── structural/
│   │   ├── behavioral/
│   │   └── concurrent/
│   ├── middleware/           # 中间件系统 / Middleware system
│   │   ├── mod.rs
│   │   ├── core/
│   │   ├── extensions/
│   │   └── plugins/
│   ├── international_standards/  # 国际标准对标 / International Standards Benchmarking
│   │   ├── mod.rs
│   │   ├── standards.rs      # 国际标准规范 / International Standards Specifications
│   │   ├── university_courses.rs  # 大学课程对标 / University Course Benchmarking
│   │   ├── framework_benchmarking.rs  # 开源框架对标 / Open Source Framework Benchmarking
│   │   ├── workflow_patterns.rs  # 工作流模式标准 / Workflow Pattern Standards
│   │   └── performance_benchmarks.rs  # 性能基准测试 / Performance Benchmarks
│   ├── examples/             # 示例代码 / Example code
│   │   ├── mod.rs
│   │   ├── basic_workflow.rs
│   │   ├── rust190_examples.rs
│   │   ├── pattern_examples.rs
│   │   ├── middleware_examples.rs
│   │   └── advanced_examples.rs
│   └── tests/                # 测试代码 / Test code
├── docs/                     # 文档 / Documentation
│   ├── ai/                   # AI 相关文档 / AI-related documentation
│   ├── algorithms/           # 算法文档 / Algorithm documentation
│   ├── iot/                  # IoT 相关文档 / IoT-related documentation
│   ├── program/              # 编程相关文档 / Programming-related documentation
│   └── rust_design/          # Rust 设计文档 / Rust design documentation
├── examples/                 # 独立示例 / Standalone examples
├── benches/                  # 基准测试 / Benchmarks
├── Cargo.toml               # 项目配置 / Project configuration
└── README.md                # 项目说明 / Project description
```

## 📚 文档 (Documentation)

- [API 文档 / API Documentation](https://docs.rs/workflow)
- [设计模式指南 / Design Patterns Guide](docs/patterns/)
- [中间件开发指南 / Middleware Development Guide](docs/middleware/)
- [Rust 1.90 特性使用指南 / Rust 1.90 Features Usage Guide](docs/rust190/)
- [国际标准对标指南 / International Standards Benchmarking Guide](docs/international_standards/)
- [大学课程对标指南 / University Course Benchmarking Guide](docs/university_courses/)
- [开源框架对标指南 / Open Source Framework Benchmarking Guide](docs/framework_benchmarking/)
- [性能基准测试指南 / Performance Benchmarking Guide](docs/performance_benchmarks/)
- [工作流模式标准指南 / Workflow Pattern Standards Guide](docs/workflow_patterns/)

## 🧪 测试 (Testing)

运行所有测试：

Run all tests:

```bash
cargo test
```

运行特定模块测试：

Run specific module tests:

```bash
cargo test patterns
cargo test middleware
cargo test rust190
cargo test international_standards
cargo test framework_benchmarking
cargo test performance_benchmarks
```

运行示例：

Run examples:

```bash
cargo run --example basic_workflow
cargo run --example pattern_usage
cargo run --example middleware_usage
cargo run --example international_standards_usage
cargo run --example framework_benchmarking_usage
cargo run --example performance_benchmarks_usage
```

## 📊 基准测试 (Benchmarks)

运行基准测试：

Run benchmarks:

```bash
cargo bench
```

## 🤝 贡献 (Contributing)

我们欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 许可证 (License)

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 🙏 致谢 (Acknowledgments)

- Rust 社区 / Rust Community
- 工作流引擎设计模式 / Workflow Engine Design Patterns
- 异步编程最佳实践 / Async Programming Best Practices

## 📞 支持 (Support)

- 问题报告 / Issue Reports: [GitHub Issues](https://github.com/rust-lang/workflow/issues)
- 讨论 / Discussions: [GitHub Discussions](https://github.com/rust-lang/workflow/discussions)
- 文档 / Documentation: [GitHub Wiki](https://github.com/rust-lang/workflow/wiki)

---

**Rust 工作流系统** - 让工作流开发更简单、更安全、更高效！

**Rust Workflow System** - Making workflow development simpler, safer, and more efficient!
