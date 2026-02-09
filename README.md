# yinmo-bot

一个用于个人 assistant 的 Rust daemon 基础框架，当前目标是：

- 任务队列调度
- 最多 `5` 个并发 worker
- 所有外部请求统一走网关（`Gateway`）
- 主体只做调度，不在本体里做复杂推理

## 选型说明

- **运行时**: `tokio`
  - 原因：异步任务、子进程、信号处理成熟。
- **队列**: `async-channel`
  - 原因：轻量、易用，适合单机内存队列。
- **错误体系**: `thiserror`
  - 原因：显式定义分层错误类型，避免 `anyhow` 黑盒化。
- **序列化**: `serde / serde_json`
  - 原因：后续和 IM、ACP、持久化对接都需要结构化数据。
- **日志**: `tracing + tracing-subscriber`
  - 原因：后续扩展多 worker、子进程观察更方便。

## 当前框架

- `src/core/gateway.rs`
  - 唯一对外入口 `ExternalGateway`
  - 目前支持 `RunCommand` 请求
- `src/core/scheduler.rs`
  - 内存任务队列 + worker 池
- `src/core/daemon.rs`
  - 启动调度核心，强制 worker 范围 `1..=5`
- `src/executor/process.rs`
  - 子进程执行器（后续可替换为 Codex ACP 执行器）
- `src/bin/scheduler_smoke.rs`
  - 并发上限烟雾验证（观测最大并发不超过 5）

## 运行

```bash
cargo run
```

```bash
cargo run --bin scheduler_smoke
```
