# 原始日志（Raw Journal）

按时间顺序追加实验现场记录，不回写历史条目。


## 正文

### 1
日期：2026-04-22
- 会话目标：阶段 1 启动，通读 C 实现，建立语义模型
- 关键决策待定：te_expr Rust 表示策略（方案 A infix enum / 方案 B 镜像 C 布局）
- 已完成：C 代码完整通读，五个阶段 1 任务已列出
  
### 2
日期：2026-04-23
#### 2.1
- 发现：默认工具链是 msvc，需要显式用 cargo +stable-x86_64-pc-windows-gnu；考虑在项目根加 rust-toolchain.toml 固定
- 完成：核心数据结构 Expr、NativeFn、NativeClosure、TeVariable 已实现并通过 4 个 smoke test
- te_eval 骨架已实现（支持 Constant/Variable/Function/Closure），parser 是 stub
#### 2.2
- 完成：内置函数表 24 个条目 + binary search find_builtin + linear find_lookup，10 tests 全绿
- 观察：log 映射 log10（非自然对数），与 C 默认行为一致（未定义 TE_NAT_LOG）
- 注意：unsigned long 在 Windows 是 32 位，Rust 统一用 u64，fac/ncr 溢出边界略不同，但不影响正确性
#### 2.3
- 关键决策：Token::Infix 放弃函数指针身份比较，改用 InfixOp enum。原因：Rust 警告 fn ptr 地址在 LTO 场景下不保证唯一，是 C→Rust翻译的语义差异点。
- 完成：lexer 层全部就绪 — next_token、State、Token、InfixOp，22 tests 通过
#### 2.4
- 完成：完整 parser（base/power/factor/term/expr/list）+ constant folding（optimize）+ te_compile 脱 stub，36 tests 通过
- 观察：optimize 用 **n = Expr::Constant(val) 原地替换 Box 内容，NLL borrow checker 正确分离了 mut-borrow（改 args）和后续te_eval + 写回的阶段
- 待验证：-2^2 = 4.0（C 默认行为，unary minus 在 base 之前），compare.ps1 可能是关键对齐点
#### 2.5
- 阶段完成：compare.ps1 等效测试 62/62 全通过，C/Rust 输出完全一致
- 语义差异点已解决：%g 格式化（6位有效数字，大数科学计数法）、错误位置报告（s.error_pos() 而非 -1）
- 工具问题：compare.ps1 在 bash 调 PowerShell 5.1 时有 UTF-8 编码问题，改用 bash 等效脚本绕过；用户直接从 PowerShell 终端运行compare.ps1 应无此问题
- 阻塞点：test_cases.csv 共 72 行，去掉头/注释后 62 个用例全通过（注意 CSV 共 73 行含 header，实际用例 62 个）