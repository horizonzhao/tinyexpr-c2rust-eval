from __future__ import annotations

import json
import subprocess
from typing import Any

from ..models import Action, Task
from .base import AgentAdapter


class CommandAgent(AgentAdapter):
    name = "command"

    def __init__(self, command: list[str], timeout_seconds: int = 300) -> None:
        if not command:
            raise ValueError("external agent command cannot be empty")
        self.command = command
        self.timeout_seconds = timeout_seconds

    def next_action(
        self, task: Task, observation: dict[str, Any], step: int
    ) -> Action:
        request = {
            "task": {
                "id": task.id,
                "title": task.title,
                "problem": task.problem,
                "repository_root": str(task.repository_root),
                "allowed_paths": task.allowed_paths,
                "read_only_paths": task.read_only_paths,
            },
            "observation": observation,
            "step": step,
            "remaining_steps": task.budget.max_steps - step + 1,
        }
        completed = subprocess.run(
            self.command,
            cwd=task.repository_root,
            input=json.dumps(request),
            capture_output=True,
            text=True,
            timeout=self.timeout_seconds,
            check=False,
        )
        if completed.returncode != 0:
            return Action(
                type="finish",
                message=f"External agent failed ({completed.returncode}): {completed.stderr.strip()}",
            )
        return Action.from_dict(json.loads(completed.stdout))
