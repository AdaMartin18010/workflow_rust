# 项目重构计划 - 完全对齐 Temporal

## 📋 重构目标

**核心目标**: 将项目完全重构为基于 Temporal 概念的 Rust 1.90 实现

**设计原则**:

1. 完全遵循 Temporal 的设计理念和 API 模式
2. 充分利用 Rust 1.90 的类型系统和零成本抽象
3. 提供与 Temporal Golang SDK 对等的功能
4. 保持与 Temporal 服务器的互操作性

---

## 🗂️ 新文档结构

```text
workflow/docs/
│
├─ temporal_rust/                          # 新的核心文档目录
│  ├─ 00_MASTER_INDEX.md                   # ✅ 已创建 - 主索引
│  ├─ 01_concept_mapping.md                # ✅ 已创建 - 概念映射
│  ├─ 02_architecture.md                   # ⏳ 待创建 - 架构设计
│  ├─ 03_type_system.md                    # ⏳ 待创建 - 类型系统
│  ├─ 04_workflow_definition.md            # ⏳ 待创建 - 工作流定义
│  ├─ 05_activity_definition.md            # ⏳ 待创建 - Activity 定义
│  ├─ 06_signals_and_queries.md            # ⏳ 待创建 - Signal/Query
│  ├─ 07_event_sourcing.md                 # ⏳ 待创建 - 事件溯源
│  ├─ 08_distributed_coordination.md       # ⏳ 待创建 - 分布式协调
│  ├─ 09_saga_pattern.md                   # ⏳ 待创建 - Saga 模式
│  ├─ 10_child_workflows.md                # ⏳ 待创建 - 子工作流
│  ├─ 11_worker_implementation.md          # ⏳ 待创建 - Worker 实现
│  ├─ 12_client_api.md                     # ⏳ 待创建 - 客户端 API
│  ├─ 13_observability.md                  # ⏳ 待创建 - 可观测性
│  ├─ 14_deployment.md                     # ⏳ 待创建 - 部署指南
│  ├─ 15_testing.md                        # ⏳ 待创建 - 测试策略
│  ├─ 16_best_practices.md                 # ⏳ 待创建 - 最佳实践
│  ├─ 17_migration_guide.md                # ⏳ 待创建 - 迁移指南
│  ├─ 18_basic_examples.md                 # ⏳ 待创建 - 基础示例
│  ├─ 19_practical_examples.md             # ⏳ 待创建 - 实战示例
│  ├─ 20_enterprise_scenarios.md           # ⏳ 待创建 - 企业场景
│  ├─ 21_tech_stack_comparison.md          # ⏳ 待创建 - 技术栈对比
│  ├─ 22_temporal_server_integration.md    # ⏳ 待创建 - 服务器集成
│  └─ 23_ecosystem_integration.md          # ⏳ 待创建 - 生态集成
│
├─ api_reference/                          # API 参考文档
│  ├─ workflow_context.md                  # ⏳ 待创建
│  ├─ activity_context.md                  # ⏳ 待创建
│  ├─ workflow_client.md                   # ⏳ 待创建
│  ├─ workflow_worker.md                   # ⏳ 待创建
│  ├─ signal_trait.md                      # ⏳ 待创建
│  └─ query_trait.md                       # ⏳ 待创建
│
├─ code_examples/                          # 代码示例目录
│  ├─ rust/                                # Rust 示例
│  │  ├─ 01_hello_world.rs                # ⏳ 待创建
│  │  ├─ 02_basic_workflow.rs             # ⏳ 待创建
│  │  ├─ 03_activity_workflow.rs          # ⏳ 待创建
│  │  ├─ 04_signal_example.rs             # ⏳ 待创建
│  │  ├─ 05_query_example.rs              # ⏳ 待创建
│  │  ├─ 06_saga_example.rs               # ⏳ 待创建
│  │  ├─ 07_child_workflow_example.rs     # ⏳ 待创建
│  │  └─ 08_enterprise_example.rs         # ⏳ 待创建
│  │
│  └─ golang/                              # Golang 对比示例
│     ├─ 01_hello_world.go                # ⏳ 待创建
│     ├─ 02_basic_workflow.go             # ⏳ 待创建
│     ├─ 03_activity_workflow.go          # ⏳ 待创建
│     ├─ 04_signal_example.go             # ⏳ 待创建
│     ├─ 05_query_example.go              # ⏳ 待创建
│     ├─ 06_saga_example.go               # ⏳ 待创建
│     ├─ 07_child_workflow_example.go     # ⏳ 待创建
│     └─ 08_enterprise_example.go         # ⏳ 待创建
│
├─ diagrams/                               # 架构图和流程图
│  ├─ architecture.svg                     # ⏳ 待创建
│  ├─ event_flow.svg                       # ⏳ 待创建
│  ├─ worker_lifecycle.svg                 # ⏳ 待创建
│  └─ deployment.svg                       # ⏳ 待创建
│
├─ TEMPORAL_BASED_DESIGN.md                # ✅ 已创建 - 基础设计文档
├─ README.md                               # 🔄 需更新 - 主文档入口
│
└─ deprecated/                             # 过时文档迁移目标
   ├─ old_design/                          # 旧设计文档
   │  ├─ workflow_fundamentals/            # 迁移：concepts.md等
   │  ├─ rust_design/                      # 迁移：rust_design*.md
   │  ├─ algorithms/                       # 迁移：workflow_algorithm*.md
   │  ├─ ai/                               # 迁移：workflow_ai*.md
   │  └─ iot/                              # 迁移：workflow_iot*.md
   │
   ├─ old_comparisons/                     # 旧对比文档
   │  ├─ TEMPORAL_FRAMEWORK_COMPARISON.md  # 迁移
   │  ├─ TEMPORAL_ALIGNMENT_ROADMAP.md     # 迁移
   │  ├─ TEMPORAL_QUICK_REFERENCE.md       # 迁移
   │  ├─ TEMPORAL_INTEGRATION_SUMMARY.md   # 迁移
   │  └─ TEMPORAL_DOCS_INDEX.md            # 迁移
   │
   ├─ rust189/                             # Rust 1.89 文档
   │  ├─ const_generics.md                 # 迁移
   │  ├─ language_features.md              # 迁移
   │  ├─ standard_library.md               # 迁移
   │  └─ x86_features.md                   # 迁移
   │
   └─ legacy_standards/                    # 旧标准文档
      └─ international_standards/          # 迁移
```

---

## 📦 新代码结构

```text
workflow/src/
│
├─ lib.rs                                  # 🔄 需重构 - 主库文件
│
├─ temporal/                               # 新的核心模块
│  ├─ mod.rs                               # 模块根
│  │
│  ├─ workflow/                            # 工作流相关
│  │  ├─ mod.rs
│  │  ├─ context.rs                        # WorkflowContext
│  │  ├─ definition.rs                     # 工作流定义
│  │  ├─ execution.rs                      # WorkflowExecution
│  │  ├─ options.rs                        # StartWorkflowOptions
│  │  └─ macros.rs                         # #[workflow] 宏
│  │
│  ├─ activity/                            # Activity 相关
│  │  ├─ mod.rs
│  │  ├─ context.rs                        # ActivityContext
│  │  ├─ definition.rs                     # Activity 定义
│  │  ├─ options.rs                        # ActivityOptions
│  │  ├─ retry.rs                          # RetryPolicy
│  │  └─ macros.rs                         # #[activity] 宏
│  │
│  ├─ signal/                              # Signal 相关
│  │  ├─ mod.rs
│  │  ├─ trait_def.rs                      # Signal trait
│  │  ├─ registry.rs                       # SignalRegistry
│  │  └─ handler.rs                        # SignalHandler
│  │
│  ├─ query/                               # Query 相关
│  │  ├─ mod.rs
│  │  ├─ trait_def.rs                      # Query trait
│  │  ├─ registry.rs                       # QueryRegistry
│  │  └─ handler.rs                        # QueryHandler
│  │
│  ├─ worker/                              # Worker 相关
│  │  ├─ mod.rs
│  │  ├─ workflow_worker.rs                # WorkflowWorker
│  │  ├─ activity_worker.rs                # ActivityWorker
│  │  ├─ config.rs                         # WorkerConfig
│  │  └─ task_poller.rs                    # 任务轮询器
│  │
│  ├─ client/                              # 客户端相关
│  │  ├─ mod.rs
│  │  ├─ workflow_client.rs                # WorkflowClient
│  │  ├─ workflow_handle.rs                # WorkflowHandle
│  │  └─ config.rs                         # ClientConfig
│  │
│  ├─ event/                               # 事件相关
│  │  ├─ mod.rs
│  │  ├─ event_types.rs                    # WorkflowEvent 枚举
│  │  ├─ event_history.rs                  # EventHistory
│  │  └─ replay.rs                         # 事件重放
│  │
│  ├─ storage/                             # 存储相关
│  │  ├─ mod.rs
│  │  ├─ trait_def.rs                      # WorkflowStorage trait
│  │  ├─ postgres.rs                       # PostgreSQL 实现
│  │  ├─ mysql.rs                          # MySQL 实现
│  │  └─ memory.rs                         # 内存实现(测试用)
│  │
│  ├─ types/                               # 核心类型
│  │  ├─ mod.rs
│  │  ├─ workflow_id.rs                    # WorkflowId
│  │  ├─ run_id.rs                         # RunId
│  │  ├─ activity_id.rs                    # ActivityId
│  │  ├─ timer_id.rs                       # TimerId
│  │  └─ event_id.rs                       # EventId
│  │
│  ├─ error/                               # 错误类型
│  │  ├─ mod.rs
│  │  ├─ workflow_error.rs                 # WorkflowError
│  │  ├─ activity_error.rs                 # ActivityError
│  │  ├─ signal_error.rs                   # SignalError
│  │  └─ query_error.rs                    # QueryError
│  │
│  └─ patterns/                            # 高级模式
│     ├─ mod.rs
│     ├─ saga.rs                           # Saga 模式
│     ├─ child_workflow.rs                 # 子工作流
│     └─ timer.rs                          # 定时器
│
├─ grpc/                                   # gRPC 通信 (与 Temporal Server)
│  ├─ mod.rs
│  ├─ client.rs                            # gRPC 客户端
│  ├─ proto/                               # Protocol Buffers 定义
│  └─ codec.rs                             # 编解码器
│
├─ observability/                          # 可观测性
│  ├─ mod.rs
│  ├─ metrics.rs                           # 指标收集
│  ├─ tracing.rs                           # 分布式追踪
│  └─ logging.rs                           # 日志记录
│
├─ prelude.rs                              # 便捷导入
│
└─ legacy/                                 # 旧代码（逐步迁移）
   ├─ engine.rs                            # 旧 WorkflowEngine
   ├─ state.rs                             # 旧状态管理
   ├─ rust189/                             # Rust 1.89 代码
   └─ rust190/                             # 旧 Rust 1.90 代码
```

---

## 🔄 迁移步骤

### 阶段1: 文档重组 (Week 1-2)

#### 步骤 1.1: 创建新文档结构 ✅ 已完成

- [x] 创建 `temporal_rust/` 目录
- [x] 创建主索引文档
- [x] 创建概念映射文档

#### 步骤 1.2: 创建核心文档 (Week 1)

- [ ] 02_architecture.md - 架构设计
- [ ] 03_type_system.md - 类型系统
- [ ] 04_workflow_definition.md - 工作流定义
- [ ] 05_activity_definition.md - Activity 定义
- [ ] 06_signals_and_queries.md - Signal/Query

#### 步骤 1.3: 创建高级文档 (Week 2)

- [ ] 07_event_sourcing.md - 事件溯源
- [ ] 08_distributed_coordination.md - 分布式协调
- [ ] 09_saga_pattern.md - Saga 模式
- [ ] 10_child_workflows.md - 子工作流

#### 步骤 1.4: 创建运行时文档 (Week 2)

- [ ] 11_worker_implementation.md - Worker 实现
- [ ] 12_client_api.md - 客户端 API
- [ ] 13_observability.md - 可观测性
- [ ] 14_deployment.md - 部署指南

#### 步骤 1.5: 创建示例文档 (Week 2)

- [ ] 18_basic_examples.md - 基础示例
- [ ] 19_practical_examples.md - 实战示例
- [ ] 20_enterprise_scenarios.md - 企业场景

#### 步骤 1.6: 迁移旧文档到 deprecated/ (Week 2)

```bash
# 创建 deprecated 目录结构
mkdir -p workflow/docs/deprecated/{old_design,old_comparisons,rust189,legacy_standards}

# 迁移旧对比文档
mv workflow/docs/TEMPORAL_FRAMEWORK_COMPARISON.md workflow/docs/deprecated/old_comparisons/
mv workflow/docs/TEMPORAL_ALIGNMENT_ROADMAP.md workflow/docs/deprecated/old_comparisons/
mv workflow/docs/TEMPORAL_QUICK_REFERENCE.md workflow/docs/deprecated/old_comparisons/
mv workflow/docs/TEMPORAL_INTEGRATION_SUMMARY.md workflow/docs/deprecated/old_comparisons/
mv workflow/docs/TEMPORAL_DOCS_INDEX.md workflow/docs/deprecated/old_comparisons/

# 迁移旧设计文档
mv workflow/docs/workflow_fundamentals/ workflow/docs/deprecated/old_design/
mv workflow/docs/rust_design/ workflow/docs/deprecated/old_design/
mv workflow/docs/algorithms/ workflow/docs/deprecated/old_design/
mv workflow/docs/ai/ workflow/docs/deprecated/old_design/
mv workflow/docs/iot/ workflow/docs/deprecated/old_design/

# 迁移 Rust 1.89 文档
mv workflow/docs/rust189/ workflow/docs/deprecated/

# 迁移标准文档
mv workflow/docs/international_standards/ workflow/docs/deprecated/legacy_standards/
```

---

### 阶段2: 代码重构 (Week 3-8)

#### 步骤 2.1: 创建新模块结构 (Week 3)

- [ ] 创建 `src/temporal/` 目录结构
- [ ] 实现核心类型 (`types/` 模块)
- [ ] 实现错误类型 (`error/` 模块)
- [ ] 实现 `prelude.rs`

#### 步骤 2.2: 实现工作流系统 (Week 4-5)

- [ ] WorkflowContext 实现
- [ ] WorkflowExecution 实现
- [ ] #[workflow] 宏实现
- [ ] 工作流选项和配置

#### 步骤 2.3: 实现 Activity 系统 (Week 5-6)

- [ ] ActivityContext 实现
- [ ] #[activity] 宏实现
- [ ] RetryPolicy 实现
- [ ] 心跳机制

#### 步骤 2.4: 实现 Signal/Query (Week 6)

- [ ] Signal trait 和 SignalRegistry
- [ ] Query trait 和 QueryRegistry
- [ ] Handler 机制

#### 步骤 2.5: 实现 Worker (Week 7)

- [ ] WorkflowWorker 实现
- [ ] ActivityWorker 实现
- [ ] 任务轮询机制
- [ ] 并发控制

#### 步骤 2.6: 实现 Client (Week 7)

- [ ] WorkflowClient 实现
- [ ] WorkflowHandle 实现
- [ ] 工作流操作 API

#### 步骤 2.7: 实现事件系统 (Week 8)

- [ ] WorkflowEvent 枚举
- [ ] EventHistory 实现
- [ ] 事件重放机制

#### 步骤 2.8: 实现存储层 (Week 8)

- [ ] WorkflowStorage trait
- [ ] PostgreSQL 实现
- [ ] MySQL 实现
- [ ] 内存实现(测试用)

---

### 阶段3: 示例和测试 (Week 9-10)

#### 步骤 3.1: Rust 示例 (Week 9)

- [ ] Hello World
- [ ] 基础工作流
- [ ] Activity 工作流
- [ ] Signal/Query 示例
- [ ] Saga 示例
- [ ] 子工作流示例
- [ ] 企业场景示例

#### 步骤 3.2: Golang 对比示例 (Week 9)

- [ ] 所有示例的 Golang 版本
- [ ] 并列对比文档

#### 步骤 3.3: 测试 (Week 10)

- [ ] 单元测试
- [ ] 集成测试
- [ ] 端到端测试
- [ ] 性能基准测试

---

### 阶段4: 集成和部署 (Week 11-12)

#### 步骤 4.1: Temporal Server 集成 (Week 11)

- [ ] gRPC 通信实现
- [ ] Protocol Buffers 定义
- [ ] 协议兼容性测试

#### 步骤 4.2: 可观测性 (Week 11)

- [ ] 指标收集 (metrics)
- [ ] 分布式追踪 (tracing)
- [ ] 日志记录

#### 步骤 4.3: 部署文档 (Week 12)

- [ ] 单机部署指南
- [ ] 集群部署指南
- [ ] Kubernetes 部署
- [ ] Docker 支持

#### 步骤 4.4: 最终整合 (Week 12)

- [ ] 更新主 README
- [ ] 完善 API 文档
- [ ] 创建快速入门指南
- [ ] 发布 v1.0.0

---

## 📊 文档迁移清单

### 需要迁移到 deprecated/ 的文档

#### 旧对比文档 (old_comparisons/)

- [x] TEMPORAL_FRAMEWORK_COMPARISON.md (已标记)
- [x] TEMPORAL_ALIGNMENT_ROADMAP.md (已标记)
- [x] TEMPORAL_QUICK_REFERENCE.md (已标记)
- [x] TEMPORAL_INTEGRATION_SUMMARY.md (已标记)
- [x] TEMPORAL_DOCS_INDEX.md (已标记)

#### 旧设计文档 (old_design/)

- [ ] workflow_fundamentals/ (所有文件)
  - [ ] concepts.md
  - [ ] patterns.md
  - [ ] state_machines.md
- [ ] rust_design/ (所有文件 - 13个文件)
- [ ] algorithms/ (所有文件 - 17个文件)
- [ ] ai/ (所有文件 - 5个文件)
- [ ] iot/ (所有目录)

#### Rust 1.89 文档 (rust189/)

- [ ] const_generics.md
- [ ] language_features.md
- [ ] standard_library.md
- [ ] x86_features.md

#### 旧标准文档 (legacy_standards/)

- [ ] international_standards/ (所有文件)

### 需要保留和更新的文档

#### 核心文档

- [x] README.md - 需要完全重写
- [x] ARCHITECTURE.md - 需要基于 Temporal 重写
- [ ] API.md - 需要基于新 API 重写
- [ ] PERFORMANCE.md - 需要更新基准测试

#### 项目管理文档

- [ ] RUST_189_ENHANCEMENT_PLAN.md - 改为 TEMPORAL_RUST_ROADMAP.md
- [ ] OVERVIEW.md - 移除或整合到新文档

---

## 🎯 成功标准

### 文档质量

- [ ] 所有 23 个核心文档完成
- [ ] 所有 API 参考文档完成
- [ ] 至少 8 个 Rust 示例
- [ ] 至少 8 个 Golang 对比示例
- [ ] 旧文档全部迁移到 deprecated/

### 代码质量

- [ ] 新模块结构完全实现
- [ ] 所有核心功能测试覆盖率 > 80%
- [ ] 性能基准测试建立
- [ ] 与 Temporal Server 互操作测试通过

### 用户体验

- [ ] 快速入门指南（< 5 分钟上手）
- [ ] 完整的 API 文档
- [ ] 至少 10 个实战示例
- [ ] 迁移指南（从 Golang SDK）

---

## 🚀 执行计划

### 本周任务 (Week 1)

1. ✅ 创建主索引文档
2. ✅ 创建概念映射文档
3. ⏳ 创建架构设计文档
4. ⏳ 创建类型系统文档
5. ⏳ 创建工作流定义文档
6. ⏳ 开始迁移旧文档

### 下周任务 (Week 2)

1. 完成所有核心文档
2. 完成旧文档迁移
3. 开始代码重构

---

## 📞 反馈渠道

- **GitHub Issues**: 报告问题和建议
- **讨论区**: 技术讨论和问答
- **Pull Requests**: 贡献代码和文档

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-26  
**负责人**: temporal-rust 核心团队
