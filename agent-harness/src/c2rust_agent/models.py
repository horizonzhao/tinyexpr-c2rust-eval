from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class VerifyCommand:
    name: str
    command: str
    cwd: str = "."
    timeout_seconds: int = 300
    weight: float = 1.0
    required: bool = True


@dataclass(frozen=True)
class Budget:
    max_steps: int = 30
    max_seconds: int = 1800


@dataclass(frozen=True)
class Task:
    id: str
    title: str
    problem: str
    repository_root: Path
    allowed_paths: tuple[str, ...]
    read_only_paths: tuple[str, ...]
    allowed_command_prefixes: tuple[str, ...]
    verify: tuple[VerifyCommand, ...]
    budget: Budget
    workspace_mode: str = "copy"
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class Action:
    type: str
    command: str | None = None
    cwd: str = "."
    path: str | None = None
    content: str | None = None
    pattern: str | None = None
    message: str = ""

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> "Action":
        return cls(
            type=str(value.get("type", "")),
            command=value.get("command"),
            cwd=str(value.get("cwd", ".")),
            path=value.get("path"),
            content=value.get("content"),
            pattern=value.get("pattern"),
            message=str(value.get("message", "")),
        )
