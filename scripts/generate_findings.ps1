# 从 logs/raw-journal.md 中提取候选发现，辅助人工提炼 FINDINGS.md
# 用法：在仓库根目录运行 .\scripts\generate_findings.ps1
#
# 说明：本脚本不自动生成 FINDINGS 条目（那需要人类判断），
#       只是从原始日志中按关键词筛出候选片段列表，人工决定哪些要提升为精选。

$ErrorActionPreference = "Stop"

$JOURNAL = ".\logs\raw-journal.md"
$OUTPUT = ".\data\findings_candidates.md"

if (-not (Test-Path $JOURNAL)) {
    Write-Host "✗ $JOURNAL 不存在" -ForegroundColor Red
    exit 1
}

# ========= 关键词：提示可能是有价值发现的词 =========
$keywords = @(
    # 情绪信号
    "意外", "惊讶", "没想到", "发现", "啊哈",
    # 问题信号
    "问题", "错", "bug", "失败", "不对", "不一致", "差异",
    # 转折信号
    "但是", "然而", "不过", "重新", "重写", "换方案",
    # 难度信号
    "卡", "困难", "复杂", "反复", "多次",
    # 技术信号
    "unsafe", "生命周期", "所有权", "联合体", "指针", "宏", "溢出", "NaN"
)

# ========= 解析日志 =========
$content = Get-Content $JOURNAL -Raw -Encoding UTF8

# 用 ### 分块
$blocks = $content -split "(?m)^### " | Where-Object { $_.Trim() }

Write-Host "扫描 $($blocks.Count) 个日志块..." -ForegroundColor Cyan

$candidates = @()

foreach ($block in $blocks) {
    $hitKeywords = @()
    foreach ($kw in $keywords) {
        if ($block -match [regex]::Escape($kw)) {
            $hitKeywords += $kw
        }
    }
    if ($hitKeywords.Count -ge 2) {
        # 至少命中 2 个关键词才作为候选（降低噪音）
        $title = ($block -split "`n")[0].Trim()
        $excerpt = ($block -split "`n" | Select-Object -First 10) -join "`n"
        $candidates += [PSCustomObject]@{
            Title = $title
            Keywords = ($hitKeywords -join ", ")
            Excerpt = $excerpt.Substring(0, [math]::Min(500, $excerpt.Length))
        }
    }
}

Write-Host "找到 $($candidates.Count) 个候选片段" -ForegroundColor Cyan

# ========= 输出 =========
$header = @"
# 发现候选列表（自动生成）

> 由 ``scripts/generate_findings.ps1`` 从 [``raw-journal.md``](../logs/raw-journal.md) 自动筛选。
> 生成时间：$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
>
> **使用方法**：
> 1. 人工阅读每个候选片段，判断是否值得提升为精选 Finding
> 2. 决定的条目迁移到 [``FINDINGS.md``](../FINDINGS.md) 并编号 F-XXX
> 3. 本文件不需要提交到仓库（每次重新生成）

找到 $($candidates.Count) 个候选。

---

"@

$body = foreach ($c in $candidates) {
    @"
## 候选：$($c.Title)

**命中关键词**：$($c.Keywords)

**片段**：

```
$($c.Excerpt)...
```

**判断**：
- [ ] 值得提升为 Finding
- [ ] 不值得（原因：_TBD_）
- [ ] 需要补充上下文再判断

---

"@
}

($header + ($body -join "`n")) | Out-File $OUTPUT -Encoding UTF8
Write-Host "✓ 候选列表写入 $OUTPUT" -ForegroundColor Green
