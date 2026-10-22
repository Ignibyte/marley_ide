# shellcheck shell=bash
# #892's visual check: the Browser drawer's Recordings, and Send to the agent. Marley's Chromium
# reaches no host but this machine; a page served in the run logs a line, warns, throws an error
# and fetches a file that is not there. The terminal is used first.
# - `marley: record this`, then the drawer's Recordings: the saved recording with its Send
#   (`892-01-recordings`, REQ-001).
# - The drawer's Console and Send to the agent: the line typed into the terminal
#   (`892-02-sent`, REQ-002).
# - Marley Day, Recordings (`892-03-day`, REQ-001).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-0}
# The toolbar's Console button and the drawer's tabs, from #887's shots.
CONSOLE_X=${CONSOLE_X:-1320}
CONSOLE_Y=${CONSOLE_Y:-87}
CONSOLE_TAB_X=${CONSOLE_TAB_X:-298}
TAB_Y=${TAB_Y:-765}
RECORDINGS_X=${RECORDINGS_X:-440}
SEND_X=${SEND_X:-1480}

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Feature Map</title><style>
body { margin: 0; padding: 32px; font: 18px sans-serif; background: #f4f1ea }
</style></head><body>
<h1>Feature Map</h1>
<script>
console.log('loaded 214 cards');
console.warn('verdict buttons wrap on a narrow card');
fetch('verdicts.json').catch(() => {});
setTimeout(() => { throw new Error('cannot read the verdict of card 2.24'); }, 50);
</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  profile_setting marley.tmux_hosts '[]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
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

  echo "== the terminal is used first, then the page"
  click 800 500
  settle 1
  palette "marley: open browser" 5
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 5

  echo "== Record This, then the drawer's Recordings"
  palette "marley: record this" 4
  click "$CONSOLE_X" "$CONSOLE_Y"
  settle 2
  click "$RECORDINGS_X" "$TAB_Y"
  settle 3
  pointer_to 900 400
  settle 1
  shot 892-01-recordings
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== the Console's Send to the agent"
  click "$CONSOLE_TAB_X" "$TAB_Y"
  settle 2
  click "$SEND_X" "$TAB_Y"
  settle 3
  pointer_to 900 400
  settle 1
  shot 892-02-sent

  echo "== Marley Day, Recordings"
  click 518 50
  settle 2
  profile_setting theme '"Marley Day"'
  settle 3
  click "$RECORDINGS_X" "$TAB_Y"
  settle 2
  shot 892-03-day
}
