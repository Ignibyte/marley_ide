# shellcheck shell=bash
# #555's visual check: a terminal's block sent to a CLI agent in another terminal. Stand-ins named
# `claude` and `codex`, first on the terminal's PATH, print each line they read (`got: <line>`,
# also to a log); the `claude` one, on reading `wait`, runs the plugin's `event.py` with a prompt
# and a PermissionRequest, so its seat waits (#549's stand-ins). Checks: a long failed block shows
# Ask the agent on its last row while another terminal runs an agent, and its click types a
# reference to `terminal_read` at the agent's prompt (REQ-001, REQ-004); `ctrl-shift-enter` on a
# selected short block types its Markdown (REQ-002, REQ-008); a token in a block reaches the agent
# redacted (REQ-003); an agent that waits gets nothing (REQ-006); two agents open the picker,
# without the block's own terminal (REQ-007); with no agent, no chip (REQ-005).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py

# The terminal area, and the chip on a long block's last row, from the shots. The terminals are
# brought forward by their tabs (alt-1 to alt-3): the rail's rows move when Needs you shows.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
CHIP_X=${CHIP_X:-1290}
CHIP_Y=${CHIP_Y:-935}
BLOCK_X=${BLOCK_X:-420}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  for name in claude codex; do
    sed -e "s|@NAME@|$name|" -e "s|@LOG@|$E2E_WORK/$name.log|" \
      -e "s|@HOOK@|$([[ $name == claude ]] && echo "$HOOK")|" >"$bin/$name" <<'FAKE'
#!/usr/bin/env python3
# A stand-in agent: each line it reads, printed and logged; on `wait`, a prompt and a permission
# request through the plugin's hook, so its seat waits.
import json
import os
import subprocess
import sys

NAME, LOG, HOOK = "@NAME@", "@LOG@", "@HOOK@"
COMMON = {"session_id": f"e2e-{os.getpid()}", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}
print(f"{NAME} (stand-in): type a line", flush=True)
for line in sys.stdin:
    line = line.rstrip("\n")
    with open(LOG, "a", encoding="utf-8") as log:
        log.write(f"got: {line}\n")
    print(f"got: {line}", flush=True)
    if line == "wait" and HOOK:
        for event in ({"hook_event_name": "UserPromptSubmit", "prompt": "Clean the build"},
                      {"hook_event_name": "PermissionRequest", "tool_name": "Bash",
                       "tool_input": {"command": "rm -rf build"}}):
            answer = subprocess.run([sys.executable, HOOK], input=json.dumps({**COMMON, **event}),
                                    capture_output=True, text=True, check=False).stdout
            sequence = json.loads(answer or "{}").get("terminalSequence")
            if sequence:
                sys.stdout.write(sequence)
                sys.stdout.flush()
FAKE
    chmod +x "$bin/$name"
    : >"$E2E_WORK/$name.log"
  done
  git init -q -b main "$repo"
  zed_agent_off
  open_path "$repo"
}

# Turns Zed's own agent off in the run's copy of the settings, so no step can start a thread of the
# user's agent: a menu walked by its keys past its end once chose Add to Agent Thread.
zed_agent_off() {
  python3 - "$E2E_PROFILE/config/settings.json" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("agent", {})["enabled"] = False
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Brings the terminal on tab `$1` forward and gives it the focus.
terminal_at() {
  press ALT "$1"
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
}

# Right-clicks the block row at `$1` and picks Send to Agent, the Block section's first item, six
# above the menu's last.
send_from_menu() {
  local steps
  click "$BLOCK_X" "$1" right
  settle 1
  press "" End
  for ((steps = 0; steps < 6; steps++)); do
    press "" Up
  done
  press "" Return
  settle 2
}

# A GitHub token of the kind the redactor knows, put together here so no file holds one.
fake_token() {
  printf 'gh%s_%s' p "$(printf 'x%.0s' {1..36})"
}

steps() {
  local lines
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  settle 2
  palette "workspace: new terminal"
  settle 3

  echo "== a long failed block offers Ask the agent, which sends a reference"
  type_text "seq 1 200; false"
  press "" Return
  settle 2
  shot 555-01-chip
  click "$CHIP_X" "$CHIP_Y"
  settle 2
  shot 555-02-reference-sent
  expect "nothing reaches the agent before Return" test ! -s "$E2E_WORK/claude.log"
  press "" Return
  settle 2
  cat "$E2E_WORK/claude.log"
  expect "the agent read the reference" \
    holds "$E2E_WORK/claude.log" "got: [terminal " "seq 1 200; false, exit 1; terminal_read terminal="
  mcp_agent terminals

  echo "== ctrl-shift-enter sends a short selected block inline"
  terminal_at 2
  type_text "false"
  press "" Return
  settle 1
  press "CTRL" Up
  settle 1
  press "CTRL SHIFT" Return
  settle 3
  shot 555-03-inline-sent
  cat "$E2E_WORK/claude.log"
  expect "the agent read the block's Markdown" \
    holds "$E2E_WORK/claude.log" 'got: ```' "got: \$ false" "got: exit 1"

  echo "== a token reaches the agent redacted"
  terminal_at 2
  type_text "echo token=$(fake_token)"
  press "" Return
  settle 1
  shot 555-04-before-send
  send_from_menu 916
  shot 555-05-redacted
  cat "$E2E_WORK/claude.log"
  expect "the agent read the token redacted" holds "$E2E_WORK/claude.log" "token=[redacted: "
  expect "and never the token" test "$(grep -c "$(fake_token)" "$E2E_WORK/claude.log")" = 0

  echo "== an agent that waits gets nothing"
  terminal_at 1
  type_text "wait"
  press "" Return
  settle 3
  lines=$(wc -l <"$E2E_WORK/claude.log")
  terminal_at 2
  send_from_menu 916
  shot 555-06-refused
  expect "nothing was typed at the waiting agent" test "$(wc -l <"$E2E_WORK/claude.log")" = "$lines"

  echo "== two agents: the picker, without the block's own terminal"
  palette "workspace: new terminal"
  settle 3
  type_text "codex"
  press "" Return
  settle 2
  terminal_at 2
  send_from_menu 916
  shot 555-07-picker
  press "" Escape
  settle 1

  echo "== no agent, no chip"
  terminal_at 1
  press CTRL d
  settle 1
  terminal_at 3
  press CTRL d
  settle 1
  terminal_at 2
  type_text "false"
  press "" Return
  settle 2
  shot 555-08-no-agent-no-chip
}
