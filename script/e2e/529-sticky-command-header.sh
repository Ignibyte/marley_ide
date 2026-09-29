# shellcheck shell=bash
# #529's visual check: a long block's command pinned over the terminal's top row while the view
# is scrolled back into its output. At the live screen there is no header (REQ-002); two pages up
# into `seq 1 400`'s output it shows with the block's check (REQ-001); at the block's start it
# goes (REQ-003); a click on it scrolls to the start and selects nothing (REQ-004); a running
# loop's header says it runs (REQ-005); `less` on the alternate screen shows none (REQ-006); with
# `marley.sticky_command_header` off there is none (REQ-007); the Marley page's Terminal section
# has the toggle (REQ-008).
compositor sway

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The header's row, over the terminal's top row, from the shots.
HEADER_Y=${HEADER_Y:-77}
# Where the pointer scrolls the settings window, and how far down its Terminal section is.
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}
SETTINGS_SCROLL=${SETTINGS_SCROLL:-20}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  seq 1 500 | sed 's/^/line /' >"$repo/long.txt"
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

# Sets `marley.sticky_command_header` in the run's copy of the settings; Marley reloads it.
sticky_header_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {})["sticky_command_header"] = sys.argv[2] == "on"
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

page_up() {
  local count=$1 i
  for ((i = 0; i < count; i++)); do
    press SHIFT Page_Up
  done
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== the live screen: no header"
  type_text "seq 1 400"
  press "" Return
  settle 1
  type_text "echo after"
  press "" Return
  settle 2
  shot 529-01-live

  echo "== scrolled back into seq's output: its command pinned at the top"
  page_up 2
  shot 529-02-pinned

  echo "== at the block's start: no header"
  page_up 12
  shot 529-03-start-in-view

  echo "== a click on the header scrolls to the block's start"
  press SHIFT Page_Down
  press SHIFT Page_Down
  press SHIFT Page_Down
  settle 1
  shot 529-04a-before-click
  click "$TERMINAL_X" "$HEADER_Y"
  settle 1
  pointer_to "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  shot 529-04-jumped

  echo "== a running block's header says it runs"
  press SHIFT End
  settle 1
  type_text "for i in \$(seq 1 400); do echo \$i; sleep 0.05; done"
  press "" Return
  # More than a screen and a page of its output, so a page up lands inside it.
  settle 8
  page_up 1
  shot 529-05-running
  settle 14

  echo "== the alternate screen: no header"
  press SHIFT End
  type_text "less long.txt"
  press "" Return
  settle 2
  page_up 2
  shot 529-06-alternate
  type_text "q"
  settle 1

  echo "== the setting off: no header"
  sticky_header_setting off
  settle 3
  page_up 2
  shot 529-07-off
  press SHIFT End
  settle 1

  echo "== the Marley page's toggle"
  palette "marley: open settings"
  settle 4
  shot 529-08a-settings
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll "$SETTINGS_SCROLL"
  settle 2
  shot 529-08-setting
}
