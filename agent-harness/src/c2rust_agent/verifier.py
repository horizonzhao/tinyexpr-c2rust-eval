from __future__ import annotations

import subprocess
import time
from dataclasses import asdict

from .models import Task, VerifyCommand
from .workspace import resolve_inside


def run_command(task: Task, spec: VerifyCommand) -> dict[str, object]:
    started = time.monotonic()
    completed = subprocess.run(
        spec.command,
        cwd=resolve_inside(task.repository_root, spec.cwd),
        shell=True,
        capture_output=True,
        text=True,
        timeout=spec.timeout_seconds,
        check=False,
    )
    return {
        "spec": asdict(spec),
        "exit_code": completed.returncode,
        "duration_ms": round((time.monotonic() - started) * 1000),
        "stdout": completed.stdout[-12000:],
        "stderr": completed.stderr[-12000:],
        "passed": completed.returncode == 0,
    }


def verify(task: Task) -> list[dict[str, object]]:
    return [run_command(task, spec) for spec in task.verify]

