from __future__ import annotations

import time
import shutil
from dataclasses import asdict, replace
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .adapters.base import AgentAdapter
from .models import Action, Task, VerifyCommand
from .policies import check_action, check_changed_files
from .report import write_report
from .scoring import calculate_score
from .trajectory import TrajectoryWriter
from .verifier import run_command, verify
from .workspace import changed_files, resolve_inside, snapshot


def _execute(task: Task, action: Action) -> dict[str, Any]:
    if action.type == "read_file":
        target = resolve_inside(task.repository_root, action.path or "")
        if not target.is_file():
            return {"ok": False, "error": f"file not found: {action.path}"}
        return {"ok": True, "path": action.path, "content": target.read_text(encoding="utf-8")[:50000]}
    if action.type == "list_files":
        target = resolve_inside(task.repository_root, action.path or ".")
        if not target.is_dir():
            return {"ok": False, "error": f"directory not found: {action.path}"}
        files = [
            path.relative_to(task.repository_root).as_posix()
            for path in sorted(target.rglob("*"))
            if path.is_file() and "target" not in path.parts
        ]
        return {"ok": True, "files": files[:500], "truncated": len(files) > 500}
    if action.type == "search_code":
        target = resolve_inside(task.repository_root, action.path or ".")
        pattern = action.pattern or ""
        matches = []
        candidates = [target] if target.is_file() else sorted(target.rglob("*"))
        for path in candidates:
            if not path.is_file() or "target" in path.parts:
                continue
            try:
                lines = path.read_text(encoding="utf-8").splitlines()
            except UnicodeDecodeError:
                continue
            for line_number, line in enumerate(lines, start=1):
                if pattern in line:
                    matches.append(
                        {
                            "path": path.relative_to(task.repository_root).as_posix(),
                            "line": line_number,
                            "text": line[:500],
                        }
                    )
                    if len(matches) >= 200:
                        return {"ok": True, "matches": matches, "truncated": True}
        return {"ok": True, "matches": matches, "truncated": False}
    if action.type == "write_file":
        target = resolve_inside(task.repository_root, action.path or "")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(action.content or "", encoding="utf-8")
        return {"ok": True, "path": action.path}
    if action.type == "run_command":
        return run_command(
            task,
            VerifyCommand(
                name="agent-command", command=action.command or "", cwd=action.cwd
            ),
        )
    if action.type == "finish":
        return {"ok": True, "message": action.message}
    return {"ok": False, "error": f"unsupported action: {action.type}"}


def run(task: Task, agent: AgentAdapter, runs_root: Path) -> Path:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    run_dir = runs_root / f"{task.id}-{agent.name}-{stamp}"
    if task.workspace_mode == "copy":
        workspace_root = run_dir / "workspace"
        shutil.copytree(
            task.repository_root,
            workspace_root,
            ignore=shutil.ignore_patterns("target", ".git", "__pycache__"),
        )
        task = replace(task, repository_root=workspace_root)
    elif task.workspace_mode != "in_place":
        raise ValueError(f"unsupported workspace_mode: {task.workspace_mode}")
    trajectory = TrajectoryWriter(run_dir / "trajectory.jsonl")
    before = snapshot(task.repository_root)
    started = time.monotonic()
    observation: dict[str, Any] = {"type": "start", "message": task.problem}
    violations: list[str] = []
    steps = 0
    agent_finished = False
    agent_error: str | None = None

    trajectory.append("run_started", task_id=task.id, agent=agent.name)
    for step in range(1, task.budget.max_steps + 1):
        if time.monotonic() - started > task.budget.max_seconds:
            observation = {"ok": False, "error": "time budget exhausted"}
            break
        steps = step
        try:
            action = agent.next_action(task, observation, step)
        except Exception as exc:
            agent_error = f"{type(exc).__name__}: {exc}"
            trajectory.append("agent_error", step=step, error=agent_error)
            break
        allowed, reason = check_action(task, action)
        trajectory.append("action", step=step, action=asdict(action), allowed=allowed, reason=reason)
        if not allowed:
            violations.append(reason)
            observation = {"ok": False, "error": reason}
            continue
        observation = _execute(task, action)
        trajectory.append("observation", step=step, observation=observation)
        step_changed = changed_files(before, snapshot(task.repository_root))
        for path in check_changed_files(task, step_changed):
            violation = f"changed forbidden path: {path}"
            if violation not in violations:
                violations.append(violation)
        if action.type == "finish":
            agent_finished = True
            break

    after = snapshot(task.repository_root)
    changed = changed_files(before, after)
    for path in check_changed_files(task, changed):
        violation = f"changed forbidden path: {path}"
        if violation not in violations:
            violations.append(violation)
    verification = verify(task)
    score = calculate_score(task, verification, violations, agent_finished, steps, changed)
    status = "passed" if score["resolved"] else "failed"
    summary = {
        "task_id": task.id,
        "agent": agent.name,
        "status": status,
        "steps": steps,
        "duration_ms": round((time.monotonic() - started) * 1000),
        "changed_files": changed,
        "policy_violations": violations,
        "verification": verification,
        "score": score,
        "agent_metrics": agent.metrics(),
        "agent_error": agent_error,
    }
    trajectory.append("run_finished", **summary)
    write_report(run_dir, summary)
    return run_dir
