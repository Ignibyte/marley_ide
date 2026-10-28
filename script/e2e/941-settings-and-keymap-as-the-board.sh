# shellcheck shell=bash
# #941's visual check: Settings and the keymap as the boards draw them. A scratch repository and a
# fresh profile, no agent and no Rusty; the run's own `keymap.json` binds Ctrl+Alt+Shift+K to
# New File, so one binding is the user's.
# - `marley: open settings`: the section list with its tiles, Layout as Marley | Zed, Agent text
#   full width, the "At the prompt" section and sentence-case titles (`941-01-settings`, REQ-001
#   to REQ-005).
# - "full width" searched: Agent text full width on, Marley's default (#822), the copied
#   `agent.limit_content_width` taken out first (`941-02-found`); switched off: "changed · reset",
#   and the run's `settings.json` holds `"limit_content_width": true` (`941-02-full-width`,
#   REQ-004).
# - `zed: open keymap` searched for "new file" (Zed's and yours) and "browser reload" (Marley's):
#   Action, Arguments, Keys, Where, From, and Find by keys (`941-03-keymap`, `941-03-keymap-marley`,
#   REQ-006 to REQ-008); Find by keys clicked: pressed (`941-04-find-by-keys`, REQ-008).
# - Browser Reload opened for new keys and Ctrl+L typed, which Focus Address Bar holds there:
#   "New keys for …"; Save once: "Also Focus Address Bar, in MarleyBrowser" and Save Anyway
#   (`941-05-record`, REQ-009).
# - Marley Day: the Settings window and the keymap (`941-06-day`, `941-06-day-keymap`).
compositor sway

STOP_AT=${STOP_AT:-0}
AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-950}
# From the first runs' shots: Agent text full width's switch, Find by keys, and the record
# dialog's Save.
WIDTH_X=${WIDTH_X:-1523}
WIDTH_Y=${WIDTH_Y:-248}
FIND_KEYS_X=${FIND_KEYS_X:-1236}
FIND_KEYS_Y=${FIND_KEYS_Y:-91}
# The first row's pencil, which opens it for new keys.
EDIT_X=${EDIT_X:-284}
EDIT_Y=${EDIT_Y:-161}
KEYS_X=${KEYS_X:-798}
KEYS_Y=${KEYS_Y:-195}
SAVE_X=${SAVE_X:-1043}
CANCEL_X=${CANCEL_X:-942}
CANCEL_Y=${CANCEL_Y:-380}
SAVE_Y=${SAVE_Y:-338}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  profile_setting marley.tmux_hosts '[]'
  # The user's own choice comes with the copied settings; the run shows Marley's default (#822's).
  python3 - "$E2E_PROFILE/config/settings.json" <<'UNSET'
import json, pathlib, sys

path = pathlib.Path(sys.argv[1])
settings = json.loads(path.read_text())
settings.get("agent", {}).pop("limit_content_width", None)
path.write_text(json.dumps(settings, indent=2))
UNSET
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-k": "workspace::NewFile"}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Settings"
  palette "marley: open settings" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 941-01-settings
  expect "settings.json does not set limit_content_width yet" bash -c "! grep -q limit_content_width '$E2E_PROFILE/config/settings.json'"
  press "CTRL" f
  settle 1
  type_text "full width"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 941-02-found
  if [[ $STOP_AT == 1 ]]; then
    return
  fi
  click "$WIDTH_X" "$WIDTH_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 941-02-full-width
  expect "settings.json holds limit_content_width true" holds "$E2E_PROFILE/config/settings.json" '"limit_content_width": true'
  press "CTRL" w
  settle 2

  echo "== the keymap"
  palette "zed: open keymap" 4
  type_text "new file"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 941-03-keymap
  press "CTRL" a
  type_text "browser reload"
  settle 2
  shot 941-03-keymap-marley
  if [[ $STOP_AT == 2 ]]; then
    return
  fi
  click "$FIND_KEYS_X" "$FIND_KEYS_Y"
  settle 1
  shot 941-04-find-by-keys
  click "$FIND_KEYS_X" "$FIND_KEYS_Y"
  settle 1

  echo "== new keys"
  click "$EDIT_X" "$EDIT_Y"
  settle 2
  # A click on the keys field records the keys pressed next.
  click "$KEYS_X" "$KEYS_Y"
  settle 1
  press "CTRL" l
  settle 1
  shot 941-05-typed
  if [[ $STOP_AT == 3 ]]; then
    return
  fi
  click "$SAVE_X" "$SAVE_Y"
  settle 2
  shot 941-05-record
  # The keys field still records, so Escape would be taken as keys: Cancel closes the dialog.
  click "$CANCEL_X" "$CANCEL_Y"
  settle 2

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 941-06-day-keymap
  palette "marley: open settings" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 941-06-day
}
