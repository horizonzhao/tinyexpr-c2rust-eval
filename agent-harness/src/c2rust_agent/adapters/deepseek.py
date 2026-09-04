from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request
from typing import Any

from ..models import Action, Task
from .base import AgentAdapter


TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read a UTF-8 text file relative to the task workspace.",
            "parameters": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "list_files",
            "description": "List files below a workspace-relative directory.",
            "parameters": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "search_code",
            "description": "Search text in workspace files.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                },
                "required": ["pattern", "path"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Replace a UTF-8 file inside an allowed writable path.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                },
                "required": ["path", "content"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "run_command",
            "description": "Run one policy-approved command in a workspace directory.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "cwd": {"type": "string"},
                },
                "required": ["command", "cwd"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "finish",
            "description": "Finish after implementing and testing the repair.",
            "parameters": {
                "type": "object",
                "properties": {"message": {"type": "string"}},
                "required": ["message"],
                "additionalProperties": False,
            },
        },
    },
]


class DeepSeekAgent(AgentAdapter):
    name = "deepseek"

    def __init__(
        self,
        model: str = "deepseek-v4-flash",
        thinking: str = "enabled",
        reasoning_effort: str = "high",
        max_tokens: int = 8192,
        timeout_seconds: int = 120,
        max_retries: int = 3,
    ) -> None:
        self.api_key = os.environ.get("DEEPSEEK_API_KEY")
        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY environment variable is required")
        self.base_url = os.environ.get("DEEPSEEK_BASE_URL", "https://api.deepseek.com").rstrip("/")
        self.model = model
        self.thinking = thinking
        self.reasoning_effort = reasoning_effort
        self.max_tokens = max_tokens
        self.timeout_seconds = timeout_seconds
        self.max_retries = max_retries
        self.messages: list[dict[str, Any]] = []
        self.pending_tool_call_id: str | None = None
        self.queued_tool_calls: list[dict[str, Any]] = []
        self._metrics = {
            "provider": "deepseek",
            "model": model,
            "api_requests": 0,
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0,
            "cache_hit_tokens": 0,
            "cache_miss_tokens": 0,
        }

    def _request(self, payload: dict[str, Any]) -> dict[str, Any]:
        request = urllib.request.Request(
            f"{self.base_url}/chat/completions",
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        for attempt in range(self.max_retries):
            try:
                self._metrics["api_requests"] += 1
                with urllib.request.urlopen(request, timeout=self.timeout_seconds) as response:
                    return json.loads(response.read().decode("utf-8"))
            except urllib.error.HTTPError as exc:
                body = exc.read().decode("utf-8", errors="replace")
                if exc.code not in {429, 500, 502, 503, 504} or attempt + 1 == self.max_retries:
                    raise RuntimeError(f"DeepSeek API HTTP {exc.code}: {body}") from exc
            except urllib.error.URLError as exc:
                if attempt + 1 == self.max_retries:
                    raise RuntimeError(f"DeepSeek API connection failed: {exc}") from exc
            time.sleep(2**attempt)
        raise RuntimeError("DeepSeek API request failed")

    def next_action(self, task: Task, observation: dict[str, Any], step: int) -> Action:
        if not self.messages:
            self.messages = [
                {
                    "role": "system",
                    "content": (
                        "You are a coding agent. Inspect the workspace, make the smallest correct repair, "
                        "run the declared checks, and call finish only when done. Call exactly one tool per turn. "
                        "Never modify read-only paths."
                    ),
                },
                {
                    "role": "user",
                    "content": json.dumps(
                        {
                            "task_id": task.id,
                            "problem": task.problem,
                            "allowed_paths": task.allowed_paths,
                            "read_only_paths": task.read_only_paths,
                            "allowed_commands": task.allowed_command_prefixes,
                        },
                        ensure_ascii=False,
                    ),
                },
            ]
        elif self.pending_tool_call_id:
            self.messages.append(
                {
                    "role": "tool",
                    "tool_call_id": self.pending_tool_call_id,
                    "content": json.dumps(observation, ensure_ascii=False),
                }
            )
            self.pending_tool_call_id = None

        if self.queued_tool_calls:
            return self._action_from_tool_call(self.queued_tool_calls.pop(0))

        payload = {
            "model": self.model,
            "messages": self.messages,
            "tools": TOOLS,
            "thinking": {"type": self.thinking},
            "reasoning_effort": self.reasoning_effort,
            "max_tokens": self.max_tokens,
        }
        if self.thinking == "disabled":
            payload["tool_choice"] = "required"
        response = self._request(payload)
        usage = response.get("usage", {})
        for key in ("prompt_tokens", "completion_tokens", "total_tokens"):
            self._metrics[key] += int(usage.get(key, 0) or 0)
        details = usage.get("prompt_tokens_details", {}) or {}
        self._metrics["cache_hit_tokens"] += int(details.get("cached_tokens", 0) or 0)
        self._metrics["cache_miss_tokens"] += int(details.get("cache_miss_tokens", 0) or 0)

        message = response["choices"][0]["message"]
        assistant_message = {
            "role": "assistant",
            "content": message.get("content") or "",
            "tool_calls": message.get("tool_calls", []),
        }
        if message.get("reasoning_content") is not None:
            assistant_message["reasoning_content"] = message["reasoning_content"]
        self.messages.append(assistant_message)
        tool_calls = message.get("tool_calls") or []
        if not tool_calls:
            return Action(type="finish", message=message.get("content") or "Model stopped without a tool call.")
        self.queued_tool_calls.extend(tool_calls)
        return self._action_from_tool_call(self.queued_tool_calls.pop(0))

    def _action_from_tool_call(self, tool_call: dict[str, Any]) -> Action:
        self.pending_tool_call_id = tool_call["id"]
        arguments = json.loads(tool_call["function"]["arguments"])
        arguments["type"] = tool_call["function"]["name"]
        return Action.from_dict(arguments)

    def metrics(self) -> dict[str, Any]:
        return dict(self._metrics)
