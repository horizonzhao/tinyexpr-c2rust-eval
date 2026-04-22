# tinyexpr C→Rust 评测实验初始化脚本
# 用法：在仓库根目录运行 .\scripts\init.ps1

$ErrorActionPreference = "Stop"
$PSDefaultParameterValues['Out-File:Encoding'] = 'utf8'

Write-Host "=== tinyexpr C->Rust 评测实验初始化 ===" -ForegroundColor Cyan

# 确认在仓库根目录
if (-not (Test-Path "README.md") -or -not (Test-Path "METHODOLOGY.md")) {
    Write-Host "错误：请在仓库根目录运行此脚本" -ForegroundColor Red
    exit 1
}

# ========= 1. 工具链检查 =========
Write-Host "`n[1/7] 检查工具链..." -ForegroundColor Yellow
$tools = @{
    "git"    = "版本控制"
    "cargo"  = "Rust 包管理"
    "rustc"  = "Rust 编译器"
    "gcc"    = "C 编译器（MinGW）"
    "claude" = "Claude Code（可选，如缺失会跳过）"
}

$missing = @()
foreach ($t in $tools.Keys) {
    $cmd = Get-Command $t -ErrorAction SilentlyContinue
    if ($null -eq $cmd) {
        if ($t -eq "claude") {
            Write-Host "  ⚠  缺少 $t（$($tools[$t])）- 可稍后安装" -ForegroundColor DarkYellow
        } else {
            Write-Host "  ✗ 缺少 $t（$($tools[$t])）" -ForegroundColor Red
            $missing += $t
        }
    } else {
        Write-Host "  ✓ $t" -ForegroundColor Green
    }
}

if ($missing.Count -gt 0) {
    Write-Host "`n必需工具缺失，见 docs/02-setup.md 安装指南" -ForegroundColor Red
    exit 1
}

# ========= 2. 记录工具版本 =========
Write-Host "`n[2/7] 记录工具版本到 data/environment.txt..." -ForegroundColor Yellow
$envFile = "data/environment.txt"
"=== 实验环境 $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===" | Out-File $envFile
"" | Out-File $envFile -Append
"# OS" | Out-File $envFile -Append
(Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, OSArchitecture | Format-List | Out-String).Trim() | Out-File $envFile -Append
"" | Out-File $envFile -Append
"# rustc" | Out-File $envFile -Append
(rustc --version) | Out-File $envFile -Append
"" | Out-File $envFile -Append
"# cargo" | Out-File $envFile -Append
(cargo --version) | Out-File $envFile -Append
"" | Out-File $envFile -Append
"# rustup default" | Out-File $envFile -Append
(rustup show active-toolchain) | Out-File $envFile -Append
"" | Out-File $envFile -Append
"# gcc" | Out-File $envFile -Append
(gcc --version | Select-Object -First 1) | Out-File $envFile -Append
"" | Out-File $envFile -Append
"# git" | Out-File $envFile -Append
(git --version) | Out-File $envFile -Append
if (Get-Command claude -ErrorAction SilentlyContinue) {
    "" | Out-File $envFile -Append
    "# claude" | Out-File $envFile -Append
    try { (claude --version) | Out-File $envFile -Append } catch { "(未能获取版本)" | Out-File $envFile -Append }
}
Write-Host "  ✓ 已写入 $envFile" -ForegroundColor Green

# ========= 3. 克隆 tinyexpr C 原版 =========
Write-Host "`n[3/7] 克隆 tinyexpr C 原版..." -ForegroundColor Yellow
if (Test-Path "tinyexpr-c") {
    Write-Host "  tinyexpr-c/ 已存在，跳过克隆" -ForegroundColor Gray
} else {
    git clone --depth 1 https://github.com/codeplea/tinyexpr.git tinyexpr-c
    Write-Host "  ✓ 克隆完成" -ForegroundColor Green
}

# 记录 tinyexpr 的 commit hash，保证可复现
Push-Location tinyexpr-c
try {
    $commit = git rev-parse HEAD
    "`n# tinyexpr commit hash" | Out-File "../$envFile" -Append
    $commit | Out-File "../$envFile" -Append
    Write-Host "  C 原版 commit: $commit" -ForegroundColor Gray
} finally {
    Pop-Location
}

# ========= 4. 编译 C 对照基准 =========
Write-Host "`n[4/7] 编译 C 对照基准..." -ForegroundColor Yellow
Push-Location tinyexpr-c
try {
    # smoke test
    gcc -O2 -o smoke.exe smoke.c tinyexpr.c -lm 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ✗ 编译 smoke.exe 失败" -ForegroundColor Red
        exit 1
    }
    Write-Host "  ✓ smoke.exe" -ForegroundColor Green

    # repl for interactive testing
    gcc -O2 -o repl.exe repl.c tinyexpr.c -lm 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ✗ 编译 repl.exe 失败" -ForegroundColor Red
        exit 1
    }
    Write-Host "  ✓ repl.exe" -ForegroundColor Green

    # 运行 smoke 验证 C 版本在当前环境工作正常
    Write-Host "  运行 C 版 smoke test 验证..." -ForegroundColor Gray
    $smokeOut = & .\smoke.exe 2>&1
    Write-Host ($smokeOut | Out-String) -ForegroundColor DarkGray
} finally {
    Pop-Location
}

# ========= 5. 创建 Rust 项目 =========
Write-Host "`n[5/7] 创建 Rust 项目..." -ForegroundColor Yellow
if (Test-Path "tinyexpr-rs") {
    Write-Host "  tinyexpr-rs/ 已存在，跳过 cargo new" -ForegroundColor Gray
} else {
    cargo new tinyexpr-rs --lib
    Write-Host "  ✓ cargo new 完成" -ForegroundColor Green

    # 添加一个 bin target 供 compare.ps1 调用
    $cargoToml = @"
[package]
name = "tinyexpr-rs"
version = "0.1.0"
edition = "2021"

[lib]
name = "tinyexpr_rs"
path = "src/lib.rs"

[[bin]]
name = "repl"
path = "src/bin/repl.rs"

[profile.release]
lto = true
"@
    $cargoToml | Out-File "tinyexpr-rs/Cargo.toml"

    # 占位 REPL，等翻译完成后填入真实实现
    New-Item -ItemType Directory -Force -Path "tinyexpr-rs/src/bin" | Out-Null
    $replStub = @'
// 占位 REPL，实际实现等翻译完 te_interp 后替换
// 从 stdin 读取一行表达式，输出求值结果
fn main() {
    use std::io::{self, BufRead, Write};
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        // TODO: 替换为真正的 te_interp 调用
        println!("(stub) input was: {}", line);
        io::stdout().flush().ok();
    }
}
'@
    $replStub | Out-File "tinyexpr-rs/src/bin/repl.rs"
    Write-Host "  ✓ 配置 Cargo.toml 和 REPL 占位实现" -ForegroundColor Green
}

# ========= 6. 链接 CLAUDE.md =========
Write-Host "`n[6/7] 配置 Claude Code 上下文..." -ForegroundColor Yellow
if (Test-Path ".claude/CLAUDE.md") {
    # Claude Code 会优先读当前目录的 CLAUDE.md
    Copy-Item ".claude/CLAUDE.md" -Destination "tinyexpr-rs/CLAUDE.md" -Force
    Write-Host "  ✓ 已把 .claude/CLAUDE.md 复制到 tinyexpr-rs/CLAUDE.md" -ForegroundColor Green
} else {
    Write-Host "  ⚠  .claude/CLAUDE.md 不存在，跳过" -ForegroundColor DarkYellow
}

# ========= 7. 总结 =========
Write-Host "`n[7/7] 初始化完成！" -ForegroundColor Cyan
Write-Host @"

目录状态：
  tinyexpr-c/          <- C 原版 + 编译好的 smoke.exe / repl.exe
  tinyexpr-rs/         <- Rust 项目骨架 + CLAUDE.md
  data/environment.txt <- 本次实验的工具版本记录

下一步：
  1. 阅读 .claude/prompts/00-startup.md 了解如何启动 Claude Code
  2. cd tinyexpr-rs && claude
  3. 把 00-startup.md 的 prompt 粘给它
  4. 开始阶段 1：读懂 C 版本

跑对照测试（等你翻译到一定程度后）：
  cd tinyexpr-rs
  cargo build --release
  cd ..
  .\scripts\compare.ps1

"@ -ForegroundColor White
