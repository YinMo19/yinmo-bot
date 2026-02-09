# Agent Working Notes

本文件定义本项目的核心工程约束。

## 1) 架构边界

- **所有与外部系统的交互必须经过 Gateway**。
- 不允许外部入口绕过 `ExternalGateway` 直接操作 `Scheduler` 或 `Executor`。
- `Daemon` 负责装配，不负责业务细节。

## 2) 并发约束

- 当前并发上限固定为 `5`。
- 配置值超出范围时，统一在 `Daemon` 层裁剪到 `1..=5`。

## 3) 错误处理约束

- 禁止使用 `anyhow` 作为对外错误模型。
- 每层模块定义显式错误类型（`thiserror`）：
  - `GatewayError`
  - `SchedulerError`
  - `ExecutorError`
  - 入口程序错误类型（如 `MainError` / `SmokeError`）
- 跨层传播时保留语义，不吞掉错误边界。

## 4) 近期演进方向

- 增加 IM Gateway（Telegram 优先）
- 增加 Codex ACP Executor
- 增加任务状态持久化（SQLite）
- 增加记忆层（profile/task/episodic）

