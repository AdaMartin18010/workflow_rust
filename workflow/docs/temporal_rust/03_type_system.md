# 类型系统设计

## 📋 文档概述

本文档详细阐述基于Temporal的Rust 1.90工作流系统的类型系统设计，包括：

- 核心类型定义
- Trait体系
- 泛型约束
- 生命周期设计
- 类型安全保证
- Rust 1.90特性应用

---

## 🎯 设计目标

### 类型安全优先

1. **编译时验证**: 尽可能在编译时捕获错误
2. **零成本抽象**: 类型系统不增加运行时开销
3. **表达力**: 类型系统能够准确表达业务逻辑
4. **可维护性**: 清晰的类型定义便于理解和维护

### Temporal概念映射

完全遵循Temporal的类型模型，提供类型安全的Rust实现。

---

## 🏗️ 核心类型体系

### 类型层次图

```text
┌──────────────────────────────────────────────────────────────────┐
│                      核心类型体系                                 │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  标识符类型 (Identifier Types)                                    │
│  ├─ WorkflowId(String)                                            │
│  ├─ RunId(Uuid)                                                   │
│  ├─ ActivityId(String)                                            │
│  ├─ TimerId(String)                                               │
│  └─ EventId(u64)                                                  │
│                                                                    │
│  执行类型 (Execution Types)                                       │
│  ├─ WorkflowExecution { workflow_id, run_id }                    │
│  ├─ ActivityExecution { activity_id, ... }                       │
│  └─ ChildWorkflowExecution { ... }                               │
│                                                                    │
│  上下文类型 (Context Types)                                       │
│  ├─ WorkflowContext                                               │
│  ├─ ActivityContext                                               │
│  └─ ChildWorkflowContext                                          │
│                                                                    │
│  选项类型 (Options Types)                                         │
│  ├─ StartWorkflowOptions                                          │
│  ├─ ActivityOptions                                               │
│  ├─ RetryPolicy                                                   │
│  ├─ ChildWorkflowOptions                                          │
│  └─ WorkerConfig                                                  │
│                                                                    │
│  事件类型 (Event Types)                                           │
│  ├─ WorkflowEvent (enum)                                          │
│  ├─ EventHistory                                                  │
│  └─ EventId                                                       │
│                                                                    │
│  Trait类型 (Trait Types)                                          │
│  ├─ Workflow                                                      │
│  ├─ Activity                                                      │
│  ├─ Signal                                                        │
│  ├─ Query                                                         │
│  └─ WorkflowStorage                                               │
│                                                                    │
│  错误类型 (Error Types)                                           │
│  ├─ WorkflowError                                                 │
│  ├─ ActivityError                                                 │
│  ├─ SignalError                                                   │
│  ├─ QueryError                                                    │
│  └─ StorageError                                                  │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

---

## 📦 核心类型详解

### 1. 标识符类型 (Identifier Types)

所有标识符都使用新类型模式(Newtype Pattern)以提供类型安全。

#### 1.1 WorkflowId

```rust
/// 工作流ID - 唯一标识一个工作流
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowId(pub String);

impl WorkflowId {
    /// 创建新的工作流ID
    pub fn new(id: impl Into<String>) -> Self {
        WorkflowId(id.into())
    }
    
    /// 生成随机ID
    pub fn generate() -> Self {
        WorkflowId(format!("workflow-{}", Uuid::new_v4()))
    }
    
    /// 获取内部字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for WorkflowId {
    fn from(s: String) -> Self {
        WorkflowId(s)
    }
}

impl From<&str> for WorkflowId {
    fn from(s: &str) -> Self {
        WorkflowId(s.to_string())
    }
}
```

**设计要点**:

- ✅ **类型安全**: 不能将String误用为WorkflowId
- ✅ **可序列化**: 支持JSON序列化
- ✅ **可哈希**: 可用作HashMap的键
- ✅ **便捷转换**: From trait实现便捷转换

#### 1.2 RunId

```rust
/// 运行ID - 标识工作流的一次具体执行
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl RunId {
    /// 生成新的运行ID
    pub fn generate() -> Self {
        RunId(Uuid::new_v4())
    }
    
    /// 从字符串解析
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(RunId(Uuid::parse_str(s)?))
    }
    
    /// 转为字符串
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**设计要点**:

- ✅ **UUID类型**: 保证全局唯一性
- ✅ **Copy trait**: 轻量级复制
- ✅ **类型安全**: 与WorkflowId区分

#### 1.3 ActivityId

```rust
/// Activity ID - 标识工作流中的一个Activity
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityId(pub String);

impl ActivityId {
    pub fn new(id: impl Into<String>) -> Self {
        ActivityId(id.into())
    }
    
    pub fn generate() -> Self {
        ActivityId(format!("activity-{}", Uuid::new_v4()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

#### 1.4 EventId

```rust
/// 事件ID - 事件历史中的序号
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl EventId {
    pub fn zero() -> Self {
        EventId(0)
    }
    
    pub fn next(&self) -> Self {
        EventId(self.0 + 1)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**设计要点**:

- ✅ **有序**: 实现Ord trait，支持排序
- ✅ **Copy**: 轻量级类型
- ✅ **递增**: 提供next()方法

### 2. 执行类型 (Execution Types)

#### 2.1 WorkflowExecution

```rust
/// 工作流执行 - 标识一次具体的工作流运行
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// 工作流ID
    pub workflow_id: WorkflowId,
    /// 运行ID
    pub run_id: RunId,
}

impl WorkflowExecution {
    /// 创建新的执行
    pub fn new(workflow_id: WorkflowId) -> Self {
        Self {
            workflow_id,
            run_id: RunId::generate(),
        }
    }
    
    /// 创建带指定run_id的执行
    pub fn with_run_id(workflow_id: WorkflowId, run_id: RunId) -> Self {
        Self {
            workflow_id,
            run_id,
        }
    }
}

impl fmt::Display for WorkflowExecution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.workflow_id, self.run_id)
    }
}
```

**设计要点**:

- ✅ **复合键**: workflow_id + run_id唯一标识一次执行
- ✅ **类型安全**: 两个ID类型明确区分
- ✅ **不可变**: 执行创建后ID不变

### 3. 上下文类型 (Context Types)

#### 3.1 WorkflowContext

```rust
/// 工作流上下文 - 提供工作流执行环境
pub struct WorkflowContext {
    /// 工作流执行信息
    pub(crate) execution: WorkflowExecution,
    
    /// 工作流类型
    pub(crate) workflow_type: String,
    
    /// 任务队列
    pub(crate) task_queue: String,
    
    /// 执行超时
    pub(crate) execution_timeout: Option<Duration>,
    
    /// 运行超时
    pub(crate) run_timeout: Option<Duration>,
    
    /// 事件历史
    pub(crate) history: Arc<RwLock<EventHistory>>,
    
    /// Signal注册表
    pub(crate) signals: Arc<SignalRegistry>,
    
    /// Query注册表
    pub(crate) queries: Arc<QueryRegistry>,
    
    /// 客户端（用于启动子工作流等）
    pub(crate) client: Arc<WorkflowClient>,
}

impl WorkflowContext {
    /// 获取工作流ID
    pub fn workflow_id(&self) -> &WorkflowId {
        &self.execution.workflow_id
    }
    
    /// 获取运行ID
    pub fn run_id(&self) -> &RunId {
        &self.execution.run_id
    }
    
    /// 获取工作流类型
    pub fn workflow_type(&self) -> &str {
        &self.workflow_type
    }
    
    /// 获取工作流信息
    pub fn get_info(&self) -> WorkflowInfo {
        WorkflowInfo {
            workflow_type: self.workflow_type.clone(),
            workflow_execution: self.execution.clone(),
            task_queue: self.task_queue.clone(),
        }
    }
    
    /// 执行Activity
    pub async fn execute_activity<A: Activity>(
        &self,
        input: A::Input,
        options: ActivityOptions,
    ) -> Result<A::Output, WorkflowError> {
        // 实现...
    }
    
    /// 等待Signal
    pub async fn await_signal<S: Signal>(&self, signal_name: &str) -> Result<S, WorkflowError> {
        // 实现...
    }
    
    /// 注册Query处理器
    pub fn set_query_handler<Q, F, Fut>(&self, handler: F)
    where
        Q: Query,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Q::Result, QueryError>> + Send + 'static,
    {
        self.queries.register::<Q, _, _>(handler);
    }
    
    /// 定时器
    pub async fn sleep(&self, duration: Duration) {
        // 实现...
    }
    
    /// 启动子工作流
    pub async fn start_child_workflow<W: Workflow>(
        &self,
        input: W::Input,
        options: ChildWorkflowOptions,
    ) -> Result<ChildWorkflowHandle<W::Output>, WorkflowError> {
        // 实现...
    }
}
```

**设计要点**:

- ✅ **生命周期安全**: 所有引用都有明确的生命周期
- ✅ **泛型方法**: execute_activity等使用泛型提供类型安全
- ✅ **Arc共享**: 可安全地在多个异步任务间共享
- ✅ **内部可变性**: 使用RwLock允许内部状态修改

#### 3.2 ActivityContext

```rust
/// Activity上下文 - 提供Activity执行环境
pub struct ActivityContext {
    /// Activity ID
    pub(crate) activity_id: ActivityId,
    
    /// 工作流执行信息
    pub(crate) workflow_execution: WorkflowExecution,
    
    /// Activity类型
    pub(crate) activity_type: String,
    
    /// 心跳句柄
    pub(crate) heartbeat: Arc<HeartbeatHandle>,
    
    /// 取消令牌
    pub(crate) cancellation: CancellationToken,
    
    /// Activity信息
    pub(crate) info: ActivityInfo,
}

impl ActivityContext {
    /// 获取Activity ID
    pub fn activity_id(&self) -> &ActivityId {
        &self.activity_id
    }
    
    /// 获取工作流执行信息
    pub fn workflow_execution(&self) -> &WorkflowExecution {
        &self.workflow_execution
    }
    
    /// 发送心跳
    pub async fn heartbeat(&self) -> Result<(), ActivityError> {
        self.heartbeat.record().await
    }
    
    /// 发送带详情的心跳
    pub async fn heartbeat_with_details(
        &self,
        details: impl Serialize,
    ) -> Result<(), ActivityError> {
        self.heartbeat.record_with_details(details).await
    }
    
    /// 检查是否被取消
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }
    
    /// 等待取消
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await
    }
    
    /// 获取Activity信息
    pub fn get_info(&self) -> &ActivityInfo {
        &self.info
    }
}
```

**设计要点**:

- ✅ **不可变引用**: 大部分方法只需要&self
- ✅ **取消安全**: 通过CancellationToken实现优雅取消
- ✅ **心跳抽象**: HeartbeatHandle封装心跳逻辑

### 4. Trait体系

#### 4.1 Workflow Trait

```rust
/// Workflow trait - 定义工作流接口
pub trait Workflow: Send + Sync + 'static {
    /// 输入类型
    type Input: DeserializeOwned + Send + 'static;
    
    /// 输出类型
    type Output: Serialize + Send + 'static;
    
    /// 工作流名称
    fn name() -> &'static str;
    
    /// 执行工作流
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send;
}
```

**设计要点**:

- ✅ **关联类型**: Input和Output明确定义
- ✅ **Send + Sync**: 支持跨线程安全传递
- ✅ **'static**: 不依赖外部生命周期
- ✅ **异步**: execute返回Future

**使用示例**:

```rust
// 定义工作流
pub struct OrderWorkflow;

impl Workflow for OrderWorkflow {
    type Input = OrderInput;
    type Output = OrderOutput;
    
    fn name() -> &'static str {
        "OrderWorkflow"
    }
    
    fn execute(
        ctx: WorkflowContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, WorkflowError>> + Send {
        async move {
            // 工作流逻辑
            let payment_result = ctx
                .execute_activity::<ProcessPaymentActivity>(
                    input.payment_info,
                    ActivityOptions::default(),
                )
                .await?;
            
            Ok(OrderOutput {
                order_id: input.order_id,
                status: OrderStatus::Completed,
            })
        }
    }
}
```

#### 4.2 Activity Trait

```rust
/// Activity trait - 定义Activity接口
pub trait Activity: Send + Sync + 'static {
    /// 输入类型
    type Input: DeserializeOwned + Send + 'static;
    
    /// 输出类型
    type Output: Serialize + Send + 'static;
    
    /// Activity名称
    fn name() -> &'static str;
    
    /// 执行Activity
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send;
}
```

**使用示例**:

```rust
// 定义Activity
pub struct ProcessPaymentActivity;

impl Activity for ProcessPaymentActivity {
    type Input = PaymentInput;
    type Output = PaymentResult;
    
    fn name() -> &'static str {
        "ProcessPayment"
    }
    
    fn execute(
        ctx: ActivityContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, ActivityError>> + Send {
        async move {
            // 发送心跳
            ctx.heartbeat().await?;
            
            // 检查取消
            if ctx.is_cancelled() {
                return Err(ActivityError::Cancelled);
            }
            
            // 执行支付逻辑
            let result = payment_service::process(&input).await?;
            
            Ok(result)
        }
    }
}
```

#### 4.3 Signal Trait

```rust
/// Signal trait - 定义Signal接口
pub trait Signal: Serialize + DeserializeOwned + Send + 'static {
    /// Signal名称
    fn name() -> &'static str;
}
```

**使用示例**:

```rust
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
```

#### 4.4 Query Trait

```rust
/// Query trait - 定义Query接口
pub trait Query: Send + 'static {
    /// Query名称
    fn name() -> &'static str;
    
    /// 结果类型
    type Result: Serialize + DeserializeOwned + Send;
}
```

**使用示例**:

```rust
pub struct WorkflowStatusQuery;

impl Query for WorkflowStatusQuery {
    fn name() -> &'static str {
        "status"
    }
    
    type Result = WorkflowStatus;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub current_step: String,
    pub progress: f64,
    pub started_at: DateTime<Utc>,
}
```

#### 4.5 WorkflowStorage Trait

```rust
/// 工作流存储trait - 持久化抽象
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
```

**设计要点**:

- ✅ **异步**: 所有方法都是异步的
- ✅ **错误处理**: 使用Result类型
- ✅ **可测试**: 易于mock
- ✅ **可扩展**: 支持多种存储后端

---

## 🔐 类型安全保证

### 1. 编译时类型检查

```rust
// ✅ 类型安全: 编译通过
let workflow_id = WorkflowId::new("order-123");
let run_id = RunId::generate();
let execution = WorkflowExecution::new(workflow_id);

// ❌ 类型错误: 编译失败
let execution = WorkflowExecution::new(run_id); // Error: 期望WorkflowId，得到RunId
```

### 2. 泛型约束

```rust
// Activity执行保证类型匹配
impl WorkflowContext {
    pub async fn execute_activity<A: Activity>(
        &self,
        input: A::Input,  // 输入类型必须匹配Activity定义
        options: ActivityOptions,
    ) -> Result<A::Output, WorkflowError> {  // 输出类型由Activity决定
        // ...
    }
}

// 使用
let result: PaymentResult = ctx
    .execute_activity::<ProcessPaymentActivity>(
        payment_input,  // 必须是PaymentInput类型
        options,
    )
    .await?;  // 返回PaymentResult类型
```

### 3. 生命周期安全

```rust
// 生命周期参数确保引用有效性
pub struct WorkflowHandle<'a, O> {
    execution: &'a WorkflowExecution,
    storage: Arc<dyn WorkflowStorage>,
    _phantom: PhantomData<O>,
}

impl<'a, O: DeserializeOwned> WorkflowHandle<'a, O> {
    pub async fn get_result(&self) -> Result<O, WorkflowError> {
        // execution引用保证在Handle生命周期内有效
    }
}
```

---

## 🚀 Rust 1.90特性应用

### 1. 改进的类型推断

```rust
// Rust 1.90: 更好的泛型推断
let handle = client.start_workflow(
    WorkflowId::new("order-123"),
    "my-queue".to_string(),
    OrderInput { /* ... */ },
    StartWorkflowOptions::default(),
).await?;

// 返回类型自动推断为 WorkflowHandle<OrderOutput>
let result = handle.get_result().await?;  // result类型: OrderOutput
```

### 2. const泛型

```rust
/// 使用const泛型定义固定大小的工作流定义
pub struct StaticWorkflowDefinition<const MAX_STATES: usize, const MAX_TRANSITIONS: usize> {
    name: String,
    states: [Option<String>; MAX_STATES],
    transitions: [Option<StateTransition>; MAX_TRANSITIONS],
    state_count: usize,
    transition_count: usize,
}

impl<const MAX_STATES: usize, const MAX_TRANSITIONS: usize> 
    StaticWorkflowDefinition<MAX_STATES, MAX_TRANSITIONS> 
{
    pub const fn new(name: String) -> Self {
        Self {
            name,
            states: [const { None }; MAX_STATES],
            transitions: [const { None }; MAX_TRANSITIONS],
            state_count: 0,
            transition_count: 0,
        }
    }
}

// 使用
let workflow: StaticWorkflowDefinition<10, 20> = StaticWorkflowDefinition::new("MyWorkflow".to_string());
```

### 3. async trait方法

```rust
// Rust 1.90: trait中可以直接使用async fn
pub trait WorkflowStorage {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        history: &EventHistory,
    ) -> Result<(), StorageError>;
}

// 不再需要#[async_trait]宏！
```

---

## 📊 类型关系图

```text
┌────────────────────────────────────────────────────────────┐
│                     类型依赖关系                            │
└────────────────────────────────────────────────────────────┘

WorkflowClient
    │
    ├─ uses ─> WorkflowStorage trait
    │           ├─ PostgresWorkflowStorage
    │           ├─ MySQLWorkflowStorage
    │           └─ InMemoryWorkflowStorage
    │
    └─ produces ─> WorkflowHandle<O>
                    └─ contains ─> WorkflowExecution
                                    ├─ WorkflowId
                                    └─ RunId

WorkflowWorker
    │
    ├─ contains ─> HashMap<String, WorkflowFactory>
    │               └─ calls ─> Workflow trait implementations
    │
    └─ contains ─> HashMap<String, ActivityFactory>
                    └─ calls ─> Activity trait implementations

WorkflowContext
    │
    ├─ contains ─> WorkflowExecution
    │               ├─ WorkflowId
    │               └─ RunId
    │
    ├─ contains ─> EventHistory
    │               └─ Vec<WorkflowEvent>
    │                   └─ contains ─> EventId
    │
    ├─ contains ─> SignalRegistry
    │               └─ handles ─> Signal trait implementations
    │
    └─ contains ─> QueryRegistry
                    └─ handles ─> Query trait implementations

ActivityContext
    │
    ├─ contains ─> ActivityId
    │
    ├─ contains ─> WorkflowExecution
    │
    └─ contains ─> CancellationToken
```

---

## 🎯 最佳实践

### 1. 使用Newtype模式

```rust
// ✅ 好: 使用Newtype提供类型安全
pub struct WorkflowId(String);
pub struct RunId(Uuid);

// ❌ 差: 直接使用原始类型
pub type WorkflowId = String;  // 容易混淆
pub type RunId = Uuid;          // 容易混淆
```

### 2. 使用关联类型

```rust
// ✅ 好: 使用关联类型
pub trait Workflow {
    type Input;
    type Output;
    // ...
}

// ❌ 差: 使用泛型参数
pub trait Workflow<Input, Output> {  // 使用时更繁琐
    // ...
}
```

### 3. 最小化生命周期参数

```rust
// ✅ 好: 使用Arc避免生命周期参数
pub struct WorkflowContext {
    history: Arc<RwLock<EventHistory>>,
    // ...
}

// ❌ 差: 过多的生命周期参数
pub struct WorkflowContext<'a, 'b, 'c> {
    history: &'a mut EventHistory,
    // ...
}
```

---

## 📚 下一步

- **应用类型**: [工作流定义](./04_workflow_definition.md)
- **实现示例**: [基础示例](./18_basic_examples.md)
- **了解错误处理**: [错误类型](./error_types.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
