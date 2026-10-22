# shellcheck shell=bash
# #850's visual check: the Browser tab as its board draws it. Marley's Chromium reaches no host
# but this machine (`offline_chromium`); a page served in the run logs a line to its console.
# - `marley: open browser`, the page: the toolbar with the project, the address, the place, and
#   Pick, Annotate and Record labelled in a group (`850-01-toolbar`, REQ-001).
# - Marley Day (`850-02-day`, REQ-001).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-0}

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Feature Map</title><style>
body { margin: 0; padding: 32px; font: 18px sans-serif; background: #15110e; color: #f3ece6 }
.card { display: inline-block; width: 220px; margin: 8px; padding: 16px; border: 1px solid #3a2f26; border-radius: 12px }
</style></head><body>
<h1>Feature Map</h1>
<div class="card">2.24 tmux rows under their project</div>
<div class="card">2.25 The rail shrinks</div>
<script>console.warn('verdict buttons wrap on a narrow card');</script>
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

  echo "== the Browser tab"
  click 800 500
  settle 1
  palette "marley: open browser" 5
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 5
  pointer_to 1500 950
  settle 1
  shot 850-01-toolbar
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 850-02-day
}
