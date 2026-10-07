# shellcheck shell=bash
# #682's visual check: a settings change an agent proposes and the user accepts. A stand-in agent
# calls `settings_change` through the Claude Code plugin's bridge, as Claude Code in a Marley
# terminal does, on the run's copy of the user settings, which opens with a comment. Proposing
# `terminal.font_size` 19 shows the question (`card`, REQ-001); Apply writes it with the comment
# kept and the terminal draws larger (`applied`, REQ-002, REQ-003); 30 and Decline leave the file
# (`declined`, REQ-004); a wrong type and an unknown key are refused with no question (`refused`,
# REQ-005, REQ-006); a question left alone answers `no_answer` after 25 seconds and goes
# (`no-answer`, REQ-007). `initialize` names the tool (REQ-008).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The question's Apply and Decline, as the first run's shot found them.
APPLY_X=${APPLY_X:-958}
APPLY_Y=${APPLY_Y:-901}
DECLINE_X=${DECLINE_X:-1030}
DECLINE_Y=${DECLINE_Y:-901}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  git init -q -b main "$repo"
  # A comment the change must keep; the runner's later insertion keeps it too.
  sed -i '1i // kept by #682' "$E2E_PROFILE/config/settings.json"
  open_path "$repo"
}

settings_file() {
  printf '%s' "$E2E_PROFILE/config/settings.json"
}

# Starts a proposal in the background; its answer goes to `$1`.
propose_later() {
  local out=$1
  shift
  mcp_agent tool settings_change "$@" >"$out" 2>&1 &
  PROPOSAL=$!
}

# The font size the settings file sets for the terminal now.
terminal_font_size() {
  python3 - "$(settings_file)" <<'PY'
import json, re, sys
text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
print(json.loads(text).get("terminal", {}).get("font_size"))
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the instructions name the tool"
  mcp_agent instructions | tee "$E2E_WORK/instructions.txt" | head -2
  expect "they name settings_change" holds "$E2E_WORK/instructions.txt" "settings_change"

  echo "== a proposal asks the user"
  propose_later "$E2E_WORK/apply.txt" '{"key": "terminal.font_size", "value": 19}'
  settle 3
  shot 682-01-card

  echo "== Apply writes it"
  click "$APPLY_X" "$APPLY_Y"
  wait "$PROPOSAL" || true
  cat "$E2E_WORK/apply.txt"
  expect "the answer is applied" holds "$E2E_WORK/apply.txt" '"result": "applied"' '"after": 19'
  expect "the file holds 19" test "$(terminal_font_size)" = 19
  expect "and its comment" grep -q '^// kept by #682' "$(settings_file)"
  settle 2
  shot 682-02-applied

  echo "== Decline leaves the file"
  propose_later "$E2E_WORK/decline.txt" '{"key": "terminal.font_size", "value": 30}'
  settle 3
  click "$DECLINE_X" "$DECLINE_Y"
  wait "$PROPOSAL" || true
  cat "$E2E_WORK/decline.txt"
  expect "the answer is declined" holds "$E2E_WORK/decline.txt" "declined"
  expect "the file still holds 19" test "$(terminal_font_size)" = 19
  settle 1
  shot 682-03-declined

  echo "== refusals ask nothing"
  mcp_agent tool settings_change '{"key": "terminal.font_size", "value": "big"}' \
    | tee "$E2E_WORK/invalid.txt"
  expect "a wrong type is invalid_value" holds "$E2E_WORK/invalid.txt" "invalid_value"
  mcp_agent tool settings_change '{"key": "terminal.no_such_key", "value": 1}' \
    | tee "$E2E_WORK/unknown.txt"
  expect "an unknown key is no_setting" holds "$E2E_WORK/unknown.txt" "no_setting"
  settle 1
  shot 682-04-refused

  echo "== no answer"
  propose_later "$E2E_WORK/silent.txt" '{"key": "terminal.font_size", "value": 21}'
  settle 3
  shot 682-05-waiting
  wait "$PROPOSAL" || true
  cat "$E2E_WORK/silent.txt"
  expect "the answer is no_answer" holds "$E2E_WORK/silent.txt" "no_answer"
  expect "the file still holds 19" test "$(terminal_font_size)" = 19
  settle 1
  shot 682-06-no-answer
}
