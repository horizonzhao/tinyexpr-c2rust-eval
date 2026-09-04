from __future__ import annotations

import argparse
import json
import shlex
from pathlib import Path

from .adapters import CommandAgent, DeepSeekAgent, ReplayAgent
from .leaderboard import write_leaderboard
from .runner import run
from .task_loader import load_task


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="c2rust-agent")
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    validate = subparsers.add_parser("validate", help="validate a task manifest")
    validate.add_argument("task", type=Path)

    run_parser = subparsers.add_parser("run", help="run an agent task")
    run_parser.add_argument("task", type=Path)
    run_parser.add_argument(
        "--agent", choices=("replay", "command", "deepseek"), default="replay"
    )
    run_parser.add_argument("--replay", type=Path)
    run_parser.add_argument("--command")
    run_parser.add_argument("--model", default="deepseek-v4-flash")
    run_parser.add_argument("--thinking", choices=("enabled", "disabled"), default="enabled")
    run_parser.add_argument("--reasoning-effort", choices=("low", "medium", "high"), default="high")
    run_parser.add_argument("--max-tokens", type=int, default=8192)
    run_parser.add_argument("--runs-dir", type=Path)

    scoreboard = subparsers.add_parser("scoreboard", help="aggregate run scores")
    scoreboard.add_argument("runs_dir", type=Path)
    scoreboard.add_argument("--output", type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.subcommand == "scoreboard":
        output = args.output or args.runs_dir
        board = write_leaderboard(args.runs_dir.resolve(), output.resolve())
        print(f"runs: {board['total_runs']}")
        print(f"leaderboard: {output.resolve() / 'leaderboard.md'}")
        return 0

    task = load_task(args.task.resolve())
    if args.subcommand == "validate":
        print(json.dumps({"valid": True, "task_id": task.id}, indent=2))
        return 0

    harness_root = Path(__file__).resolve().parents[2]
    if args.agent == "replay":
        replay_path = args.replay or harness_root / "replays" / f"{task.id}.json"
        agent = ReplayAgent(replay_path)
    elif args.agent == "command":
        if not args.command:
            raise SystemExit("--command is required for command agent")
        agent = CommandAgent(shlex.split(args.command, posix=False))
    else:
        agent = DeepSeekAgent(
            model=args.model,
            thinking=args.thinking,
            reasoning_effort=args.reasoning_effort,
            max_tokens=args.max_tokens,
        )
    run_dir = run(task, agent, args.runs_dir or harness_root / "runs")
    summary = json.loads((run_dir / "summary.json").read_text(encoding="utf-8"))
    print(f"run: {run_dir}")
    print(f"status: {summary['status']}")
    print(f"score: {summary['score']['total']}/{summary['score']['max']}")
    return 0 if summary["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
