# shellcheck shell=bash
# #684's visual check: the Marley agent as Claude Code's own interface in a terminal tab. A fake
# `claude` (MARLEY_CLAUDE) writes the arguments it got to a file and prints them. With
# `marley.assistant.enabled` off, the command palette lists no "open marley agent in terminal"
# (`hidden`, REQ-001); on, the command opens a terminal running the fake with the instructions
# file and the five tools turned off (`terminal`, REQ-002), and the file holds the Marley agent's
# instructions (REQ-003).
compositor sway

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # A `claude` that gives a version for Marley's check, says it is signed out (so no offer comes),
  # and otherwise records and prints its arguments.
  cat >"$bin/claude" <<SH
#!/bin/sh
case "\$1 \$2" in
  "--version ") printf '2.1.293 (Claude Code)\n' ; exit 0 ;;
  "auth status") printf '{"loggedIn": false}\n' ; exit 0 ;;
esac
printf '%s\n' "\$@" >"$E2E_WORK/claude-args.txt"
echo "fake claude got: \$*"
sleep 600
SH
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  # Off to start with, and decided, so no offer comes.
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
settings.setdefault("marley", {})["assistant"] = {"enabled": False}
json.dump(settings, open(path, "w"), indent=2)
PY
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 2
}

# Sets `marley.assistant.enabled` in the copy's settings.
assistant() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, re, sys
path, value = sys.argv[1], sys.argv[2] == "true"
text = open(path).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text)
settings.setdefault("marley", {}).setdefault("assistant", {})["enabled"] = value
json.dump(settings, open(path, "w"), indent=2)
PY
}

# Whether the fake's arguments carry the instructions file and the five tools, and the file the
# Marley agent's instructions.
arguments_are_right() {
  python3 - "$E2E_WORK/claude-args.txt" <<'PY'
import sys
args = open(sys.argv[1]).read().split("\n")
print("  arguments:", args[:12])
at = args.index("--append-system-prompt-file") if "--append-system-prompt-file" in args else -1
path = args[at + 1] if at >= 0 else ""
tools = args[args.index("--disallowedTools") + 1:] if "--disallowedTools" in args else []
text = open(path).read() if path else ""
print("  instructions file:", path, "holds the role:", "You are Marley's own agent" in text)
ok = (at >= 0 and "You are Marley's own agent" in text
      and {"Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"} <= set(tools))
sys.exit(0 if ok else 1)
PY
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== off: the palette has no such command"
  palette "open marley agent"
  shot 684-01-hidden
  press "" Escape
  settle 1

  echo "== on: the command opens the agent in a terminal"
  assistant true
  settle 3
  palette "open marley agent"
  press "" Return
  settle 6
  shot 684-02-terminal
  expect "the fake claude ran with the instructions and the tools off" arguments_are_right
}
