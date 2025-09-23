//! 持久化模块 / Persistence Module
//! 提供工作流状态与历史的持久化抽象与适配器接口

use async_trait::async_trait;

/// 工作流状态快照 / Workflow state snapshot
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshot {
    pub workflow_id: String,
    pub state: serde_json::Value,
    pub updated_at: i64,
}

/// 幂等键记录 / Idempotency record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdempotencyRecord {
    pub key: String,
    pub created_at: i64,
}

#[async_trait]
pub trait PersistenceAdapter: Send + Sync {
    async fn save_state(&self, snapshot: StateSnapshot) -> anyhow::Result<()>;
    async fn load_state(&self, workflow_id: &str) -> anyhow::Result<Option<StateSnapshot>>;
    async fn put_idempotency_key(&self, key: &str, ttl_seconds: u64) -> anyhow::Result<bool>;
}

/// 内存适配器（默认实现）/ In-memory adapter (default)
pub struct InMemoryAdapter {
    states: parking_lot::RwLock<std::collections::HashMap<String, StateSnapshot>>,
    keys: parking_lot::RwLock<std::collections::HashMap<String, i64>>,
}

impl Default for InMemoryAdapter {
    fn default() -> Self { Self::new() }
}

impl InMemoryAdapter {
    pub fn new() -> Self {
        Self {
            states: parking_lot::RwLock::new(Default::default()),
            keys: parking_lot::RwLock::new(Default::default()),
        }
    }
}

#[async_trait]
impl PersistenceAdapter for InMemoryAdapter {
    async fn save_state(&self, snapshot: StateSnapshot) -> anyhow::Result<()> {
        self.states.write().insert(snapshot.workflow_id.clone(), snapshot);
        Ok(())
    }

    async fn load_state(&self, workflow_id: &str) -> anyhow::Result<Option<StateSnapshot>> {
        Ok(self.states.read().get(workflow_id).cloned())
    }

    async fn put_idempotency_key(&self, key: &str, _ttl_seconds: u64) -> anyhow::Result<bool> {
        let now = chrono::Utc::now().timestamp();
        let mut keys = self.keys.write();
        if keys.contains_key(key) { return Ok(false); }
        keys.insert(key.to_string(), now);
        Ok(true)
    }
}

/// Redis 适配器（可选）/ Redis adapter (optional)
#[cfg(feature = "database")]
pub mod redis_adapter {
    use super::*;
    use redis::AsyncCommands;

    pub struct RedisAdapter {
        client: redis::Client,
        namespace: String,
    }

    impl RedisAdapter {
        pub fn new(url: &str, namespace: impl Into<String>) -> anyhow::Result<Self> {
            Ok(Self { client: redis::Client::open(url)?, namespace: namespace.into() })
        }

        fn key(&self, k: &str) -> String { format!("{}:{}", self.namespace, k) }
    }

    #[async_trait]
    impl PersistenceAdapter for RedisAdapter {
        async fn save_state(&self, snapshot: StateSnapshot) -> anyhow::Result<()> {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            let key = self.key(&format!("state:{}", snapshot.workflow_id));
            let val = serde_json::to_string(&snapshot)?;
            conn.set::<_, _, ()>(key, val).await?;
            Ok(())
        }

        async fn load_state(&self, workflow_id: &str) -> anyhow::Result<Option<StateSnapshot>> {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            let key = self.key(&format!("state:{}", workflow_id));
            let val: Option<String> = conn.get(key).await?;
            Ok(match val { Some(v) => Some(serde_json::from_str(&v)?), None => None })
        }

        async fn put_idempotency_key(&self, key: &str, ttl_seconds: u64) -> anyhow::Result<bool> {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            let key = self.key(&format!("idem:{}", key));
            let added: bool = redis::cmd("SET").arg(&key).arg("1").arg("NX").arg("EX").arg(ttl_seconds).query_async(&mut conn).await?;
            Ok(added)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_adapter_roundtrip() {
        let adapter = InMemoryAdapter::new();
        let snap = StateSnapshot { workflow_id: "wf1".into(), state: serde_json::json!({"s":"ok"}), updated_at: 0 };
        adapter.save_state(snap.clone()).await.unwrap();
        let got = adapter.load_state("wf1").await.unwrap().unwrap();
        assert_eq!(got.workflow_id, "wf1");
    }
}


