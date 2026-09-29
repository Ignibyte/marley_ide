# shellcheck shell=bash
# #556's visual check: an agent runs commands at the user's shell prompt as blocks. A stand-in
# agent reaches Marley's MCP server through the Claude Code plugin's bridge and calls
# terminal_run on the project's terminal. An allowlisted command runs at once and answers with
# its block, which carries the agent's mark (REQ-001, REQ-009); one outside both lists runs under
# the default setting (REQ-002); a denylisted one waits on a card with Run and Refuse and a toast,
# typing nothing (REQ-003), until Escape refuses it (REQ-004) or Enter runs it (REQ-005). Typing
# at the prompt and a program in the foreground refuse (REQ-006, REQ-007); a block still running
# at the wait answers running (REQ-008). Ctrl-I takes the terminal over and refuses, and hands it
# back (REQ-010); the ask setting puts a command outside the lists on the card (REQ-011); the
# setting is on the Marley page (REQ-012).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A click in the terminal, the card's Refuse at the terminal's bottom left (#593), and the
# settings window's search field, from the shots.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
REFUSE_X=${REFUSE_X:-322}
REFUSE_Y=${REFUSE_Y:-954}
SEARCH_X=${SEARCH_X:-912}
SEARCH_Y=${SEARCH_Y:-59}

# Sets `marley.<$1>` to the JSON value `$2` in the run's copy of the settings.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {})[sys.argv[2]] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  git init -q -b main "$repo"
  echo "hello from notes" >"$repo/notes.txt"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Starts a run in the background; its output goes to `$1`.
run_later() {
  local out=$1
  shift
  mcp_agent terminal-run repo "$@" >"$out" 2>&1 &
  RUNNER=$!
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== an allowlisted command runs at once, as a block with the agent's mark"
  mcp_agent terminal-run repo "cat notes.txt" | tee "$E2E_WORK/run1.txt"
  expect "it ran and answered with its block" holds "$E2E_WORK/run1.txt" "'cat notes.txt', exit 0" \
    "| hello from notes"
  settle 1
  shot 556-01-ran-allowlisted

  echo "== outside both lists, the default runs it"
  # The line ends in a newline, so the prompt that follows starts a row of its own.
  mcp_agent terminal-run repo "printf 'x%.0s' {1..30}; echo" | tee "$E2E_WORK/run2.txt"
  expect "it ran with no card" holds "$E2E_WORK/run2.txt" "exit 0" "| xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
  settle 1
  shot 556-02-outside-lists

  echo "== a denylisted command waits on the card"
  run_later "$E2E_WORK/run3.txt" "rm notes.txt"
  settle 3
  shot 556-03-card
  expect "nothing was typed while it waits" test -f "$E2E_WORK/repo/notes.txt"

  echo "== Escape refuses it"
  press "" Escape
  wait "$RUNNER" || true
  cat "$E2E_WORK/run3.txt"
  expect "the call was refused" holds "$E2E_WORK/run3.txt" "refused to run"
  expect "and the file is still there" test -f "$E2E_WORK/repo/notes.txt"
  settle 1
  shot 556-04-refused

  echo "== Enter runs it"
  run_later "$E2E_WORK/run4.txt" "rm notes.txt"
  settle 3
  press "" Return
  wait "$RUNNER" || true
  cat "$E2E_WORK/run4.txt"
  expect "Enter ran it" holds "$E2E_WORK/run4.txt" "'rm notes.txt', exit 0"
  expect "and the file is gone" test ! -e "$E2E_WORK/repo/notes.txt"
  settle 1
  shot 556-05-ran-on-enter

  echo "== typing at the prompt refuses"
  type_text "ech"
  settle 1
  mcp_agent terminal-run repo ls | tee "$E2E_WORK/run5.txt"
  expect "the typing was kept" holds "$E2E_WORK/run5.txt" "typed at the prompt"
  shot 556-06-typing-refused
  press "CTRL" u
  settle 1

  echo "== a program in the foreground refuses"
  type_text "sleep 8"
  press "" Return
  settle 2
  mcp_agent terminal-run repo ls | tee "$E2E_WORK/run6.txt"
  expect "the program was named" holds "$E2E_WORK/run6.txt" "sleep runs in the terminal's foreground"
  shot 556-07-program-refused
  settle 8

  echo "== a block still running at the wait"
  mcp_agent terminal-run repo "sleep 12" --wait 2 | tee "$E2E_WORK/run7.txt"
  expect "it answered running" holds "$E2E_WORK/run7.txt" "'sleep 12', exit None, running True"
  shot 556-08-still-running
  settle 12
  mcp_agent terminal-read "sleep 12" | tee "$E2E_WORK/read7.txt"
  expect "terminal_read reads it after" holds "$E2E_WORK/read7.txt" "'sleep 12'"

  echo "== terminal_blocks marks the agent's blocks"
  mcp_agent blocks | tee "$E2E_WORK/blocks.txt"
  expect "the agent's block is marked" holds "$E2E_WORK/blocks.txt" "'cat notes.txt', exit 0, running False, kept True, verified True, host None, agent True"
  expect "the user's is not" holds "$E2E_WORK/blocks.txt" "'sleep 8', exit 0, running False, kept True, verified True, host None, agent False"

  echo "== Ctrl-I takes over, and hands back"
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL" i
  settle 1
  shot 556-09-taken-over
  mcp_agent terminal-run repo ls | tee "$E2E_WORK/run8.txt"
  expect "a run while the user has control is refused" holds "$E2E_WORK/run8.txt" "taken over"
  press "CTRL" i
  settle 1
  mcp_agent terminal-run repo "ls -a" | tee "$E2E_WORK/run9.txt"
  expect "after the hand-back it runs" holds "$E2E_WORK/run9.txt" "'ls -a', exit 0"
  settle 1
  shot 556-10-handed-back

  echo "== ask puts a command outside the lists on the card"
  marley_setting agent_commands_outside_lists '"ask"'
  settle 3
  run_later "$E2E_WORK/run10.txt" "printf hi"
  settle 3
  shot 556-11-ask-outside-lists
  click "$REFUSE_X" "$REFUSE_Y"
  wait "$RUNNER" || true
  cat "$E2E_WORK/run10.txt"
  expect "Refuse refused it" holds "$E2E_WORK/run10.txt" "refused to run"

  echo "== the setting on the Marley page"
  palette "marley: open settings"
  settle 4
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "Agent Commands"
  settle 2
  shot 556-12-settings
}
