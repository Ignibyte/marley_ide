# shellcheck shell=bash
# #547's e2e test: #519's second slice. Claude Code's list (a scratch CLAUDE_CONFIG_DIR) has
# Marley's plugin at 1.1.0, so the agent bar offers the update; a click runs the stand-in
# `claude`'s `plugin` commands, which log themselves and write the version Marley ships into the
# list. The stand-in
# then acts out a turn through the plugin's real hook, and the stand-in agent reads
# `fleet_snapshot` as it goes: the terminal's seat working, then idle, then done once the
# stand-in exits, then gone once the terminal closes. With `marley.no_update_after_minutes` at 1,
# a working row with no event for 70 seconds reads `no update in 1 m`.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
# The plugin's version as Marley ships it, which the update writes into Claude Code's list.
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The agent bar's plugin chip, measured from the first runs (519's shots put its left end at 452).
CHIP_X=${CHIP_X:-520}
CHIP_Y=${CHIP_Y:-953}

# Gives the profile's `marley` settings block the key, or adds the block.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import sys
path, key, value = sys.argv[1:]
text = open(path).read()
entry = f'"{key}": {value},'
if '"marley": {' in text:
    text = text.replace('"marley": {', '"marley": {\n    ' + entry, 1)
else:
    lines = text.split("\n")
    at = next(i for i, line in enumerate(lines) if line.startswith("{"))
    lines.insert(at + 1, '  "marley": {' + entry + '},')
    text = "\n".join(lines)
open(path, "w").write(text)
PY
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  mkdir -p "$home" "$bin" "$config/plugins"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/1.1.0", "version": "1.1.0"}]}}
JSON
  cat >"$config/plugins/known_marketplaces.json" <<JSON
{"marley": {"source": {"source": "directory", "path": "$E2E_PROFILE/claude-code"}, "installLocation": "$E2E_PROFILE/claude-code", "lastUpdated": "2026-09-25T00:00:00.000Z"}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  cat >"$E2E_WORK/steps.json" <<'JSON'
[
  {"label": "a prompt, and a test run", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Run the test suite"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "cargo test"}, "tool_use_id": "t1"}
  ]},
  {"label": "the turn ends", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": {"command": "cargo test"}, "tool_use_id": "t1"},
    {"hook_event_name": "Stop", "last_assistant_message": "All 12 tests pass."}
  ]}
]
JSON
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@LOG@|$E2E_WORK/plugin.log|" \
    -e "s|@SHIPPED@|$SHIPPED|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code. `claude plugin ...` logs its arguments; `plugin update marley@marley`
# also writes the shipped version into Claude Code's list, as the real update does. Otherwise, at
# each line it reads, it runs the plugin's hook with the next step's payloads, as Claude Code runs
# a hook, and writes each answer's sequence to its terminal.
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write(" ".join(sys.argv[1:]) + "\n")
    if sys.argv[1:] == ["plugin", "update", "marley@marley"]:
        path = os.path.join(os.environ["CLAUDE_CONFIG_DIR"], "plugins/installed_plugins.json")
        listed = json.load(open(path, encoding="utf-8"))
        listed["plugins"]["marley@marley"][0]["version"] = "@SHIPPED@"
        json.dump(listed, open(path, "w", encoding="utf-8"))
    sys.exit(0)

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"session_id": "e2e-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}

print("Claude Code (stand-in): press Enter for each step", flush=True)
for number, step in enumerate(STEPS, 1):
    if not sys.stdin.readline():
        break
    for event in step["events"]:
        answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps({**COMMON, **event}),
                                capture_output=True, text=True, check=False).stdout
        sequence = json.loads(answer or "{}").get("terminalSequence")
        if sequence:
            sys.stdout.write(sequence)
            sys.stdout.flush()
    print(f"step {number}: {step['label']}", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude"
  # The `claude` Marley runs for the plugin's commands. Its PATH may come from the login shell,
  # which puts the real one first, so the stand-in is named outright.
  export MARLEY_CLAUDE=$bin/claude
  marley_setting no_update_after_minutes 1
  git init -q -b events "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

# Whether the plugin log has the marketplace refresh before the plugin's update.
updated_in_order() {
  cat "$E2E_WORK/plugin.log"
  [[ $(head -1 "$E2E_WORK/plugin.log") == "plugin marketplace update marley" ]] &&
    [[ $(sed -n 2p "$E2E_WORK/plugin.log") == "plugin update marley@marley" ]]
}

# The stand-in agent's reading of `fleet_snapshot`, kept as $E2E_WORK/<label>.txt.
fleet() {
  mcp_agent fleet | tee "$E2E_WORK/$1.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  settle 3
  shot 547-01-update-chip
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  shot 547-02-updated
  expect "the update refreshed the marketplace, then updated the plugin" updated_in_order
  press "" Return
  settle 2
  fleet working
  expect "fleet_snapshot lists the working seat" \
    holds "$E2E_WORK/working.txt" ": working, prompt 'Run the test suite', tool 'Bash: cargo test'"
  # A minute without an event, and the minute timer's refresh.
  settle 70
  shot 547-03-no-update
  press "" Return
  settle 2
  fleet idle
  expect "fleet_snapshot follows the seat to idle" \
    holds "$E2E_WORK/idle.txt" ": idle, prompt 'Run the test suite', message 'All 12 tests pass.'"
  # End of input: the stand-in exits and the shell has the terminal again.
  press CTRL d
  settle 3
  fleet exited
  expect "the seat is done once Claude Code has left" holds "$E2E_WORK/exited.txt" ": done"
  type_text "exit"
  press "" Return
  settle 3
  fleet closed
  expect "a closed terminal's seat is gone" holds "$E2E_WORK/closed.txt" "no seats"
}
