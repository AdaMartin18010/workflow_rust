# 架构设计

## 📋 文档概述

本文档详细阐述基于Temporal的Rust 1.90工作流系统的架构设计，包括：

- 系统整体架构
- 分层设计
- 核心组件
- 数据流
- 部署模式
- 与Temporal Server的关系

---

## 🏗️ 系统整体架构

### 架构全景图

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                    Temporal-Rust 工作流系统架构                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      应用层 (Application Layer)                  │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │   │
│  │  │  #[workflow] │  │  #[activity] │  │    Signal    │           │   │
│  │  │   Workflow   │  │   Activity   │  │    Query     │           │   │
│  │  │  Definitions │  │  Definitions │  │  Handlers    │           │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ▲                                         │
│                              │ 使用                                    │
│                              │                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      SDK层 (SDK Layer)                          │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │ WorkflowContext │              │ ActivityContext │           │   │
│  │  │   工作流上下文   │              │   活动上下文     │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │ WorkflowClient  │              │ WorkflowWorker  │           │   │
│  │  │   工作流客户端   │              │   工作流执行器   │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ▲                                           │
│                              │ 依赖                                      │
│                              │                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      运行时层 (Runtime Layer)                    │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │  Task Poller    │              │ Event Processor │           │   │
│  │  │   任务轮询器     │              │   事件处理器     │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │ Signal Registry │              │ Query Registry  │           │   │
│  │  │   信号注册表     │              │   查询注册表     │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │  Tokio Runtime  │              │  Connection Pool│           │   │
│  │  │   异步运行时     │              │   连接池         │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ▲                                           │
│                              │ 使用                                      │
│                              │                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      通信层 (Communication Layer)                 │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │  gRPC Client    │              │ Protocol Buffers│           │   │
│  │  │  (tonic)        │◄────────────►│  (prost)        │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ▲                                           │
│                              │ 与Temporal Server通信                    │
│                              │                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      持久化层 (Persistence Layer)                 │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │  ┌─────────────────┐              ┌─────────────────┐           │   │
│  │  │ WorkflowStorage │              │  Event History  │           │   │
│  │  │   Trait         │◄────────────►│   事件历史       │           │   │
│  │  └─────────────────┘              └─────────────────┘           │   │
│  │           ▲                                                       │   │
│  │           │ 实现                                                  │   │
│  │  ┌────────┴────────┬────────────┬────────────┐                  │   │
│  │  │                 │            │            │                  │   │
│  │  ▼                 ▼            ▼            ▼                  │   │
│  │ PostgreSQL       MySQL       Memory      Temporal Server         │   │
│  │ Storage         Storage      Storage      (可选)                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      可观测层 (Observability Layer)               │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │   │
│  │  │    Tracing      │  │     Metrics     │  │     Logging     │ │   │
│  │  │  分布式追踪      │  │    指标收集      │  │    日志记录      │ │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

### 设计原则

1. **Temporal兼容性**: 完全遵循Temporal的设计模式和概念模型
2. **模块化**: 清晰的层次划分，职责分离
3. **可扩展性**: 易于添加新功能和集成
4. **类型安全**: 充分利用Rust的类型系统
5. **性能优先**: 零成本抽象，最小化运行时开销
6. **可观测性**: 内置监控、追踪和日志

---

## 📐 分层架构详解

### 1. 应用层 (Application Layer)

**职责**: 业务逻辑定义

**核心组件**:

```rust
// 工作流定义
#[workflow]
pub async fn business_workflow(
    ctx: WorkflowContext,
    input: BusinessInput,
) -> Result<BusinessOutput, WorkflowError> {
    // 业务逻辑
}

// Activity定义
#[activity]
pub async fn business_activity(
    ctx: ActivityContext,
    input: ActivityInput,
) -> Result<ActivityOutput, ActivityError> {
    // 具体操作
}
```

**特点**:

- 声明式API
- 类型安全
- 宏简化定义
- 与Temporal Go SDK风格一致

### 2. SDK层 (SDK Layer)

**职责**: 提供工作流开发API

**核心组件**:

#### 2.1 WorkflowContext

```rust
pub struct WorkflowContext {
    workflow_id: WorkflowId,
    run_id: RunId,
    workflow_type: String,
    task_queue: String,
    history: Arc<RwLock<EventHistory>>,
    signals: Arc<SignalRegistry>,
    queries: Arc<QueryRegistry>,
}

impl WorkflowContext {
    /// 执行Activity
    pub async fn execute_activity<A: Activity>(
        &self,
        input: A::Input,
        options: ActivityOptions,
    ) -> Result<A::Output, WorkflowError>;
    
    /// 等待Signal
    pub async fn await_signal<S: Signal>(&self) -> Result<S, WorkflowError>;
    
    /// 注册Query处理器
    pub fn set_query_handler<Q: Query, F>(&self, handler: F);
    
    /// 定时器
    pub async fn sleep(&self, duration: Duration);
    
    /// 启动子工作流
    pub async fn start_child_workflow<W: Workflow>(
        &self,
        input: W::Input,
        options: ChildWorkflowOptions,
    ) -> Result<ChildWorkflowHandle<W::Output>, WorkflowError>;
}
```

#### 2.2 WorkflowClient

```rust
pub struct WorkflowClient {
    storage: Arc<dyn WorkflowStorage>,
    grpc_client: Option<Arc<TemporalGrpcClient>>,
    task_dispatcher: Arc<TaskDispatcher>,
}

impl WorkflowClient {
    /// 启动工作流
    pub async fn start_workflow<W: Workflow>(
        &self,
        workflow_id: WorkflowId,
        task_queue: String,
        input: W::Input,
        options: StartWorkflowOptions,
    ) -> Result<WorkflowHandle<W::Output>, ClientError>;
    
    /// 发送Signal
    pub async fn signal_workflow<S: Signal>(
        &self,
        workflow_id: &WorkflowId,
        signal: S,
    ) -> Result<(), ClientError>;
    
    /// 查询工作流
    pub async fn query_workflow<Q: Query>(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<Q::Result, ClientError>;
}
```

#### 2.3 WorkflowWorker

```rust
pub struct WorkflowWorker {
    config: WorkerConfig,
    task_queue: String,
    workflows: Arc<RwLock<HashMap<String, WorkflowFactory>>>,
    activities: Arc<RwLock<HashMap<String, ActivityFactory>>>,
    storage: Arc<dyn WorkflowStorage>,
    task_poller: Arc<TaskPoller>,
}

impl WorkflowWorker {
    /// 注册工作流
    pub fn register_workflow<W: Workflow>(&mut self);
    
    /// 注册Activity
    pub fn register_activity<A: Activity>(&mut self);
    
    /// 启动Worker
    pub async fn start(self: Arc<Self>) -> Result<(), WorkerError>;
}
```

### 3. 运行时层 (Runtime Layer)

**职责**: 任务调度和事件处理

**核心组件**:

#### 3.1 TaskPoller

```rust
pub struct TaskPoller {
    task_queue: String,
    storage: Arc<dyn WorkflowStorage>,
    max_concurrent_tasks: usize,
}

impl TaskPoller {
    /// 轮询工作流任务
    pub async fn poll_workflow_tasks(&self) -> Result<Option<WorkflowTask>, PollError>;
    
    /// 轮询Activity任务
    pub async fn poll_activity_tasks(&self) -> Result<Option<ActivityTask>, PollError>;
}
```

#### 3.2 EventProcessor

```rust
pub struct EventProcessor {
    storage: Arc<dyn WorkflowStorage>,
}

impl EventProcessor {
    /// 处理工作流事件
    pub async fn process_workflow_event(
        &self,
        event: WorkflowEvent,
    ) -> Result<(), ProcessError>;
    
    /// 重放事件历史
    pub async fn replay_history(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<WorkflowState, ProcessError>;
}
```

#### 3.3 SignalRegistry 和 QueryRegistry

```rust
pub struct SignalRegistry {
    handlers: Arc<RwLock<HashMap<String, SignalHandler>>>,
    pending: Arc<RwLock<HashMap<String, VecDeque<Value>>>>,
}

pub struct QueryRegistry {
    handlers: Arc<RwLock<HashMap<String, QueryHandler>>>,
}
```

### 4. 通信层 (Communication Layer)

**职责**: 与Temporal Server通信

**核心组件**:

#### 4.1 TemporalGrpcClient

```rust
pub struct TemporalGrpcClient {
    workflow_service: WorkflowServiceClient<Channel>,
}

impl TemporalGrpcClient {
    /// 启动工作流执行
    pub async fn start_workflow_execution(
        &self,
        request: StartWorkflowExecutionRequest,
    ) -> Result<StartWorkflowExecutionResponse, tonic::Status>;
    
    /// 轮询工作流任务
    pub async fn poll_workflow_task_queue(
        &self,
        request: PollWorkflowTaskQueueRequest,
    ) -> Result<PollWorkflowTaskQueueResponse, tonic::Status>;
    
    /// 完成工作流任务
    pub async fn respond_workflow_task_completed(
        &self,
        request: RespondWorkflowTaskCompletedRequest,
    ) -> Result<RespondWorkflowTaskCompletedResponse, tonic::Status>;
}
```

### 5. 持久化层 (Persistence Layer)

**职责**: 状态持久化和事件存储

**核心抽象**:

#### 5.1 WorkflowStorage Trait

```rust
#[async_trait]
pub trait WorkflowStorage: Send + Sync {
    /// 保存工作流执行
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        history: &EventHistory,
    ) -> Result<(), StorageError>;
    
    /// 加载工作流执行
    async fn load_workflow_execution(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<(WorkflowExecution, EventHistory), StorageError>;
    
    /// 追加事件
    async fn append_event(
        &self,
        workflow_id: &WorkflowId,
        event: WorkflowEvent,
    ) -> Result<EventId, StorageError>;
    
    /// 获取事件历史
    async fn get_event_history(
        &self,
        workflow_id: &WorkflowId,
        from_event_id: Option<EventId>,
    ) -> Result<Vec<WorkflowEvent>, StorageError>;
}
```

#### 5.2 实现

- **PostgresWorkflowStorage**: PostgreSQL实现
- **MySQLWorkflowStorage**: MySQL实现
- **InMemoryWorkflowStorage**: 内存实现（测试用）

### 6. 可观测层 (Observability Layer)

**职责**: 监控、追踪和日志

**组件**:

- **Tracing**: 使用`tracing` crate实现结构化日志
- **Metrics**: 使用`metrics` crate收集性能指标
- **OpenTelemetry**: 分布式追踪

---

## 🔄 数据流图

### 工作流启动流程

```text
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ 1. start_workflow()
       ▼
┌─────────────────────────────┐
│     WorkflowClient          │
│  - 创建 WorkflowExecution   │
│  - 记录 Started 事件        │
└──────┬──────────────────────┘
       │ 2. 保存到存储
       ▼
┌─────────────────────────────┐
│     WorkflowStorage         │
│  - 保存执行记录             │
│  - 保存事件历史             │
└──────┬──────────────────────┘
       │ 3. 分发任务
       ▼
┌─────────────────────────────┐
│     TaskQueue               │
│  - 任务入队                 │
└──────┬──────────────────────┘
       │ 4. Worker轮询
       ▼
┌─────────────────────────────┐
│     WorkflowWorker          │
│  - 轮询任务队列             │
│  - 获取工作流任务           │
└──────┬──────────────────────┘
       │ 5. 执行工作流
       ▼
┌─────────────────────────────┐
│     WorkflowContext         │
│  - 创建执行环境             │
│  - 执行工作流函数           │
└──────┬──────────────────────┘
       │ 6. 执行Activities
       ▼
┌─────────────────────────────┐
│     ActivityContext         │
│  - 执行Activity函数         │
│  - 记录Activity事件         │
└──────┬──────────────────────┘
       │ 7. 完成/失败
       ▼
┌─────────────────────────────┐
│     EventProcessor          │
│  - 处理完成事件             │
│  - 更新状态                 │
│  - 持久化结果               │
└─────────────────────────────┘
```

### Signal处理流程

```text
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ 1. signal_workflow()
       ▼
┌─────────────────────────────┐
│     WorkflowClient          │
│  - 创建 Signal 事件         │
└──────┬──────────────────────┘
       │ 2. 记录事件
       ▼
┌─────────────────────────────┐
│     WorkflowStorage         │
│  - 保存 Signal 事件         │
└──────┬──────────────────────┘
       │ 3. 通知Worker
       ▼
┌─────────────────────────────┐
│     SignalRegistry          │
│  - 查找Signal处理器         │
│  - 触发处理器               │
└──────┬──────────────────────┘
       │ 4. 唤醒工作流
       ▼
┌─────────────────────────────┐
│     WorkflowContext         │
│  - await_signal() 返回      │
│  - 继续执行工作流           │
└─────────────────────────────┘
```

### Query处理流程

```text
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ 1. query_workflow()
       ▼
┌─────────────────────────────┐
│     WorkflowClient          │
│  - 创建 Query 请求          │
└──────┬──────────────────────┘
       │ 2. 路由到Worker
       ▼
┌─────────────────────────────┐
│     QueryRegistry           │
│  - 查找Query处理器          │
└──────┬──────────────────────┘
       │ 3. 执行处理器
       ▼
┌─────────────────────────────┐
│     Query Handler           │
│  - 读取工作流状态           │
│  - 返回结果                 │
└──────┬──────────────────────┘
       │ 4. 返回给Client
       ▼
┌─────────────┐
│   Client    │
│  - 获取结果 │
└─────────────┘
```

---

## 🌐 部署架构

### 单机部署模式

```text
┌────────────────────────────────────────────────────────────┐
│                       单机部署                              │
├────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            Application Process                       │  │
│  ├──────────────────────────────────────────────────────┤  │
│  │  ┌────────────────┐        ┌────────────────┐       │  │
│  │  │ WorkflowClient │        │ WorkflowWorker │       │  │
│  │  └────────────────┘        └────────────────┘       │  │
│  │                                                       │  │
│  │  ┌────────────────────────────────────────────────┐ │  │
│  │  │          In-Memory Storage (开发/测试)         │ │  │
│  │  └────────────────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└────────────────────────────────────────────────────────────┘
```

**特点**:

- 简单部署
- 适合开发和测试
- 无外部依赖

### 独立部署模式

```text
┌────────────────────────────────────────────────────────────────────┐
│                          独立部署                                   │
├────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐          ┌──────────────────┐                │
│  │  Client Process  │          │  Worker Process  │                │
│  │                  │          │  ┌────────────┐  │                │
│  │ WorkflowClient   │          │  │   Worker   │  │                │
│  └─────────┬────────┘          │  └────────────┘  │                │
│            │                   └────────┬──────────┘                │
│            │                            │                            │
│            │                            │                            │
│            ▼                            ▼                            │
│  ┌──────────────────────────────────────────────────┐              │
│  │            PostgreSQL / MySQL                     │              │
│  │         (WorkflowStorage Implementation)          │              │
│  └──────────────────────────────────────────────────┘              │
│                                                                      │
└────────────────────────────────────────────────────────────────────┘
```

**特点**:

- Client和Worker分离
- 使用外部数据库
- 支持多Worker实例
- 适合生产环境

### 与Temporal Server集成

```text
┌────────────────────────────────────────────────────────────────────────┐
│                    与 Temporal Server 集成                              │
├────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────┐                    ┌──────────────────┐          │
│  │  Client Process  │                    │  Worker Process  │          │
│  │  ┌────────────┐  │                    │  ┌────────────┐  │          │
│  │  │WorkflowClient│ gRPC               │  │   Worker   │  │          │
│  │  └────────────┘  │◄───────────────────►  └────────────┘  │          │
│  └──────────────────┘         │           └──────────────────┘          │
│                                │                     ▲                   │
│                                ▼                     │                   │
│  ┌────────────────────────────────────────────────┐ │                   │
│  │            Temporal Server (gRPC API)          │ │ gRPC              │
│  │  ┌──────────────┐  ┌──────────────┐           │ │                   │
│  │  │   Frontend   │  │    History   │           │─┘                   │
│  │  │   Service    │  │    Service   │           │                     │
│  │  └──────────────┘  └──────────────┘           │                     │
│  └────────────────────────────────────────────────┘                     │
│                           │                                              │
│                           ▼                                              │
│  ┌────────────────────────────────────────────────┐                     │
│  │         Temporal Server Database               │                     │
│  │      (Cassandra / PostgreSQL / MySQL)          │                     │
│  └────────────────────────────────────────────────┘                     │
│                                                                          │
└────────────────────────────────────────────────────────────────────────┘
```

**特点**:

- 完全兼容Temporal生态
- 使用Temporal Server的持久化
- 支持多语言Worker互操作
- 企业级特性（搜索、可见性等）

---

## 🔌 组件交互详解

### WorkflowClient ↔ WorkflowStorage

```rust
// Client启动工作流
impl WorkflowClient {
    pub async fn start_workflow<W: Workflow>(
        &self,
        workflow_id: WorkflowId,
        task_queue: String,
        input: W::Input,
        options: StartWorkflowOptions,
    ) -> Result<WorkflowHandle<W::Output>, ClientError> {
        // 1. 创建执行
        let execution = WorkflowExecution {
            workflow_id,
            run_id: RunId(Uuid::new_v4()),
        };
        
        // 2. 创建Started事件
        let event = WorkflowEvent::WorkflowExecutionStarted {
            workflow_id: execution.workflow_id.clone(),
            workflow_type: W::name().to_string(),
            input: serde_json::to_value(&input)?,
            timestamp: Utc::now(),
            event_id: EventId(0),
        };
        
        let mut history = EventHistory::new();
        history.append(event);
        
        // 3. 保存到存储
        self.storage.save_workflow_execution(&execution, &history).await?;
        
        // 4. 分发任务
        self.task_dispatcher.dispatch_workflow_task(WorkflowTask {
            workflow_execution: execution.clone(),
            workflow_type: W::name().to_string(),
            task_queue,
            input: serde_json::to_value(&input)?,
        }).await?;
        
        // 5. 返回Handle
        Ok(WorkflowHandle::new(execution, self.storage.clone()))
    }
}
```

### WorkflowWorker ↔ TaskPoller

```rust
// Worker轮询任务
impl WorkflowWorker {
    async fn poll_workflow_tasks(&self) -> Result<(), WorkerError> {
        let semaphore = Arc::new(Semaphore::new(
            self.config.max_concurrent_workflow_executions
        ));
        
        loop {
            // 1. 获取许可
            let permit = semaphore.clone().acquire_owned().await?;
            
            // 2. 轮询任务
            match self.task_poller.poll_workflow_tasks().await {
                Ok(Some(task)) => {
                    let worker = self.clone();
                    tokio::spawn(async move {
                        let _permit = permit;
                        // 3. 执行任务
                        worker.execute_workflow_task(task).await.ok();
                    });
                }
                Ok(None) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    tracing::error!("Failed to poll workflow task: {}", e);
                }
            }
        }
    }
}
```

### WorkflowContext ↔ EventHistory

```rust
// 工作流执行记录事件
impl WorkflowContext {
    pub async fn execute_activity<A: Activity>(
        &self,
        input: A::Input,
        options: ActivityOptions,
    ) -> Result<A::Output, WorkflowError> {
        let activity_id = ActivityId(Uuid::new_v4().to_string());
        
        // 1. 记录Scheduled事件
        self.history.write().unwrap().append(
            WorkflowEvent::ActivityTaskScheduled {
                activity_id: activity_id.clone(),
                activity_type: A::name().to_string(),
                input: serde_json::to_value(&input)?,
                timestamp: Utc::now(),
                event_id: EventId(0),
            }
        );
        
        // 2. 执行Activity (通过Worker)
        let result = self.execute_activity_internal::<A>(
            activity_id.clone(),
            input,
            options
        ).await;
        
        // 3. 记录完成/失败事件
        match &result {
            Ok(output) => {
                self.history.write().unwrap().append(
                    WorkflowEvent::ActivityTaskCompleted {
                        activity_id,
                        result: serde_json::to_value(output)?,
                        timestamp: Utc::now(),
                        event_id: EventId(0),
                    }
                );
            }
            Err(error) => {
                self.history.write().unwrap().append(
                    WorkflowEvent::ActivityTaskFailed {
                        activity_id,
                        error: error.to_string(),
                        retry_count: 0,
                        timestamp: Utc::now(),
                        event_id: EventId(0),
                    }
                );
            }
        }
        
        result
    }
}
```

---

## 🎯 设计决策

### 1. 为什么选择嵌入式存储抽象？

**原因**:

- ✅ 灵活部署: 支持单机和分布式部署
- ✅ 开发友好: 无需外部依赖即可开发测试
- ✅ 性能优化: 可针对特定场景优化存储
- ✅ 渐进式迁移: 可从嵌入式逐步迁移到Temporal Server

**权衡**:

- ⚠️ 实现复杂: 需要自己实现存储层
- ⚠️ 运维成本: 需要管理数据库

### 2. 为什么使用gRPC与Temporal Server通信？

**原因**:

- ✅ 官方协议: Temporal官方使用gRPC
- ✅ 性能优异: 高效的二进制协议
- ✅ 类型安全: Protocol Buffers提供强类型
- ✅ 跨语言: 支持多语言互操作

### 3. 为什么选择Tokio作为异步运行时？

**原因**:

- ✅ 成熟稳定: Rust生态最成熟的异步运行时
- ✅ 性能优异: 高性能、低延迟
- ✅ 生态丰富: 大量库支持
- ✅ 易于使用: 良好的API设计

### 4. 为什么使用trait抽象存储层？

**原因**:

- ✅ 可测试: 易于mock和单元测试
- ✅ 可扩展: 支持多种存储实现
- ✅ 解耦: SDK层不依赖具体存储
- ✅ 灵活: 可根据需求选择不同实现

---

## 📈 性能考虑

### 内存管理

- **Arc/RwLock**: 最小化锁竞争
- **事件历史**: 使用追加方式，避免全量复制
- **连接池**: 复用数据库连接

### 并发控制

```rust
pub struct WorkerConfig {
    /// 最大并发工作流执行数
    pub max_concurrent_workflow_executions: usize,
    
    /// 最大并发Activity执行数
    pub max_concurrent_activity_executions: usize,
    
    /// 轮询间隔
    pub poll_interval: Duration,
    
    /// 任务队列大小
    pub task_queue_size: usize,
}
```

### 批处理优化

- 事件批量写入
- 任务批量轮询
- 连接复用

---

## 🔐 安全考虑

### 认证和授权

- gRPC TLS加密
- 令牌认证
- 工作流权限控制

### 数据保护

- 敏感数据加密
- 审计日志
- 数据访问控制

---

## 📚 下一步

- **继续学习**: [类型系统设计](./03_type_system.md)
- **查看API**: [工作流定义](./04_workflow_definition.md)
- **了解实现**: [Worker实现](./11_worker_implementation.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
