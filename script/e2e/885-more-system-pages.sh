# shellcheck shell=bash
# #885's visual check: the System tab's other pages as the SystemPages board draws them. A scratch
# repository; the run's Marley with its MCP server, Rusty off, no fleet provider, and stand-in
# `claude` and `codex` that print a version.
# - Troubleshooting: the shells, Rusty off, Claude Code and Codex found, the plugin, the other
#   CLIs and git (`885-01-trouble`, REQ-001, REQ-002).
# - Marley's MCP endpoint: running, its address, its file, the paired programs (`885-02-mcp`,
#   REQ-001).
# - Install and update, Shell integration, Marley's Chromium, Fleet providers and For developers
#   in turn (`885-03a-install` to `885-03e-developers`, REQ-001).
# - Marley Day, Troubleshooting (`885-04-day`, REQ-001).
compositor sway

# The page list's rows, from #855's shots: This build at 113, then one every 29 pixels.
LIST_X=${LIST_X:-340}
list_y() { echo $((113 + 29 * $1)); }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '#!/bin/sh\necho "2.1.200 (Claude Code)"\n' >"$bin/claude"
  printf '#!/bin/sh\necho "codex-cli 0.80.0"\n' >"$bin/codex"
  chmod +x "$bin/claude" "$bin/codex"
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

# Shows the page at `$1` in the list and shoots it as `$2`.
page_shot() {
  click "$LIST_X" "$(list_y "$1")"
  settle 3
  pointer_to 1500 950
  settle 1
  shot "$2"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the System tab"
  palette "marley: open system pages" 5

  echo "== the pages"
  page_shot 3 885-01-trouble
  page_shot 5 885-02-mcp
  page_shot 2 885-03a-install
  page_shot 4 885-03b-shell
  page_shot 6 885-03c-chromium
  page_shot 7 885-03d-fleet
  page_shot 8 885-03e-developers

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  page_shot 3 885-04-day
}
