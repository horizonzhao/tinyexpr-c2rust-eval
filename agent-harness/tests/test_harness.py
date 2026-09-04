from pathlib import Path

import pytest

from c2rust_agent.models import Action
from c2rust_agent.adapters.deepseek import DeepSeekAgent
from c2rust_agent.leaderboard import build_leaderboard
from c2rust_agent.policies import check_action
from c2rust_agent.scoring import calculate_score
from c2rust_agent.task_loader import load_task
from c2rust_agent.workspace import changed_files, resolve_inside, snapshot


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


def test_snapshot_does_not_ignore_a_workspace_because_parent_is_named_runs(
    tmp_path: Path,
) -> None:
    workspace = tmp_path / "runs" / "task" / "workspace"
    workspace.mkdir(parents=True)
    (workspace / "source.rs").write_text("fn main() {}", encoding="utf-8")
    assert "source.rs" in snapshot(workspace)


def test_repair_task_set_contains_eight_tasks() -> None:
    repair_tasks = sorted((ROOT / "agent-harness" / "tasks").glob("repair-*.yaml"))
    assert len(repair_tasks) == 8
    assert all(load_task(path).workspace_mode == "copy" for path in repair_tasks)


def test_scoring_returns_a_weighted_100_point_result() -> None:
    task = load_task(TASK_PATH)
    verification = [
        {
            "passed": True,
            "spec": {"name": "tests", "weight": 3.0, "required": True},
        },
        {
            "passed": False,
            "spec": {"name": "lint", "weight": 1.0, "required": False},
        },
    ]
    score = calculate_score(task, verification, [], True, 1, [])
    assert score["total"] == 82.5
    assert score["resolved"]
    assert score["breakdown"]["correctness"] == 52.5


def test_leaderboard_aggregates_agent_results(tmp_path: Path) -> None:
    run_dir = tmp_path / "run-1"
    run_dir.mkdir()
    (run_dir / "summary.json").write_text(
        '{"agent":"demo","steps":2,"duration_ms":100,"score":{"total":90,"resolved":true}}',
        encoding="utf-8",
    )
    board = build_leaderboard(tmp_path)
    assert board["total_runs"] == 1
    assert board["agents"][0]["resolve_rate"] == 100.0
    assert board["agents"][0]["average_score"] == 90.0


def test_deepseek_adapter_converts_tool_call_to_action(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("DEEPSEEK_API_KEY", "test-key")
    agent = DeepSeekAgent()
    captured = {}

    def fake_request(payload):
        captured.update(payload)
        return {
        "choices": [
            {
                "message": {
                    "content": None,
                    "tool_calls": [
                        {
                            "id": "call-1",
                            "type": "function",
                            "function": {
                                "name": "read_file",
                                "arguments": '{"path":"src/lib.rs"}',
                            },
                        }
                    ],
                }
            }
        ],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
        }

    agent._request = fake_request
    task = load_task(ROOT / "agent-harness" / "tasks" / "repair-error-position.yaml")
    action = agent.next_action(task, {"type": "start"}, 1)
    assert action.type == "read_file"
    assert action.path == "src/lib.rs"
    assert agent.metrics()["total_tokens"] == 15
    assert "tool_choice" not in captured
    assert all("strict" not in tool["function"] for tool in captured["tools"])


def test_deepseek_nonthinking_mode_requires_a_tool(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("DEEPSEEK_API_KEY", "test-key")
    agent = DeepSeekAgent(thinking="disabled")
    captured = {}

    def fake_request(payload):
        captured.update(payload)
        return {
            "choices": [{"message": {"content": "done", "tool_calls": []}}],
            "usage": {},
        }

    agent._request = fake_request
    task = load_task(ROOT / "agent-harness" / "tasks" / "repair-error-position.yaml")
    agent.next_action(task, {"type": "start"}, 1)
    assert captured["tool_choice"] == "required"
