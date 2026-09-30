# shellcheck shell=bash
# #599's e2e test: the Marley guide. A scratch project; the title bar shows the `?` before Sign In
# (`titlebar`, REQ-001). A click on it opens the guide in a Browser tab of the project (`guide`,
# REQ-002); the contents filter narrows to `teardown` (`filter`) and a click on a match shows its
# article, a summary then steps (`article`, REQ-006). With the terminal's tab in front, the `?`
# brings the same guide tab forward and opens no second one (`again`, REQ-003). Closed, the guide
# opens again from `marley: open guide` (`palette`, REQ-005). In a new window with no folder the
# command hands the page's `file://` URL to the system browser, a fake `xdg-open` first on the PATH
# that writes it to `fallback.txt` (REQ-004).
compositor sway

# The `?` in the title bar, the page's contents filter and its first match, the terminal's tab,
# from the first run's shots.
GUIDE_X=${GUIDE_X:-1506}
GUIDE_Y=${GUIDE_Y:-17}
FILTER_X=${FILTER_X:-397}
FILTER_Y=${FILTER_Y:-212}
MATCH_X=${MATCH_X:-360}
MATCH_Y=${MATCH_Y:-291}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-400}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin project=$E2E_WORK/project
  mkdir -p "$home" "$bin" "$project"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# A project\n' >"$project/README.md"
  cat >"$bin/xdg-open" <<FAKE
#!/bin/sh
printf '%s\n' "\$@" >>"$E2E_WORK/fallback.txt"
FAKE
  chmod +x "$bin/xdg-open"
  export PATH="$bin:$PATH"
  open_path "$project"
}

palette_command() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the title bar's ?"
  shot titlebar

  echo "== the ? opens the guide in a Browser tab of the project"
  click "$GUIDE_X" "$GUIDE_Y"
  settle 12
  shot guide

  echo "== the contents filter, then a match's article"
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  type_text "teardown"
  settle 2
  shot filter
  click "$MATCH_X" "$MATCH_Y"
  settle 2
  shot article

  echo "== with the terminal in front, the ? brings the same tab forward"
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  shot terminal
  click "$GUIDE_X" "$GUIDE_Y"
  settle 3
  shot again

  echo "== closed, marley: open guide opens it again"
  press "CTRL" w
  settle 2
  shot closed
  palette_command "marley: open guide"
  settle 6
  shot palette

  echo "== in a window with no folder, the system browser"
  palette_command "workspace: new window"
  settle 6
  palette_command "marley: open guide"
  settle 4
  shot fallback
  cat "$E2E_WORK/fallback.txt" 2>/dev/null || echo "(no fallback.txt)"
  expect "the system browser got the guide's file URL" \
    holds "$E2E_WORK/fallback.txt" "file://" "/guide/index.html"
}
