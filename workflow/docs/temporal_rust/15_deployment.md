# 部署指南

## 📋 文档概述

本文档详细阐述Temporal工作流系统的部署方案，包括：

- 单机部署
- Docker部署
- Kubernetes部署
- 配置管理
- 高可用架构
- 最佳实践

---

## 🎯 部署架构概览

```text
┌─────────────────────────────────────────────────────────────┐
│                   Temporal 部署架构                          │
└─────────────────────────────────────────────────────────────┘

                      Load Balancer
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
   ┌─────────┐       ┌─────────┐       ┌─────────┐
   │ Worker  │       │ Worker  │       │ Worker  │
   │ Node 1  │       │ Node 2  │       │ Node 3  │
   └─────────┘       └─────────┘       └─────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                           ▼
                  ┌─────────────────┐
                  │ Temporal Server │
                  │   (Frontend)    │
                  └─────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
   ┌─────────┐       ┌─────────┐       ┌─────────┐
   │ History │       │ Matching│       │ Frontend│
   │ Service │       │ Service │       │ Service │
   └─────────┘       └─────────┘       └─────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                           ▼
                  ┌─────────────────┐
                  │   PostgreSQL    │
                  │    Cluster      │
                  └─────────────────┘
```

---

## 🖥️ 单机部署

### 1. 系统要求

```yaml
最低要求:
  CPU: 2核
  内存: 4GB
  磁盘: 20GB SSD

推荐配置:
  CPU: 4核+
  内存: 8GB+
  磁盘: 50GB+ SSD
  
操作系统:
  - Ubuntu 20.04+
  - Debian 11+
  - CentOS 8+
  - macOS 11+
```

### 2. 安装依赖

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql \
    postgresql-contrib

# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 数据库设置

```bash
# 启动PostgreSQL
sudo systemctl start postgresql
sudo systemctl enable postgresql

# 创建数据库和用户
sudo -u postgres psql << EOF
CREATE DATABASE temporal;
CREATE USER temporal_user WITH PASSWORD 'temporal_password';
GRANT ALL PRIVILEGES ON DATABASE temporal TO temporal_user;
\q
EOF

# 初始化schema
psql -h localhost -U temporal_user -d temporal -f schema/temporal.sql
```

### 4. 编译和运行

```bash
# 克隆项目
git clone https://github.com/your-org/temporal-rust.git
cd temporal-rust/workflow

# 编译（Release模式）
cargo build --release

# 配置环境变量
cat > .env << EOF
DATABASE_URL=postgresql://temporal_user:temporal_password@localhost/temporal
RUST_LOG=info
TEMPORAL_FRONTEND_ADDRESS=0.0.0.0:7233
EOF

# 运行Worker
./target/release/workflow-worker --config config/worker.toml
```

### 5. 配置文件示例

```toml
# config/worker.toml
[worker]
task_queue = "default"
max_concurrent_workflow_tasks = 100
max_concurrent_activity_tasks = 100

[database]
url = "postgresql://temporal_user:temporal_password@localhost/temporal"
max_connections = 20
min_connections = 5

[logging]
level = "info"
format = "json"

[metrics]
enabled = true
prometheus_port = 9090
```

---

## 🐳 Docker部署

### 1. Dockerfile

```dockerfile
# Dockerfile
FROM rust:1.90 as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY workflow/Cargo.toml ./workflow/

# 创建虚拟源文件以缓存依赖
RUN mkdir -p workflow/src && \
    echo "fn main() {}" > workflow/src/main.rs && \
    cargo build --release && \
    rm -rf workflow/src

# 复制实际源代码
COPY workflow/src ./workflow/src

# 编译应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        libssl3 \
        libpq5 && \
    rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -m -u 1000 workflow && \
    mkdir -p /app && \
    chown -R workflow:workflow /app

WORKDIR /app

# 从builder复制编译好的二进制
COPY --from=builder /app/target/release/workflow-worker /app/

# 切换到非root用户
USER workflow

# 暴露端口
EXPOSE 7233 9090

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

# 启动命令
ENTRYPOINT ["/app/workflow-worker"]
CMD ["--config", "/app/config/worker.toml"]
```

### 2. Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  # PostgreSQL数据库
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: temporal
      POSTGRES_USER: temporal_user
      POSTGRES_PASSWORD: temporal_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./schema:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U temporal_user"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - temporal-network

  # Temporal Worker
  worker:
    build:
      context: .
      dockerfile: Dockerfile
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://temporal_user:temporal_password@postgres/temporal
      RUST_LOG: info
      TEMPORAL_FRONTEND_ADDRESS: 0.0.0.0:7233
    ports:
      - "7233:7233"
      - "9090:9090"
    volumes:
      - ./config:/app/config:ro
    networks:
      - temporal-network
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  # Prometheus监控
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - temporal-network

  # Grafana可视化
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - temporal-network

volumes:
  postgres_data:
  prometheus_data:
  grafana_data:

networks:
  temporal-network:
    driver: bridge
```

### 3. 启动和管理

```bash
# 构建镜像
docker-compose build

# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f worker

# 查看服务状态
docker-compose ps

# 停止服务
docker-compose down

# 重启Worker
docker-compose restart worker

# 扩展Worker实例
docker-compose up -d --scale worker=3
```

---

## ☸️ Kubernetes部署

### 1. 命名空间

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: temporal
  labels:
    name: temporal
```

### 2. ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: temporal-config
  namespace: temporal
data:
  worker.toml: |
    [worker]
    task_queue = "default"
    max_concurrent_workflow_tasks = 100
    max_concurrent_activity_tasks = 100

    [database]
    url = "postgresql://temporal_user:temporal_password@postgres-service:5432/temporal"
    max_connections = 20

    [logging]
    level = "info"
    format = "json"

    [metrics]
    enabled = true
    prometheus_port = 9090
```

### 3. Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: temporal-secrets
  namespace: temporal
type: Opaque
data:
  # base64编码的密码
  postgres-password: dGVtcG9yYWxfcGFzc3dvcmQ=  # temporal_password
```

### 4. PostgreSQL部署

```yaml
# postgres-deployment.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: temporal
spec:
  serviceName: postgres-service
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
          name: postgres
        env:
        - name: POSTGRES_DB
          value: temporal
        - name: POSTGRES_USER
          value: temporal_user
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: temporal-secrets
              key: postgres-password
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 2
            memory: 4Gi
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 20Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: temporal
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  clusterIP: None
```

### 5. Worker部署

```yaml
# worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: temporal-worker
  namespace: temporal
  labels:
    app: temporal-worker
spec:
  replicas: 3
  selector:
    matchLabels:
      app: temporal-worker
  template:
    metadata:
      labels:
        app: temporal-worker
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: worker
        image: your-registry/temporal-worker:latest
        ports:
        - containerPort: 7233
          name: grpc
        - containerPort: 9090
          name: metrics
        env:
        - name: DATABASE_URL
          value: postgresql://temporal_user:$(POSTGRES_PASSWORD)@postgres-service:5432/temporal
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: temporal-secrets
              key: postgres-password
        - name: RUST_LOG
          value: info
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        resources:
          requests:
            cpu: 1
            memory: 2Gi
          limits:
            cpu: 2
            memory: 4Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: temporal-config
---
apiVersion: v1
kind: Service
metadata:
  name: temporal-worker-service
  namespace: temporal
spec:
  selector:
    app: temporal-worker
  ports:
  - name: grpc
    port: 7233
    targetPort: 7233
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

### 6. Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: temporal-ingress
  namespace: temporal
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - temporal.example.com
    secretName: temporal-tls
  rules:
  - host: temporal.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: temporal-worker-service
            port:
              number: 7233
```

### 7. HorizontalPodAutoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: temporal-worker-hpa
  namespace: temporal
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: temporal-worker
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
      - type: Pods
        value: 2
        periodSeconds: 30
      selectPolicy: Max
```

### 8. 部署命令

```bash
# 创建命名空间
kubectl apply -f namespace.yaml

# 创建ConfigMap和Secret
kubectl apply -f configmap.yaml
kubectl apply -f secret.yaml

# 部署PostgreSQL
kubectl apply -f postgres-deployment.yaml

# 等待PostgreSQL就绪
kubectl wait --for=condition=ready pod -l app=postgres -n temporal --timeout=300s

# 部署Worker
kubectl apply -f worker-deployment.yaml

# 部署Ingress
kubectl apply -f ingress.yaml

# 部署HPA
kubectl apply -f hpa.yaml

# 查看部署状态
kubectl get all -n temporal

# 查看日志
kubectl logs -f -l app=temporal-worker -n temporal

# 进入Pod调试
kubectl exec -it temporal-worker-xxx -n temporal -- /bin/bash
```

---

## 🔧 配置管理

### 环境变量

```bash
# 开发环境
export ENVIRONMENT=development
export RUST_LOG=debug
export DATABASE_URL=postgresql://localhost/temporal_dev

# 测试环境
export ENVIRONMENT=testing
export RUST_LOG=info
export DATABASE_URL=postgresql://test-db/temporal_test

# 生产环境
export ENVIRONMENT=production
export RUST_LOG=warn
export DATABASE_URL=postgresql://prod-db/temporal_prod
export ENABLE_TLS=true
```

### 配置文件层次

```text
config/
├── default.toml       # 默认配置
├── development.toml   # 开发环境覆盖
├── testing.toml       # 测试环境覆盖
└── production.toml    # 生产环境覆盖
```

---

## 🚀 最佳实践

### 1. 高可用部署

```yaml
# 至少3个Worker实例
replicas: 3

# 跨多个可用区
affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
    - labelSelector:
        matchExpressions:
        - key: app
          operator: In
          values:
          - temporal-worker
      topologyKey: topology.kubernetes.io/zone
```

### 2. 资源限制

```yaml
resources:
  requests:
    cpu: 1
    memory: 2Gi
  limits:
    cpu: 2
    memory: 4Gi
```

### 3. 优雅关闭

```rust
// 注册信号处理
tokio::spawn(async move {
    tokio::signal::ctrl_c().await.ok();
    worker.shutdown().await;
});
```

### 4. 健康检查

```rust
// /health端点
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": Utc::now()
    }))
}
```

---

## 📚 总结

### 部署方案对比

| 方案 | 适用场景 | 优点 | 缺点 |
|------|----------|------|------|
| **单机** | 开发/测试 | 简单快速 | 不具备高可用 |
| **Docker** | 小规模生产 | 易于管理 | 扩展性有限 |
| **Kubernetes** | 大规模生产 | 高可用、自动扩展 | 复杂度高 |

---

## 📚 下一步

- **最佳实践**: [设计原则](./16_best_practices.md)
- **迁移指南**: [从其他系统迁移](./17_migration_guide.md)

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**维护者**: temporal-rust 文档团队
