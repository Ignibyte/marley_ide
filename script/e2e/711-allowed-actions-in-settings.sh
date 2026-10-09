# shellcheck shell=bash
# #711's visual check: Settings → Marley → Agent Control → Allowed Actions edits
# `marley.agent_control.actions_allowed`. The profile names `editor::SelectAll` and
# `nothing::Here`.
# - `711-01-listed` (REQ-001): both rows, `nothing::Here` marked "not an action".
# - `711-02-added` (REQ-002): `pane::SplitRight` typed into the field and confirmed, now a row and
#   in settings.json.
# - `711-03-removed` (REQ-003): `nothing::Here`'s remove clicked, its row gone, and settings.json
#   without it.
compositor sway

# The Settings window's field (as 711-01 shows it) and `nothing::Here`'s remove button (as 711-02
# shows it, after the list grew), from the first runs' shots.
FIELD_X=${FIELD_X:-1440}
FIELD_Y=${FIELD_Y:-422}
REMOVE_X=${REMOVE_X:-1555}
REMOVE_Y=${REMOVE_Y:-376}

setup() {
  mkdir -p "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  profile_setting marley.agent_control.actions_allowed '["editor::SelectAll", "nothing::Here"]'
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether the run's settings list `marley.agent_control.actions_allowed` as the JSON `$1`.
allowed_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
actual = json.loads(text).get("marley", {}).get("agent_control", {}).get("actions_allowed")
sys.exit(0 if actual == json.loads(sys.argv[2]) else 1)
SETTINGS
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the Allowed Actions item"
  palette "marley: open settings"
  settle 4
  # The window opens with its page list focused; Ctrl+F puts the keys in its search.
  press "CTRL" f
  settle 1
  type_text "Allowed Actions"
  settle 2
  shot 711-01-listed

  echo "== a name added"
  click "$FIELD_X" "$FIELD_Y"
  settle 1
  type_text "pane::SplitRight"
  press "" Return
  settle 2
  shot 711-02-added
  expect "settings.json holds the added name" \
    allowed_is '["editor::SelectAll", "nothing::Here", "pane::SplitRight"]'

  echo "== a name removed"
  click "$REMOVE_X" "$REMOVE_Y"
  settle 2
  pointer_to 1100 700
  settle 1
  shot 711-03-removed
  expect "settings.json lost the removed name" \
    allowed_is '["editor::SelectAll", "pane::SplitRight"]'
}
