# External Agent Protocol

The command adapter starts the configured executable once per agent step. The
request is a JSON object containing the task, current observation, step number,
and remaining budget. The process must print exactly one JSON object to stdout.

Supported actions:

```json
{"type":"run_command","command":"cargo test --all","cwd":"tinyexpr-rs"}
{"type":"write_file","path":"tinyexpr-rs/src/example.rs","content":"..."}
{"type":"finish","message":"Implementation and verification complete."}
```

`cwd` and `path` are repository-relative. The task policy decides which paths
and command prefixes are allowed. A rejected action is recorded and returned
to the agent as the next observation.

