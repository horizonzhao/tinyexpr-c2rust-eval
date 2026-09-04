from __future__ import annotations

import subprocess
import time
from dataclasses import asdict

from .models import Task, VerifyCommand
from .workspace import resolve_inside


def run_command(task: Task, spec: VerifyCommand) -> dict[str, object]:
    started = time.monotonic()
    try:
        completed = subprocess.run(
            spec.command,
            cwd=resolve_inside(task.repository_root, spec.cwd),
            shell=True,
            capture_output=True,
            text=True,
            timeout=spec.timeout_seconds,
            check=False,
        )
        exit_code: int | None = completed.returncode
        stdout = completed.stdout[-12000:]
        stderr = completed.stderr[-12000:]
        failure = None if completed.returncode == 0 else "nonzero_exit"
    except subprocess.TimeoutExpired as exc:
        exit_code = None
        stdout = (exc.stdout or "")[-12000:]
        stderr = (exc.stderr or "")[-12000:]
        failure = "timeout"
    except OSError as exc:
        exit_code = None
        stdout = ""
        stderr = str(exc)
        failure = "execution_error"
    return {
        "spec": asdict(spec),
        "exit_code": exit_code,
        "duration_ms": round((time.monotonic() - started) * 1000),
        "stdout": stdout,
        "stderr": stderr,
        "passed": exit_code == 0,
        "failure": failure,
    }


def verify(task: Task) -> list[dict[str, object]]:
    return [run_command(task, spec) for spec in task.verify]
