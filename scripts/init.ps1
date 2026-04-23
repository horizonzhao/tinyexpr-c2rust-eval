# tinyexpr C->Rust evaluation init script
# Run from repository root: .\scripts\init.ps1

$ErrorActionPreference = "Stop"
$PSDefaultParameterValues['Out-File:Encoding'] = 'utf8'

Write-Host "=== tinyexpr C->Rust evaluation initialization ===" -ForegroundColor Cyan

if (-not (Test-Path "README.md") -or -not (Test-Path "METHODOLOGY.md")) {
    Write-Host "Error: run this script from repository root." -ForegroundColor Red
    exit 1
}

Write-Host "`n[1/7] Checking toolchain..." -ForegroundColor Yellow
$tools = @{
    "git"    = "version control"
    "cargo"  = "Rust package manager"
    "rustc"  = "Rust compiler"
    "gcc"    = "C compiler (MinGW)"
    "claude" = "Claude Code (optional)"
}

$missing = @()
foreach ($t in $tools.Keys) {
    $cmd = Get-Command $t -ErrorAction SilentlyContinue
    if ($null -eq $cmd) {
        if ($t -eq "claude") {
            Write-Host "  [WARN] Missing $t ($($tools[$t])) - optional for init" -ForegroundColor DarkYellow
        } else {
            Write-Host "  [ERR] Missing $t ($($tools[$t]))" -ForegroundColor Red
            $missing += $t
        }
    } else {
        Write-Host "  [OK] $t" -ForegroundColor Green
    }
}

if ($missing.Count -gt 0) {
    Write-Host "`nRequired tools are missing. See docs/02-setup.md." -ForegroundColor Red
    exit 1
}

Write-Host "`n[2/7] Recording environment to data/environment.txt..." -ForegroundColor Yellow
$envFile = "data/environment.txt"
"=== Environment $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===" | Out-File $envFile
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
"# rustup active toolchain" | Out-File $envFile -Append
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
    try { (claude --version) | Out-File $envFile -Append } catch { "(version unavailable)" | Out-File $envFile -Append }
}
Write-Host "  [OK] Wrote $envFile" -ForegroundColor Green

Write-Host "`n[3/7] Cloning tinyexpr C reference..." -ForegroundColor Yellow
if (Test-Path "tinyexpr-c") {
    Write-Host "  tinyexpr-c/ already exists, skip clone." -ForegroundColor Gray
} else {
    git clone --depth 1 https://github.com/codeplea/tinyexpr.git tinyexpr-c
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path "tinyexpr-c")) {
        Write-Host "  [WARN] Clone failed, retrying once..." -ForegroundColor DarkYellow
        Start-Sleep -Seconds 2
        git clone --depth 1 https://github.com/codeplea/tinyexpr.git tinyexpr-c
    }

    if ($LASTEXITCODE -ne 0 -or -not (Test-Path "tinyexpr-c")) {
        Write-Host "  [ERR] Clone failed. Please check network/proxy and rerun." -ForegroundColor Red
        exit 1
    }

    Write-Host "  [OK] Clone done." -ForegroundColor Green
}

Push-Location tinyexpr-c
try {
    $commit = git rev-parse HEAD
    "`n# tinyexpr commit hash" | Out-File "../$envFile" -Append
    $commit | Out-File "../$envFile" -Append
    Write-Host "  tinyexpr commit: $commit" -ForegroundColor Gray
} finally {
    Pop-Location
}

Write-Host "`n[4/7] Building C baseline binaries..." -ForegroundColor Yellow
Push-Location tinyexpr-c
try {
    gcc -O2 -o smoke.exe smoke.c tinyexpr.c -lm 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [ERR] Failed building smoke.exe" -ForegroundColor Red
        exit 1
    }
    Write-Host "  [OK] smoke.exe" -ForegroundColor Green

    gcc -O2 -o repl.exe repl.c tinyexpr.c -lm 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [ERR] Failed building repl.exe" -ForegroundColor Red
        exit 1
    }
    Write-Host "  [OK] repl.exe" -ForegroundColor Green

    Write-Host "  Running smoke.exe check..." -ForegroundColor Gray
    $smokeOut = & .\smoke.exe 2>&1
    Write-Host ($smokeOut | Out-String) -ForegroundColor DarkGray
} finally {
    Pop-Location
}

Write-Host "`n[5/7] Creating Rust project skeleton..." -ForegroundColor Yellow
if (Test-Path "tinyexpr-rs") {
    Write-Host "  tinyexpr-rs/ already exists, skip cargo new." -ForegroundColor Gray
} else {
    cargo new tinyexpr-rs --lib
    Write-Host "  [OK] cargo new done." -ForegroundColor Green

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

    New-Item -ItemType Directory -Force -Path "tinyexpr-rs/src/bin" | Out-Null
    $replStub = @'
// Placeholder REPL. Replace after te_interp is translated.
fn main() {
    use std::io::{self, BufRead, Write};
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        println!("(stub) input was: {}", line);
        io::stdout().flush().ok();
    }
}
'@
    $replStub | Out-File "tinyexpr-rs/src/bin/repl.rs"
    Write-Host "  [OK] Wrote Cargo.toml and REPL stub." -ForegroundColor Green
}

Write-Host "`n[6/7] Copying Claude context file..." -ForegroundColor Yellow
if (Test-Path ".claude/CLAUDE.md") {
    Copy-Item ".claude/CLAUDE.md" -Destination "tinyexpr-rs/CLAUDE.md" -Force
    Write-Host "  [OK] Copied .claude/CLAUDE.md -> tinyexpr-rs/CLAUDE.md" -ForegroundColor Green
} else {
    Write-Host "  [WARN] .claude/CLAUDE.md not found, skipped." -ForegroundColor DarkYellow
}

Write-Host "`n[7/7] Initialization completed." -ForegroundColor Cyan
Write-Host @"

Directory state:
  tinyexpr-c/          C reference + smoke.exe / repl.exe
  tinyexpr-rs/         Rust skeleton + CLAUDE.md
  data/environment.txt Toolchain snapshot

Next:
  1) Read .claude/prompts/00-startup.md
  2) cd tinyexpr-rs
  3) claude
  4) Paste 00-startup prompt

When ready for cross-check:
  cd tinyexpr-rs
  cargo build --release
  cd ..
  .\scripts\compare.ps1

"@ -ForegroundColor White
