# shellcheck shell=bash
# #884's visual check: the Marley Guide in Marley's colours, each article a card, as the
# SearchMore board draws it. Marley's Chromium reaches no host but this machine.
# Marley's Chromium starts in the light scheme here, so a stand-in agent sets the page's scheme
# with `browser_color_scheme`, as #756 does.
# - `marley: open guide`, the dark scheme: Night's colours, the nav and an article card
#   (`884-01-guide`, REQ-001).
# - Marley Day and the light scheme: Day's colours (`884-02-day`, REQ-001).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  offline_chromium
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
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

  echo "== the guide"
  palette "marley: open guide" 8
  mcp_agent tool browser_color_scheme '{"scheme": "dark"}'
  settle 3
  pointer_to 1500 950
  settle 1
  shot 884-01-guide

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  mcp_agent tool browser_color_scheme '{"scheme": "light"}'
  settle 5
  shot 884-02-day
}
