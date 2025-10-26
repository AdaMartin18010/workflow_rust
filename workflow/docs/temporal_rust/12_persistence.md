# 持久化实现

## 📋 文档概述

本文档详细阐述Temporal的持久化层实现，包括：

- 持久化架构
- 事件存储
- Rust 1.90实现
- Golang实现对比
- 多种存储后端
- 性能优化

---

## 🎯 持久化核心概念

### 为什么需要持久化？

Temporal的核心能力依赖于持久化：

1. **持久性 (Durability)**: 工作流状态在进程崩溃后可恢复
2. **事件溯源 (Event Sourcing)**: 通过事件历史重建状态
3. **确定性重放 (Deterministic Replay)**: 支持代码更新
4. **可观测性**: 查询历史执行记录

```text
┌─────────────────────────────────────────────────────────────┐
│                    持久化架构                                │
└─────────────────────────────────────────────────────────────┘

Worker                  Temporal Service            Storage
  │                           │                        │
  ├─ Execute Workflow ───────▶│                        │
  │                           ├─ Save Event ──────────▶│
  │                           │                        │
  │  ◀──── Crash ─────       │                        │
  │                           │                        │
  ├─ Restart ────────────────▶│                        │
  │                           ├─ Load Events ◀────────┤
  │                           │                        │
  ├─ Replay Events ◀─────────┤                        │
  │                           │                        │
  └─ Continue Execution ─────▶│                        │
```

---

## 🦀 Rust实现

### 存储接口定义

```rust
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// 工作流存储trait
#[async_trait]
pub trait WorkflowStorage: Send + Sync {
    /// 保存工作流执行
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        metadata: &WorkflowMetadata,
    ) -> Result<(), StorageError>;
    
    /// 加载工作流执行
    async fn load_workflow_execution(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<Option<WorkflowExecutionRecord>, StorageError>;
    
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
        limit: Option<usize>,
    ) -> Result<Vec<WorkflowEvent>, StorageError>;
    
    /// 更新工作流状态
    async fn update_workflow_state(
        &self,
        workflow_id: &WorkflowId,
        state: WorkflowLifecycleState,
    ) -> Result<(), StorageError>;
    
    /// 保存Activity心跳
    async fn save_activity_heartbeat(
        &self,
        activity_id: &ActivityId,
        details: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<(), StorageError>;
    
    /// 查询工作流
    async fn query_workflows(
        &self,
        query: WorkflowQuery,
    ) -> Result<Vec<WorkflowExecutionRecord>, StorageError>;
}

/// 工作流元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub workflow_type: String,
    pub task_queue: String,
    pub started_at: DateTime<Utc>,
    pub timeout_config: TimeoutConfig,
    pub tags: HashMap<String, String>,
}

/// 工作流执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionRecord {
    pub execution: WorkflowExecution,
    pub metadata: WorkflowMetadata,
    pub state: WorkflowLifecycleState,
    pub event_history: Vec<WorkflowEvent>,
    pub updated_at: DateTime<Utc>,
}

/// 工作流查询
#[derive(Debug, Clone)]
pub struct WorkflowQuery {
    pub workflow_type: Option<String>,
    pub state: Option<WorkflowLifecycleState>,
    pub started_after: Option<DateTime<Utc>>,
    pub started_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
```

### PostgreSQL实现

```rust
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;

/// PostgreSQL存储实现
pub struct PostgresWorkflowStorage {
    pool: PgPool,
}

impl PostgresWorkflowStorage {
    /// 创建新实例
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        Ok(Self { pool })
    }
    
    /// 初始化数据库schema
    pub async fn init_schema(&self) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflow_executions (
                workflow_id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                workflow_type TEXT NOT NULL,
                task_queue TEXT NOT NULL,
                state TEXT NOT NULL,
                started_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                completed_at TIMESTAMPTZ,
                metadata JSONB NOT NULL,
                UNIQUE(workflow_id, run_id)
            );
            
            CREATE TABLE IF NOT EXISTS workflow_events (
                id BIGSERIAL PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                event_id BIGINT NOT NULL,
                event_type TEXT NOT NULL,
                event_data JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                UNIQUE(workflow_id, event_id)
            );
            
            CREATE INDEX IF NOT EXISTS idx_workflow_events_workflow_id 
                ON workflow_events(workflow_id, event_id);
            
            CREATE TABLE IF NOT EXISTS activity_heartbeats (
                activity_id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                details JSONB,
                timestamp TIMESTAMPTZ NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_activity_heartbeats_workflow_id 
                ON activity_heartbeats(workflow_id);
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        Ok(())
    }
}

#[async_trait]
impl WorkflowStorage for PostgresWorkflowStorage {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        metadata: &WorkflowMetadata,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO workflow_executions (
                workflow_id, run_id, workflow_type, task_queue, 
                state, started_at, updated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (workflow_id) 
            DO UPDATE SET 
                run_id = EXCLUDED.run_id,
                updated_at = EXCLUDED.updated_at,
                metadata = EXCLUDED.metadata
            "#
        )
        .bind(&execution.workflow_id.as_str())
        .bind(&execution.run_id.to_string())
        .bind(&metadata.workflow_type)
        .bind(&metadata.task_queue)
        .bind("Running")
        .bind(metadata.started_at)
        .bind(Utc::now())
        .bind(serde_json::to_value(metadata).unwrap())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn load_workflow_execution(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<Option<WorkflowExecutionRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT workflow_id, run_id, workflow_type, task_queue, 
                   state, started_at, updated_at, metadata
            FROM workflow_executions
            WHERE workflow_id = $1
            "#
        )
        .bind(workflow_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        if let Some(row) = row {
            let execution = WorkflowExecution::with_run_id(
                WorkflowId::new(row.get::<String, _>("workflow_id")),
                RunId::parse(&row.get::<String, _>("run_id")).unwrap(),
            );
            
            let metadata: WorkflowMetadata = serde_json::from_value(
                row.get::<serde_json::Value, _>("metadata")
            ).unwrap();
            
            // 加载事件历史
            let event_history = self.get_event_history(workflow_id, None, None).await?;
            
            Ok(Some(WorkflowExecutionRecord {
                execution,
                metadata,
                state: parse_state(row.get("state")),
                event_history,
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn append_event(
        &self,
        workflow_id: &WorkflowId,
        event: WorkflowEvent,
    ) -> Result<EventId, StorageError> {
        let row = sqlx::query(
            r#"
            INSERT INTO workflow_events (
                workflow_id, event_id, event_type, event_data, timestamp
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#
        )
        .bind(workflow_id.as_str())
        .bind(event.event_id.0 as i64)
        .bind(format!("{:?}", event.event_type))
        .bind(serde_json::to_value(&event.event_type).unwrap())
        .bind(event.timestamp)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        Ok(EventId(row.get::<i64, _>("id") as u64))
    }
    
    async fn get_event_history(
        &self,
        workflow_id: &WorkflowId,
        from_event_id: Option<EventId>,
        limit: Option<usize>,
    ) -> Result<Vec<WorkflowEvent>, StorageError> {
        let mut query = String::from(
            "SELECT event_id, event_type, event_data, timestamp FROM workflow_events WHERE workflow_id = $1"
        );
        
        if from_event_id.is_some() {
            query.push_str(" AND event_id >= $2");
        }
        
        query.push_str(" ORDER BY event_id ASC");
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        let mut sql_query = sqlx::query(&query).bind(workflow_id.as_str());
        
        if let Some(from_id) = from_event_id {
            sql_query = sql_query.bind(from_id.0 as i64);
        }
        
        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        let events = rows
            .into_iter()
            .map(|row| {
                WorkflowEvent {
                    event_id: EventId(row.get::<i64, _>("event_id") as u64),
                    timestamp: row.get("timestamp"),
                    event_type: serde_json::from_value(
                        row.get::<serde_json::Value, _>("event_data")
                    ).unwrap(),
                }
            })
            .collect();
        
        Ok(events)
    }
    
    async fn update_workflow_state(
        &self,
        workflow_id: &WorkflowId,
        state: WorkflowLifecycleState,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE workflow_executions 
            SET state = $1, updated_at = $2
            WHERE workflow_id = $3
            "#
        )
        .bind(format!("{:?}", state))
        .bind(Utc::now())
        .bind(workflow_id.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn save_activity_heartbeat(
        &self,
        activity_id: &ActivityId,
        details: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO activity_heartbeats (activity_id, workflow_id, details, timestamp)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (activity_id)
            DO UPDATE SET details = EXCLUDED.details, timestamp = EXCLUDED.timestamp
            "#
        )
        .bind(activity_id.as_str())
        .bind("unknown")  // 实际应该从activity_id提取
        .bind(details)
        .bind(timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn query_workflows(
        &self,
        query: WorkflowQuery,
    ) -> Result<Vec<WorkflowExecutionRecord>, StorageError> {
        // 构建动态查询
        let mut sql = String::from("SELECT * FROM workflow_executions WHERE 1=1");
        let mut bindings: Vec<String> = vec![];
        
        if let Some(workflow_type) = &query.workflow_type {
            sql.push_str(&format!(" AND workflow_type = ${}", bindings.len() + 1));
            bindings.push(workflow_type.clone());
        }
        
        if let Some(state) = &query.state {
            sql.push_str(&format!(" AND state = ${}", bindings.len() + 1));
            bindings.push(format!("{:?}", state));
        }
        
        sql.push_str(" ORDER BY started_at DESC");
        
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        
        // 执行查询并转换结果
        // (简化实现)
        Ok(vec![])
    }
}
```

### 内存存储实现（测试用）

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 内存存储实现（用于测试）
pub struct InMemoryWorkflowStorage {
    executions: Arc<RwLock<HashMap<WorkflowId, WorkflowExecutionRecord>>>,
    events: Arc<RwLock<HashMap<WorkflowId, Vec<WorkflowEvent>>>>,
}

impl InMemoryWorkflowStorage {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WorkflowStorage for InMemoryWorkflowStorage {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
        metadata: &WorkflowMetadata,
    ) -> Result<(), StorageError> {
        let record = WorkflowExecutionRecord {
            execution: execution.clone(),
            metadata: metadata.clone(),
            state: WorkflowLifecycleState::Running,
            event_history: vec![],
            updated_at: Utc::now(),
        };
        
        self.executions
            .write()
            .await
            .insert(execution.workflow_id.clone(), record);
        
        Ok(())
    }
    
    async fn load_workflow_execution(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<Option<WorkflowExecutionRecord>, StorageError> {
        let executions = self.executions.read().await;
        Ok(executions.get(workflow_id).cloned())
    }
    
    async fn append_event(
        &self,
        workflow_id: &WorkflowId,
        event: WorkflowEvent,
    ) -> Result<EventId, StorageError> {
        let mut events = self.events.write().await;
        let workflow_events = events.entry(workflow_id.clone()).or_insert_with(Vec::new);
        workflow_events.push(event.clone());
        Ok(event.event_id)
    }
    
    async fn get_event_history(
        &self,
        workflow_id: &WorkflowId,
        from_event_id: Option<EventId>,
        limit: Option<usize>,
    ) -> Result<Vec<WorkflowEvent>, StorageError> {
        let events = self.events.read().await;
        
        if let Some(workflow_events) = events.get(workflow_id) {
            let mut filtered: Vec<_> = workflow_events.iter()
                .filter(|e| {
                    if let Some(from_id) = from_event_id {
                        e.event_id >= from_id
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();
            
            if let Some(limit) = limit {
                filtered.truncate(limit);
            }
            
            Ok(filtered)
        } else {
            Ok(vec![])
        }
    }
    
    async fn update_workflow_state(
        &self,
        workflow_id: &WorkflowId,
        state: WorkflowLifecycleState,
    ) -> Result<(), StorageError> {
        let mut executions = self.executions.write().await;
        
        if let Some(record) = executions.get_mut(workflow_id) {
            record.state = state;
            record.updated_at = Utc::now();
        }
        
        Ok(())
    }
    
    // 其他方法实现...
    async fn save_activity_heartbeat(
        &self,
        _activity_id: &ActivityId,
        _details: serde_json::Value,
        _timestamp: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        Ok(())
    }
    
    async fn query_workflows(
        &self,
        _query: WorkflowQuery,
    ) -> Result<Vec<WorkflowExecutionRecord>, StorageError> {
        let executions = self.executions.read().await;
        Ok(executions.values().cloned().collect())
    }
}
```

---

## 🐹 Golang实现对比

### Temporal Go SDK持久化

Golang的Temporal SDK使用Temporal Service的持久化层，不需要直接实现存储接口。

```go
// Temporal Service配置（管理员操作）
import (
    "go.temporal.io/server/common/config"
)

// PostgreSQL配置
cfg := &config.Persistence{
    DefaultStore: "default",
    DataStores: map[string]config.DataStore{
        "default": {
            SQL: &config.SQL{
                PluginName:   "postgres",
                DatabaseName: "temporal",
                ConnectAddr:  "localhost:5432",
                User:         "temporal",
                Password:     "temporal",
            },
        },
    },
}
```

---

## 🎯 最佳实践

### 1. 连接池管理

```rust
// ✅ 好: 使用连接池
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

### 2. 事务管理

```rust
// 保存工作流执行和事件在一个事务中
async fn save_workflow_with_event(
    &self,
    execution: &WorkflowExecution,
    event: WorkflowEvent,
) -> Result<(), StorageError> {
    let mut tx = self.pool.begin().await?;
    
    // 保存执行
    sqlx::query("INSERT INTO workflow_executions ...")
        .execute(&mut tx)
        .await?;
    
    // 保存事件
    sqlx::query("INSERT INTO workflow_events ...")
        .execute(&mut tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

### 3. 索引优化

```sql
-- 工作流查询优化
CREATE INDEX idx_workflows_type_state 
    ON workflow_executions(workflow_type, state);

CREATE INDEX idx_workflows_started_at 
    ON workflow_executions(started_at DESC);

-- 事件查询优化
CREATE INDEX idx_events_workflow_event 
    ON workflow_events(workflow_id, event_id);
```

### 4. 数据归档

```rust
/// 归档完成的工作流
pub async fn archive_completed_workflows(
    &self,
    before: DateTime<Utc>,
) -> Result<usize, StorageError> {
    // 移动到归档表
    let result = sqlx::query(
        r#"
        INSERT INTO workflow_executions_archive
        SELECT * FROM workflow_executions
        WHERE state IN ('Completed', 'Failed', 'Cancelled')
          AND completed_at < $1
        "#
    )
    .bind(before)
    .execute(&self.pool)
    .await?;
    
    let archived = result.rows_affected();
    
    // 删除原始记录
    sqlx::query(
        r#"
        DELETE FROM workflow_executions
        WHERE state IN ('Completed', 'Failed', 'Cancelled')
          AND completed_at < $1
        "#
    )
    .bind(before)
    .execute(&self.pool)
    .await?;
    
    Ok(archived as usize)
}
```

---

## 📚 总结

### 持久化关键点

1. **事件溯源**: 通过事件历史重建状态
2. **性能优化**: 连接池、索引、查询优化
3. **数据一致性**: 使用事务保证
4. **扩展性**: 支持分库分表
5. **归档策略**: 定期归档历史数据

### Rust vs Golang

- **Rust**: 需要自己实现存储层（更灵活）
- **Golang**: 使用Temporal Service的持久化（开箱即用）

---

## 📚 下一步

- **部署策略**: [生产部署](./deployment.md)
- **监控告警**: [可观测性](./monitoring.md)
- **性能调优**: [性能优化](./performance.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
