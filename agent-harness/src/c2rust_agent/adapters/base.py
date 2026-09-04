from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any

from ..models import Action, Task


class AgentAdapter(ABC):
    name = "unknown"

    @abstractmethod
    def next_action(
        self, task: Task, observation: dict[str, Any], step: int
    ) -> Action:
        raise NotImplementedError

