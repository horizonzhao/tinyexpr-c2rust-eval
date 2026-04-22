# 实验方法（Methodology）

本文定义研究目标、实验流程、验证机制和评分口径，以保证结论可解释且可复现。

## 1. 研究目标

本评测回答四个问题：

- Claude Code 在协作条件下能否完成小型 C→Rust 翻译并通过验证。
- 各类子任务（语法映射、所有权决策、错误处理重构）中的表现差异。
- 需要何种人工介入及其频率。
- 哪些失败模式可转化为未来工具改进方向。

## 2. 样本与边界

被测样本为 `tinyexpr`（约 500 LOC，依赖简单，具备可执行对照测试）。  
该样本适合模块级翻译评测，但不覆盖大型多模块项目、复杂宏系统、并发与系统调用场景。

## 3. 工具链

| 类别 | 选择 |
|---|---|
| AI 助手 | Claude Code |
| Rust | `stable-x86_64-pc-windows-gnu` |
| C 编译器 | MinGW-w64 gcc |
| 平台 | Windows |

版本号在实验执行时写入 `data/environment.txt`。

## 4. 执行流程

实验固定为五阶段：

1. 理解 C 实现并建立语义模型。
2. 搭建 Rust 类型与接口骨架。
3. 逐函数翻译并即时单测。
4. 运行 C/Rust 对照测试并修正差异。
5. 汇总数据、提炼发现并形成结论。

交互层面采用小步迭代：每轮仅处理一个函数或一组紧耦合定义，避免跨层并发修改。

## 5. 验证与指标

### 5.1 验证机制

- `cargo test`：验证 Rust 局部正确性。
- `scripts/compare.ps1`：验证 C/Rust 行为一致性。
- 浮点比较采用统一容差策略（记录于脚本与报告）。

### 5.2 量化指标（`data/metrics.csv`）

- 规模：`c_loc`、`rust_loc`
- 效率：`rounds`、`first_try_compiled`、`first_try_tested`
- 安全性：`used_unsafe`
- 协作成本：`human_intervention`
- 问题类型：`tags`

### 5.3 定性指标

在 `FINDINGS.md` 中记录每条发现的：

- 严重度（1-5）
- 模型识别方式（主动/提示后/未识别）
- 可迁移启示（对流程或工具的改进价值）

## 6. 数据治理

- 原始记录：`logs/raw-journal.md`（仅追加，不回写历史）。
- 会话统计：`data/session_stats.md`。
- 结构化结论：`FINDINGS.md`、`CONCLUSIONS.md`。

## 7. 偏差控制

本实验显式承认并报告实验者偏差、样本偏差、模型版本偏差与交互偏差。  
外推边界统一在 `LIMITATIONS.md` 中声明。
