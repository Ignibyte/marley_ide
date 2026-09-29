# shellcheck shell=bash
# #525's visual check, after the fact: an agent reads and types into a running program. A Python
# REPL in the project's terminal; a stand-in agent reaches Marley's MCP server through the Claude
# Code plugin's bridge. terminal_screen names the program and shows its rows (REQ-001); the first
# write shows a card with Allow and Deny under the terminal and types nothing before Allow
# (REQ-002), then types (REQ-003) and leaves a bar with Take Over (REQ-005); a later write types
# without asking (REQ-004). Ctrl-I takes over and refuses writes, and hands back with a new
# generation that refuses the old one (REQ-006, REQ-007). ask_every_write asks each time and Deny
# types nothing (REQ-008); an unanswered write is refused after 25 seconds (REQ-010); never_ask
# types with no card (REQ-009); the shell at its prompt is refused (REQ-011); a program started
# later asks again (REQ-002). Since #594 each line an agent submits runs, its output checked on
# the screen, and the rich input's Enter runs a line in Python's REPL under an agent's name.
# Since #595 the bar goes with the program it names, and Ctrl-I at the shell's prompt completes.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A click in the terminal, and the card's Allow and Deny at the terminal's bottom left (#593),
# from the shots.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
ALLOW_X=${ALLOW_X:-290}
ALLOW_Y=${ALLOW_Y:-954}
DENY_X=${DENY_X:-345}
DENY_Y=${DENY_Y:-954}

# Merges the JSON object `$1` into `marley` in the profile copy's settings, one level deep for
# `system_one` (565's `system_one_setting`, widened).
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
def skip_blank(index):
    while index < len(text):
        if text[index] in " \t\r\n":
            index += 1
        elif text.startswith("//", index):
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = len(text) if end < 0 else end + 2
        else:
            break
    return index


out, index, in_string = [], 0, False
while index < len(text):
    character = text[index]
    if in_string:
        out.append(character)
        if character == "\\" and index + 1 < len(text):
            out.append(text[index + 1])
            index += 2
            continue
        if character == '"':
            in_string = False
        index += 1
        continue
    if character == '"':
        in_string = True
    elif text.startswith("//", index) or text.startswith("/*", index):
        index = skip_blank(index)
        continue
    elif character == ",":
        ahead = skip_blank(index + 1)
        if ahead < len(text) and text[ahead] in "}]":
            index += 1
            continue
    out.append(character)
    index += 1
cleaned = "".join(out).strip()
settings = json.loads(cleaned) if cleaned else {}
marley = settings.setdefault("marley", {})
for key, value in changes.items():
    if key == "system_one":
        layer = marley.setdefault("system_one", {})
        for inner, setting in value.items():
            if inner == "uses":
                layer.setdefault("uses", {}).update(setting)
            else:
                layer[inner] = setting
    else:
        marley[key] = value
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
}

# Sets `marley.agent_terminal_writes` and waits for Marley to read it.
writes_mode() {
  marley_setting "{\"agent_terminal_writes\": \"$1\"}"
  settle 3
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
  open_path "$repo"
}

# Starts a write in the background into the terminal titled with `$1`; its output goes to `$2`.
write_later() {
  local title=$1 out=$2
  shift 2
  mcp_agent terminal-type "$title" "$@" >"$out" 2>&1 &
  WRITER=$!
}

python_repl() {
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "python3 -q"
  press "" Return
  settle 3
}

# Whether the screen of the terminal titled with `$1` shows a row that is `$2`, the program's
# output rather than the line typed: the screen goes to `$3`.
screen_shows() {
  mcp_agent terminal-screen "$1" >"$3"
  cat "$3"
  grep -qE "^  \| $2 *\$" "$3"
}

steps() {
  local before
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== terminal_screen reads a running program"
  python_repl
  mcp_agent terminal-screen python | tee "$E2E_WORK/screen1.txt"
  expect "terminal_screen names the program" holds "$E2E_WORK/screen1.txt" "program python"
  expect "and shows the REPL's prompt" holds "$E2E_WORK/screen1.txt" ">>>"
  shot 525-01-repl

  echo "== the first write asks, and types on Allow"
  write_later python "$E2E_WORK/type1.txt" "print(6 * 7)" --submit
  settle 3
  shot 525-02-ask
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$WRITER" || true
  cat "$E2E_WORK/type1.txt"
  expect "Allow typed the write" holds "$E2E_WORK/type1.txt" "bytes into"
  settle 1
  expect "and the REPL ran it" screen_shows python 42 "$E2E_WORK/screen-42.txt"
  shot 525-03-typed

  echo "== a later write to the same program does not ask"
  mcp_agent terminal-type python "print('again')" --submit | tee "$E2E_WORK/type2.txt"
  expect "the second write typed without a card" holds "$E2E_WORK/type2.txt" "bytes into"
  settle 1
  expect "and ran" screen_shows python again "$E2E_WORK/screen-again.txt"
  shot 525-04-no-ask

  echo "== Ctrl-I takes over"
  mcp_agent terminal-screen python | tee "$E2E_WORK/screen2.txt"
  before=$(sed -n 's/.*generation \([0-9]*\),.*/\1/p' "$E2E_WORK/screen2.txt" | head -1)
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL" i
  settle 1
  shot 525-05-taken-over
  mcp_agent terminal-type python "print(1)" --submit | tee "$E2E_WORK/type3.txt"
  expect "a write while the user has control is refused" holds "$E2E_WORK/type3.txt" "taken over"

  echo "== Hand Back: the old generation is refused, the new one types"
  press "CTRL" i
  settle 1
  mcp_agent terminal-type python "print(2)" --submit --generation "$before" | tee "$E2E_WORK/type4.txt"
  expect "a write naming the old generation is refused" holds "$E2E_WORK/type4.txt" "not $before"
  mcp_agent terminal-type python "print('handed back')" --submit | tee "$E2E_WORK/type5.txt"
  expect "a write naming the new one types" holds "$E2E_WORK/type5.txt" "bytes into"
  settle 1
  expect "and ran" screen_shows python "handed back" "$E2E_WORK/screen-handed.txt"
  shot 525-06-handed-back

  echo "== ask_every_write asks each time, and Deny types nothing"
  writes_mode ask_every_write
  write_later python "$E2E_WORK/type6.txt" "print('asked again')" --submit
  settle 3
  shot 525-07-asks-every-write
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$WRITER" || true
  cat "$E2E_WORK/type6.txt"
  expect "Allow typed it" holds "$E2E_WORK/type6.txt" "bytes into"
  write_later python "$E2E_WORK/type7.txt" "print('denied')" --submit
  settle 3
  click "$DENY_X" "$DENY_Y"
  wait "$WRITER" || true
  cat "$E2E_WORK/type7.txt"
  expect "Deny refused it" holds "$E2E_WORK/type7.txt" "refused this write"
  settle 1
  expect "and nothing of it reached the REPL" test "$(mcp_agent terminal-screen python | grep -c denied)" = 0
  shot 525-08-denied

  echo "== an unanswered write is refused after 25 seconds"
  write_later python "$E2E_WORK/type8.txt" "print('unanswered')" --submit
  wait "$WRITER" || true
  cat "$E2E_WORK/type8.txt"
  expect "the unanswered write is refused" holds "$E2E_WORK/type8.txt" "did not answer"
  settle 1
  shot 525-09-timed-out

  echo "== never_ask types with no card"
  writes_mode never_ask
  mcp_agent terminal-type python "print('never asked')" --submit | tee "$E2E_WORK/type9.txt"
  expect "never_ask typed it" holds "$E2E_WORK/type9.txt" "bytes into"
  settle 1
  expect "and ran" screen_shows python "never asked" "$E2E_WORK/screen-never.txt"
  shot 525-10-never-ask

  echo "== the shell at its prompt is refused"
  writes_mode ask_first_write
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL" d
  settle 2
  mcp_agent terminal-type bash "echo no" --submit | tee "$E2E_WORK/type10.txt"
  expect "a write at the shell's prompt is refused" holds "$E2E_WORK/type10.txt" "never at the shell's prompt"
  shot 525-11-shell-refused

  echo "== #595: Ctrl-I at the shell's prompt completes, with no bar left from Python"
  type_text "ech"
  press "CTRL" i
  settle 1
  shot 525-11b-tab-completes
  expect "Ctrl-I reached the shell" \
    test "$(mcp_agent terminal-screen repo | grep -c '^  | \$ echo')" -ge 1
  press "CTRL" u
  settle 1

  echo "== a program started later asks again"
  python_repl
  write_later python "$E2E_WORK/type11.txt" "print('new program')" --submit
  settle 3
  shot 525-12-new-program-asks
  click "$DENY_X" "$DENY_Y"
  wait "$WRITER" || true
  cat "$E2E_WORK/type11.txt"
  expect "Deny refused the new program's first write" holds "$E2E_WORK/type11.txt" "refused this write"

  echo "== #594: the rich input's Enter runs the line in a REPL under an agent's name"
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL" d
  settle 2
  type_text "exec -a claude python3 -q"
  press "" Return
  settle 4
  mcp_agent terminals
  press "CTRL" g
  settle 1
  type_text "print(6 * 7 + 1)"
  settle 1
  shot 525-13-rich-input-open
  # The only terminal: its title follows the program's name.
  expect "the text waits in the rich input" \
    test "$(mcp_agent terminal-screen repo | grep -c '6 \* 7 + 1')" = 0
  press "" Return
  settle 2
  expect "the rich input's line ran" screen_shows repo 43 "$E2E_WORK/screen-rich.txt"
  shot 525-14-rich-input-ran
}
