from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def write_report(run_dir: Path, summary: dict[str, Any]) -> None:
    (run_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=True), encoding="utf-8"
    )
    verification = summary.get("verification", [])
    lines = [
        f"# Run: {summary['task_id']}",
        "",
        f"- Agent: `{summary['agent']}`",
        f"- Status: **{summary['status']}**",
        f"- Steps: {summary['steps']}",
        f"- Duration: {summary['duration_ms']} ms",
        f"- Changed files: {len(summary['changed_files'])}",
        f"- Policy violations: {len(summary['policy_violations'])}",
        "",
        "## Verification",
        "",
        "| Command | Result | Duration |",
        "|---|---:|---:|",
    ]
    for item in verification:
        command = str(item["spec"]["command"]).replace("|", "\\|")
        result = "PASS" if item["passed"] else "FAIL"
        lines.append(f"| `{command}` | {result} | {item['duration_ms']} ms |")
    lines.extend(["", "## Changed Files", ""])
    lines.extend(f"- `{path}`" for path in summary["changed_files"])
    if not summary["changed_files"]:
        lines.append("_None_")
    (run_dir / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
