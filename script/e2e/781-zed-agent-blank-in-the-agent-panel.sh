# shellcheck shell=bash
# #781's visual check: a Zed Agent thread with no model says so, and Configure opens the providers'
# settings, not Zed's Agent Panel in the right dock. On a copy of the user's profile, where Zed's
# agent has no model it can reach; the copy's thread list emptied.
# - New Agent, "zed", Enter, Enter: the thread's tab shows the callout, The Zed Agent has no model
#   yet, with Configure Providers (`callout`, REQ-002).
# - Select a Model, Configure: the Settings window on LLM Providers; no Agent Panel at the right
#   (`configure`, REQ-001).
# - The settings closed, the callout's Configure Providers: the same page (`callout-button`,
#   REQ-002).
compositor sway

# The note Marley shows when its key opens the picker from a terminal, Select a Model, the
# picker's Configure and the callout's button, as the reproduction's and first run's shots found
# them.
NOTE_CLOSE_X=${NOTE_CLOSE_X:-1563}
NOTE_CLOSE_Y=${NOTE_CLOSE_Y:-879}
SELECT_MODEL_X=${SELECT_MODEL_X:-1262}
SELECT_MODEL_Y=${SELECT_MODEL_Y:-950}
CONFIGURE_X=${CONFIGURE_X:-1157}
CONFIGURE_Y=${CONFIGURE_Y:-917}
CALLOUT_BUTTON_X=${CALLOUT_BUTTON_X:-1534}
CALLOUT_BUTTON_Y=${CALLOUT_BUTTON_Y:-89}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

# Closes the Settings window with its key, when it is the focused window.
close_settings() {
  press "CTRL" w
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== New Agent, Zed Agent"
  press "CTRL ALT" n
  settle 2
  type_text "zed"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 8
  click "$NOTE_CLOSE_X" "$NOTE_CLOSE_Y"
  settle 1
  shot 781-01-callout

  echo "== Select a Model, Configure"
  click "$SELECT_MODEL_X" "$SELECT_MODEL_Y"
  settle 2
  shot 781-01a-picker
  click "$CONFIGURE_X" "$CONFIGURE_Y"
  settle 4
  shot 781-02-configure

  echo "== the callout's Configure Providers"
  close_settings
  # The model picker stays open behind the Settings window.
  press "" Escape
  settle 1
  shot 781-02a-closed
  click "$CALLOUT_BUTTON_X" "$CALLOUT_BUTTON_Y"
  settle 4
  shot 781-03-callout-button
}
