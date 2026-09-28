#!/usr/bin/env python3
# Marley's event hook for Claude Code (#519). Claude Code runs it at each hook event with the
# event's JSON on stdin. In a Marley terminal it answers with a `terminalSequence`: an OSC 777
# notify titled `marley-event` whose body is the base64 of a short summary of the event, which
# Marley folds into the terminal's rail row. Anywhere else, and on any failure, it answers `{}`,
# which asks Claude Code for nothing.
import base64
import json
import os
import re
import sys

TITLE = "marley-event"
# Claude Code refuses a sequence over 4,096 bytes; Marley's scanner reads 4 KiB. The summary's
# JSON stays under this, so its base64 and the escape around it fit both.
MAX_SUMMARY = 2900
MAX_PROMPT = 300
MAX_MESSAGE = 300
MAX_PREVIEW = 200
# An AskUserQuestion's options, which the inbox's question route reads (#570).
MAX_OPTIONS = 8
MAX_OPTION = 40
# The input field that says what a tool acts on, by tool.
PREVIEW_KEYS = {
    "Read": "file_path",
    "Write": "file_path",
    "Edit": "file_path",
    "MultiEdit": "file_path",
    "NotebookEdit": "notebook_path",
    "Bash": "command",
    "Glob": "pattern",
    "Grep": "pattern",
    "WebFetch": "url",
    "WebSearch": "query",
    "Task": "description",
}
COMMON = ("session_id", "prompt_id", "agent_id", "permission_mode", "cwd", "transcript_path")
# Where a sentence ends, once runs of whitespace are single spaces.
SENTENCE_END = re.compile(r"[.!?] ")


def cut(text, limit):
    text = " ".join(str(text).split())
    return text if len(text) <= limit else text[: limit - 1] + "…"


# A final message's question or status sits at its end, so a long one keeps its last whole
# sentences after its start (#566). Marley redacts after this cut, so the end starts at a
# sentence and the start stops at a word: neither cut parts a secret from the name that marks it.
def cut_ends(text, limit):
    text = " ".join(str(text).split())
    if len(text) <= limit:
        return text
    room = limit // 2 - 3
    starts = [match.end() for match in SENTENCE_END.finditer(text) if len(text) - match.end() <= room]
    if not starts:
        return cut(text, limit)
    tail = text[starts[0]:]
    head = text[: limit - 3 - len(tail)]
    if not text[len(head)].isspace() and " " in head:
        head = head.rsplit(" ", 1)[0]
    return head.rstrip() + " … " + tail


def preview(tool, tool_input):
    if not isinstance(tool_input, dict):
        return None
    if tool == "AskUserQuestion":
        questions = tool_input.get("questions")
        if isinstance(questions, list) and questions and isinstance(questions[0], dict):
            question = questions[0].get("question")
            if isinstance(question, str):
                return cut(question, MAX_PREVIEW)
    key = PREVIEW_KEYS.get(tool)
    value = tool_input.get(key) if key else None
    if not isinstance(value, str):
        value = next((item for item in tool_input.values() if isinstance(item, str)), None)
    return cut(value, MAX_PREVIEW) if value else None


def options(tool_input):
    if not isinstance(tool_input, dict):
        return None
    questions = tool_input.get("questions")
    if not (isinstance(questions, list) and questions and isinstance(questions[0], dict)):
        return None
    listed = questions[0].get("options")
    if not isinstance(listed, list):
        return None
    labels = [cut(option["label"], MAX_OPTION) for option in listed
              if isinstance(option, dict) and isinstance(option.get("label"), str)]
    return labels[:MAX_OPTIONS] or None


def summary(event):
    name = event.get("hook_event_name")
    out = {"v": 1, "event": name}
    for key in COMMON:
        if isinstance(event.get(key), str):
            out[key] = event[key]
    if name == "UserPromptSubmit" and isinstance(event.get("prompt"), str):
        out["prompt"] = cut(event["prompt"], MAX_PROMPT)
    if name in ("PreToolUse", "PostToolUse", "PostToolUseFailure", "PermissionRequest"):
        tool = event.get("tool_name")
        if isinstance(tool, str):
            out["tool"] = tool
            shown = preview(tool, event.get("tool_input"))
            if shown:
                out["preview"] = shown
            if tool == "AskUserQuestion":
                labels = options(event.get("tool_input"))
                if labels:
                    out["options"] = labels
        if isinstance(event.get("tool_use_id"), str):
            out["tool_use_id"] = event["tool_use_id"]
    if name == "PostToolUseFailure" and event.get("is_interrupt") is True:
        out["is_interrupt"] = True
    if name in ("Stop", "StopFailure", "SubagentStop"):
        message = event.get("last_assistant_message")
        if isinstance(message, str):
            out["message"] = cut_ends(message, MAX_MESSAGE)
    if name == "StopFailure" and isinstance(event.get("error"), str):
        out["error"] = cut(event["error"], 80)
    for key in ("source", "trigger", "reason"):
        if isinstance(event.get(key), str):
            out[key] = event[key]
    # Bound the frame. Paths are short but have no bound, so an overlong one goes first; then a
    # question's options, which no row shows; then the fields the row shows, longest first.
    for key in ("transcript_path", "cwd", "options", "message", "preview", "prompt"):
        if len(json.dumps(out, separators=(",", ":"))) <= MAX_SUMMARY:
            break
        out.pop(key, None)
    return out


def main():
    if os.environ.get("TERM_PROGRAM") != "zed":
        print("{}")
        return
    event = json.loads(sys.stdin.read(1 << 20))
    if not isinstance(event, dict):
        print("{}")
        return
    body = json.dumps(summary(event), separators=(",", ":"))
    if len(body) > MAX_SUMMARY:
        print("{}")
        return
    encoded = base64.b64encode(body.encode()).decode()
    print(json.dumps({"terminalSequence": f"\x1b]777;notify;{TITLE};{encoded}\x07"}))


try:
    main()
except Exception:
    print("{}")
