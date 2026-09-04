# C2Rust Agent Harness

This directory adds a model-agnostic coding-agent evaluation layer to the
existing tinyexpr C-to-Rust experiment. It does not modify the reference C
implementation, the Rust translation, or the original experiment records.

## MVP capabilities

- YAML task manifests with command, path, step, and time budgets.
- Replay and external-command agent adapters.
- Per-step policy enforcement and deterministic verification.
- Workspace snapshots and changed-file detection.
- JSONL trajectories plus machine-readable and Markdown summaries.

## Quick start

From the repository root:

```powershell
python -m pip install -e .\agent-harness
c2rust-agent validate .\agent-harness\tasks\tinyexpr-baseline.yaml
c2rust-agent run .\agent-harness\tasks\tinyexpr-baseline.yaml --agent replay
```

Results are written under `agent-harness/runs/` by default. The included
baseline task is read-only: it runs the existing Rust tests and records the
result as a complete agent trajectory.

An external agent can be integrated with `--agent command`. The configured
process receives one JSON request on stdin and must return one JSON action on
stdout per invocation. See `protocol.md` for the wire format.

