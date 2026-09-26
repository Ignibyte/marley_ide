# shellcheck shell=bash
# #500's e2e test: a Browser tab from the rail's +, and the chip that connects Claude Code to
# Marley. The scratch repository opens in the Marley layout, its terminal a stand-in Claude Code
# (the shell becomes `claude` by name). The project's + lists New Browser Tab after New Terminal;
# choosing it opens a blank page in a new Browser tab of the project with the focus in the address
# bar, so an address typed at once loads. Back in the terminal, the agent bar's chip reads
# "Connect Claude Code to Marley", and its tooltip names the browser among the tools. The chip is
# never clicked: it would install the plugin into the real Claude Code configuration. Chromium
# runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The project's + in the rail, the terminal's tab, and the chip, measured from the first run.
PLUS_X=236
PLUS_Y=96
TERMINAL_TAB_X=400
TERMINAL_TAB_Y=50
CHIP_X=550
CHIP_Y=953

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  # The terminal's shell becomes `claude` by name, and the agent bar shows its chip.
  printf '%s\n' "exec -a claude bash -c 'while :; do printf .; sleep 1; done'" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>From the rail</title></head>
<body style="margin:0;font:20px sans-serif;background:#eef6ee"><h2 style="margin:40px">Opened from the rail's +</h2></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b browser "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the project's + lists New Browser Tab"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  # The menu opens on New Terminal; one step down is New Browser Tab.
  press "" Down
  settle 1
  shot 500-01-menu
  echo "== New Browser Tab opens a blank page with the focus in its address bar"
  press "" Return
  settle 6
  type_text "$SITE/index.html"
  press "" Return
  settle 4
  shot 500-02-opened
  echo "== the terminal's agent bar offers to connect Claude Code to Marley"
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  shot 500-03-chip
}
