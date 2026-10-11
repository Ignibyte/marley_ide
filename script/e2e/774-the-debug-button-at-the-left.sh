# shellcheck shell=bash
# #774's visual check: the Debug button at the status bar's left. A scratch repo.
# - At start: the Debug icon at the status bar's left end, none among the right-hand buttons
#   (`start`, REQ-001).
# - A click: the bottom dock open on the Debug panel, the button toggled (`open`, REQ-002).
# - A second click: the bottom dock closed (`closed`, REQ-003).
# - `marley: use zed layout`: no Debug icon at the left, Zed's own among the right-hand buttons
#   (`zed-layout`, REQ-005).
# - Back in the Marley layout, a right-click on it, then Hide Button: no Debug icon anywhere, and
#   the copy's settings hold `debugger.button: false` (`menu`, `hidden`, REQ-004).
compositor sway

# The Debug button and its menu's Hide Button, as the first run's shots found them;
# `STOP_AT_START=1` ends the run at the first shot, `STOP_AT_MENU=1` at the menu's.
DEBUG_X=${DEBUG_X:-279}
BAR_Y=${BAR_Y:-985}
HIDE_X=${HIDE_X:-347}
HIDE_Y=${HIDE_Y:-976}
STOP_AT_START=${STOP_AT_START:-0}
STOP_AT_MENU=${STOP_AT_MENU:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  open_path "$repo"
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether the run's copy of the settings sets `debugger.button` to false.
debugger_button_off() {
  python3 - "$E2E_PROFILE/config/settings.json" <<'SETTINGS'
import json, re, sys

text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
debugger = json.loads(text).get("debugger") or {}
print(f"debugger settings: {debugger}")
sys.exit(0 if debugger.get("button") is False else 1)
SETTINGS
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4
  shot 774-01-start
  if [[ $STOP_AT_START == 1 ]]; then
    return
  fi

  echo "== the Debug button"
  click "$DEBUG_X" "$BAR_Y"
  settle 3
  shot 774-02-open

  echo "== the Debug button again"
  click "$DEBUG_X" "$BAR_Y"
  settle 2
  shot 774-03-closed

  echo "== Zed's layout"
  palette "marley: use zed layout"
  settle 4
  shot 774-04-zed-layout

  echo "== the Marley layout, then Hide Button"
  palette "marley: use marley layout"
  settle 4
  click "$DEBUG_X" "$BAR_Y" right
  settle 1
  shot 774-05-menu
  if [[ $STOP_AT_MENU == 1 ]]; then
    return
  fi
  click "$HIDE_X" "$HIDE_Y"
  settle 2
  shot 774-06-hidden
  expect "the settings turn the Debug button off" debugger_button_off
}
