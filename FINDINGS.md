# 精选发现（Findings）

本文从 `logs/raw-journal.md` 提炼高价值观察，统一用于结论归纳与后续复现实验对比。

## 标签集

`#ownership` `#lifetime` `#union` `#fnptr` `#error` `#test-diff` `#idiom` `#float` `#recursion` `#prompt`

---

### F-001 C 函数指针身份比较在 Rust 中不可靠

- `tags`：#fnptr #idiom
- `severity`：4
- `model_behavior`：proactive
- `rounds`：2
- `evidence`：`src/lib.rs` — `Token::Infix`；编译器警告 `unpredictable_function_pointer_comparisons`

**Observation**  
C parser 用 `s->function == add` 识别运算符（函数指针相等比较）。首轮翻译将 `Token::Infix` 设计为 `fn(f64,f64)->f64`，Rust 编译器产生警告：LTO 可能合并或分离函数地址，比较结果不可靠。

**Analysis**  
C 依赖链接器/编译单元内地址唯一性，是隐含假设。Rust 明确不保证此属性。模型在看到 warning 后主动重设计，引入 `InfixOp` enum 替换函数指针身份比较，同时用 `as_native_fn()` 方法将 enum 转回可调用函数。

**Implication**  
C→Rust 翻译中所有"函数指针作为 token 标识"的模式均需替换为 enum variant。这是可预测的系统性改动，可在代码生成阶段前置检测。

---

### F-002 C union+flexible array → Rust enum：零 unsafe

- `tags`：#union #ownership
- `severity`：3
- `model_behavior`：proactive
- `rounds`：1
- `evidence`：`tinyexpr.h` `te_expr` struct；`src/lib.rs` `Expr` enum

**Observation**  
C 的 `te_expr` 用 `int type` + `union {double; const double*; const void*}` + `void *parameters[1]`（柔性数组）表示多态 AST 节点。Rust 翻译为 `enum Expr { Constant(f64), Variable(*const f64), Function{...}, Closure{...} }`，数据直接放 variant。

**Analysis**  
Rust enum 在类型层面消除了 C 的标签+union 组合，无需 unsafe（除 Variable 指针解引用外）。`Vec<Box<Expr>>` 替换柔性数组，分配语义更清晰。

**Implication**  
C 的 tagged union 是 Rust enum 的直接翻译目标，且更安全。`void *parameters[1]` 模式也应统一用 `Vec` 替代。

---

### F-003 `%g` 格式化行为与 Rust 默认浮点输出存在系统性差异

- `tags`：#float #test-diff
- `severity`：4
- `model_behavior`：proactive
- `rounds`：1
- `evidence`：`src/bin/repl.rs` `format_g`；对比：`1000000` → C `1e+06` vs Rust `1000000`

**Observation**  
C `printf("%g")` 使用 6 位有效数字，指数 ≥6 或 <−4 时切换科学计数法，指数带符号且至少两位（`1e+06`、`1e-05`）。Rust `format!("{}", x)` 输出最短可还原的十进制表示，不遵循 `%g` 规则，导致大数输出差异。

**Analysis**  
若直接用 `{}` 输出，`compare.ps1` 中所有涉及大数/小数的用例将 FAIL。模型主动识别差异并实现 `format_g`，通过先格式化为科学记数法提取指数再按规则输出。

**Implication**  
任何 C→Rust 翻译中涉及浮点输出对比的验证，必须显式处理 `%g`/`%f`/`%e` 与 Rust formatter 的差异。建议在翻译框架中将浮点输出格式列为必检项。

---

### F-004 `new_expr` + `te_free` 系被完整消除

- `tags`：#ownership
- `severity`：2
- `model_behavior`：proactive
- `rounds`：1
- `evidence`：C `new_expr`(15 LOC) + `te_free_parameters`(12) + `te_free`(5) → Rust 0 LOC

**Observation**  
C 手动管理 AST 节点内存（`malloc` + `free`），包含 fallthrough switch 递归释放子节点。Rust 的 `Box<Expr>` + `Vec<Box<Expr>>` + `Drop` 自动处理所有释放逻辑。

**Analysis**  
所有权语义直接对应 C 的树形 free。模型正确识别 `te_free` 等价于 Drop，无需翻译。

**Implication**  
C 的树形内存释放函数（递归 free）是 Rust Drop 的直接映射，可作为翻译时的正向信号：遇到 `te_free` 类函数无需翻译，直接标注删除。

---

### F-005 `optimize` 原地替换：NLL borrow checker 自动处理

- `tags`：#ownership
- `severity`：3
- `model_behavior`：proactive
- `rounds`：1
- `evidence`：`src/lib.rs` `optimize`；C `n->type = TE_CONSTANT; n->value = val`

**Observation**  
C `optimize` 原地将纯函数节点替换为常量（直接修改 `type` 字段）。Rust 需要先 mut borrow `args` 做递归优化，再不可变 borrow 调用 `te_eval`，最后再 mut 写回 `**n = Expr::Constant(val)`。

**Analysis**  
NLL（Non-Lexical Lifetimes）正确分离了三个阶段的借用周期，编译通过，无需额外重构。`**n = Expr::Constant(val)` 原地替换 Box 内容是关键写法。

**Implication**  
Rust 的 NLL 对"先 mut 操作子字段，再整体替换"的模式支持良好。无需提前引入 `std::mem::replace` 或辅助变量。

---

### F-006 错误位置计算：单测通过但集成测试暴露语义差异

- `tags`：#error #test-diff
- `severity`：3
- `model_behavior`：prompted
- `rounds`：2
- `evidence`：`src/lib.rs` `te_compile`；C `*error = s.next - s.start`；初始 Rust `*error = -1`

**Observation**  
C `te_compile` 在 `list()` 返回 NULL（内存分配失败）时设 `*error = -1`，在解析错误时设 `*error = 当前位置`。Rust 将两种情况都写为 `-1`，36 个单元测试通过，但集成对比测试暴露错误位置不匹配（C 输出 `Error at position 2`，Rust 输出 `Error at position -1`）。

**Analysis**  
Rust 中 `Box::new` 不会失败，两个分支合并为一个，应统一用 `s.error_pos()`。单测覆盖了成功路径和"有无错误"，但未精确断言错误位置值。

**Implication**  
错误码/位置的精确语义是单测覆盖盲区，需在集成层（compare.ps1 级）验证。翻译时应将 `*error` 语义显式注释，避免默认值掩盖差异。

---

### F-007 工具链默认值不符合约束

- `tags`：#prompt
- `severity`：2
- `model_behavior`：proactive
- `rounds`：1
- `evidence`：`rustup show active-toolchain` → `stable-x86_64-pc-windows-msvc`；CLAUDE.md 约束为 `windows-gnu`

**Observation**  
机器默认 Rust 工具链为 MSVC，MSVC 链接器报错。实验要求 GNU 工具链，需显式 `cargo +stable-x86_64-pc-windows-gnu`。

**Analysis**  
工具链版本管理是可通过 `rust-toolchain.toml` 固定的环境问题，不是翻译质量问题。

**Implication**  
翻译实验启动前应写入 `rust-toolchain.toml` 固定工具链，消除此类噪音。
