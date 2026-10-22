# shellcheck shell=bash
# #887's visual check: the Browser's console drawer as the Browser board draws it. Marley's Chromium
# reaches no host but this machine; a page served in the run logs a line, warns, throws an error
# and fetches a file that is not there.
# - The toolbar's Console button, counting the errors, and the drawer's Console
#   (`887-01-console`, REQ-001).
# - The drawer's Network (`887-02-network`, REQ-001).
# - Marley Day (`887-03-day`, REQ-001).
# - A page that asks `confirm()`: its dialog's Cancel and OK as the prompts' buttons
#   (`887-04-dialog`, REQ-002); Escape answers Cancel.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-0}
# The toolbar's Console button and the drawer's Network tab, from the first run's shots.
CONSOLE_X=${CONSOLE_X:-1320}
CONSOLE_Y=${CONSOLE_Y:-87}
NETWORK_X=${NETWORK_X:-380}
NETWORK_Y=${NETWORK_Y:-760}

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
  cat >"$E2E_WORK/site/confirm.html" <<'HTML'
<!doctype html><html><head><title>Reset</title></head><body>
<script>setTimeout(() => confirm('Reset every verdict to Not tried?'), 300);</script>
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

  echo "== the page"
  click 800 500
  settle 1
  palette "marley: open browser" 5
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 5
  pointer_to 900 700
  settle 1
  shot 887-00-page
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== the drawer's Console"
  click "$CONSOLE_X" "$CONSOLE_Y"
  settle 2
  pointer_to 900 400
  settle 1
  shot 887-01-console

  echo "== its Network"
  click "$NETWORK_X" "$NETWORK_Y"
  settle 2
  pointer_to 900 400
  settle 1
  shot 887-02-network

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 887-03-day

  echo "== the page's own dialog"
  press CTRL l
  settle 1
  type_text "$SITE/confirm.html"
  press "" Return
  settle 4
  pointer_to 1500 950
  settle 1
  shot 887-04-dialog
  press "" Escape
  settle 1
}
