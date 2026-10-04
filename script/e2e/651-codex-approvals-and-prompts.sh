# shellcheck shell=bash
# #651's visual check: Codex's approvals answered from the inbox. #650's stand-in `codex`
# (`codex-fixture.sh`), named by `MARLEY_CODEX`, with `marley.codex_app_server` on in the run's
# copy, plays Codex's server and TUI; each request is a cue typed into its TUI: `command <line>`,
# `file <paths>`, `perms write:<path> network`, `elicit <server> <message>`, `input <question>`,
# and `y` or `n` answers the last request in the TUI. The server logs every answer with the client
# that sent it, takes the first, and tells every subscriber the request is resolved. The inbox's
# risk use runs in `shadow` on the System One replay, as #568's run sets it, so the entries carry
# Marley's chips. Never the user's Codex, and no model turn.
#
# `651-01-command`, `651-02-allowed`, `651-03-answered-in-terminal`, `651-04-file-change`,
# `651-05-denied`, `651-06a-permissions`, `651-06-permissions`, `651-07-elicitation`,
# `651-08-opened`, `651-09-unlisted`, `651-10-server-gone`.
compositor sway

# shellcheck source=script/e2e/codex-fixture.sh
. script/e2e/codex-fixture.sh

# Places in the window, from the first run's shots: the inbox's first entry, and its buttons.
ENTRY_X=${ENTRY_X:-110}
ENTRY_Y=${ENTRY_Y:-123}
# The buttons sit right-aligned under the entry: Allow first of four, Deny third, and Dismiss
# last of an elicitation's two.
ALLOW_X=${ALLOW_X:-100}
ALLOW_Y=${ALLOW_Y:-158}
DENY_X=${DENY_X:-228}
DENY_Y=${DENY_Y:-158}
DISMISS_X=${DISMISS_X:-220}
DISMISS_Y=${DISMISS_Y:-158}
# A place in the terminal, to give it the keys back after a click in the rail.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}

server_log() { cat "$E2E_WORK/server.log" 2>/dev/null; }

# How many lines of the server's log hold `$1`.
logged() { server_log | grep -cF -- "$1" || true; }

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# A Codex from the New Agent picker.
picker_codex() {
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 6
}

# Types `$1` and Enter into the terminal, after giving it the keys.
cue() {
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "$1"
  press "" Return
  settle "${2:-3}"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions
  mkdir -p "$home" "$bin" "$versions" "$E2E_PROFILE/system_one"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_codex_stand_in "$versions/0.155.1"
  ln -sfn "$versions/0.155.1" "$bin/codex"
  export MARLEY_CODEX=$bin/codex
  profile_setting marley.codex_app_server true
  # The inbox's risk use in shadow, on the replay: Marley's chips, and no model asked.
  : >"$E2E_PROFILE/system_one/replay.jsonl"
  profile_setting marley.system_one "{\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"inbox\": \"shadow\"}}"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  server_log
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== a Codex on its own App Server"
  picker_codex
  settle 4
  expect "Marley subscribed to the lead" test "$(logged 'resumed by marley while idle')" = 1

  echo "== a command"
  cue "command rm -rf build"
  shot 651-01-command
  expect "the server asked the command" test "$(logged 'asked 101 item/commandExecution/requestApproval')" = 1

  echo "== Allow"
  click "$ALLOW_X" "$ALLOW_Y"
  settle 3
  shot 651-02-allowed
  expect "Marley answered 101 with accept" test "$(logged 'answered 101 by marley: {"decision": "accept"}')" = 1

  echo "== answered in the terminal"
  cue "command rm -rf dist" 2
  cue n
  shot 651-03-answered-in-terminal
  expect "the TUI answered 102" test "$(logged 'answered 102 by codex-tui')" = 1
  expect "Marley sent nothing for 102" test "$(logged '102 by marley')" = 0

  echo "== a file change"
  cue "file README.md src/main.rs"
  shot 651-04-file-change

  echo "== Deny"
  click "$DENY_X" "$DENY_Y"
  settle 3
  shot 651-05-denied
  expect "Marley answered 103 with decline" test "$(logged 'answered 103 by marley: {"decision": "decline"}')" = 1

  echo "== permissions"
  cue "perms write:/tmp/out network"
  shot 651-06a-permissions
  click "$ALLOW_X" "$ALLOW_Y"
  settle 3
  shot 651-06-permissions
  expect "Marley granted 104 as asked, for the turn" test "$(server_log | grep -F 'answered 104 by marley' | grep -F '"scope": "turn"' | grep -cF '/tmp/out' || true)" = 1

  echo "== an elicitation"
  cue "elicit docs Pick a branch for the docs build"
  shot 651-07-elicitation
  click "$DISMISS_X" "$DISMISS_Y"
  settle 3
  expect "Marley dismissed 105" test "$(logged 'answered 105 by marley: {"action": "cancel"}')" = 1

  echo "== the entry opens its terminal"
  cue "elicit docs Another question"
  palette "workspace: new terminal"
  settle 2
  click "$ENTRY_X" "$ENTRY_Y"
  settle 2
  shot 651-08-opened
  cue n

  echo "== a question the inbox does not list"
  cue "input Which branch?" 10
  shot 651-09-unlisted
  expect "Marley sent nothing for 107" test "$(logged '107 by marley')" = 0
  cue n

  echo "== the server goes"
  cue "command rm -rf out"
  cue crash 4
  shot 651-10-server-gone
  expect "Marley answered only the four it was asked to" test "$(server_log | grep -cE '(answered|late) [0-9]+ by marley' || true)" = 4
}
