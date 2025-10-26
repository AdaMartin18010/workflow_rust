# Temporal框架对齐实施路线图

## 📋 概述

本文档是《Temporal框架对标分析与本项目使用梳理》的实施计划，详细规划了如何将本项目的工作流能力向Temporal框架对齐的具体步骤。

**目标**: 在保持Rust语言优势的同时，提供类Temporal的开发体验和企业级特性。

---

## 🎯 总体目标

### 短期目标 (Q1 2025 - 3个月)

- 实现Activity抽象层
- 添加Signal和Query机制
- 增强持久化能力（事件溯源）
- 达到60%+ Temporal特性对齐

### 中期目标 (Q2-Q3 2025 - 6个月)

- 实现版本管理系统
- 添加子工作流支持
- 完善可观测性
- 达到75%+ Temporal特性对齐

### 长期目标 (Q4 2025 - 12个月)

- 分布式部署能力
- 与Temporal互操作
- 生产就绪
- 达到85%+ 核心特性对齐

---

## 📅 详细实施计划

## 第一阶段: 核心能力补齐 (Week 1-12)

### Week 1-4: Activity抽象层 🔴

#### 任务1.1: 设计Activity API

**优先级**: 🔴 P0  
**估算工时**: 3天  
**负责人**: 核心团队

**设计要点:**

```rust
// src/activity/mod.rs

#[async_trait]
pub trait Activity: Send + Sync {
    /// Activity的输入类型
    type Input: DeserializeOwned + Send;
    /// Activity的输出类型
    type Output: Serialize + Send;
    /// Activity的错误类型
    type Error: std::error::Error + Send;
    
    /// Activity的唯一名称
    fn name(&self) -> &str;
    
    /// 执行Activity
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
    
    /// 重试策略（可选）
    fn retry_policy(&self) -> Option<RetryPolicy> {
        Some(RetryPolicy::default())
    }
    
    /// 执行超时（可选）
    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_secs(60))
    }
    
    /// 是否幂等
    fn is_idempotent(&self) -> bool {
        false
    }
}

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始重试间隔
    pub initial_interval: Duration,
    /// 最大重试间隔
    pub max_interval: Duration,
    /// 退避系数
    pub backoff_coefficient: f64,
    /// 可重试的错误类型
    pub retryable_errors: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            backoff_coefficient: 2.0,
            retryable_errors: vec![],
        }
    }
}
```

**交付物:**

- [ ] Activity trait定义
- [ ] RetryPolicy实现
- [ ] 单元测试（覆盖率>80%）
- [ ] 设计文档

#### 任务1.2: 实现ActivityExecutor

**优先级**: 🔴 P0  
**估算工时**: 5天  

```rust
// src/activity/executor.rs

pub struct ActivityExecutor {
    /// 注册的Activity
    activities: Arc<RwLock<HashMap<String, Arc<dyn DynActivity>>>>,
    /// 执行历史
    history: Arc<Mutex<Vec<ActivityExecution>>>,
    /// 性能监控
    metrics: Arc<ActivityMetrics>,
}

impl ActivityExecutor {
    pub fn new() -> Self {
        Self {
            activities: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(ActivityMetrics::new()),
        }
    }
    
    /// 注册Activity
    pub fn register<A>(&self, activity: A) 
    where 
        A: Activity + 'static,
        A::Input: DeserializeOwned,
        A::Output: Serialize,
    {
        let name = activity.name().to_string();
        let boxed: Arc<dyn DynActivity> = Arc::new(ActivityAdapter::new(activity));
        self.activities.write().unwrap().insert(name, boxed);
    }
    
    /// 执行Activity（带重试）
    pub async fn execute_with_retry(
        &self,
        activity_name: &str,
        input: Value,
    ) -> Result<Value, ActivityError> {
        let activity = self.activities.read().unwrap()
            .get(activity_name)
            .ok_or(ActivityError::NotFound(activity_name.to_string()))?
            .clone();
        
        let policy = activity.retry_policy()
            .unwrap_or_else(RetryPolicy::default);
        
        let mut attempt = 0;
        let mut interval = policy.initial_interval;
        
        loop {
            attempt += 1;
            let start_time = Instant::now();
            
            // 执行Activity
            let result = tokio::time::timeout(
                activity.timeout().unwrap_or(Duration::from_secs(60)),
                activity.execute_dyn(input.clone())
            ).await;
            
            // 记录指标
            let duration = start_time.elapsed();
            self.metrics.record_execution(activity_name, attempt, duration);
            
            match result {
                Ok(Ok(output)) => {
                    // 成功
                    self.record_success(activity_name, attempt, duration).await;
                    return Ok(output);
                }
                Ok(Err(err)) if attempt >= policy.max_attempts => {
                    // 达到最大重试次数
                    self.record_failure(activity_name, attempt, &err).await;
                    return Err(ActivityError::MaxAttemptsExceeded(Box::new(err)));
                }
                Ok(Err(err)) if self.is_retryable(&err, &policy) => {
                    // 可重试错误
                    tracing::warn!(
                        activity = activity_name,
                        attempt = attempt,
                        error = %err,
                        "Activity execution failed, retrying..."
                    );
                    
                    tokio::time::sleep(interval).await;
                    interval = std::cmp::min(
                        Duration::from_secs_f64(
                            interval.as_secs_f64() * policy.backoff_coefficient
                        ),
                        policy.max_interval,
                    );
                }
                Ok(Err(err)) => {
                    // 不可重试错误
                    self.record_failure(activity_name, attempt, &err).await;
                    return Err(ActivityError::NonRetryable(Box::new(err)));
                }
                Err(_) => {
                    // 超时
                    if attempt >= policy.max_attempts {
                        self.record_timeout(activity_name, attempt).await;
                        return Err(ActivityError::Timeout);
                    }
                    
                    tracing::warn!(
                        activity = activity_name,
                        attempt = attempt,
                        "Activity execution timeout, retrying..."
                    );
                    
                    tokio::time::sleep(interval).await;
                    interval = std::cmp::min(
                        Duration::from_secs_f64(
                            interval.as_secs_f64() * policy.backoff_coefficient
                        ),
                        policy.max_interval,
                    );
                }
            }
        }
    }
    
    fn is_retryable(&self, error: &dyn std::error::Error, policy: &RetryPolicy) -> bool {
        if policy.retryable_errors.is_empty() {
            // 如果没有指定，默认所有错误都可重试
            return true;
        }
        
        let error_str = error.to_string();
        policy.retryable_errors.iter()
            .any(|pattern| error_str.contains(pattern))
    }
    
    async fn record_success(&self, name: &str, attempts: u32, duration: Duration) {
        let mut history = self.history.lock().unwrap();
        history.push(ActivityExecution {
            name: name.to_string(),
            attempts,
            duration,
            status: ExecutionStatus::Success,
            timestamp: chrono::Utc::now(),
        });
    }
    
    async fn record_failure(&self, name: &str, attempts: u32, error: &dyn std::error::Error) {
        let mut history = self.history.lock().unwrap();
        history.push(ActivityExecution {
            name: name.to_string(),
            attempts,
            duration: Duration::ZERO,
            status: ExecutionStatus::Failed(error.to_string()),
            timestamp: chrono::Utc::now(),
        });
    }
    
    async fn record_timeout(&self, name: &str, attempts: u32) {
        let mut history = self.history.lock().unwrap();
        history.push(ActivityExecution {
            name: name.to_string(),
            attempts,
            duration: Duration::ZERO,
            status: ExecutionStatus::Timeout,
            timestamp: chrono::Utc::now(),
        });
    }
}

/// 执行历史记录
#[derive(Debug, Clone)]
struct ActivityExecution {
    name: String,
    attempts: u32,
    duration: Duration,
    status: ExecutionStatus,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
enum ExecutionStatus {
    Success,
    Failed(String),
    Timeout,
}

/// 性能指标
struct ActivityMetrics {
    executions: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl ActivityMetrics {
    fn new() -> Self {
        Self {
            executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn record_execution(&self, name: &str, _attempt: u32, duration: Duration) {
        let mut executions = self.executions.lock().unwrap();
        executions.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
            
        // 同时记录到metrics系统
        metrics::histogram!("activity_duration_seconds", "name" => name.to_string())
            .record(duration.as_secs_f64());
    }
}
```

**交付物:**

- [ ] ActivityExecutor实现
- [ ] 重试逻辑实现
- [ ] 超时控制
- [ ] 性能监控集成
- [ ] 集成测试

#### 任务1.3: 与WorkflowEngine集成

**优先级**: 🔴 P0  
**估算工时**: 3天  

```rust
// src/engine.rs (修改)

impl WorkflowEngine {
    /// 添加Activity执行器字段
    activity_executor: Option<Arc<ActivityExecutor>>,
    
    /// 设置Activity执行器
    pub fn with_activity_executor(mut self, executor: ActivityExecutor) -> Self {
        self.activity_executor = Some(Arc::new(executor));
        self
    }
    
    /// 执行Activity
    pub async fn execute_activity(
        &self,
        instance_id: &str,
        activity_name: &str,
        input: Value,
    ) -> Result<Value, WorkflowError> {
        let executor = self.activity_executor.as_ref()
            .ok_or(WorkflowError::ActivityExecutorNotConfigured)?;
        
        // 执行Activity
        let result = executor.execute_with_retry(activity_name, input).await
            .map_err(|e| WorkflowError::ActivityExecutionFailed(e.to_string()))?;
        
        // 更新工作流状态
        // ...
        
        Ok(result)
    }
}
```

**交付物:**

- [ ] WorkflowEngine集成
- [ ] 端到端测试
- [ ] 示例代码
- [ ] 文档更新

### Week 5-8: Signal与Query机制 🔴

#### 任务2.1: Signal机制设计与实现

**优先级**: 🔴 P0  
**估算工时**: 5天  

```rust
// src/signal.rs

/// Signal处理器trait
#[async_trait]
pub trait SignalHandler: Send + Sync {
    /// Signal数据类型
    type Data: DeserializeOwned + Send;
    
    /// 处理Signal
    async fn handle(&mut self, data: Self::Data) -> Result<(), SignalError>;
}

/// Signal管理器
pub struct SignalManager {
    /// 每个工作流实例的Signal通道
    channels: Arc<RwLock<HashMap<String, SignalChannel>>>,
}

struct SignalChannel {
    /// Signal队列
    queue: Arc<Mutex<VecDeque<SignalEnvelope>>>,
    /// Signal处理器
    handlers: HashMap<String, Box<dyn DynSignalHandler>>,
    /// 唤醒通知
    waker: Arc<Notify>,
}

#[derive(Clone)]
struct SignalEnvelope {
    signal_name: String,
    data: Value,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl SignalManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册Signal处理器
    pub fn register_handler<H>(
        &self,
        instance_id: &str,
        signal_name: &str,
        handler: H,
    ) where
        H: SignalHandler + 'static,
    {
        let mut channels = self.channels.write().unwrap();
        let channel = channels.entry(instance_id.to_string())
            .or_insert_with(|| SignalChannel {
                queue: Arc::new(Mutex::new(VecDeque::new())),
                handlers: HashMap::new(),
                waker: Arc::new(Notify::new()),
            });
        
        channel.handlers.insert(
            signal_name.to_string(),
            Box::new(SignalHandlerAdapter::new(handler))
        );
    }
    
    /// 发送Signal
    pub async fn send_signal(
        &self,
        instance_id: &str,
        signal_name: &str,
        data: Value,
    ) -> Result<(), SignalError> {
        let channels = self.channels.read().unwrap();
        let channel = channels.get(instance_id)
            .ok_or(SignalError::InstanceNotFound(instance_id.to_string()))?;
        
        // 添加到队列
        let envelope = SignalEnvelope {
            signal_name: signal_name.to_string(),
            data,
            timestamp: chrono::Utc::now(),
        };
        
        channel.queue.lock().unwrap().push_back(envelope);
        
        // 唤醒等待的工作流
        channel.waker.notify_one();
        
        Ok(())
    }
    
    /// 等待Signal
    pub async fn await_signal(
        &self,
        instance_id: &str,
        signal_name: &str,
    ) -> Result<Value, SignalError> {
        let channels = self.channels.read().unwrap();
        let channel = channels.get(instance_id)
            .ok_or(SignalError::InstanceNotFound(instance_id.to_string()))?;
        
        let waker = channel.waker.clone();
        let queue = channel.queue.clone();
        
        drop(channels); // 释放读锁
        
        loop {
            // 检查队列中是否有匹配的Signal
            {
                let mut q = queue.lock().unwrap();
                if let Some(pos) = q.iter().position(|e| e.signal_name == signal_name) {
                    let envelope = q.remove(pos).unwrap();
                    return Ok(envelope.data);
                }
            }
            
            // 等待新Signal
            waker.notified().await;
        }
    }
}
```

**交付物:**

- [ ] Signal机制实现
- [ ] SignalManager实现
- [ ] 与WorkflowEngine集成
- [ ] 测试用例

#### 任务2.2: Query机制设计与实现

**优先级**: 🔴 P0  
**估算工时**: 4天  

```rust
// src/query.rs

/// Query处理器trait
#[async_trait]
pub trait QueryHandler: Send + Sync {
    /// Query结果类型
    type Result: Serialize + Send;
    
    /// 处理Query
    async fn handle(&self) -> Result<Self::Result, QueryError>;
}

/// Query管理器
pub struct QueryManager {
    /// Query处理器
    handlers: Arc<RwLock<HashMap<String, HashMap<String, Box<dyn DynQueryHandler>>>>>,
}

impl QueryManager {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册Query处理器
    pub fn register_handler<H>(
        &self,
        instance_id: &str,
        query_name: &str,
        handler: H,
    ) where
        H: QueryHandler + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers.entry(instance_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(
                query_name.to_string(),
                Box::new(QueryHandlerAdapter::new(handler))
            );
    }
    
    /// 执行Query
    pub async fn execute_query(
        &self,
        instance_id: &str,
        query_name: &str,
    ) -> Result<Value, QueryError> {
        let handlers = self.handlers.read().unwrap();
        let instance_handlers = handlers.get(instance_id)
            .ok_or(QueryError::InstanceNotFound(instance_id.to_string()))?;
        
        let handler = instance_handlers.get(query_name)
            .ok_or(QueryError::QueryNotFound(query_name.to_string()))?;
        
        // 执行Query（不应该修改状态）
        handler.handle_dyn().await
    }
}
```

**交付物:**

- [ ] Query机制实现
- [ ] QueryManager实现
- [ ] 与WorkflowEngine集成
- [ ] 测试用例

### Week 9-12: 持久化增强 🔴

#### 任务3.1: 事件溯源架构

**优先级**: 🔴 P0  
**估算工时**: 7天  

```rust
// src/persistence/event_sourcing.rs

/// 工作流事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// 工作流启动
    WorkflowStarted {
        workflow_id: String,
        workflow_name: String,
        input: Value,
        timestamp: i64,
    },
    /// Activity启动
    ActivityStarted {
        workflow_id: String,
        activity_name: String,
        input: Value,
        timestamp: i64,
    },
    /// Activity完成
    ActivityCompleted {
        workflow_id: String,
        activity_name: String,
        output: Value,
        timestamp: i64,
    },
    /// Activity失败
    ActivityFailed {
        workflow_id: String,
        activity_name: String,
        error: String,
        timestamp: i64,
    },
    /// Signal接收
    SignalReceived {
        workflow_id: String,
        signal_name: String,
        data: Value,
        timestamp: i64,
    },
    /// 状态转换
    StateTransitioned {
        workflow_id: String,
        from_state: String,
        to_state: String,
        timestamp: i64,
    },
    /// 工作流完成
    WorkflowCompleted {
        workflow_id: String,
        output: Value,
        timestamp: i64,
    },
    /// 工作流失败
    WorkflowFailed {
        workflow_id: String,
        error: String,
        timestamp: i64,
    },
}

/// 事件存储
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 追加事件
    async fn append_event(&self, event: WorkflowEvent) -> Result<u64, EventStoreError>;
    
    /// 获取事件流
    async fn get_events(
        &self,
        workflow_id: &str,
        from_sequence: u64,
    ) -> Result<Vec<WorkflowEvent>, EventStoreError>;
    
    /// 获取最新序列号
    async fn get_latest_sequence(&self, workflow_id: &str) -> Result<u64, EventStoreError>;
}

/// 内存事件存储实现
pub struct InMemoryEventStore {
    events: Arc<RwLock<HashMap<String, Vec<(u64, WorkflowEvent)>>>>,
    sequences: Arc<RwLock<HashMap<String, u64>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            sequences: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_event(&self, event: WorkflowEvent) -> Result<u64, EventStoreError> {
        let workflow_id = event.workflow_id().to_string();
        
        let mut events = self.events.write().unwrap();
        let mut sequences = self.sequences.write().unwrap();
        
        // 获取下一个序列号
        let sequence = sequences.entry(workflow_id.clone())
            .and_modify(|s| *s += 1)
            .or_insert(1);
        
        // 追加事件
        events.entry(workflow_id.clone())
            .or_insert_with(Vec::new)
            .push((*sequence, event));
        
        Ok(*sequence)
    }
    
    async fn get_events(
        &self,
        workflow_id: &str,
        from_sequence: u64,
    ) -> Result<Vec<WorkflowEvent>, EventStoreError> {
        let events = self.events.read().unwrap();
        
        if let Some(workflow_events) = events.get(workflow_id) {
            let filtered = workflow_events.iter()
                .filter(|(seq, _)| *seq >= from_sequence)
                .map(|(_, event)| event.clone())
                .collect();
            
            Ok(filtered)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn get_latest_sequence(&self, workflow_id: &str) -> Result<u64, EventStoreError> {
        let sequences = self.sequences.read().unwrap();
        Ok(*sequences.get(workflow_id).unwrap_or(&0))
    }
}

/// 状态重建器
pub struct StateRebuilder {
    event_store: Arc<dyn EventStore>,
}

impl StateRebuilder {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self { event_store }
    }
    
    /// 从事件重建工作流状态
    pub async fn rebuild_state(
        &self,
        workflow_id: &str,
    ) -> Result<WorkflowInstance, StateRebuildError> {
        // 获取所有事件
        let events = self.event_store.get_events(workflow_id, 0).await?;
        
        // 从事件重建状态
        let mut state_builder = WorkflowStateBuilder::new(workflow_id);
        
        for event in events {
            state_builder.apply_event(event)?;
        }
        
        state_builder.build()
    }
}
```

**交付物:**

- [ ] 事件溯源架构实现
- [ ] EventStore trait和实现
- [ ] 状态重建机制
- [ ] WAL支持
- [ ] 性能测试

---

## 第二阶段: 企业特性增强 (Week 13-24)

### Week 13-16: 版本管理 🟡

#### 任务4.1: 工作流版本系统

**估算工时**: 8天

```rust
// src/versioning/mod.rs

use semver::{Version, VersionReq};

pub struct WorkflowVersion {
    pub definition: WorkflowDefinition,
    pub version: Version,
    pub compatible_with: Vec<VersionReq>,
    pub deprecated: bool,
    pub migration_rules: Vec<MigrationRule>,
}

pub struct MigrationRule {
    pub from_version: VersionReq,
    pub to_version: Version,
    pub migration_fn: Box<dyn Fn(WorkflowInstance) -> Result<WorkflowInstance, MigrationError>>,
}

pub struct VersionManager {
    versions: Arc<RwLock<HashMap<String, BTreeMap<Version, WorkflowVersion>>>>,
}

impl VersionManager {
    pub fn register_version(&self, workflow_name: String, version: WorkflowVersion) {
        // 注册版本
    }
    
    pub fn get_compatible_version(
        &self,
        workflow_name: &str,
        required_version: &VersionReq,
    ) -> Option<&WorkflowVersion> {
        // 查找兼容版本
    }
    
    pub async fn migrate_instance(
        &self,
        instance: WorkflowInstance,
        target_version: &Version,
    ) -> Result<WorkflowInstance, MigrationError> {
        // 迁移实例到新版本
    }
}
```

### Week 17-20: 子工作流支持 🟡

#### 任务5.1: 子工作流机制

**估算工时**: 6天

```rust
// src/child_workflow.rs

pub struct ChildWorkflowBuilder<'a> {
    parent_id: String,
    engine: &'a WorkflowEngine,
    workflow_name: String,
    input: Option<Value>,
    options: ChildWorkflowOptions,
}

pub struct ChildWorkflowOptions {
    pub inherit_parent_timeout: bool,
    pub propagate_failures: bool,
    pub wait_for_cancellation: bool,
}

impl WorkflowEngine {
    pub fn child_workflow<'a>(
        &'a self,
        parent_id: &str,
        workflow_name: &str,
    ) -> ChildWorkflowBuilder<'a> {
        ChildWorkflowBuilder {
            parent_id: parent_id.to_string(),
            engine: self,
            workflow_name: workflow_name.to_string(),
            input: None,
            options: ChildWorkflowOptions::default(),
        }
    }
}

impl<'a> ChildWorkflowBuilder<'a> {
    pub fn input(mut self, input: Value) -> Self {
        self.input = Some(input);
        self
    }
    
    pub async fn execute(self) -> Result<ChildWorkflowHandle, WorkflowError> {
        // 启动子工作流
    }
}
```

### Week 21-24: 可观测性增强 🟡

#### 任务6.1: 分布式追踪集成

**估算工时**: 5天

```rust
// src/observability/tracing.rs

use opentelemetry::trace::{Tracer, Span};

pub struct WorkflowTracer {
    tracer: Box<dyn Tracer>,
}

impl WorkflowEngine {
    pub fn trace_workflow_execution(
        &self,
        instance_id: &str,
    ) -> WorkflowSpan {
        let span = self.tracer.start(format!("workflow.{}", instance_id));
        WorkflowSpan { span }
    }
}
```

---

## 第三阶段: 生产就绪 (Week 25-48)

### Week 25-32: 分布式部署 🟢

#### 任务7.1: 集群支持

**估算工时**: 15天

### Week 33-40: Temporal互操作 🟢

#### 任务8.1: Temporal兼容层

**估算工时**: 12天

### Week 41-48: 性能优化与稳定性 🟢

#### 任务9.1: 生产环境验证

**估算工时**: 20天

---

## 📊 进度跟踪

### 关键里程碑

| 里程碑 | 目标日期 | 状态 | 完成度 |
|--------|---------|------|--------|
| M1: Activity抽象完成 | Week 4 | 📋 待开始 | 0% |
| M2: Signal/Query完成 | Week 8 | 📋 待开始 | 0% |
| M3: 事件溯源完成 | Week 12 | 📋 待开始 | 0% |
| M4: v2.0 Beta发布 | Week 12 | 📋 待开始 | 0% |
| M5: 版本管理完成 | Week 16 | 📋 待开始 | 0% |
| M6: 子工作流完成 | Week 20 | 📋 待开始 | 0% |
| M7: v2.0 RC发布 | Week 24 | 📋 待开始 | 0% |
| M8: 分布式部署完成 | Week 32 | 📋 待开始 | 0% |
| M9: Temporal互操作完成 | Week 40 | 📋 待开始 | 0% |
| M10: v2.0正式发布 | Week 48 | 📋 待开始 | 0% |

### 风险与缓解措施

| 风险 | 影响 | 概率 | 缓解措施 |
|-----|------|------|---------|
| 设计复杂度超预期 | 🔴 高 | 🟡 中 | 分阶段实施，MVP优先 |
| 性能回归 | 🔴 高 | 🟢 低 | 持续性能测试，基准监控 |
| 向后兼容性问题 | 🟡 中 | 🟡 中 | 版本隔离，迁移工具 |
| 资源不足 | 🟡 中 | 🟢 低 | 社区贡献，优先级调整 |

---

## 📚 参考资源

### 开发指南

- [Activity开发指南](./guides/activity_development.md)
- [Signal/Query使用指南](./guides/signal_query_guide.md)
- [持久化配置指南](./guides/persistence_config.md)

### 示例代码

- [Activity示例](../examples/activity_examples/)
- [Signal/Query示例](../examples/signal_query_examples/)
- [事件溯源示例](../examples/event_sourcing_examples/)

---

**文档版本**: 1.0  
**最后更新**: 2025-10-26  
**维护者**: workflow_rust核心团队
