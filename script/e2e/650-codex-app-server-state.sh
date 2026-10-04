# shellcheck shell=bash
# #650's visual check: Codex's state from its own App Server. A stand-in `codex`, one Python
# program the scenario writes, plays Codex's three parts. `--version` prints its own file's name,
# so copies under `versions/` named `0.155.1` and `0.150.0` are two releases, and `bin/codex`
# links to one, as Codex's installer moves its link (#648's way); `MARLEY_CODEX` names the link.
# `app-server --listen unix://P` serves the WebSocket and the JSON-RPC on P with the rules Marley
# rests on: the first `initialize` names the originator, a thread's subscribers are the
# connections joined when it starts and those that resume it, `thread/started` and the status go
# to every joined connection, turns, tokens and requests to subscribers, and
# `thread/settings/updated` to subscribers of the experimental API. It logs each method with the
# client that sent it to `server.log`. `--remote unix://P` is the TUI: it prints its arguments,
# starts its thread and a side thread, and sends each line typed into it to the server as a cue
# the server plays (`work`, `approve`, `input`, `done`, `fail`, `full`, `scoped`, `crash`).
# Without `--remote` it prints its arguments and reads lines, as #532's fake does. Never the
# user's Codex, and no network.
#
# `650-01-off`, `650-02-launched`, `650-03-working`, `650-04-approval`, `650-05-input`,
# `650-06-opened`, `650-07-done`, `650-08-failed`, `650-09-close-asks`, `650-10-full`,
# `650-11-scoped`, `650-12-server-gone`, `650-13-outside`, `650-14-page`, `650-15-versions`.
compositor sway

# shellcheck source=script/e2e/codex-fixture.sh
. script/e2e/codex-fixture.sh

# Places in the window, from the first run's shots. The rail sorts its rows by what each agent
# needs, so a row moves; the steps that would click one use the palette instead.
INBOX_FIRST_X=${INBOX_FIRST_X:-110}
INBOX_FIRST_Y=${INBOX_FIRST_Y:-123}
CHIP_B_X=${CHIP_B_X:-183}
CHIP_B_Y=${CHIP_B_Y:-277}
BAR_CHIP_X=${BAR_CHIP_X:-640}
BAR_CHIP_Y=${BAR_CHIP_Y:-954}

server_log() { cat "$E2E_WORK/server.log" 2>/dev/null; }
launches() { cat "$E2E_WORK/launches.log" 2>/dev/null; }

# How many lines of the server's log hold `$1`.
logged() { server_log | grep -cF -- "$1" || true; }

marley_log() { cat "$E2E_PROFILE/logs/Marley.log" 2>/dev/null; }

# A Codex from the New Agent picker.
picker_codex() {
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 6
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# The pid of the `$1`th server the stand-in started, and its socket.
server_pid() { sed -n "${1}p" "$E2E_WORK/servers.txt" | cut -d' ' -f1; }
server_socket() { sed -n "${1}p" "$E2E_WORK/servers.txt" | cut -d' ' -f2; }

# Types `$1` and Enter into the terminal in front.
cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

close_settings() {
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
}

# Whether every `initialize` from Marley in the server's log comes after the TUI's on the same
# server.
marley_joined_second() {
  python3 - "$E2E_WORK/server.log" <<'ORDER'
import collections, sys

seen = collections.defaultdict(list)
for line in open(sys.argv[1], encoding="utf-8"):
    parts = line.split()
    if len(parts) >= 3 and parts[2] == "initialize":
        seen[parts[0]].append(parts[1])
ok = all(names.index("marley") > names.index("codex-tui")
         for names in seen.values() if "marley" in names and "codex-tui" in names)
joined = any("marley" in names for names in seen.values())
sys.exit(0 if ok and joined else 1)
ORDER
}

# Whether Marley's requests in the server's log are only the five it may send.
marley_methods_allowed() {
  ! server_log | awk '$2 == "marley" && $3 !~ /^(initialize|initialized|thread\/loaded\/list|thread\/read|thread\/resume|thread\/unsubscribe)$/' | grep -q .
}

# Whether the server process `$1` has ended.
gone() { ! kill -0 "$1" 2>/dev/null; }


setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions
  mkdir -p "$home" "$bin" "$versions"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_codex_stand_in "$versions/0.155.1"
  cp "$versions/0.155.1" "$versions/0.150.0"
  ln -sfn "$versions/0.155.1" "$bin/codex"
  export MARLEY_CODEX=$bin/codex
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-k": ["zed::OpenSettingsAt", {"path": "marley.codex_app_server"}], "ctrl-alt-shift-j": ["zed::OpenSettingsAt", {"path": "marley.allow_untested_versions.codex_app_server"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  marley_log | grep "codex app server" || true
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the switch off"
  picker_codex
  settle 3
  shot 650-01-off
  expect "Codex started with no --remote" test "$(launches | grep -c -- '--remote' || true)" = 0

  echo "== the switch on: Codex A"
  profile_setting marley.codex_app_server true
  settle 3
  picker_codex
  settle 4
  shot 650-02-launched
  expect "A's TUI was typed with --remote and --cd" test "$(launches | grep -c -- '--remote unix://.*/codex/.*\.sock --cd ' || true)" = 1
  expect "the server came up" test "$(logged 'listening on')" = 1
  expect "Marley joined after the TUI" marley_joined_second

  echo "== work"
  cue work 6
  shot 650-03-working

  echo "== Codex B, and an approval in it"
  picker_codex
  settle 4
  cue approve 3
  shot 650-04-approval

  echo "== input in A"
  palette "pane: activate previous item"
  cue input 3
  shot 650-05-input

  echo "== the inbox opens B"
  click "$INBOX_FIRST_X" "$INBOX_FIRST_Y"
  settle 2
  shot 650-06-opened

  echo "== done in B"
  cue "done" 3
  shot 650-07-done

  echo "== work then fail in B"
  cue work 2
  cue fail 3
  shot 650-08-failed

  echo "== a working Codex asks before it closes"
  cue work 6
  palette "pane: close active item"
  shot 650-09-close-asks
  press "" Escape
  settle 2

  echo "== full access from the thread"
  cue full 1
  cue work 3
  pointer_to 800 400
  settle 1
  pointer_to "$CHIP_B_X" "$CHIP_B_Y"
  settle 2
  shot 650-10-full

  echo "== Codex C started with full access, then scoped"
  profile_setting marley.agent_permissions_by_project "{\"$E2E_WORK/repo\": {\"codex\": \"full_access\"}}"
  settle 3
  picker_codex
  settle 4
  shot 650-11a-full-from-start
  cue scoped 1
  cue work 3
  shot 650-11-scoped
  expect "C's arguments still ask for full access" test "$(launches | grep -c -- '--sandbox danger-full-access' || true)" -ge 1

  echo "== the server crashes"
  cue crash 4
  shot 650-12-server-gone

  echo "== B closed when idle"
  palette "pane: activate previous item"
  cue "done" 3
  palette "pane: close active item"
  settle 4
  expect "B's server ended with its terminal" gone "$(server_pid 2)"
  expect "B's socket is gone" test ! -e "$(server_socket 2)"

  echo "== an untested Codex"
  ln -sfn "$E2E_WORK/versions/0.150.0" "$E2E_WORK/bin/codex"
  settle 11
  picker_codex
  settle 3
  pointer_to 800 400
  settle 1
  pointer_to "$BAR_CHIP_X" "$BAR_CHIP_Y"
  settle 2
  shot 650-13-outside
  expect "the untested Codex started with no --remote" test "$(launches | tail -1 | grep -c -- '--remote' || true)" = 0

  echo "== the settings"
  press "CTRL ALT SHIFT" k
  settle 4
  shot 650-14-page
  close_settings
  press "CTRL ALT SHIFT" j
  settle 4
  shot 650-15-versions
  close_settings

  echo "== the logs"
  expect "Marley sent only its five methods" marley_methods_allowed
  expect "Marley answered no request" test "$(logged 'marley answered')" = 0
  expect "Marley resumed only idle threads" test "$(logged 'resumed by marley while active')" = 0

  echo "== quit"
  local folder
  folder=$(dirname "$(dirname "$(server_socket 1)")")
  # A still waits on its input, which the close guard would ask about at the quit.
  profile_setting marley.ask_before_ending_a_working_agent false
  settle 3
  quit_marley
  settle 2
  expect "A's server ended with Marley" gone "$(server_pid 1)"
  expect "Marley's socket folder is gone" test ! -e "$folder"
  server_log
}
