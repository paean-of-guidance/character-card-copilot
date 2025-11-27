# Backend Refactor Plan

## 愿景与原则
- 后端按照领域（Domain）、应用（Application）、基础设施（Infrastructure）三层划分，避免巨石式模块
- 所有模块保持职责单一，禁止在同一文件处理模型、服务、Tauri 命令等多重逻辑
- 增量迁移，确保每一步都能编译、通过现有测试，并记录完成度
- 关键结构和命令接口在迁移后补充单元/集成测试，防止回归

## 当前主要痛点
1. `character_session.rs` 超过 1000 行，集合状态、业务、IO、事件发送等多重职责
2. `events.rs` 同时定义 payload 与 tauri `emit` 逻辑，无法在非 Tauri 场景复用
3. `command_system` 目录中文件交叉依赖，没有清晰的服务边界
4. ToolRegistry、CharacterStorage、AIChatService 等直接耦合在会话逻辑中，难以替换或模拟
5. 缺乏正式的迁移计划与进度追踪，导致多人协作困难
## 目标架构草图
```
src-tauri/src/backend
├── domain
│   ├── sessions/{session.rs, config.rs}
│   ├── commands/{command.rs, metadata.rs}
│   └── events/payloads.rs
├── application
│   ├── session_service.rs
│   ├── command_service.rs
│   ├── tool_service.rs
│   └── event_bus.rs
└── infrastructure
    ├── tauri/
    │   ├── session_commands.rs
    │   ├── command_commands.rs
    │   └── event_emitter.rs
    └── persistence/
        ├── session_repository.rs
        └── command_registry.rs
```
- `context_builder`、`ai_chat`、`ToolRegistry` 将通过应用层服务注入，避免交叉引用
- 命令系统保留 `builtin` 子目录，但每个命令独立文件并仅依赖 domain 层
## 阶段任务与进度
| 阶段 | 目标 | 负责人 | 状态 | 备注 |
| --- | --- | --- | --- | --- |
| 0 | 编写文档、盘点依赖、确认目录结构 |  | ☑ 已完成 | 本文档 + 架构图，输出依赖清单 |
| 1 | domain 层拆分：会话、命令、事件 payload |  | ⏳ 进行中 | 初步完成 backend/domain/sessions（config、session）迁移 |
| 2 | application 层服务化：SessionService、CommandService、ToolService、EventBus |  | ☐ 未开始 | 需要补测试，保留旧 API 调用 |
| 3 | infrastructure 层瘦身：Tauri 命令文件拆分、EventEmitter 适配重写 |  | ☐ 未开始 | 仅做参数解析 + 调用服务 |
| 4 | 命令系统模块化：注册机制、builtin 重构、配置化 |  | ☐ 未开始 | 计划引入 once_cell / inventory |
| 5 | 清理遗留：删除旧模块、统一日志与 tracing、补全文档 |  | ☐ 未开始 | 收尾，确保 `cargo clippy` 通过 |

## 近期 TODO（滚动维护）
- [ ] 输出 Session/Command 依赖图（使用 mermaid 或文字描述）
- [ ] 梳理 `character_session.rs` 中可直接拆分的函数列表
- [ ] 为事件 payload 引入单元测试，覆盖序列化字段
- [ ] 讨论命令注册机制（宏 vs manifest），确定可扩展方案
- [ ] 在 README 或 docs 中引用本计划，便于新同学了解现状

## 记录规范
- 每完成一个阶段或子任务，在表格/待办勾选并补充备注
- 如果计划调整（例如新增阶段），必须在此文档更新并写明原因
- 开发中遇到阻塞项，直接在 TODO 下新增条目并标记 `⚠`，确保及时跟进
