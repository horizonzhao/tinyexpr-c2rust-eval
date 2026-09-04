# Claude Code 在 C→Rust 翻译任务上的评测

本仓库记录将 [codeplea/tinyexpr](https://github.com/codeplea/tinyexpr)（约 500 LoC）翻译为 Rust 的全过程，并据此评估 Claude Code 在中小规模 C→Rust 迁移中的能力边界。

## 研究问题

- 在人类协作条件下，Claude Code 能否稳定产出可编译、可测试、语义可对照的 Rust 代码。
- 哪类子任务可由模型独立完成，哪类任务需要人工介入。
- 交互协议、测试机制与人工决策如何影响最终结果。

## 数据与产物

- `tinyexpr-c/`：C 基准实现
- `tinyexpr-rs/`：Rust 翻译产物
- `data/metrics.csv`：函数级量化指标
- `data/test_cases.csv`：C/Rust 对照测试集
- `logs/raw-journal.md`：原始过程日志

## Coding Agent 评测框架

仓库新增了独立的 `agent-harness/` 模块，将原有单次实验扩展为可重复运行的 coding agent 评测框架，同时保持 C 基准、Rust 产物和历史实验记录不变。

### 已有能力

- 使用 YAML 定义评测任务，以及命令、路径、步骤和时间预算。
- 支持 Replay Agent 和外部命令 Agent 两种适配方式。
- 对每一步操作执行权限检查和确定性验证。
- 记录工作区快照，识别新增、删除和修改的文件。
- 生成 JSONL 完整执行轨迹、JSON 汇总和 Markdown 报告。
- 在 GitHub Actions 中自动测试框架并校验基线任务。

### 目录结构

```text
agent-harness/
├── pyproject.toml
├── replays/                       # 可复现的预定义 Agent 动作
├── tasks/                         # YAML 评测任务
├── tests/                         # 框架自身测试
└── src/c2rust_agent/
    ├── adapters/                  # Replay 与外部命令适配器
    ├── cli.py                     # 命令行入口
    ├── runner.py                  # Agent 执行循环
    ├── policies.py                # 路径和命令权限检查
    ├── workspace.py               # 工作区快照与变更检测
    ├── verifier.py                # 确定性验证器
    ├── trajectory.py              # JSONL 轨迹记录
    └── report.py                  # 评测报告生成
```

### 快速开始

在仓库根目录执行：

```powershell
python -m pip install -e .\agent-harness
c2rust-agent validate .\agent-harness\tasks\tinyexpr-baseline.yaml
c2rust-agent run .\agent-harness\tasks\tinyexpr-baseline.yaml --agent replay
```

默认结果写入 `agent-harness/runs/`。当前自带的 `tinyexpr-baseline` 是只读任务：它运行现有 Rust 测试，并将命令、结果、耗时、文件变化和策略检查记录为完整 Agent 轨迹。

如果本地环境不允许安装命令行入口，可以直接运行：

```powershell
$env:PYTHONPATH = ".\agent-harness\src"
python -m c2rust_agent.cli validate .\agent-harness\tasks\tinyexpr-baseline.yaml
python -m c2rust_agent.cli run .\agent-harness\tasks\tinyexpr-baseline.yaml --agent replay
```

### 外部 Agent 接入协议

使用 `--agent command` 可以接入任意外部 coding agent。Command Adapter 在每个步骤启动一次指定进程，通过标准输入发送一个 JSON 请求，其中包含：

- 当前任务及问题描述
- 允许修改和禁止修改的路径
- 上一步观察结果
- 当前步骤和剩余步骤预算

外部进程必须在标准输出中返回一个 JSON 动作。当前支持三类动作：

```json
{"type":"run_command","command":"cargo test --all","cwd":"tinyexpr-rs"}
{"type":"write_file","path":"tinyexpr-rs/src/example.rs","content":"..."}
{"type":"finish","message":"实现和验证已经完成。"}
```

其中 `cwd` 和 `path` 都是相对于仓库根目录的路径。任务策略决定允许访问的路径和命令前缀；被拒绝的动作会写入轨迹，并作为下一步观察结果返回给 Agent。

调用示例：

```powershell
c2rust-agent run .\agent-harness\tasks\example.yaml `
  --agent command `
  --command "你的-agent-命令"
```

### 运行产物

每次运行会产生一个独立目录：

```text
agent-harness/runs/<任务>-<Agent>-<UTC时间>/
├── trajectory.jsonl              # 每一步动作、观察和策略结果
├── summary.json                  # 机器可读的最终指标
└── report.md                     # 便于人工查看的评测报告
```

运行只有在 Agent 正常结束、验证命令全部通过且没有策略违规时，才会被标记为 `passed`。

## 复现实验

```powershell
git clone https://github.com/你的用户名/tinyexpr-c2rust-eval.git
cd tinyexpr-c2rust-eval
.\scripts\init.ps1
cd tinyexpr-rs
cargo test
..\scripts\compare.ps1
```

仓库根目录已提供 `rust-toolchain.toml`，固定工具链为 `stable-x86_64-pc-windows-gnu`；进入仓库后执行 `cargo` 会自动使用该工具链。

完整流程见 `CONTRIBUTING.md` 与 `docs/02-setup.md`。

## 文档导航

- `METHODOLOGY.md`：实验设计、流程与指标定义
- `FINDINGS.md`：可复核的关键发现
- `CONCLUSIONS.md`：性能结论
- `LIMITATIONS.md`：外推边界与偏差来源
- `STRUCTURE.md`：仓库结构与信息流

## 结论适用范围

本评测为单样本、单实验者研究，结论主要适用于与 tinyexpr 结构相近的中小型 C 项目；不直接代表大规模、强宏依赖或系统级 C 代码迁移场景。

## 许可

本仓库文档与脚本使用 MIT 许可。`tinyexpr-c/` 遵循上游项目的 zlib 许可。详见 `LICENSE`。
