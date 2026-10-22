# shellcheck shell=bash
# #857's visual check: empty and error states as the States board's card. A scratch repository;
# no Claude Code transcripts (the run's own config folder), no Forge provider, MARLEY_CHROMIUM
# naming a file that is not there, and `marley.harness` naming a command that is not there.
# - The rail filtered for "dpkg": the card and Clear the filter (`857-01-filter`, REQ-001).
# - The rail's Threads view searched for "dpkg": the card and Clear the search (`857-02-threads`,
#   REQ-001). (Its "No conversations yet" card is not shot: the run's copy of the database keeps
#   one Rusty thread, as #877 found.)
# - The Browser tab with no Chromium: the card and Open the guide (`857-03-browser`, REQ-001).
# - Forge's board with no provider: the card and Open Settings; the rail's harness card beside it
#   (`857-04-forge`, REQ-001).
# - Marley Day (`857-05-day`, REQ-001).
compositor sway

FILTER_X=${FILTER_X:-100}
FILTER_Y=${FILTER_Y:-50}
THREADS_X=${THREADS_X:-38}
THREADS_Y=${THREADS_Y:-17}
PROJECTS_X=${PROJECTS_X:-14}
PROJECTS_Y=${PROJECTS_Y:-17}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  export MARLEY_CHROMIUM=$E2E_WORK/no-chromium
  profile_setting marley.harness "{\"command\": \"$E2E_WORK/no-rh\", \"args\": [\"mcp\"]}"
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

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the rail filtered for dpkg"
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  type_text "dpkg"
  settle 2
  pointer_to 900 700
  settle 1
  shot 857-01-filter
  press "" Escape
  settle 1

  echo "== the rail's Threads view"
  click "$THREADS_X" "$THREADS_Y"
  settle 3
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  type_text "dpkg"
  settle 2
  pointer_to 900 700
  settle 1
  shot 857-02-threads
  click "$PROJECTS_X" "$PROJECTS_Y"
  settle 2

  echo "== the Browser tab with no Chromium"
  palette "marley: open browser" 6
  pointer_to 900 900
  settle 1
  shot 857-03-browser

  echo "== Forge's board with no provider"
  palette "marley: open forge board" 4
  pointer_to 900 900
  settle 1
  shot 857-04-forge

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 857-05-day
}
