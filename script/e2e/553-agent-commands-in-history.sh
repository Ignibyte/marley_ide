# shellcheck shell=bash
# #553's visual check: whether an agent's commands enter the shell's history and Marley's
# suggestions. A stand-in agent calls terminal_run through the plugin's bridge. By default its
# command enters bash's history and file and is suggested (REQ-001, REQ-002), and a user's own
# line typed with a leading space is kept as bash's own settings say (REQ-006). With
# `marley.agent_commands_in_history` off, a terminal opened after the change runs the command as
# a marked block (REQ-003), keeps it out of bash's history and file and out of zsh's file
# (REQ-004), and suggests nothing from it (REQ-005). The setting is on the Marley page (REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
SEARCH_X=${SEARCH_X:-912}
SEARCH_Y=${SEARCH_Y:-59}

# Sets the settings key path `$1` (dot-separated) to the JSON value `$2` in the run's copy of the
# settings.
set_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  # History written at each prompt, so the file can be read while the shell runs.
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
HISTFILE=$HOME/.bash_history
PROMPT_COMMAND='history -a'
RC
  cat >"$home/.zshrc" <<'RC'
PROMPT='$ '
HISTFILE=$HOME/.zsh_history
HISTSIZE=100
SAVEHIST=100
setopt INC_APPEND_HISTORY
RC
  : >"$home/.bash_history"
  : >"$home/.zsh_history"
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

in_file() { grep -qxF "$1" "$E2E_WORK/home/$2"; }

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== the default: the agent's command enters the history"
  mcp_agent terminal-run repo "echo agent-on" | tee "$E2E_WORK/run1.txt"
  expect "it ran" holds "$E2E_WORK/run1.txt" "'echo agent-on', exit 0"
  type_text "history | tail -3"
  press "" Return
  settle 2
  shot 553-01-default-in-history
  expect "the history file holds it" in_file "echo agent-on" .bash_history

  echo "== a user's own spaced line, with the setting on, is bash's to keep"
  type_text " echo user-spaced"
  press "" Return
  settle 2
  expect "HISTCONTROL unset keeps it" in_file " echo user-spaced" .bash_history

  echo "== and it is suggested"
  # Short of the command's end, so the ghost text shows past the cursor, which covers one cell.
  type_text "echo agen"
  settle 2
  shot 553-02-default-suggested
  press "CTRL" u
  settle 1

  echo "== off, in a bash terminal opened after the change"
  set_setting marley.agent_commands_in_history false
  settle 3
  palette "workspace: new terminal"
  settle 4
  mcp_agent terminal-run repo "echo agent-off-quiet" | tee "$E2E_WORK/run2.txt"
  expect "it ran as a block" holds "$E2E_WORK/run2.txt" "'echo agent-off-quiet', exit 0"
  settle 1
  shot 553-03-off-block
  mcp_agent blocks | tee "$E2E_WORK/blocks.txt"
  expect "its block carries the command, unspaced, and the mark" holds "$E2E_WORK/blocks.txt" \
    "'echo agent-off-quiet', exit 0, running False, kept True, verified True, host None, agent True"
  type_text "history | tail -3"
  press "" Return
  settle 2
  shot 553-04-off-not-in-history
  expect "bash's file lacks it" bash -c "! grep -q 'agent-off' '$E2E_WORK/home/.bash_history'"
  cat "$E2E_WORK/home/.bash_history"

  echo "== nor is it suggested"
  # Its rest would show as `f-quiet`; the file's `echo agent-on` does not fit.
  type_text "echo agent-of"
  settle 2
  shot 553-05-off-not-suggested
  press "CTRL" u
  settle 1

  echo "== off, in zsh"
  set_setting terminal.shell '{"program": "zsh"}'
  settle 3
  palette "workspace: new terminal"
  settle 5
  mcp_agent terminal-run repo "echo agent-zsh" | tee "$E2E_WORK/run3.txt"
  expect "zsh ran it" holds "$E2E_WORK/run3.txt" "'echo agent-zsh', exit 0"
  type_text "echo after-zsh"
  press "" Return
  settle 2
  shot 553-04b-zsh
  cat "$E2E_WORK/home/.zsh_history"
  expect "zsh saved the user's next line" grep -q "after-zsh" "$E2E_WORK/home/.zsh_history"
  expect "and not the agent's" bash -c "! grep -q 'agent-zsh' '$E2E_WORK/home/.zsh_history'"

  echo "== the setting on the Marley page"
  palette "marley: open settings"
  settle 4
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "Agent Commands in History"
  settle 2
  shot 553-06-setting
}
