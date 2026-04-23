# 02. 环境搭建（Setup）

本文给出最小可复现实验环境。

## 1. 工具链

- Rust：`stable-x86_64-pc-windows-gnu`
- C 编译器：MinGW-w64 gcc
- 其他：Git、PowerShell、Claude Code

版本信息建议写入 `data/environment.txt`。
仓库根目录的 `rust-toolchain.toml` 已固定 Rust 工具链；在仓库内执行 `cargo` 时通常无需手动 `rustup default`。

## 2. 初始化

```powershell
cd D:\projects\tinyexpr-c2rust-eval
.\scripts\init.ps1
```

`init.ps1` 完成以下操作：检查依赖、克隆 `tinyexpr-c/`、编译 C 基准、创建 `tinyexpr-rs/`、同步交互协议文件。

## 3. 就绪验证

```powershell
.\tinyexpr-c\smoke.exe
.\tinyexpr-c\repl.exe
cd tinyexpr-rs
cargo build
claude --version
```

若上述命令均成功，环境可用于翻译与对照评测。

## 4. 常见问题

- `gcc` 不可用：检查 `C:\msys64\mingw64\bin` 是否在 PATH。
- Rust 链接异常：确认工具链为 GNU 版本而非 MSVC。
- C 编译参数异常：优先复查 `scripts/init.ps1` 的编译选项与本机 gcc 版本。
