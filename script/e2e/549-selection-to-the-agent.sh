# shellcheck shell=bash
# #549's e2e test: the editor's selection sent to a CLI agent in a terminal. Stand-ins named
# `claude` and `codex`, first on the terminal's PATH, print each line they read (`got: <line>`,
# also to a log); the `claude` one, on reading `wait`, runs the plugin's `event.py` with a prompt
# and a PermissionRequest, so its seat waits. Checks: with one agent, `ctrl->` in the file's editor
# types `@src/auth.txt#L2-4` at the agent's prompt, and the agent reads it only after Return; with
# two, a picker opens, and the codex row gets the absolute `…/src/auth.txt:2-4 ` since its folder,
# `sub/`, does not hold the file; rich input open on the claude terminal takes the reference; an
# agent that waits gets nothing and a toast.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo/src" "$repo/sub"
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
  for line in 1 2 3 4 5 6 7 8 9 10; do
    echo "line $line of auth"
  done >"$repo/src/auth.txt"
  git init -q -b send "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Opens src/auth.txt through the file finder, which also brings back its tab when it is open; the
# finder comes from the palette, since a terminal with the focus takes ctrl-p for its program.
open_auth() {
  palette "file finder: toggle"
  settle 1
  type_text "auth"
  settle 1
  press "" Return
  settle 2
}

# Selects lines 2 to 4 of the open file.
select_lines() {
  press CTRL Home
  press "" Down
  press SHIFT Down
  press SHIFT Down
  press SHIFT End
  settle 1
}

# Zed's Add to Agent Thread, `ctrl->`: ctrl and the `greater` key, which is what the binding
# names (ctrl, shift and `period` does not match it).
send_key() {
  press CTRL greater
  settle 2
}

# The rail's row of the claude terminal, the project's first, measured from the first run.
CLAUDE_ROW_X=100
CLAUDE_ROW_Y=136

# In the picker, the row whose text holds `$1`.
pick() {
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  settle 2
  open_auth
  select_lines
  shot 549-00-selected
  echo "== one agent: ctrl-> types the reference"
  send_key
  shot 549-01-sent
  expect "nothing reaches the agent before Return" test ! -s "$E2E_WORK/claude.log"
  press "" Return
  settle 2
  shot 549-02-unsent-until-enter
  cat "$E2E_WORK/claude.log"
  expect "the agent read Claude Code's reference, relative to its folder" \
    holds "$E2E_WORK/claude.log" "got: @src/auth.txt#L2-4"
  echo "== two agents: the picker"
  palette "workspace: new terminal"
  settle 3
  type_text "cd sub && codex"
  press "" Return
  settle 2
  open_auth
  send_key
  shot 549-03-picker
  pick "codex"
  shot 549-04-picked-outside-cwd
  press "" Return
  settle 2
  cat "$E2E_WORK/codex.log"
  expect "codex read Zed's form with the absolute path" \
    holds "$E2E_WORK/codex.log" "got: $E2E_WORK/repo/src/auth.txt:2-4"
  echo "== rich input open on the claude terminal takes the reference"
  click "$CLAUDE_ROW_X" "$CLAUDE_ROW_Y"
  settle 1
  press CTRL g
  settle 1
  open_auth
  send_key
  pick "claude"
  shot 549-07-rich-input
  press "" Escape
  settle 1
  expect "the reference went into rich input, not the terminal" \
    test "$(grep -c "got: @src" "$E2E_WORK/claude.log")" -eq 1
  echo "== an agent that waits gets nothing"
  type_text "wait"
  press "" Return
  settle 3
  open_auth
  send_key
  pick "claude"
  shot 549-05-refused
  click "$CLAUDE_ROW_X" "$CLAUDE_ROW_Y"
  settle 1
  press "" Return
  settle 2
  tail -2 "$E2E_WORK/claude.log"
  expect "nothing was typed at the waiting agent's prompt" \
    bash -c "tail -1 '$E2E_WORK/claude.log' | grep -qx 'got: '"
  # With no agent, ctrl-> goes on to Zed's Agent Panel, which starts a thread of the panel's last
  # agent; in the run's copy of the user's profile that is the user's own Claude agent, which no
  # scenario may start, so that step was checked once by hand (#549's notes) and is not run here.
}
