# shellcheck shell=bash
# #648's visual check: Marley reads the version of the `claude` and `codex` it runs, and keeps
# Claude Code's prompt tags off on a version they were not tested on, saying why in the agent bar.
# `$E2E_WORK/versions/` holds copies of one stand-in named `2.1.287`, `2.2.0`, `garbled` and
# `2.1.300`; given `--version` each prints its own name and ` (Claude Code)`, and otherwise acts
# out a session as #519's does: at each Enter it runs the plugin's real `hooks/event.py` with the
# next step's payloads and writes each answer's sequence to its terminal. `$E2E_WORK/bin/claude`
# links to one of them and moves as Claude Code's installer moves its link; `MARLEY_CLAUDE` names
# the link, and `MARLEY_CODEX` a stand-in `codex` printing `codex-cli 0.155.1`. A scratch
# `CLAUDE_CONFIG_DIR` lists Marley's plugin at the version Marley ships, so its chip stays away.
# Never the user's Claude Code or Codex. Every settings change is an outside edit (L-607).
#
# `648-01-tested`, `648-02-untested`, `648-03-why`, `648-04-setting`, `648-05-allowed`,
# `648-06-unreadable`, `648-07-back-in-range`.
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)

# Where the version chip sits in the agent bar, from the first run's shots.
CHIP_X=${CHIP_X:-520}
CHIP_Y=${CHIP_Y:-952}

# Whether the run's copy of the settings holds the JSON value `$2` at the dotted key path `$1`.
setting_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, sys

node = json.loads(pathlib.Path(sys.argv[1]).read_text())
for key in sys.argv[2].split("."):
    node = node.get(key) if isinstance(node, dict) else None
sys.exit(0 if node == json.loads(sys.argv[3]) else 1)
SETTINGS
}

marley_log() { cat "$E2E_PROFILE/logs/Marley.log" 2>/dev/null; }

# How many of Marley's log lines hold `$1`.
logged() { marley_log | grep -cF -- "$1" || true; }

# The link Marley's `claude` resolves to, moved to `$1`.
install_version() {
  ln -sfn "$E2E_WORK/versions/$1" "$E2E_WORK/bin/claude"
}

# A run of the stand-in in the project's terminal: the session's two steps, the second an
# injected prompt.
run_claude() {
  type_text "claude"
  press "" Return
  settle 3
  press "" Return
  settle 2
  press "" Return
  settle 3
}

# Ends the stand-in's run, as Ctrl+D ends its input.
end_claude() {
  press "CTRL" d
  settle 2
}

close_settings() {
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
}

write_stand_in() {
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code. `--version` prints this file's name, as Claude Code prints its version;
# otherwise each line it reads runs the plugin's hook with the next step's payloads.
import json
import os
import subprocess
import sys

if sys.argv[1:] == ["--version"]:
    print(f"{os.path.basename(os.path.realpath(__file__))} (Claude Code)")
    sys.exit(0)
STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"session_id": "e2e-version-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
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
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions
  local config=$E2E_WORK/claude-config name
  mkdir -p "$home" "$bin" "$versions" "$config/plugins"
  expect "the harness's copy allows the prompt tags" \
    setting_is marley.allow_untested_versions.claude_prompt_tags true
  profile_setting marley.allow_untested_versions '{}'
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$E2E_WORK/steps.json" <<'JSON'
[
  {"label": "a prompt, and Bash running", "events": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Add a README to the project"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"}, "tool_use_id": "t1"}
  ]},
  {"label": "a task notification", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "<task-notification>\nThe build finished.\n</task-notification>"}
  ]}
]
JSON
  for name in 2.1.287 2.2.0 garbled 2.1.300; do
    write_stand_in "$versions/$name"
  done
  install_version 2.1.287
  printf '#!/bin/sh\necho "codex-cli 0.155.1"\n' >"$versions/codex"
  chmod +x "$versions/codex"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$versions/codex CLAUDE_CONFIG_DIR=$config
  git init -q -b versions "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== a tested version"
  run_claude
  shot 648-01-tested
  expect "Marley read Claude Code's version" test "$(logged 'agent versions: Claude Code 2.1.287 at')" = 1
  expect "Marley read Codex's version" test "$(logged 'agent versions: Codex 0.155.1 at')" = 1

  echo "== an untested version"
  end_claude
  install_version 2.2.0
  settle 11
  run_claude
  shot 648-02-untested
  expect "2.1.287 was read once" test "$(logged 'agent versions: Claude Code 2.1.287 at')" = 1
  expect "2.2.0 was read" test "$(logged 'agent versions: Claude Code 2.2.0 at')" = 1

  echo "== why"
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  shot 648-03-why

  echo "== the setting"
  click "$CHIP_X" "$CHIP_Y"
  settle 4
  shot 648-04-setting
  close_settings

  echo "== allowed"
  profile_setting marley.allow_untested_versions '{"claude_prompt_tags": true}'
  settle 3
  end_claude
  run_claude
  shot 648-05-allowed

  echo "== unreadable"
  profile_setting marley.allow_untested_versions '{}'
  end_claude
  install_version garbled
  settle 11
  run_claude
  # Off the chip first: the pointer still rests where 648-03 left it, and a hover needs a move.
  pointer_to 800 400
  settle 1
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  shot 648-06-unreadable
  expect "the garbled version was logged" test "$(logged 'printed "garbled (Claude Code)"')" -ge 1

  echo "== back in range"
  end_claude
  install_version 2.1.300
  settle 11
  run_claude
  shot 648-07-back-in-range
  marley_log | grep "agent versions:" || true
}
