param(
    [string]$Model = "deepseek-v4-flash",
    [int]$MaxTokens = 8192,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$RepoRoot = Split-Path -Parent $PSScriptRoot
$AgentExe = Join-Path $RepoRoot ".venv\Scripts\c2rust-agent.exe"
$TasksDir = Join-Path $RepoRoot "agent-harness\tasks"
$RunsDir = Join-Path $RepoRoot "agent-harness\runs"

if (-not (Test-Path $AgentExe)) {
    throw "Agent executable not found: $AgentExe. Create the virtual environment and install agent-harness first."
}

if (-not $env:DEEPSEEK_API_KEY) {
    throw "DEEPSEEK_API_KEY is not set in the current PowerShell session."
}

New-Item -ItemType Directory -Path $RunsDir -Force | Out-Null
$Tasks = Get-ChildItem "$TasksDir\repair-*.yaml" | Sort-Object Name
$Results = @()

foreach ($Task in $Tasks) {
    $TaskId = $Task.BaseName
    $Existing = Get-ChildItem $RunsDir -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "$TaskId-deepseek-*" } |
        ForEach-Object {
            $Path = Join-Path $_.FullName "summary.json"
            if (Test-Path $Path) {
                Get-Content $Path -Raw | ConvertFrom-Json
            }
        } |
        Where-Object { $_.status -eq "passed" } |
        Select-Object -First 1

    if (-not $Force -and $Existing) {
        Write-Host "[SKIP] $TaskId already has a passing result" -ForegroundColor Yellow
        $Summary = $Existing
        $Source = "existing"
    }
    else {
        Write-Host "[RUN] $TaskId" -ForegroundColor Cyan
        & $AgentExe run $Task.FullName `
            --agent deepseek `
            --model $Model `
            --thinking enabled `
            --reasoning-effort high `
            --max-tokens $MaxTokens `
            --runs-dir $RunsDir

        $Latest = Get-ChildItem $RunsDir -Directory |
            Where-Object { $_.Name -like "$TaskId-deepseek-*" } |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1
        $SummaryPath = if ($Latest) { Join-Path $Latest.FullName "summary.json" }
        $Summary = if ($SummaryPath -and (Test-Path $SummaryPath)) {
            Get-Content $SummaryPath -Raw | ConvertFrom-Json
        }
        else {
            $null
        }
        $Source = "new"
    }

    if ($Summary) {
        $Results += [PSCustomObject]@{
            Task       = $TaskId
            Status     = $Summary.status
            Score      = $Summary.score.total
            Steps      = $Summary.steps
            DurationMs = $Summary.duration_ms
            Tokens     = $Summary.agent_metrics.total_tokens
            Requests   = $Summary.agent_metrics.api_requests
            Source     = $Source
        }
    }
    else {
        $Results += [PSCustomObject]@{
            Task       = $TaskId
            Status     = "no-summary"
            Score      = 0
            Steps      = 0
            DurationMs = 0
            Tokens     = 0
            Requests   = 0
            Source     = $Source
        }
    }

    Start-Sleep -Seconds 2
}

$CsvPath = Join-Path $RunsDir "$Model-summary.csv"
$Results | Export-Csv $CsvPath -NoTypeInformation -Encoding UTF8
& $AgentExe scoreboard $RunsDir --output $RunsDir

$Passed = @($Results | Where-Object Status -eq "passed").Count
$AverageScore = ($Results | Measure-Object Score -Average).Average
$TotalTokens = ($Results | Measure-Object Tokens -Sum).Sum
$TotalRequests = ($Results | Measure-Object Requests -Sum).Sum

Write-Host "`n=== DeepSeek benchmark results ===" -ForegroundColor Cyan
$Results | Format-Table -AutoSize
Write-Host "Resolved tasks: $Passed / $($Results.Count)"
Write-Host ("Average score: {0:N2}" -f $AverageScore)
Write-Host "Total tokens: $TotalTokens"
Write-Host "API requests: $TotalRequests"
Write-Host "CSV: $CsvPath"
Write-Host "Leaderboard: $RunsDir\leaderboard.md"
