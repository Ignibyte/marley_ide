# shellcheck shell=bash
# #912's visual check: Welcome to Marley on a first start. `first_run`: the profile copy has none
# of the user's database, so nothing restores and `first_open` is unset. A stand-in `claude`
# (it logs how it was run) is `MARLEY_CLAUDE`, with a scratch `CLAUDE_CONFIG_DIR`, so the version
# check and Connect the plugin never run the box's own Claude Code or touch its config (PR-687);
# Codex's folder is left off the PATH.
# - `912-01-night`: Welcome to Marley in the Home group: the mark, the title and its line, the
#   four rows with the setting in effect lit, Open a Folder and Go to Home (REQ-001, REQ-002,
#   REQ-004).
# - `912-02-day`: Marley Day clicked: the theme written and its card lit (REQ-002, REQ-003).
# - `912-03-plugin`: Connect the plugin clicked: the stand-in ran `plugin install` (REQ-004).
# - `912-04-home`: Go to Home: the welcome tab closed, Home in front (REQ-005).
# - `912-05-again`: `marley: show welcome`: the screen again (REQ-006).
# - `912-06-next-start`: Marley started again: no welcome screen (REQ-007).
compositor sway
first_run

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-900}
# From the probe run's shot.
DAY_X=${DAY_X:-1070}
DAY_Y=${DAY_Y:-400}
PLUGIN_X=${PLUGIN_X:-1154}
PLUGIN_Y=${PLUGIN_Y:-460}
HOME_X=${HOME_X:-980}
HOME_Y=${HOME_Y:-677}

setup() {
  local bin=$E2E_WORK/bin home=$E2E_WORK/home
  mkdir -p "$bin" "$home"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/claude" <<FAKE
#!/bin/sh
# A stand-in Claude Code: it logs how it was run, says a version, and lists no plugin.
printf '%s\n' "\$*" >>"$E2E_WORK/claude.log"
[ "\$1" = "--version" ] && echo "2.0.0 (Claude Code)"
exit 0
FAKE
  chmod +x "$bin/claude"
  PATH=$(printf '%s' "$bin:$PATH" | tr ':' '\n' | grep -v '/installs/codex/' | paste -sd:)
  export PATH
  # The plugin's install and the version check run the stand-in, on a scratch Claude config.
  export MARLEY_CLAUDE=$bin/claude CLAUDE_CONFIG_DIR=$E2E_WORK/claude-config
  mkdir -p "$CLAUDE_CONFIG_DIR"
  profile_setting theme '"Marley Night"'
  profile_setting marley.layout '"marley"'
  profile_setting marley.tmux_hosts '[]'
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  away
  shot 912-01-night
  [ -n "${STOP_AT_FIRST:-}" ] && return

  echo "== Marley Day"
  click "$DAY_X" "$DAY_Y"
  settle 4
  away
  shot 912-02-day

  echo "== Connect the plugin"
  click "$PLUGIN_X" "$PLUGIN_Y"
  settle 4
  echo "claude ran: $(tr '\n' ';' <"$E2E_WORK/claude.log" 2>/dev/null)"
  away
  shot 912-03-plugin

  echo "== Go to Home"
  click "$HOME_X" "$HOME_Y"
  settle 3
  away
  shot 912-04-home

  echo "== marley: show welcome"
  palette "marley: show welcome" 3
  away
  shot 912-05-again

  echo "== the next start"
  quit_marley
  settle 3
  launch_marley_with
  settle 12
  away
  shot 912-06-next-start
}
