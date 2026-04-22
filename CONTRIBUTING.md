# 贡献与复现指南（Contributing）

欢迎两类贡献：复现实验、扩展实验。

## 1. 快速复现

### 1.1 前置环境

- Windows + PowerShell
- Rust `stable-x86_64-pc-windows-gnu`
- MinGW-w64 gcc
- Git
- Claude Code

### 1.2 执行步骤

```powershell
git clone https://github.com/你的用户名/tinyexpr-c2rust-eval.git
cd tinyexpr-c2rust-eval
.\scripts\init.ps1
cd tinyexpr-rs
claude
```

启动后使用 `.claude/prompts/00-startup.md`，按 `docs/03-phase-by-phase.md` 推进。

### 1.3 结果提交

请在 PR 中包含：

- `tinyexpr-rs/` 翻译结果
- `logs/raw-journal.md` 原始记录
- `data/metrics.csv` 与 `data/session_stats.md`
- 复现总结（与本仓库结论的一致与差异）

## 2. 扩展方向

- 更换被测项目（规模、风格、依赖类型多样化）
- 更换 AI 工具并保持同一方法学
- 增强评测协议（fuzz、性能基准、盲评）

## 3. 质量要求

- Rust 代码需通过 `cargo fmt`、`cargo clippy`、`cargo test`。
- 文档以中文为主，表述准确、可复核、避免口语化夸张。
- 所有结论需给出可追溯证据（日志、指标或测试输出）。

## 4. 交流规范

- 以证据为中心，避免人身化表述。
- 讨论结论时需同时说明适用边界（参见 `LIMITATIONS.md`）。
