from __future__ import annotations

import time
from dataclasses import asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .adapters.base import AgentAdapter
from .models import Action, Task, VerifyCommand
from .policies import check_action, check_changed_files
from .report import write_report
from .trajectory import TrajectoryWriter
from .verifier import run_command, verify
from .workspace import changed_files, resolve_inside, snapshot


def _execute(task: Task, action: Action) -> dict[str, Any]:
    if action.type == "write_file":
        target = resolve_inside(task.repository_root, action.path or "")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(action.content or "", encoding="utf-8")
        return {"ok": True, "path": action.path}
    if action.type == "run_command":
        return run_command(
            task,
            VerifyCommand(command=action.command or "", cwd=action.cwd),
        )
    if action.type == "finish":
        return {"ok": True, "message": action.message}
    return {"ok": False, "error": f"unsupported action: {action.type}"}


def run(task: Task, agent: AgentAdapter, runs_root: Path) -> Path:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = runs_root / f"{task.id}-{agent.name}-{stamp}"
    trajectory = TrajectoryWriter(run_dir / "trajectory.jsonl")
    before = snapshot(task.repository_root)
    started = time.monotonic()
    observation: dict[str, Any] = {"type": "start", "message": task.problem}
    violations: list[str] = []
    steps = 0
    agent_finished = False

    trajectory.append("run_started", task_id=task.id, agent=agent.name)
    for step in range(1, task.budget.max_steps + 1):
        if time.monotonic() - started > task.budget.max_seconds:
            observation = {"ok": False, "error": "time budget exhausted"}
            break
        steps = step
        action = agent.next_action(task, observation, step)
        allowed, reason = check_action(task, action)
        trajectory.append("action", step=step, action=asdict(action), allowed=allowed, reason=reason)
        if not allowed:
            violations.append(reason)
            observation = {"ok": False, "error": reason}
            continue
        observation = _execute(task, action)
        trajectory.append("observation", step=step, observation=observation)
        if action.type == "finish":
            agent_finished = True
            break

    after = snapshot(task.repository_root)
    changed = changed_files(before, after)
    violations.extend(f"changed forbidden path: {path}" for path in check_changed_files(task, changed))
    verification = verify(task)
    passed = all(item["passed"] for item in verification)
    status = "passed" if passed and not violations and agent_finished else "failed"
    summary = {
        "task_id": task.id,
        "agent": agent.name,
        "status": status,
        "steps": steps,
        "duration_ms": round((time.monotonic() - started) * 1000),
        "changed_files": changed,
        "policy_violations": violations,
        "verification": verification,
    }
    trajectory.append("run_finished", **summary)
    write_report(run_dir, summary)
    return run_dir
