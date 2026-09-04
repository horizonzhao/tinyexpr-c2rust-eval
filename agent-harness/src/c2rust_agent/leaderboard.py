from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def build_leaderboard(runs_root: Path) -> dict[str, Any]:
    summaries = []
    skipped_runs = 0
    for path in sorted(runs_root.glob("*/summary.json")):
        summary = json.loads(path.read_text(encoding="utf-8"))
        if "score" not in summary:
            skipped_runs += 1
            continue
        summaries.append(summary)

    agents: dict[str, list[dict[str, Any]]] = {}
    for summary in summaries:
        agents.setdefault(summary["agent"], []).append(summary)

    rows = []
    for agent, runs in sorted(agents.items()):
        count = len(runs)
        rows.append(
            {
                "agent": agent,
                "runs": count,
                "resolved": sum(1 for run in runs if run["score"]["resolved"]),
                "resolve_rate": round(
                    100.0 * sum(1 for run in runs if run["score"]["resolved"]) / count,
                    2,
                ),
                "average_score": round(
                    sum(float(run["score"]["total"]) for run in runs) / count, 2
                ),
                "average_steps": round(sum(int(run["steps"]) for run in runs) / count, 2),
                "average_duration_ms": round(
                    sum(int(run["duration_ms"]) for run in runs) / count
                ),
            }
        )
    rows.sort(key=lambda row: (-row["resolve_rate"], -row["average_score"], row["agent"]))
    return {"total_runs": len(summaries), "skipped_runs": skipped_runs, "agents": rows}


def write_leaderboard(runs_root: Path, output_dir: Path) -> dict[str, Any]:
    board = build_leaderboard(runs_root)
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "leaderboard.json").write_text(
        json.dumps(board, indent=2, ensure_ascii=True), encoding="utf-8"
    )
    lines = [
        "# Coding Agent 排行榜",
        "",
        "| Agent | 解决数 | 解决率 | 平均分 | 平均步骤 | 平均耗时 |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for row in board["agents"]:
        lines.append(
            f"| {row['agent']} | {row['resolved']}/{row['runs']} | "
            f"{row['resolve_rate']}% | {row['average_score']} | "
            f"{row['average_steps']} | {row['average_duration_ms']} ms |"
        )
    (output_dir / "leaderboard.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return board
