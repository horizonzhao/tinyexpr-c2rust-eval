# 从 data/metrics.csv 汇总指标到 data/metrics_summary.md
# 用法：在仓库根目录运行 .\scripts\collect_metrics.ps1

$ErrorActionPreference = "Stop"

$METRICS = ".\data\metrics.csv"
$SUMMARY = ".\data\metrics_summary.md"

if (-not (Test-Path $METRICS)) {
    Write-Host "✗ $METRICS 不存在" -ForegroundColor Red
    exit 1
}

# 跳过注释行
$lines = Get-Content $METRICS -Encoding UTF8 | Where-Object { $_ -and -not $_.StartsWith("#") }
if ($lines.Count -lt 2) {
    Write-Host "⚠  metrics.csv 仅含表头，尚未录入数据" -ForegroundColor Yellow
    $records = @()
} else {
    $records = $lines | ConvertFrom-Csv
}

$total = $records.Count
Write-Host "共 $total 条记录" -ForegroundColor Cyan

# ========= 统计 =========
if ($total -eq 0) {
    $firstTryCompiled = 0
    $firstTryTested = 0
    $usedUnsafe = 0
    $rounds1 = 0
    $rounds2 = 0
    $rounds3plus = 0
    $interventionNone = 0
    $interventionHint = 0
    $interventionRedesign = 0
    $cTotalLoc = 0
    $rsTotalLoc = 0
    $avgRounds = 0
} else {
    $firstTryCompiled = ($records | Where-Object { $_.first_try_compiled -eq "yes" }).Count
    $firstTryTested = ($records | Where-Object { $_.first_try_tested -eq "yes" }).Count
    $usedUnsafe = ($records | Where-Object { $_.used_unsafe -eq "yes" }).Count
    $rounds1 = ($records | Where-Object { [int]$_.rounds -le 1 }).Count
    $rounds2 = ($records | Where-Object { [int]$_.rounds -eq 2 }).Count
    $rounds3plus = ($records | Where-Object { [int]$_.rounds -ge 3 }).Count
    $interventionNone = ($records | Where-Object { $_.human_intervention -eq "none" }).Count
    $interventionHint = ($records | Where-Object { $_.human_intervention -eq "hint" }).Count
    $interventionRedesign = ($records | Where-Object { $_.human_intervention -eq "redesign" }).Count
    $cTotalLoc = ($records | Measure-Object -Property c_loc -Sum).Sum
    $rsTotalLoc = ($records | Measure-Object -Property rust_loc -Sum).Sum
    $avgRounds = [math]::Round(($records | Measure-Object -Property rounds -Average).Average, 2)
}

function Pct([int]$part, [int]$whole) {
    if ($whole -eq 0) { return "N/A" }
    return "$([math]::Round(100.0 * $part / $whole, 1))%"
}

# ========= 标签统计 =========
$tagCounts = @{}
foreach ($r in $records) {
    if ($r.tags) {
        $r.tags -split ';' | ForEach-Object {
            $t = $_.Trim()
            if ($t -and $t -ne "none") {
                if (-not $tagCounts.ContainsKey($t)) { $tagCounts[$t] = 0 }
                $tagCounts[$t]++
            }
        }
    }
}

# ========= 生成报告 =========
$report = @"
# 指标汇总（自动生成）

> 由 ``scripts/collect_metrics.ps1`` 从 [``metrics.csv``](metrics.csv) 生成。
> 生成时间：$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

## 整体

| 指标 | 数值 |
|-----|------|
| 翻译函数总数 | $total |
| C 原版总行数 | $cTotalLoc |
| Rust 翻译总行数 | $rsTotalLoc |
| 代码膨胀率 | $(if ($cTotalLoc -gt 0) { "{0:P1}" -f ($rsTotalLoc / $cTotalLoc) } else { "N/A" }) |
| 平均修正轮次 | $avgRounds |

## 首次通过率

| 指标 | 数量 / 总数 | 比例 |
|-----|------------|------|
| 首次编译通过 | $firstTryCompiled / $total | $(Pct $firstTryCompiled $total) |
| 首次测试通过 | $firstTryTested / $total | $(Pct $firstTryTested $total) |

## 修正轮次分布

| 轮次 | 数量 | 比例 |
|-----|-----|------|
| 1 轮（一次通过） | $rounds1 | $(Pct $rounds1 $total) |
| 2 轮 | $rounds2 | $(Pct $rounds2 $total) |
| 3 轮及以上 | $rounds3plus | $(Pct $rounds3plus $total) |

## 人工介入分布

| 介入类型 | 数量 | 比例 |
|---------|-----|------|
| 无介入（none） | $interventionNone | $(Pct $interventionNone $total) |
| 仅提示（hint） | $interventionHint | $(Pct $interventionHint $total) |
| 重新设计（redesign） | $interventionRedesign | $(Pct $interventionRedesign $total) |

## unsafe 使用

| 指标 | 数值 |
|-----|------|
| 含 ``unsafe`` 的函数 | $usedUnsafe / $total |
| 比例 | $(Pct $usedUnsafe $total) |

## 难题标签频次

"@

if ($tagCounts.Count -eq 0) {
    $report += "_暂无标签数据_`n"
} else {
    $report += "| 标签 | 出现次数 |`n|-----|--------|`n"
    foreach ($kv in ($tagCounts.GetEnumerator() | Sort-Object Value -Descending)) {
        $report += "| ``$($kv.Key)`` | $($kv.Value) |`n"
    }
}

$report | Out-File $SUMMARY -Encoding UTF8
Write-Host "✓ 已生成 $SUMMARY" -ForegroundColor Green
