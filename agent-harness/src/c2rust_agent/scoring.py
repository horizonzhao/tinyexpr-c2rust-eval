from __future__ import annotations

import math
from typing import Any

from .models import Task


def calculate_score(
    task: Task,
    verification: list[dict[str, Any]],
    violations: list[str],
    agent_finished: bool,
    steps: int,
    changed: list[str],
) -> dict[str, Any]:
    total_weight = sum(float(item["spec"]["weight"]) for item in verification)
    passed_weight = sum(
        float(item["spec"]["weight"]) for item in verification if item["passed"]
    )
    correctness = 70.0 * passed_weight / total_weight if total_weight else 0.0
    policy = 10.0 if not violations else 0.0
    completion = 10.0 if agent_finished else 0.0
    efficient_steps = max(3, math.ceil(task.budget.max_steps * 0.25))
    efficiency = 5.0 if steps <= efficient_steps else 3.0 if steps <= task.budget.max_steps / 2 else 1.0
    if violations:
        change_discipline = 0.0
    elif len(changed) <= 3:
        change_discipline = 5.0
    elif len(changed) <= 6:
        change_discipline = 3.0
    else:
        change_discipline = 1.0
    required_passed = all(
        item["passed"] or not item["spec"]["required"] for item in verification
    )
    return {
        "total": round(correctness + policy + completion + efficiency + change_discipline, 2),
        "max": 100.0,
        "resolved": required_passed and not violations and agent_finished,
        "breakdown": {
            "correctness": round(correctness, 2),
            "policy": policy,
            "completion": completion,
            "efficiency": round(efficiency, 2),
            "change_discipline": change_discipline,
        },
        "verification_passed": sum(1 for item in verification if item["passed"]),
        "verification_total": len(verification),
        "changed_file_count": len(changed),
    }
