from __future__ import annotations

import shlex
from pathlib import PurePosixPath

from .models import Action, Task
from .workspace import resolve_inside


def _is_under(path: str, prefixes: tuple[str, ...]) -> bool:
    candidate = PurePosixPath(path.replace("\\", "/"))
    return any(candidate == PurePosixPath(p) or PurePosixPath(p) in candidate.parents for p in prefixes)


def check_action(task: Task, action: Action) -> tuple[bool, str]:
    if action.type == "finish":
        return True, "allowed"

    if action.type == "write_file":
        if not action.path:
            return False, "write_file requires path"
        try:
            relative = resolve_inside(task.repository_root, action.path).relative_to(
                task.repository_root
            ).as_posix()
        except ValueError as exc:
            return False, str(exc)
        if _is_under(relative, task.read_only_paths):
            return False, f"path is read-only: {relative}"
        if not _is_under(relative, task.allowed_paths):
            return False, f"path is outside allowed_paths: {relative}"
        return True, "allowed"

    if action.type == "run_command":
        if not action.command:
            return False, "run_command requires command"
        if any(char in action.command for char in (";", "&", "|", ">", "<", "`", "\n", "\r")):
            return False, "shell control characters are not allowed"
        try:
            resolve_inside(task.repository_root, action.cwd)
            tokens = shlex.split(action.command, posix=False)
        except ValueError as exc:
            return False, str(exc)
        normalized_tokens = tuple(token.strip('"\'') for token in tokens)
        prefix_tokens = [
            tuple(shlex.split(prefix, posix=False))
            for prefix in task.allowed_command_prefixes
        ]
        if not any(normalized_tokens[: len(prefix)] == prefix for prefix in prefix_tokens):
            normalized = " ".join(normalized_tokens)
            return False, f"command prefix is not allowed: {normalized}"
        return True, "allowed"

    return False, f"unsupported action type: {action.type}"


def check_changed_files(task: Task, paths: list[str]) -> list[str]:
    violations = []
    for path in paths:
        if _is_under(path, task.read_only_paths) or not _is_under(path, task.allowed_paths):
            violations.append(path)
    return violations
