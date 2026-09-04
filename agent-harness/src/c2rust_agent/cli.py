from __future__ import annotations

import argparse
import json
import shlex
from pathlib import Path

from .adapters import CommandAgent, ReplayAgent
from .runner import run
from .task_loader import load_task


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="c2rust-agent")
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    validate = subparsers.add_parser("validate", help="validate a task manifest")
    validate.add_argument("task", type=Path)

    run_parser = subparsers.add_parser("run", help="run an agent task")
    run_parser.add_argument("task", type=Path)
    run_parser.add_argument("--agent", choices=("replay", "command"), default="replay")
    run_parser.add_argument("--replay", type=Path)
    run_parser.add_argument("--command")
    run_parser.add_argument("--runs-dir", type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    task = load_task(args.task.resolve())
    if args.subcommand == "validate":
        print(json.dumps({"valid": True, "task_id": task.id}, indent=2))
        return 0

    harness_root = Path(__file__).resolve().parents[2]
    if args.agent == "replay":
        replay_path = args.replay or harness_root / "replays" / f"{task.id}.json"
        agent = ReplayAgent(replay_path)
    else:
        if not args.command:
            raise SystemExit("--command is required for command agent")
        agent = CommandAgent(shlex.split(args.command, posix=False))
    run_dir = run(task, agent, args.runs_dir or harness_root / "runs")
    summary = json.loads((run_dir / "summary.json").read_text(encoding="utf-8"))
    print(f"run: {run_dir}")
    print(f"status: {summary['status']}")
    return 0 if summary["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())

