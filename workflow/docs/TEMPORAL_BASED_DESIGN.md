# 基于Temporal框架的Rust 1.90工作流系统完整设计

## 📋 文档概述

**设计理念**: 完全遵循Temporal框架的设计哲学，使用Rust 1.90实现
**目标**: 创建Temporal-native的Rust工作流库
**版本**: 1.0.0-temporal-native

---

## 1. Temporal核心概念在Rust中的映射

### 1.1 架构对应关系

```text
┌─────────────────────────────────────────────────────────────────┐
│         Temporal概念 → Rust 1.90实现映射                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Temporal Workflow    →  #[workflow] async fn                   │
│  Temporal Activity    →  #[activity] async fn                   │
│  Temporal Worker      →  WorkflowWorker struct                  │
│  Task Queue           →  TaskQueue<T> (async channel)           │
│  Signal               →  Signal<T> (typed channel)              │
│  Query                →  Query<T> trait                         │
│  Child Workflow       →  ChildWorkflowHandle<T>                 │
│  Timer                →  WorkflowTimer (persistent)             │
│  Event History        →  EventLog (event sourcing)              │
│  Saga                 →  Saga<T> with compensation              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 核心类型系统

```rust
// 完全基于Temporal的类型设计

/// 工作流上下文 - Temporal WorkflowContext的Rust实现
pub struct WorkflowContext {
    /// 工作流ID
    workflow_id: WorkflowId,
    /// 运行ID
    run_id: RunId,
    /// 工作流类型
    workflow_type: String,
    /// 任务队列
    task_queue: String,
    /// 执行超时
    execution_timeout: Option<Duration>,
    /// 事件历史
    history: Arc<RwLock<EventHistory>>,
    /// Signal通道
    signals: Arc<SignalRegistry>,
    /// Query处理器
    queries: Arc<QueryRegistry>,
}

/// 工作流ID类型
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowId(String);

/// 运行ID类型
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunId(Uuid);

/// Activity上下文 - Temporal ActivityContext的Rust实现
pub struct ActivityContext {
    /// Activity ID
    activity_id: ActivityId,
    /// 工作流执行信息
    workflow_execution: WorkflowExecution,
    /// Activity类型
    activity_type: String,
    /// 心跳
    heartbeat: Arc<HeartbeatHandle>,
    /// 取消令牌
    cancellation: CancellationToken,
}

/// 工作流执行信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub workflow_id: WorkflowId,
    pub run_id: RunId,
}

/// Activity ID
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityId(String);
```

---

## 2. 基于Temporal的工作流定义

### 2.1 工作流宏定义

```rust
// 工作流定义宏 - 完全模仿Temporal的@workflow装饰器

/// 工作流定义宏
/// 
/// 用法:
/// ```rust
/// #[workflow]
/// async fn order_workflow(ctx: WorkflowContext, input: OrderInput) -> Result<OrderOutput> {
///     // 工作流逻辑
/// }
/// ```
#[proc_macro_attribute]
pub fn workflow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. 解析函数定义
    // 2. 验证签名 (必须是async fn，第一个参数是WorkflowContext)
    // 3. 生成工作流注册代码
    // 4. 添加确定性检查
    // 5. 生成事件记录代码
}

/// Activity定义宏
/// 
/// 用法:
/// ```rust
/// #[activity]
/// async fn process_payment(ctx: ActivityContext, input: PaymentInput) -> Result<PaymentResult> {
///     // Activity逻辑
/// }
/// ```
#[proc_macro_attribute]
pub fn activity(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. 解析函数定义
    // 2. 添加重试逻辑包装
    // 3. 添加超时控制
    // 4. 添加心跳支持
    // 5. 生成Activity注册代码
}
```

### 2.2 完整的工作流示例

```rust
use temporal_rust::prelude::*;

/// 订单处理工作流 - 完全Temporal风格
#[workflow]
pub async fn order_processing_workflow(
    ctx: WorkflowContext,
    input: OrderInput,
) -> Result<OrderOutput, WorkflowError> {
    // 1. 执行Activity - 验证订单
    let validation_result = ctx
        .execute_activity::<ValidateOrderActivity>(
            ActivityInput::new(input.clone()),
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(30),
                retry_policy: Some(RetryPolicy {
                    initial_interval: Duration::from_secs(1),
                    backoff_coefficient: 2.0,
                    maximum_interval: Duration::from_secs(60),
                    maximum_attempts: 3,
                    non_retryable_error_types: vec!["ValidationError".to_string()],
                }),
                ..Default::default()
            },
        )
        .await?;

    if !validation_result.is_valid {
        return Err(WorkflowError::ValidationFailed(validation_result.reason));
    }

    // 2. 并行执行多个Activities
    let (inventory_result, payment_result) = tokio::try_join!(
        ctx.execute_activity::<ReserveInventoryActivity>(
            ActivityInput::new(input.clone()),
            ActivityOptions::default(),
        ),
        ctx.execute_activity::<ProcessPaymentActivity>(
            ActivityInput::new(input.clone()),
            ActivityOptions::default(),
        ),
    )?;

    // 3. 使用Signal等待外部确认
    ctx.await_signal::<ApprovalSignal>("order_approval").await?;

    // 4. 发货
    let shipment_result = ctx
        .execute_activity::<ShipOrderActivity>(
            ActivityInput::new(input.clone()),
            ActivityOptions::default(),
        )
        .await?;

    // 5. 返回结果
    Ok(OrderOutput {
        order_id: input.order_id,
        status: OrderStatus::Completed,
        shipment_tracking: shipment_result.tracking_number,
    })
}

/// 验证订单Activity
#[activity]
pub async fn validate_order_activity(
    ctx: ActivityContext,
    input: OrderInput,
) -> Result<ValidationResult, ActivityError> {
    // 发送心跳
    ctx.heartbeat().await?;

    // 执行验证逻辑
    let is_valid = validate_order_logic(&input).await?;

    // 检查是否被取消
    if ctx.is_cancelled() {
        return Err(ActivityError::Cancelled);
    }

    Ok(ValidationResult {
        is_valid,
        reason: if is_valid { None } else { Some("Invalid order".to_string()) },
    })
}

/// 预订库存Activity
#[activity]
pub async fn reserve_inventory_activity(
    ctx: ActivityContext,
    input: OrderInput,
) -> Result<InventoryResult, ActivityError> {
    // 实际库存预订逻辑
    inventory_service::reserve(&input.items).await
        .map_err(|e| ActivityError::ExecutionError(e.to_string()))
}

/// 处理支付Activity
#[activity]
pub async fn process_payment_activity(
    ctx: ActivityContext,
    input: OrderInput,
) -> Result<PaymentResult, ActivityError> {
    // 实际支付处理逻辑
    payment_service::process(input.payment_info).await
        .map_err(|e| ActivityError::ExecutionError(e.to_string()))
}

/// 发货Activity
#[activity]
pub async fn ship_order_activity(
    ctx: ActivityContext,
    input: OrderInput,
) -> Result<ShipmentResult, ActivityError> {
    // 实际发货逻辑
    shipping_service::ship(&input).await
        .map_err(|e| ActivityError::ExecutionError(e.to_string()))
}
```

---

## 3. Signal和Query实现

### 3.1 Signal系统设计

```rust
/// Signal定义trait
pub trait Signal: Serialize + DeserializeOwned + Send + 'static {
    /// Signal名称
    fn name() -> &'static str;
}

/// Signal注册表
pub struct SignalRegistry {
    handlers: Arc<RwLock<HashMap<String, SignalHandler>>>,
    pending: Arc<RwLock<HashMap<String, VecDeque<Value>>>>,
}

type SignalHandler = Box<dyn Fn(Value) -> BoxFuture<'static, Result<(), SignalError>> + Send + Sync>;

impl SignalRegistry {
    /// 注册Signal处理器
    pub fn register<S, F, Fut>(&self, handler: F)
    where
        S: Signal,
        F: Fn(S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), SignalError>> + Send + 'static,
    {
        let boxed: SignalHandler = Box::new(move |value: Value| {
            let signal: S = serde_json::from_value(value).expect("Signal deserialization failed");
            Box::pin(handler(signal))
        });
        
        self.handlers.write().unwrap().insert(S::name().to_string(), boxed);
    }

    /// 发送Signal
    pub async fn send<S: Signal>(&self, signal: S) -> Result<(), SignalError> {
        let signal_name = S::name();
        let value = serde_json::to_value(&signal)?;

        // 如果有处理器，立即处理
        if let Some(handler) = self.handlers.read().unwrap().get(signal_name) {
            handler(value).await
        } else {
            // 否则加入待处理队列
            self.pending
                .write()
                .unwrap()
                .entry(signal_name.to_string())
                .or_insert_with(VecDeque::new)
                .push_back(value);
            Ok(())
        }
    }

    /// 等待Signal
    pub async fn await_signal<S: Signal>(&self) -> Result<S, SignalError> {
        let signal_name = S::name();
        
        loop {
            // 检查待处理队列
            if let Some(value) = self.pending
                .write()
                .unwrap()
                .get_mut(signal_name)
                .and_then(|q| q.pop_front())
            {
                return Ok(serde_json::from_value(value)?);
            }

            // 等待新Signal
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

/// 在WorkflowContext中使用Signal
impl WorkflowContext {
    /// 等待Signal
    pub async fn await_signal<S: Signal>(&self, signal_name: &str) -> Result<S, WorkflowError> {
        self.signals.await_signal().await
            .map_err(|e| WorkflowError::SignalError(e))
    }

    /// 发送Signal到其他工作流
    pub async fn signal_workflow<S: Signal>(
        &self,
        workflow_id: &WorkflowId,
        signal: S,
    ) -> Result<(), WorkflowError> {
        // 通过客户端发送Signal
        self.client.signal_workflow(workflow_id, S::name(), signal).await
    }
}

/// Signal使用示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSignal {
    pub approved: bool,
    pub approver: String,
    pub comment: Option<String>,
}

impl Signal for ApprovalSignal {
    fn name() -> &'static str {
        "approval"
    }
}

// 在工作流中使用
#[workflow]
async fn approval_workflow(ctx: WorkflowContext) -> Result<ApprovalResult> {
    // 等待审批Signal
    let approval = ctx.await_signal::<ApprovalSignal>("approval").await?;
    
    if approval.approved {
        Ok(ApprovalResult::Approved {
            approver: approval.approver,
        })
    } else {
        Ok(ApprovalResult::Rejected {
            reason: approval.comment.unwrap_or_default(),
        })
    }
}
```

### 3.2 Query系统设计

```rust
/// Query定义trait
pub trait Query: Serialize + DeserializeOwned + Send + 'static {
    /// Query名称
    fn name() -> &'static str;
    /// Query结果类型
    type Result: Serialize + DeserializeOwned + Send;
}

/// Query注册表
pub struct QueryRegistry {
    handlers: Arc<RwLock<HashMap<String, QueryHandler>>>,
}

type QueryHandler = Box<dyn Fn() -> BoxFuture<'static, Result<Value, QueryError>> + Send + Sync>;

impl QueryRegistry {
    /// 注册Query处理器
    pub fn register<Q, F, Fut>(&self, handler: F)
    where
        Q: Query,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Q::Result, QueryError>> + Send + 'static,
    {
        let boxed: QueryHandler = Box::new(move || {
            Box::pin(async move {
                let result = handler().await?;
                Ok(serde_json::to_value(&result)?)
            })
        });
        
        self.handlers.write().unwrap().insert(Q::name().to_string(), boxed);
    }

    /// 执行Query
    pub async fn execute<Q: Query>(&self) -> Result<Q::Result, QueryError> {
        let query_name = Q::name();
        
        let handler = self.handlers
            .read()
            .unwrap()
            .get(query_name)
            .ok_or_else(|| QueryError::NotFound(query_name.to_string()))?
            .clone();

        let value = handler().await?;
        Ok(serde_json::from_value(value)?)
    }
}

/// 在WorkflowContext中使用Query
impl WorkflowContext {
    /// 设置Query处理器
    pub fn set_query_handler<Q, F, Fut>(&self, handler: F)
    where
        Q: Query,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Q::Result, QueryError>> + Send + 'static,
    {
        self.queries.register::<Q, _, _>(handler);
    }

    /// 查询其他工作流
    pub async fn query_workflow<Q: Query>(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<Q::Result, WorkflowError> {
        self.client.query_workflow(workflow_id, Q::name()).await
            .map_err(|e| WorkflowError::QueryError(e))
    }
}

/// Query使用示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusQuery;

impl Query for WorkflowStatusQuery {
    fn name() -> &'static str {
        "status"
    }
    type Result = WorkflowStatus;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub current_state: String,
    pub progress: f64,
    pub started_at: DateTime<Utc>,
}

// 在工作流中使用
#[workflow]
async fn monitored_workflow(ctx: WorkflowContext) -> Result<Output> {
    let status = Arc::new(RwLock::new(WorkflowStatus {
        current_state: "initializing".to_string(),
        progress: 0.0,
        started_at: Utc::now(),
    }));

    let status_clone = status.clone();
    
    // 设置Query处理器
    ctx.set_query_handler::<WorkflowStatusQuery, _, _>(move || {
        let status = status_clone.clone();
        async move {
            Ok(status.read().unwrap().clone())
        }
    });

    // 执行工作流逻辑，更新状态
    *status.write().unwrap() = WorkflowStatus {
        current_state: "processing".to_string(),
        progress: 0.5,
        started_at: Utc::now(),
    };

    // ... 更多逻辑
    
    Ok(Output {})
}
```

---

## 4. 事件溯源和持久化

### 4.1 事件历史设计

```rust
/// 工作流事件 - Temporal Event History的Rust实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// 工作流开始
    WorkflowExecutionStarted {
        workflow_id: WorkflowId,
        workflow_type: String,
        input: Value,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Activity调度
    ActivityTaskScheduled {
        activity_id: ActivityId,
        activity_type: String,
        input: Value,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Activity开始
    ActivityTaskStarted {
        activity_id: ActivityId,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Activity完成
    ActivityTaskCompleted {
        activity_id: ActivityId,
        result: Value,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Activity失败
    ActivityTaskFailed {
        activity_id: ActivityId,
        error: String,
        retry_count: u32,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Timer启动
    TimerStarted {
        timer_id: TimerId,
        duration: Duration,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Timer触发
    TimerFired {
        timer_id: TimerId,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// Signal接收
    WorkflowSignalReceived {
        signal_name: String,
        signal_data: Value,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// 子工作流启动
    ChildWorkflowExecutionStarted {
        child_workflow_id: WorkflowId,
        workflow_type: String,
        input: Value,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// 工作流完成
    WorkflowExecutionCompleted {
        result: Value,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
    
    /// 工作流失败
    WorkflowExecutionFailed {
        error: String,
        timestamp: DateTime<Utc>,
        event_id: EventId,
    },
}

/// 事件ID
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventId(u64);

/// 事件历史
pub struct EventHistory {
    events: Vec<WorkflowEvent>,
    next_event_id: AtomicU64,
}

impl EventHistory {
    /// 追加事件
    pub fn append(&mut self, mut event: WorkflowEvent) -> EventId {
        let event_id = EventId(self.next_event_id.fetch_add(1, Ordering::SeqCst));
        
        // 设置事件ID
        match &mut event {
            WorkflowEvent::WorkflowExecutionStarted { event_id: id, .. } |
            WorkflowEvent::ActivityTaskScheduled { event_id: id, .. } |
            WorkflowEvent::ActivityTaskStarted { event_id: id, .. } |
            WorkflowEvent::ActivityTaskCompleted { event_id: id, .. } |
            WorkflowEvent::ActivityTaskFailed { event_id: id, .. } |
            WorkflowEvent::TimerStarted { event_id: id, .. } |
            WorkflowEvent::TimerFired { event_id: id, .. } |
            WorkflowEvent::WorkflowSignalReceived { event_id: id, .. } |
            WorkflowEvent::ChildWorkflowExecutionStarted { event_id: id, .. } |
            WorkflowEvent::WorkflowExecutionCompleted { event_id: id, .. } |
            WorkflowEvent::WorkflowExecutionFailed { event_id: id, .. } => {
                *id = event_id;
            }
        }
        
        self.events.push(event);
        event_id
    }

    /// 获取所有事件
    pub fn get_events(&self) -> &[WorkflowEvent] {
        &self.events
    }

    /// 从事件重建工作流状态
    pub fn replay(&self) -> WorkflowReplayState {
        let mut state = WorkflowReplayState::new();
        
        for event in &self.events {
            state.apply_event(event);
        }
        
        state
    }
}

/// 工作流重放状态
pub struct WorkflowReplayState {
    completed_activities: HashSet<ActivityId>,
    completed_timers: HashSet<TimerId>,
    received_signals: Vec<(String, Value)>,
}

impl WorkflowReplayState {
    pub fn new() -> Self {
        Self {
            completed_activities: HashSet::new(),
            completed_timers: HashSet::new(),
            received_signals: Vec::new(),
        }
    }

    pub fn apply_event(&mut self, event: &WorkflowEvent) {
        match event {
            WorkflowEvent::ActivityTaskCompleted { activity_id, .. } => {
                self.completed_activities.insert(activity_id.clone());
            }
            WorkflowEvent::TimerFired { timer_id, .. } => {
                self.completed_timers.insert(timer_id.clone());
            }
            WorkflowEvent::WorkflowSignalReceived { signal_name, signal_data, .. } => {
                self.received_signals.push((signal_name.clone(), signal_data.clone()));
            }
            _ => {}
        }
    }
}
```

### 4.2 持久化存储

```rust
/// 工作流存储trait - Temporal Persistence的Rust抽象
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

    /// 保存Activity心跳
    async fn save_activity_heartbeat(
        &self,
        activity_id: &ActivityId,
        details: Value,
    ) -> Result<(), StorageError>;
}

/// PostgreSQL存储实现
pub struct PostgresWorkflowStorage {
    pool: PgPool,
}

#[async_trait]
impl WorkflowStorage for PostgresWorkflowStorage {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        history: &EventHistory,
    ) -> Result<(), StorageError> {
        let mut tx = self.pool.begin().await?;

        // 保存工作流执行记录
        sqlx::query!(
            r#"
            INSERT INTO workflow_executions (workflow_id, run_id, status, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (workflow_id, run_id) DO UPDATE
            SET status = $3
            "#,
            execution.workflow_id.0,
            execution.run_id.0,
            "running",
            Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        // 保存事件历史
        for event in history.get_events() {
            let event_json = serde_json::to_value(event)?;
            sqlx::query!(
                r#"
                INSERT INTO workflow_events (workflow_id, run_id, event_id, event_data, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                execution.workflow_id.0,
                execution.run_id.0,
                event.event_id().0 as i64,
                event_json,
                Utc::now(),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn load_workflow_execution(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<(WorkflowExecution, EventHistory), StorageError> {
        // 加载工作流执行
        let exec_row = sqlx::query!(
            r#"
            SELECT workflow_id, run_id
            FROM workflow_executions
            WHERE workflow_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            workflow_id.0,
        )
        .fetch_one(&self.pool)
        .await?;

        let execution = WorkflowExecution {
            workflow_id: WorkflowId(exec_row.workflow_id),
            run_id: RunId(Uuid::parse_str(&exec_row.run_id)?),
        };

        // 加载事件历史
        let event_rows = sqlx::query!(
            r#"
            SELECT event_data
            FROM workflow_events
            WHERE workflow_id = $1 AND run_id = $2
            ORDER BY event_id ASC
            "#,
            execution.workflow_id.0,
            execution.run_id.0.to_string(),
        )
        .fetch_all(&self.pool)
        .await?;

        let mut history = EventHistory::new();
        for row in event_rows {
            let event: WorkflowEvent = serde_json::from_value(row.event_data)?;
            history.append(event);
        }

        Ok((execution, history))
    }

    // ... 其他方法实现
}
```

---

## 5. Worker实现

### 5.1 WorkflowWorker设计

```rust
/// 工作流Worker - Temporal Worker的Rust实现
pub struct WorkflowWorker {
    /// Worker配置
    config: WorkerConfig,
    /// 任务队列
    task_queue: String,
    /// 已注册的工作流
    workflows: Arc<RwLock<HashMap<String, WorkflowFactory>>>,
    /// 已注册的Activities
    activities: Arc<RwLock<HashMap<String, ActivityFactory>>>,
    /// 存储
    storage: Arc<dyn WorkflowStorage>,
    /// 取消令牌
    cancellation: CancellationToken,
}

type WorkflowFactory = Arc<dyn Fn(WorkflowContext, Value) -> BoxFuture<'static, Result<Value, WorkflowError>> + Send + Sync>;
type ActivityFactory = Arc<dyn Fn(ActivityContext, Value) -> BoxFuture<'static, Result<Value, ActivityError>> + Send + Sync>;

/// Worker配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// 最大并发工作流执行数
    pub max_concurrent_workflow_executions: usize,
    /// 最大并发Activity执行数
    pub max_concurrent_activity_executions: usize,
    /// Worker身份
    pub identity: String,
}

impl WorkflowWorker {
    /// 创建新Worker
    pub fn new(
        task_queue: String,
        storage: Arc<dyn WorkflowStorage>,
        config: WorkerConfig,
    ) -> Self {
        Self {
            config,
            task_queue,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            activities: Arc::new(RwLock::new(HashMap::new())),
            storage,
            cancellation: CancellationToken::new(),
        }
    }

    /// 注册工作流
    pub fn register_workflow<F, Fut, I, O>(
        &mut self,
        workflow_type: String,
        factory: F,
    ) where
        F: Fn(WorkflowContext, I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, WorkflowError>> + Send + 'static,
        I: DeserializeOwned + Send + 'static,
        O: Serialize + Send + 'static,
    {
        let factory: WorkflowFactory = Arc::new(move |ctx, input| {
            let input: I = serde_json::from_value(input).expect("Workflow input deserialization failed");
            Box::pin(async move {
                let output = factory(ctx, input).await?;
                Ok(serde_json::to_value(&output)?)
            })
        });

        self.workflows.write().unwrap().insert(workflow_type, factory);
    }

    /// 注册Activity
    pub fn register_activity<F, Fut, I, O>(
        &mut self,
        activity_type: String,
        factory: F,
    ) where
        F: Fn(ActivityContext, I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, ActivityError>> + Send + 'static,
        I: DeserializeOwned + Send + 'static,
        O: Serialize + Send + 'static,
    {
        let factory: ActivityFactory = Arc::new(move |ctx, input| {
            let input: I = serde_json::from_value(input).expect("Activity input deserialization failed");
            Box::pin(async move {
                let output = factory(ctx, input).await?;
                Ok(serde_json::to_value(&output)?)
            })
        });

        self.activities.write().unwrap().insert(activity_type, factory);
    }

    /// 启动Worker
    pub async fn start(self: Arc<Self>) -> Result<(), WorkerError> {
        let workflow_worker = self.clone();
        let activity_worker = self.clone();

        // 启动工作流轮询
        let workflow_handle = tokio::spawn(async move {
            workflow_worker.poll_workflow_tasks().await
        });

        // 启动Activity轮询
        let activity_handle = tokio::spawn(async move {
            activity_worker.poll_activity_tasks().await
        });

        // 等待完成或取消
        tokio::select! {
            _ = workflow_handle => {},
            _ = activity_handle => {},
            _ = self.cancellation.cancelled() => {},
        }

        Ok(())
    }

    /// 轮询工作流任务
    async fn poll_workflow_tasks(&self) -> Result<(), WorkerError> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_workflow_executions));

        loop {
            if self.cancellation.is_cancelled() {
                break;
            }

            // 获取许可
            let permit = semaphore.clone().acquire_owned().await?;

            // 轮询任务
            match self.poll_workflow_task_once().await {
                Ok(Some(task)) => {
                    let worker = self.clone();
                    tokio::spawn(async move {
                        let _permit = permit;
                        worker.execute_workflow_task(task).await.ok();
                    });
                }
                Ok(None) => {
                    // 没有任务，等待
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    tracing::error!("Failed to poll workflow task: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    /// 执行工作流任务
    async fn execute_workflow_task(&self, task: WorkflowTask) -> Result<(), WorkflowError> {
        // 创建工作流上下文
        let ctx = WorkflowContext::new(
            task.workflow_execution.clone(),
            task.workflow_type.clone(),
            self.task_queue.clone(),
            self.storage.clone(),
        );

        // 获取工作流工厂
        let factory = self.workflows
            .read()
            .unwrap()
            .get(&task.workflow_type)
            .ok_or_else(|| WorkflowError::WorkflowNotRegistered(task.workflow_type.clone()))?
            .clone();

        // 执行工作流
        let result = factory(ctx, task.input).await;

        // 处理结果
        match result {
            Ok(output) => {
                // 记录完成事件
                self.storage.append_event(
                    &task.workflow_execution.workflow_id,
                    WorkflowEvent::WorkflowExecutionCompleted {
                        result: output,
                        timestamp: Utc::now(),
                        event_id: EventId(0), // Will be set by append_event
                    },
                ).await?;
            }
            Err(error) => {
                // 记录失败事件
                self.storage.append_event(
                    &task.workflow_execution.workflow_id,
                    WorkflowEvent::WorkflowExecutionFailed {
                        error: error.to_string(),
                        timestamp: Utc::now(),
                        event_id: EventId(0),
                    },
                ).await?;
            }
        }

        Ok(())
    }

    // ... Activity轮询和执行类似
}
```

---

## 6. 客户端实现

### 6.1 WorkflowClient设计

```rust
/// 工作流客户端 - Temporal Client的Rust实现
pub struct WorkflowClient {
    /// 存储
    storage: Arc<dyn WorkflowStorage>,
    /// 任务分发器
    task_dispatcher: Arc<TaskDispatcher>,
}

impl WorkflowClient {
    /// 启动工作流
    pub async fn start_workflow<W, I, O>(
        &self,
        workflow_id: WorkflowId,
        task_queue: String,
        input: I,
        options: StartWorkflowOptions,
    ) -> Result<WorkflowHandle<O>, ClientError>
    where
        W: Workflow<Input = I, Output = O>,
        I: Serialize + Send + 'static,
        O: DeserializeOwned + Send + 'static,
    {
        // 创建工作流执行
        let execution = WorkflowExecution {
            workflow_id: workflow_id.clone(),
            run_id: RunId(Uuid::new_v4()),
        };

        // 记录启动事件
        let event = WorkflowEvent::WorkflowExecutionStarted {
            workflow_id: workflow_id.clone(),
            workflow_type: W::name().to_string(),
            input: serde_json::to_value(&input)?,
            timestamp: Utc::now(),
            event_id: EventId(0),
        };

        let mut history = EventHistory::new();
        history.append(event);

        // 保存到存储
        self.storage.save_workflow_execution(&execution, &history).await?;

        // 分发任务到Worker
        self.task_dispatcher.dispatch_workflow_task(WorkflowTask {
            workflow_execution: execution.clone(),
            workflow_type: W::name().to_string(),
            task_queue,
            input: serde_json::to_value(&input)?,
        }).await?;

        // 返回Handle
        Ok(WorkflowHandle::new(execution, self.storage.clone()))
    }

    /// 发送Signal
    pub async fn signal_workflow<S: Signal>(
        &self,
        workflow_id: &WorkflowId,
        signal: S,
    ) -> Result<(), ClientError> {
        // 记录Signal事件
        let event = WorkflowEvent::WorkflowSignalReceived {
            signal_name: S::name().to_string(),
            signal_data: serde_json::to_value(&signal)?,
            timestamp: Utc::now(),
            event_id: EventId(0),
        };

        self.storage.append_event(workflow_id, event).await?;
        Ok(())
    }

    /// 查询工作流
    pub async fn query_workflow<Q: Query>(
        &self,
        workflow_id: &WorkflowId,
        query_name: &str,
    ) -> Result<Q::Result, ClientError> {
        // 向Worker发送查询请求
        // 这里需要与Worker通信
        // 简化实现，实际需要RPC机制
        todo!("Query implementation requires RPC")
    }

    /// 获取工作流Handle
    pub fn get_workflow_handle<O: DeserializeOwned>(
        &self,
        workflow_id: WorkflowId,
        run_id: RunId,
    ) -> WorkflowHandle<O> {
        WorkflowHandle::new(
            WorkflowExecution { workflow_id, run_id },
            self.storage.clone(),
        )
    }
}

/// 工作流Handle
pub struct WorkflowHandle<O> {
    execution: WorkflowExecution,
    storage: Arc<dyn WorkflowStorage>,
    _phantom: PhantomData<O>,
}

impl<O: DeserializeOwned> WorkflowHandle<O> {
    fn new(execution: WorkflowExecution, storage: Arc<dyn WorkflowStorage>) -> Self {
        Self {
            execution,
            storage,
            _phantom: PhantomData,
        }
    }

    /// 获取工作流ID
    pub fn workflow_id(&self) -> &WorkflowId {
        &self.execution.workflow_id
    }

    /// 获取运行ID
    pub fn run_id(&self) -> &RunId {
        &self.execution.run_id
    }

    /// 等待完成
    pub async fn get_result(&self) -> Result<O, WorkflowError> {
        loop {
            // 加载事件历史
            let (_, history) = self.storage
                .load_workflow_execution(&self.execution.workflow_id)
                .await?;

            // 检查是否完成
            if let Some(last_event) = history.get_events().last() {
                match last_event {
                    WorkflowEvent::WorkflowExecutionCompleted { result, .. } => {
                        return Ok(serde_json::from_value(result.clone())?);
                    }
                    WorkflowEvent::WorkflowExecutionFailed { error, .. } => {
                        return Err(WorkflowError::ExecutionFailed(error.clone()));
                    }
                    _ => {}
                }
            }

            // 等待一段时间后重试
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// 发送Signal
    pub async fn signal<S: Signal>(&self, signal: S) -> Result<(), WorkflowError> {
        let event = WorkflowEvent::WorkflowSignalReceived {
            signal_name: S::name().to_string(),
            signal_data: serde_json::to_value(&signal)?,
            timestamp: Utc::now(),
            event_id: EventId(0),
        };

        self.storage.append_event(&self.execution.workflow_id, event).await?;
        Ok(())
    }

    /// 取消工作流
    pub async fn cancel(&self) -> Result<(), WorkflowError> {
        // 实现取消逻辑
        todo!("Cancel workflow")
    }
}
```

---

## 7. 完整使用示例

### 7.1 基本使用流程

```rust
use temporal_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 设置存储
    let storage = Arc::new(PostgresWorkflowStorage::new("postgres://...").await?);

    // 2. 创建Worker
    let mut worker = WorkflowWorker::new(
        "my-task-queue".to_string(),
        storage.clone(),
        WorkerConfig::default(),
    );

    // 3. 注册工作流和Activities
    worker.register_workflow(
        "OrderProcessingWorkflow".to_string(),
        order_processing_workflow,
    );
    
    worker.register_activity(
        "ValidateOrderActivity".to_string(),
        validate_order_activity,
    );
    
    worker.register_activity(
        "ProcessPaymentActivity".to_string(),
        process_payment_activity,
    );

    // 4. 启动Worker (在后台线程)
    let worker = Arc::new(worker);
    let worker_handle = tokio::spawn({
        let worker = worker.clone();
        async move {
            worker.start().await
        }
    });

    // 5. 创建客户端
    let client = WorkflowClient::new(storage.clone());

    // 6. 启动工作流
    let handle = client.start_workflow::<OrderProcessingWorkflow, _, _>(
        WorkflowId(format!("order-{}", Uuid::new_v4())),
        "my-task-queue".to_string(),
        OrderInput {
            order_id: "ORDER-123".to_string(),
            customer_id: "CUST-456".to_string(),
            items: vec![/* ... */],
            payment_info: PaymentInfo {/* ... */},
        },
        StartWorkflowOptions::default(),
    ).await?;

    println!("Started workflow: {:?}", handle.workflow_id());

    // 7. 发送Signal (可选)
    tokio::time::sleep(Duration::from_secs(5)).await;
    handle.signal(ApprovalSignal {
        approved: true,
        approver: "admin".to_string(),
        comment: Some("Approved".to_string()),
    }).await?;

    // 8. 等待结果
    let result = handle.get_result().await?;
    println!("Workflow completed: {:?}", result);

    Ok(())
}
```

---

## 8. 文档结构规划

建议的文档目录结构：

```text
workflow/docs/
├── temporal_rust/                    # 基于Temporal的Rust实现
│   ├── 01_overview.md               # 概述
│   ├── 02_architecture.md           # 架构设计
│   ├── 03_workflow_definition.md    # 工作流定义
│   ├── 04_activity_definition.md    # Activity定义
│   ├── 05_signals_and_queries.md    # Signal和Query
│   ├── 06_event_sourcing.md         # 事件溯源
│   ├── 07_worker_implementation.md  # Worker实现
│   ├── 08_client_usage.md           # 客户端使用
│   ├── 09_testing.md                # 测试
│   └── 10_deployment.md             # 部署
├── api_reference/                    # API参考
│   ├── workflow_context.md
│   ├── activity_context.md
│   ├── workflow_client.md
│   └── worker_config.md
├── examples/                         # 示例
│   ├── basic_workflow.md
│   ├── saga_pattern.md
│   ├── signal_query_example.md
│   └── child_workflow_example.md
└── deprecated/                       # 过时文档(迁移旧文档到这里)
    ├── old_design/
    └── legacy_examples/
```

---

**文档版本**: 1.0.0-temporal-native  
**最后更新**: 2025-10-26  
**作者**: workflow_rust团队
