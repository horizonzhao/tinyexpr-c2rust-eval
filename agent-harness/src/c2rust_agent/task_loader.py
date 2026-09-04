from __future__ import annotations

from pathlib import Path

import yaml

from .models import Budget, Task, VerifyCommand


def load_task(path: Path) -> Task:
    raw = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError("task manifest must contain a YAML mapping")

    required = ("id", "title", "problem", "repository_root", "verify")
    missing = [key for key in required if key not in raw]
    if missing:
        raise ValueError(f"task manifest is missing: {', '.join(missing)}")

    policy = raw.get("policy", {})
    budget_raw = raw.get("budget", {})
    verify = tuple(
        VerifyCommand(
            command=str(item["command"]),
            cwd=str(item.get("cwd", ".")),
            timeout_seconds=int(item.get("timeout_seconds", 300)),
        )
        for item in raw["verify"]
    )
    repository_root = (path.parent / str(raw["repository_root"])).resolve()

    return Task(
        id=str(raw["id"]),
        title=str(raw["title"]),
        problem=str(raw["problem"]),
        repository_root=repository_root,
        allowed_paths=tuple(policy.get("allowed_paths", [])),
        read_only_paths=tuple(policy.get("read_only_paths", [])),
        allowed_command_prefixes=tuple(policy.get("allowed_command_prefixes", [])),
        verify=verify,
        budget=Budget(
            max_steps=int(budget_raw.get("max_steps", 30)),
            max_seconds=int(budget_raw.get("max_seconds", 1800)),
        ),
        metadata=dict(raw.get("metadata", {})),
    )

