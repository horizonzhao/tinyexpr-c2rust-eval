from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from ..models import Action, Task
from .base import AgentAdapter


class ReplayAgent(AgentAdapter):
    name = "replay"

    def __init__(self, actions_path: Path) -> None:
        raw = json.loads(actions_path.read_text(encoding="utf-8"))
        if not isinstance(raw, list):
            raise ValueError("replay file must contain a JSON array")
        self.actions = [Action.from_dict(item) for item in raw]

    def next_action(
        self, task: Task, observation: dict[str, Any], step: int
    ) -> Action:
        if step <= len(self.actions):
            return self.actions[step - 1]
        return Action(type="finish", message="Replay exhausted.")

