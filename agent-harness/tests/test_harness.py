from pathlib import Path

import pytest

from c2rust_agent.models import Action
from c2rust_agent.policies import check_action
from c2rust_agent.task_loader import load_task
from c2rust_agent.workspace import changed_files, resolve_inside


ROOT = Path(__file__).resolve().parents[2]
TASK_PATH = ROOT / "agent-harness" / "tasks" / "tinyexpr-baseline.yaml"


def test_load_baseline_task() -> None:
    task = load_task(TASK_PATH)
    assert task.id == "tinyexpr-baseline"
    assert task.repository_root == ROOT
    assert task.verify[0].cwd == "tinyexpr-rs"


def test_policy_allows_declared_command() -> None:
    task = load_task(TASK_PATH)
    allowed, _ = check_action(
        task,
        Action(type="run_command", command="cargo test --all", cwd="tinyexpr-rs"),
    )
    assert allowed


def test_policy_rejects_lookalike_command_prefix() -> None:
    task = load_task(TASK_PATH)
    allowed, _ = check_action(
        task,
        Action(type="run_command", command="cargo testevil", cwd="tinyexpr-rs"),
    )
    assert not allowed


def test_policy_rejects_shell_command_chaining() -> None:
    task = load_task(TASK_PATH)
    allowed, reason = check_action(
        task,
        Action(type="run_command", command="cargo test; echo bypass", cwd="tinyexpr-rs"),
    )
    assert not allowed
    assert "control characters" in reason


def test_policy_rejects_write_in_read_only_tree() -> None:
    task = load_task(TASK_PATH)
    allowed, reason = check_action(
        task,
        Action(type="write_file", path="tinyexpr-rs/src/lib.rs", content="bad"),
    )
    assert not allowed
    assert "read-only" in reason


def test_resolve_inside_rejects_escape() -> None:
    with pytest.raises(ValueError):
        resolve_inside(ROOT, "../outside")


def test_changed_files_detects_add_remove_and_modify() -> None:
    assert changed_files({"a": "1", "b": "2"}, {"a": "9", "c": "3"}) == [
        "a",
        "b",
        "c",
    ]
