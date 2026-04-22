# Claude Code 在 C→Rust 翻译任务上的评测

本仓库记录将 [codeplea/tinyexpr](https://github.com/codeplea/tinyexpr)（约 500 LOC）翻译为 Rust 的全过程，并据此评估 Claude Code 在中小规模 C→Rust 迁移中的能力边界。

## 研究问题

- 在人类协作条件下，Claude Code 能否稳定产出可编译、可测试、语义可对照的 Rust 代码。
- 哪类子任务可由模型独立完成，哪类任务需要人工介入。
- 交互协议、测试机制与人工决策如何影响最终结果。

## 数据与产物

- `tinyexpr-c/`：C 基准实现（黄金标准）
- `tinyexpr-rs/`：Rust 翻译产物
- `data/metrics.csv`：函数级量化指标
- `data/test_cases.csv`：C/Rust 对照测试集
- `logs/raw-journal.md`：原始过程日志

## 复现实验

```powershell
git clone https://github.com/你的用户名/tinyexpr-c2rust-eval.git
cd tinyexpr-c2rust-eval
.\scripts\init.ps1
cd tinyexpr-rs
cargo test
..\scripts\compare.ps1
```

完整流程见 `CONTRIBUTING.md` 与 `docs/02-setup.md`。

## 文档导航

- `METHODOLOGY.md`：实验设计、流程与指标定义
- `FINDINGS.md`：可复核的关键发现
- `CONCLUSIONS.md`：能力画像与结论
- `LIMITATIONS.md`：外推边界与偏差来源
- `STRUCTURE.md`：仓库结构与信息流

## 结论适用范围

本评测为单样本、单实验者研究，结论主要适用于与 tinyexpr 结构相近的中小型 C 项目；不直接代表大规模、强宏依赖或系统级 C 代码迁移场景。

## 许可

本仓库文档与脚本使用 MIT 许可。`tinyexpr-c/` 遵循上游项目的 zlib 许可。详见 `LICENSE`。
