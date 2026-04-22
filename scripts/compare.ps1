# C vs Rust 对照测试脚本
# 用法：在仓库根目录运行 .\scripts\compare.ps1
#
# 前置条件：
#   - tinyexpr-c/repl.exe 存在（init.ps1 生成）
#   - tinyexpr-rs/target/release/repl.exe 存在（cargo build --release 生成）

$ErrorActionPreference = "Continue"

$C_REPL = ".\tinyexpr-c\repl.exe"
$RS_REPL = ".\tinyexpr-rs\target\release\repl.exe"
$TEST_CSV = ".\data\test_cases.csv"
$RESULT_CSV = ".\data\compare_result.csv"

# ========= 前置检查 =========
if (-not (Test-Path $C_REPL)) {
    Write-Host "✗ 缺少 C 版 repl.exe，请先运行 .\scripts\init.ps1" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $RS_REPL)) {
    Write-Host "✗ 缺少 Rust 版二进制。请先执行：" -ForegroundColor Yellow
    Write-Host "    cd tinyexpr-rs && cargo build --release" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $TEST_CSV)) {
    Write-Host "✗ 缺少测试用例文件 $TEST_CSV" -ForegroundColor Red
    exit 1
}

# ========= 加载测试用例 =========
# 跳过以 # 开头的行
$lines = Get-Content $TEST_CSV -Encoding UTF8 | Where-Object { $_ -and -not $_.StartsWith("#") }
$header = $lines[0]
$cases = $lines[1..($lines.Count - 1)] | ForEach-Object {
    $fields = $_ -split ',', 4  # category,expression,expected_behavior,notes
    if ($fields.Count -ge 2) {
        [PSCustomObject]@{
            Category   = $fields[0].Trim()
            Expression = $fields[1].Trim('"').Trim()
            Expected   = if ($fields.Count -ge 3) { $fields[2].Trim() } else { "" }
            Notes      = if ($fields.Count -ge 4) { $fields[3].Trim() } else { "" }
        }
    }
} | Where-Object { $_ -ne $null }

Write-Host "=== 开始对照测试（共 $($cases.Count) 个用例）===" -ForegroundColor Cyan

# ========= 执行对照 =========
function Invoke-Repl {
    param([string]$exe, [string]$input)
    try {
        $output = $input | & $exe 2>&1 | Out-String
        return $output.Trim()
    } catch {
        return "ERROR: $_"
    }
}

function Compare-Output {
    param([string]$c, [string]$r)
    # 完全一致
    if ($c -eq $r) { return @{ Match = $true; Reason = "exact" } }

    # 尝试按浮点数比较（容差 1e-12 相对误差）
    $cNum = 0.0; $rNum = 0.0
    $cIsNum = [double]::TryParse($c, [ref]$cNum)
    $rIsNum = [double]::TryParse($r, [ref]$rNum)
    if ($cIsNum -and $rIsNum) {
        if ([double]::IsNaN($cNum) -and [double]::IsNaN($rNum)) {
            return @{ Match = $true; Reason = "both NaN" }
        }
        if ([double]::IsInfinity($cNum) -and [double]::IsInfinity($rNum)) {
            if ([double]::IsPositiveInfinity($cNum) -eq [double]::IsPositiveInfinity($rNum)) {
                return @{ Match = $true; Reason = "same inf" }
            }
        }
        $absDiff = [math]::Abs($cNum - $rNum)
        $absMax = [math]::Max([math]::Abs($cNum), [math]::Abs($rNum))
        if ($absMax -eq 0) {
            return @{ Match = ($absDiff -eq 0); Reason = "zero exact" }
        }
        $relDiff = $absDiff / $absMax
        if ($relDiff -lt 1e-12) {
            return @{ Match = $true; Reason = "within 1e-12" }
        }
        return @{ Match = $false; Reason = "reldiff=$relDiff" }
    }

    return @{ Match = $false; Reason = "string mismatch" }
}

$results = @()
$pass = 0
$fail = 0
$catStats = @{}

foreach ($case in $cases) {
    $cOut = Invoke-Repl $C_REPL $case.Expression
    $rOut = Invoke-Repl $RS_REPL $case.Expression
    $cmp = Compare-Output $cOut $rOut

    if (-not $catStats.ContainsKey($case.Category)) {
        $catStats[$case.Category] = @{ Pass = 0; Fail = 0 }
    }

    if ($cmp.Match) {
        $pass++
        $catStats[$case.Category].Pass++
        $status = "PASS"
        $color = "Green"
    } else {
        $fail++
        $catStats[$case.Category].Fail++
        $status = "FAIL"
        $color = "Red"
    }

    Write-Host ("[{0}] [{1,-12}] {2}" -f $status, $case.Category, $case.Expression) -ForegroundColor $color
    if (-not $cmp.Match) {
        Write-Host ("       C : {0}" -f $cOut) -ForegroundColor DarkGray
        Write-Host ("       RS: {0}" -f $rOut) -ForegroundColor DarkGray
        Write-Host ("       原因: {0}" -f $cmp.Reason) -ForegroundColor DarkGray
    }

    $results += [PSCustomObject]@{
        Category   = $case.Category
        Expression = $case.Expression
        C_Output   = $cOut
        Rust_Output = $rOut
        Match      = $cmp.Match
        Reason     = $cmp.Reason
    }
}

# ========= 总结 =========
Write-Host "`n=== 结果: $pass 通过 / $fail 失败 / 共 $($cases.Count) ===" -ForegroundColor Cyan

Write-Host "`n分类统计：" -ForegroundColor Cyan
foreach ($cat in ($catStats.Keys | Sort-Object)) {
    $s = $catStats[$cat]
    $total = $s.Pass + $s.Fail
    $rate = if ($total -gt 0) { [math]::Round(100.0 * $s.Pass / $total, 1) } else { 0 }
    $color = if ($s.Fail -eq 0) { "Green" } elseif ($s.Pass -eq 0) { "Red" } else { "Yellow" }
    Write-Host ("  {0,-12}: {1,3}/{2,-3} ({3}%)" -f $cat, $s.Pass, $total, $rate) -ForegroundColor $color
}

# ========= 导出 CSV =========
$results | Export-Csv -Path $RESULT_CSV -Encoding UTF8 -NoTypeInformation
Write-Host "`n详细结果已写入 $RESULT_CSV" -ForegroundColor Gray

# ========= CI 退出码 =========
if ($fail -gt 0) {
    exit 1
}
