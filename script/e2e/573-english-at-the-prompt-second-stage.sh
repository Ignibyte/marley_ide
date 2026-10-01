# shellcheck shell=bash
# #573's visual check: the System One layer's reading of a line typed at a shell's prompt that
# #557's rules leave open, on the replay provider with the scratch repository listed. The shell's
# prompt editor docks at the prompt (#627); a fake `rm` first on the shell's PATH logs its
# arguments and removes nothing.
#
# With the use off, #557's hint shows after the editor's line and no call is made (REQ-001,
# REQ-011). In suggest an open line left 250 ms is asked once and its reading shows with a `?`
# (REQ-002, REQ-003); typing on makes no call for the prefix (REQ-004); a command and a line with a
# token make none (REQ-005, REQ-006); Enter at once runs the line with no call (REQ-007). In act the
# middle names its command and colours its words (REQ-008), and the grid's slot shows the words
# with the editor closed (REQ-009). Decisions lists the calls (REQ-010), an unlisted project makes
# none (REQ-011), the outcomes follow the calls (REQ-012), and the mode is on the Marley page
# (REQ-014).
compositor sway

# The settings window's search field.
SEARCH_X=${SEARCH_X:-912}
SEARCH_Y=${SEARCH_Y:-59}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_PROFILE/system_one"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '#!/usr/bin/env bash\necho "$*" >>%q\n' "$E2E_WORK/rm.log" >"$bin/rm"
  chmod +x "$bin/rm"
  # A token assembled here, so no file of the repository holds one.
  printf 'gh%s_%s' p "$(printf 'Fake%.0s' {1..9})" >"$E2E_WORK/token"
  cat >"$E2E_PROFILE/system_one/replay.jsonl" <<'JSONL'
{"set": "typed_line/1", "match": "line: find all the large files in this repo", "answers": {"kind": {"type": "choice", "choice": "request", "confidence": 0.9, "probabilities": {"request": 0.9}}}}
{"set": "typed_line/1", "match": "line: kill the dev server", "answers": {"kind": {"type": "choice", "choice": "request", "confidence": 0.85, "probabilities": {"request": 0.85}}}}
{"set": "typed_line/1", "match": "line: rm the old build folder", "repeat": true, "answers": {"kind": {"type": "choice", "choice": "command_then_english", "confidence": 0.93, "probabilities": {"command_then_english": 0.93}}}}
JSONL
  system_one_setting "{\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"typed_line\": \"off\"}}"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

# Merges the JSON object `$1` into `marley.system_one` in the run's copy of the settings, `uses`
# one level deeper.
system_one_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
layer = settings.setdefault("marley", {}).setdefault("system_one", {})
for key, value in json.loads(sys.argv[2]).items():
    if key == "uses":
        layer.setdefault("uses", {}).update(value)
    else:
        layer[key] = value
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

mode() {
  system_one_setting "{\"uses\": {\"typed_line\": \"$1\"}}"
  settle 2
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The day's rows of the typed line's calls, and every outcome row.
calls() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep '"use":"typed_line"' || true
}
outcomes() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"outcome"' || true
}

# How many of the calls' rows hold `$1`.
calls_with() { calls | grep -cF -- "$1" || true; }

# Writes the last call row to `$1.json` and prints its gist.
last_call() {
  calls | tail -n 1 >"$E2E_WORK/$1.json"
  python3 - "$E2E_WORK/$1.json" <<'PY'
import json, sys

row = json.load(open(sys.argv[1]))
print(f"  {row['use']} {row['mode']} {row['provider']} {row['set']}: {row['reading']}")
print("  state:", (row.get("state") or "").replace("\n", " | "))
PY
}

# Empties the shell's editor.
clear_line() {
  press CTRL c
  settle 0.5
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click 700 500
  settle 2

  echo "== off: #557's hint in the editor"
  type_text "kill the dev server"
  settle 1.5
  shot english
  expect "off made no call" test "$(calls | wc -l)" = 0

  echo "== suggest: an open line"
  mode suggest
  clear_line
  type_text "find all the large files in this repo"
  settle 1.5
  shot open-line
  expect "the open line was asked once" test "$(calls_with 'find all the large')" = 1
  last_call open
  expect "the call names the set, the mode, the line and the facts" holds "$E2E_WORK/open.json" \
    '"set":"typed_line/1"' '"mode":"suggest"' '"provider":"replay"' \
    'line: find all the large files in this repo' 'first word: find' 'words: 8'
  clear_line

  echo "== typing on cancels the prefix's call"
  type_text "kill the dev"
  type_text " server"
  settle 1.5
  shot typing-cancels
  expect "one call, for the whole line" test "$(calls_with 'kill the dev')" = 1
  expect "and it is the whole line" test "$(calls_with 'line: kill the dev server')" = 1
  type_text " now"
  settle 0.5
  clear_line

  echo "== a command"
  type_text "ls -la"
  settle 1.5
  shot command
  expect "a command made no call" test "$(calls_with 'ls -la')" = 0
  clear_line

  echo "== a line holding a token"
  type_text "curl the $(cat "$E2E_WORK/token")"
  settle 1.5
  shot secret
  expect "a token's line made no call" test "$(calls_with 'curl the')" = 0
  clear_line

  echo "== Enter at once"
  type_text "rm the old build folder"
  press "" Return
  settle 2
  shot entered
  expect "the shell ran the line" holds "$E2E_WORK/rm.log" "the old build folder"
  expect "no call for a line entered at once" test "$(calls_with 'rm the old build')" = 0

  echo "== act: the middle's warning"
  mode act
  type_text "rm the old build folder"
  settle 1.5
  shot act-warning
  last_call act
  expect "the act call" holds "$E2E_WORK/act.json" '"mode":"act"' 'line: rm the old build folder'
  press "" Return
  settle 2

  echo "== the grid's slot"
  press "" Escape
  settle 1
  type_text "rm the old build folder"
  settle 1.5
  shot grid
  press CTRL u
  type_text "true"
  press "" Return
  settle 2

  echo "== an unlisted project"
  system_one_setting '{"projects": []}'
  settle 2
  type_text "echo what is this"
  settle 1.5
  shot unlisted
  expect "an unlisted project's line made no call" test "$(calls_with 'echo what is this')" = 0
  clear_line

  echo "== Decisions"
  palette "marley: open decisions"
  settle 3
  shot decisions

  echo "== the outcomes"
  outcomes
  expect "the outcomes followed the calls" bash -c "$(declare -f outcomes); outcomes |
    grep -q '\"outcome\":\"cleared\"' && outcomes | grep -q '\"outcome\":\"edited\"' &&
    outcomes | grep -q '\"outcome\":\"entered\"' && outcomes | grep -q '\"outcome\":\"exit 0\"'"

  echo "== the mode on the Marley page"
  palette "marley: open settings"
  settle 4
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "Typed Line"
  settle 2
  shot settings
}
