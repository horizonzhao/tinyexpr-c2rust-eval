# tinyexpr C→Rust 迁移与 Coding Agent 评测

本仓库以 [codeplea/tinyexpr](https://github.com/codeplea/tinyexpr) 为样本，研究 AI Coding Agent 在 C→Rust 迁移中的能力、失败模式和可复现评测方法。

项目包含两层相互衔接的实验：

1. **迁移实验**：在人工协作约束下，将约 500 行 C 表达式解析器翻译为 Rust，并用单元测试与 C/Rust 对照测试验证语义等价性。
2. **Agent 评测**：把迁移过程中发现的典型问题提炼成 8 个独立修复任务，通过隔离工作区、权限策略、确定性验证、执行轨迹和统一评分评测 Coding Agent。

因此，这个仓库不只是一个 Rust 移植结果，也是一套从真实迁移案例中构造 Coding Agent benchmark 的完整样例。

## 核心问题

- Coding Agent 能否产出可编译、可测试、行为与 C 基准一致的 Rust 实现？
- 哪些迁移问题可以通过编译器和单元测试发现，哪些必须依靠跨语言对照测试？
- 所有权、函数指针、闭包上下文、浮点格式和错误语义等问题应如何验证？
- 如何把一次人工协作实验转化为可重复运行、可审计、可比较的 Agent 评测？

## 整体逻辑

```text
tinyexpr C 基准
      │
      ▼
人工协作完成 C→Rust 翻译
      │
      ├── cargo test：验证 Rust 内部行为
      ├── cargo clippy：检查静态质量与迁移风险
      └── compare.ps1：比较 C/Rust 外部行为
      │
      ▼
记录差异、人工介入和失败模式
      │
      ▼
从真实问题提炼 8 个最小修复任务
      │
      ▼
Coding Agent 在隔离工作区内检索、修改、测试
      │
      ├── Policy：限制命令与可写路径
      ├── Verifier：执行确定性验证
      ├── Trajectory：记录完整动作与观察
      └── Scoring：生成统一 100 分制结果
      │
      ▼
summary.json + report.md + leaderboard
```

两层实验共用同一条证据链：C 基准定义预期语义，Rust 实现暴露迁移难点，真实难点再成为 Agent 修复任务。这样可以避免凭空设计与 C→Rust 场景无关的玩具 benchmark。

## 仓库结构

```text
.
├── tinyexpr-c/                    # C 基准实现
├── tinyexpr-rs/                   # Rust 翻译产物、REPL 与单元测试
├── agent-harness/
│   ├── benchmarks/                # 8 个包含真实缺陷的 Rust 工作区模板
│   ├── replays/                   # 可重复执行的预定义 Agent 动作
│   ├── tasks/                     # YAML 任务、权限、预算和验证规则
│   ├── src/c2rust_agent/          # Runner、Adapter、Policy、Verifier、评分与报告
│   └── tests/                     # Harness 自身测试
├── data/                          # 指标、对照用例与环境信息
├── docs/                          # 实验过程与专题分析
├── logs/                          # 原始实验日志
├── scripts/                       # 初始化、对照、指标和批量评测脚本
├── METHODOLOGY.md                 # 实验设计和指标口径
├── FINDINGS.md                    # 可追溯的关键发现
├── CONCLUSIONS.md                 # 综合结论
└── LIMITATIONS.md                 # 偏差与适用边界
```

## 第一层：C→Rust 迁移实验

### 验证方式

Rust 实现接受三层验证：

1. `cargo test --all` 验证 lexer、parser、求值、内置函数和浮点输出等局部行为。
2. `cargo clippy --all-targets -- -D warnings` 将警告作为迁移质量门槛。
3. `scripts/compare.ps1` 把同一批表达式分别输入 C 和 Rust REPL，比较数值、错误位置、特殊浮点值和输出格式。

对照测试覆盖运算优先级、内置数学函数、嵌套表达式、非法输入、小数、科学计数法、NaN 和无穷值。浮点数使用相对误差比较，非数值输出要求规范化后一致。

### 主要发现

- C 的 tagged union 与柔性数组可以自然映射为 Rust `enum`、`Box` 和 `Vec`。
- 树形 `malloc/free` 可由 Rust 所有权和 `Drop` 消除。
- C 函数指针身份比较不能机械翻译，需要使用显式枚举表达运算符身份。
- Rust 默认浮点格式不等价于 C `%g`，仅靠计算结果正确不足以保证 CLI 行为一致。
- 错误位置、闭包上下文和优化器纯度属于“可以编译但语义可能错误”的高风险区域。

详细证据见 [FINDINGS.md](FINDINGS.md)，实验过程见 [docs/03-phase-by-phase.md](docs/03-phase-by-phase.md)。

## 第二层：Coding Agent 评测框架

### 一次运行如何执行

```text
读取任务 YAML
  → 创建或选择工作区
  → 加载 Agent Adapter
  → 向 Agent 提供任务、策略、预算与上一步观察
  → Policy 检查动作是否合法
  → 执行读文件、写文件或受控命令
  → 重复直到 finish、超时或超过步骤预算
  → Verifier 执行 fmt、clippy 和 test
  → Workspace Snapshot 计算实际文件变化
  → Scoring 计算得分和 resolved 状态
  → 写入轨迹、摘要和 Markdown 报告
```

Runner 不依赖 Agent 自报成功。只有必需验证通过、没有策略违规且 Agent 正常提交 `finish`，任务才会标记为 `resolved`。

### Agent 适配方式

| Adapter | 用途 |
|---|---|
| `replay` | 重放固定动作，用于框架回归测试和缺陷基线 |
| `command` | 通过 stdin/stdout JSON 协议接入任意外部 Agent |
| `deepseek` | 使用 DeepSeek Tool Calls API 执行真实代码检索、修改和验证 |

DeepSeek Adapter 暴露受控的 `list_files`、`read_file`、`search_code`、`write_file`、`run_command` 和 `finish` 工具。所有写入和命令仍需通过任务策略，而不是直接获得完整 shell 权限。

### 任务定义

每个 YAML 任务同时描述：

- 待修复问题和工作区模板；
- 允许修改与只读路径；
- 允许执行的命令前缀；
- 最大步骤数和执行时间；
- 格式、静态检查和行为测试及其权重；
- 缺陷分类与原始迁移发现。

任务运行时会复制 benchmark 模板到独立工作区。Agent 可以读取测试，但只能修改 `src/`；测试和 Cargo 配置保持只读，避免通过删除测试或降低检查标准获得高分。

### 修复任务集

| 任务 | 缺陷类型 | 迁移风险 |
|---|---|---|
| `repair-error-position` | 普通解析错误被统一报告为 `-1` | 错误语义 |
| `repair-format-g` | Rust 默认输出与 C `%g` 不一致 | 外部行为 |
| `repair-fn-pointer` | 直接比较函数指针身份 | 编译器与链接语义 |
| `repair-power-associativity` | 连续幂运算结合方向错误 | Parser 语义 |
| `repair-optimizer-purity` | 常量折叠错误消除非纯调用 | 优化正确性 |
| `repair-closure-context` | 调用闭包时丢失注册上下文 | FFI/闭包迁移 |
| `repair-factorial-domain` | 非整数和非法值被静默强转 | 数值定义域 |
| `repair-ast-print` | AST 打印仍是占位实现 | 功能完整性 |

### 统一评分

| 维度 | 分值 | 规则 |
|---|---:|---|
| 正确性 | 70 | 按验证项权重计算通过比例 |
| 策略合规 | 10 | 出现越权动作则该项为 0 |
| 正常完成 | 10 | Agent 明确提交 `finish` |
| 执行效率 | 5 | 根据步骤预算消耗分档 |
| 修改克制 | 5 | 根据违规情况和修改文件数量分档 |

分数用于比较结果质量，`resolved` 用于表达是否真正解决任务。高分但缺少正常结束、存在策略违规或必需验证失败的运行，不会被视为成功修复。

## 快速开始

### 环境要求

- Windows 与 PowerShell
- Python 3.10+
- Rust stable，包含 `rustfmt` 和 `clippy`
- MinGW-w64 GCC，用于构建 C 基准

仓库根目录的 `rust-toolchain.toml` 指定 `stable-x86_64-pc-windows-gnu`。

### 1. 初始化并验证 Rust 迁移

```powershell
git clone https://github.com/horizonzhao/tinyexpr-c2rust-eval.git
cd tinyexpr-c2rust-eval
.\scripts\init.ps1

cd tinyexpr-rs
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cd ..

.\scripts\compare.ps1
```

### 2. 安装 Harness

```powershell
python -m venv .venv
.\.venv\Scripts\python.exe -m pip install -e .\agent-harness pytest
```

### 3. 验证任务与运行基线

```powershell
.\.venv\Scripts\c2rust-agent.exe validate .\agent-harness\tasks\tinyexpr-baseline.yaml
.\.venv\Scripts\c2rust-agent.exe run .\agent-harness\tasks\tinyexpr-baseline.yaml --agent replay
```

修复任务的 Replay 只负责稳定暴露预置缺陷，因此预期得到 `failed`，而不是自动完成修复：

```powershell
.\.venv\Scripts\c2rust-agent.exe run .\agent-harness\tasks\repair-error-position.yaml --agent replay
```

### 4. 运行 DeepSeek Agent

API Key 只应设置在当前终端环境变量中，不要写入 YAML、命令参数或仓库文件：

```powershell
$env:DEEPSEEK_API_KEY = "your-api-key"

.\.venv\Scripts\c2rust-agent.exe run `
  .\agent-harness\tasks\repair-error-position.yaml `
  --agent deepseek `
  --model deepseek-v4-flash `
  --thinking enabled `
  --reasoning-effort high `
  --max-tokens 8192
```

批量执行全部修复任务：

```powershell
.\scripts\run_deepseek_benchmark.ps1
```

脚本默认跳过已有成功结果；使用 `-Force` 可强制重新运行。

### 5. 接入外部 Agent

```powershell
.\.venv\Scripts\c2rust-agent.exe run `
  .\agent-harness\tasks\repair-error-position.yaml `
  --agent command `
  --command "your-agent-command"
```

Command Adapter 每一步通过标准输入发送 JSON 上下文，外部进程通过标准输出返回一个动作：

```json
{"type":"run_command","command":"cargo test --all","cwd":"."}
{"type":"write_file","path":"src/lib.rs","content":"..."}
{"type":"finish","message":"Implementation and verification completed."}
```

## 运行产物

每次运行都会生成独立目录：

```text
agent-harness/runs/<task>-<agent>-<UTC timestamp>/
├── trajectory.jsonl              # 每一步请求、动作、观察和策略结果
├── summary.json                  # 状态、得分、耗时、文件变化和 Agent 指标
└── report.md                     # 便于人工审阅的报告
```

聚合多次运行：

```powershell
.\.venv\Scripts\c2rust-agent.exe scoreboard .\agent-harness\runs
```

该命令生成 `leaderboard.json` 和 `leaderboard.md`，汇总解决率、平均分、平均步骤和平均耗时。

## 当前评测结果

使用 `deepseek-v4-flash` 顺序执行 8 个修复任务，结果如下：

| 指标 | 结果 |
|---|---:|
| 成功解决 | 8/8（100%） |
| 平均得分 | 98/100 |
| 平均步骤 | 9 |
| 平均耗时 | 22.1 秒/任务 |
| API 请求 | 52 次 |
| 总 Token | 116,557 |
| 策略违规 | 0 |

这组结果证明了 Tool Calls、隔离工作区、权限控制、代码修改、验证、轨迹和评分链路已经连通。它**不等价于模型在一般 C→Rust 任务上有 100% 成功率**：当前任务以小型、单文件修复为主，每个任务仅有一次正式运行，测试对 Agent 可读，难度和评分权重仍需更多模型与重复实验校准。

## 如何理解结果

本仓库目前可以支持以下结论：

- 在 tinyexpr 这一小型算法项目上，AI 辅助 C→Rust 迁移可以达到可编译、可测试和高行为一致性。
- Coding Agent 可以在受限写入和命令权限下完成真实语义修复，并留下可审计轨迹。
- 编译成功不能替代行为验证；函数指针、浮点格式、错误位置和副作用优化尤其需要专门测试。

本仓库目前不能支持以下外推：

- 不能代表 10K+ LOC、多模块、强宏、并发、I/O 或系统调用密集的 C 项目。
- 不能根据单模型单次运行判断稳定成功率或模型排名。
- 尚未系统覆盖性能、fuzz、隐藏测试、跨平台差异和长期可维护性。

完整边界见 [LIMITATIONS.md](LIMITATIONS.md)。

## 文档导航

- [METHODOLOGY.md](METHODOLOGY.md)：实验设计、流程和指标定义
- [FINDINGS.md](FINDINGS.md)：关键问题、证据和可迁移启示
- [CONCLUSIONS.md](CONCLUSIONS.md)：原始迁移实验的综合结论
- [LIMITATIONS.md](LIMITATIONS.md)：样本、实验者、工具和方法学边界
- [STRUCTURE.md](STRUCTURE.md)：目录职责和信息流
- [CONTRIBUTING.md](CONTRIBUTING.md)：复现要求和扩展方式
- [docs/04-deep-dive](docs/04-deep-dive)：Parser、所有权、union 和浮点语义专题

## 许可

本仓库文档、Rust 代码、Harness 和脚本使用 MIT 许可。`tinyexpr-c/` 保留上游 tinyexpr 的 zlib 许可，详见 [LICENSE](LICENSE)。
