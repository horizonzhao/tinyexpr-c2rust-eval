# 仓库结构说明（Repository Structure）

本文提供目录功能说明与推荐阅读顺序。

## 1. 顶层文档

- `README.md`：项目概览与入口。
- `METHODOLOGY.md`：实验设计与指标口径。
- `FINDINGS.md`：结构化发现。
- `CONCLUSIONS.md`：综合结论。
- `LIMITATIONS.md`：适用边界。
- `CONTRIBUTING.md`：复现与扩展方式。

## 2. 核心目录

- `tinyexpr-c/`：C 基准实现（只读参考）。
- `tinyexpr-rs/`：Rust 翻译产物与测试。
- `docs/`：背景、过程与专题分析。
- `logs/`：原始过程日志（仅追加）。
- `data/`：指标、测试集与会话统计。
- `scripts/`：初始化、对照、汇总自动化脚本。
- `.claude/`：交互协议与提示模板。

## 3. 信息流

`tinyexpr-c/` → 翻译到 `tinyexpr-rs/` → 验证（单测 + 对照）→ 记录到 `logs/` 与 `data/` → 提炼 `FINDINGS.md` → 汇总到 `CONCLUSIONS.md`。

## 4. 推荐阅读路径

- 快速了解：`README.md` → `CONCLUSIONS.md` → `FINDINGS.md`
- 审核可信度：`METHODOLOGY.md` → `LIMITATIONS.md` → `logs/raw-journal.md`
- 复现实施：`CONTRIBUTING.md` → `docs/02-setup.md` → `scripts/init.ps1`
