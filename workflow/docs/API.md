# Rust 1.90 工作流系统 API 文档

## 概述

本文档详细介绍了 Rust 1.90 工作流系统的 API 接口，包括所有核心模块、类型定义、函数签名和使用示例。

## 核心模块

### 1. Rust 1.90 特性模块 (`rust190`)

#### 1.1 JIT 优化处理器 (`JITOptimizedProcessor`)

```rust
pub struct JITOptimizedProcessor {
    data: Vec<i32>,
    processed_count: usize,
    processing_time: Duration,
}

impl JITOptimizedProcessor {
    /// 创建新的 JIT 优化处理器
    pub fn new(data: Vec<i32>) -> Self
    
    /// 处理数据，利用 JIT 编译器优化
    pub fn process_data(&mut self) -> Vec<i32>
    
    /// 获取处理统计信息
    pub fn get_stats(&self) -> ProcessingStats
}
```

**使用示例：**

```rust
use workflow::rust190::JITOptimizedProcessor;

let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]);
let result = processor.process_data();
println!("处理结果: {:?}", result);
```

#### 1.2 异步流处理器 (`AsyncStreamProcessor`)

```rust
pub struct AsyncStreamProcessor {
    buffer: Vec<AsyncData>,
    processing_delay: Duration,
}

impl AsyncStreamProcessor {
    /// 创建新的异步流处理器
    pub fn new(processing_delay: Duration) -> Self
    
    /// 添加数据到处理队列
    pub fn add_data(&mut self, data: AsyncData)
    
    /// 创建异步数据流
    pub async fn create_stream(&self) -> Vec<AsyncData>
    
    /// 处理流数据
    pub async fn process_stream_parallel<F, R>(&self, processor: F) -> Vec<R>
    where
        F: Fn(AsyncData) -> R + Clone + Send + Sync,
        R: Send,
}
```

**使用示例：**

```rust
use workflow::rust190::AsyncStreamProcessor;
use std::time::Duration;

let mut processor = AsyncStreamProcessor::new(Duration::from_millis(10));
processor.add_data(AsyncData {
    id: 1,
    content: "task_a".to_string(),
    timestamp: chrono::Utc::now(),
    priority: 1,
});

let results = processor.create_stream().await;
```

#### 1.3 性能监控器 (`PerformanceMonitor`)

```rust
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<HashMap<String, PerformanceMetrics>>>,
    overall_stats: Arc<Mutex<OverallPerformanceStats>>,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self
    
    /// 记录性能指标
    pub async fn record_metrics(&self, metrics: PerformanceMetrics)
    
    /// 获取特定操作的指标
    pub async fn get_metrics(&self, operation_name: &str) -> Option<PerformanceMetrics>
    
    /// 获取整体性能统计
    pub async fn get_overall_stats(&self) -> OverallPerformanceStats
}
```

**使用示例：**

```rust
use workflow::rust190::PerformanceMonitor;

let monitor = PerformanceMonitor::new();
let metrics = PerformanceMetrics {
    operation_name: "test_operation".to_string(),
    execution_time: Duration::from_millis(100),
    memory_usage: 1024,
    cpu_usage: 0.5,
    throughput: 10.0,
    error_count: 0,
};
monitor.record_metrics(metrics).await;
```

#### 1.4 const 特性处理器 (`ConstContextProcessor`)

```rust
pub struct ConstContextProcessor {
    data: Vec<i32>,
}

impl ConstContextProcessor {
    /// 创建新的 const 上下文处理器
    pub fn new() -> Self
    
    /// 创建 const 配置
    pub fn create_config() -> WorkflowConfig
    
    /// 处理 const 数据
    pub fn process_const_data(data: &[i32]) -> i32
}
```

**使用示例：**

```rust
use workflow::rust190::ConstContextProcessor;

let processor = ConstContextProcessor::new();
let config = ConstContextProcessor::create_config();
let data = [1, 2, 3, 4, 5];
let sum = ConstContextProcessor::process_const_data(&data);
```

### 2. 稳定 API 模块 (`stable_apis`)

#### 2.1 BufRead 处理器 (`BufReadProcessor`)

```rust
pub struct BufReadProcessor {
    reader: BufReader<Cursor<Vec<u8>>>,
}

impl BufReadProcessor {
    /// 创建新的 BufRead 处理器
    pub fn new(data: Vec<u8>) -> Self
    
    /// 跳过空白字符
    pub fn skip_whitespace(&mut self) -> Result<usize, std::io::Error>
    
    /// 跳过数字字符
    pub fn skip_digits(&mut self) -> Result<usize, std::io::Error>
    
    /// 读取一行并跳过前导空白
    pub fn read_line_skip_whitespace(&mut self) -> Result<String, std::io::Error>
}
```

#### 2.2 ControlFlow 处理器 (`ControlFlowProcessor`)

```rust
pub struct ControlFlowProcessor {
    data: Vec<i32>,
}

impl ControlFlowProcessor {
    /// 创建新的 ControlFlow 处理器
    pub fn new(data: Vec<i32>) -> Self
    
    /// 使用 ControlFlow 处理数据
    pub fn process_with_control_flow(&self, target: i32) -> Result<Option<i32>, String>
}
```

### 3. 会话类型模块 (`session_types`)

#### 3.1 会话类型工作流引擎 (`SessionTypesWorkflowEngine`)

```rust
pub struct SessionTypesWorkflowEngine {
    sessions: HashMap<String, WorkflowSession>,
    session_manager: SessionManager,
}

impl SessionTypesWorkflowEngine {
    /// 创建新的会话类型工作流引擎
    pub fn new() -> Self
    
    /// 创建会话
    pub async fn create_session(
        &mut self,
        protocol: SessionProtocol,
        participants: Vec<Participant>,
    ) -> Result<String, String>
    
    /// 启动会话
    pub async fn start_session(&mut self, session_id: &str) -> Result<(), String>
    
    /// 获取会话状态
    pub fn get_session_state(&self, session_id: &str) -> Option<SessionState>
    
    /// 完成会话
    pub async fn complete_session(&mut self, session_id: &str) -> Result<(), String>
}
```

## 数据类型

### 核心数据类型

```rust
/// 异步数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncData {
    pub id: u64,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub priority: u32,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub execution_time: Duration,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub throughput: f64,
    pub error_count: u32,
}

/// 整体性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallPerformanceStats {
    pub total_operations: usize,
    pub total_execution_time: Duration,
    pub total_memory_usage: usize,
    pub average_cpu_usage: f64,
    pub average_throughput: f64,
    pub total_errors: u32,
    pub uptime: Duration,
}

/// 工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub max_retries: u32,
    pub timeout_seconds: u32,
    pub batch_size: usize,
    pub enable_logging: bool,
}

/// 会话状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Initial,
    Active,
    Waiting,
    Completed,
    Failed,
}

/// 参与者角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Initiator,
    Responder,
    Observer,
}

/// 会话协议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionProtocol {
    RequestResponse,
    PublishSubscribe,
    Stream,
    Bidirectional,
}
```

## 错误处理

### 错误类型

```rust
/// 工作流错误
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("工作流未找到: {0}")]
    WorkflowNotFound(String),
    
    #[error("会话未找到: {0}")]
    SessionNotFound(String),
    
    #[error("执行超时: {0}")]
    ExecutionTimeout(String),
    
    #[error("配置错误: {0}")]
    ConfigurationError(String),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

## 使用模式

### 1. 基本工作流处理

```rust
use workflow::rust190::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 JIT 优化处理器
    let mut jit_processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]);
    let jit_result = jit_processor.process_data();
    
    // 2. 创建异步流处理器
    let mut stream_processor = AsyncStreamProcessor::new(Duration::from_millis(10));
    stream_processor.add_data(AsyncData {
        id: 1,
        content: "task_a".to_string(),
        timestamp: chrono::Utc::now(),
        priority: 1,
    });
    let stream_result = stream_processor.create_stream().await;
    
    // 3. 性能监控
    let monitor = PerformanceMonitor::new();
    let metrics = PerformanceMetrics {
        operation_name: "workflow_execution".to_string(),
        execution_time: Duration::from_millis(100),
        memory_usage: 1024,
        cpu_usage: 0.5,
        throughput: 10.0,
        error_count: 0,
    };
    monitor.record_metrics(metrics).await;
    
    Ok(())
}
```

### 2. 会话类型工作流

```rust
use workflow::rust190::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = SessionTypesWorkflowEngine::new();
    
    let participants = vec![
        Participant {
            id: "client".to_string(),
            name: "Client".to_string(),
            role: ParticipantRole::Initiator,
            endpoint: "client_endpoint".to_string(),
        },
        Participant {
            id: "server".to_string(),
            name: "Server".to_string(),
            role: ParticipantRole::Responder,
            endpoint: "server_endpoint".to_string(),
        },
    ];
    
    let session_id = engine.create_session(SessionProtocol::RequestResponse, participants).await?;
    engine.start_session(&session_id).await?;
    
    let state = engine.get_session_state(&session_id);
    println!("会话状态: {:?}", state);
    
    engine.complete_session(&session_id).await?;
    
    Ok(())
}
```

### 3. 性能基准测试

```rust
use workflow::rust190::*;
use std::time::Duration;

async fn benchmark_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = PerformanceMonitor::new();
    
    // 模拟工作流执行
    for i in 0..1000 {
        let start = std::time::Instant::now();
        
        // 执行工作流步骤
        let mut processor = JITOptimizedProcessor::new(vec![i, i + 1, i + 2]);
        let _result = processor.process_data();
        
        let execution_time = start.elapsed();
        
        let metrics = PerformanceMetrics {
            operation_name: format!("workflow_step_{}", i),
            execution_time,
            memory_usage: 1024,
            cpu_usage: 0.3,
            throughput: 1000.0 / execution_time.as_secs_f64(),
            error_count: 0,
        };
        
        monitor.record_metrics(metrics).await;
    }
    
    let stats = monitor.get_overall_stats().await;
    println!("性能统计: {:?}", stats);
    
    Ok(())
}
```

## 最佳实践

### 1. 错误处理

- 始终使用 `Result` 类型处理可能失败的操作
- 使用 `?` 操作符进行错误传播
- 提供有意义的错误消息

### 2. 异步编程

- 使用 `async/await` 语法处理异步操作
- 合理使用 `tokio::runtime::Runtime` 进行异步上下文管理
- 避免在异步函数中使用阻塞操作

### 3. 性能优化

- 使用 `black_box` 防止编译器优化影响基准测试结果
- 合理设置批处理大小和超时时间
- 监控内存使用和CPU占用率

### 4. 类型安全

- 利用 Rust 的类型系统确保编译时安全
- 使用 `const` 函数进行编译时计算
- 合理使用泛型和 trait 约束

## 版本兼容性

- **Rust 版本**: 1.90+
- **Tokio 版本**: 1.0+
- **Serde 版本**: 1.0+
- **Chrono 版本**: 0.4+

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件。
