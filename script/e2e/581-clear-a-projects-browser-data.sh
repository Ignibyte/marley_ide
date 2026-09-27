# shellcheck shell=bash
# #581's e2e test: Clear Browser Data for one project. Marley opens alpha, which signs in as
# `alpha`; beta, handed to the running Marley (#513), signs in as `beta` and opens a second tab.
# From beta's row in the rail, Clear Browser Data… asks first, naming beta (REQ-001), and Escape
# changes nothing (REQ-002). Clear closes beta's tabs, closes and stops its Chromium and deletes its
# profile, keeping project.json, and says so (REQ-003); beta's next tab is signed out (REQ-004)
# while alpha stays signed in (REQ-005). A clear that cannot remove the profile says why
# (REQ-006). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# beta's project header in the rail: beta, handed over last, is listed above alpha
# (L-claude-507-the-rails-project-row-is-its-header-001).
BETA_ROW_X=100
BETA_ROW_Y=95
# A point in beta's pane, clear of any page's text.
PANE_X=1300
PANE_Y=850

setup() {
  local repo
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  write_login_site site
  SITE=http://127.0.0.1:$(serve_site site)
  write_mcp_agent
  for repo in alpha beta; do
    git init -q -b main "$E2E_WORK/$repo"
  done
  open_path "$E2E_WORK/alpha"
}

teardown() {
  # REQ-006 makes beta's folder read-only; the run's cleanup must remove it.
  chmod -R u+w "$(realpath "$E2E_PROFILE")/browser" 2>/dev/null || true
  browser_teardown
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Hands `path` to the running Marley, as a second launch does (#513).
hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

# Goes to `url` in the focused Browser tab.
go_to() {
  press CTRL l
  settle 1
  type_text "$1"
  press "" Return
  settle "${2:-4}"
}

# Opens a new Browser tab in the active workspace and goes to `url`.
new_tab_at() {
  palette "marley: new browser tab"
  settle 5
  type_text "$1"
  press "" Return
  settle "${2:-4}"
}

unit_is() {
  [[ $(systemctl --user is-active "$2" 2>/dev/null || true) == "$1" ]]
}

# The tabs as the agent tools list them, kept in `$E2E_WORK/tabs.txt`.
tabs() {
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
}

# Whether a tab of project `$1` in the newest listing shows `$2`.
tab_of() {
  grep -F "project $1" "$E2E_WORK/tabs.txt" | grep -qF -- "$2"
}

# How many tabs of project `$1` in the newest listing show `$2`.
tabs_of() {
  grep -F "project $1" "$E2E_WORK/tabs.txt" | grep -cF -- "$2" || true
}

# The run's browser folders, without the profiles' contents.
listing() {
  echo "== the browser folder"
  (cd "$(realpath "$E2E_PROFILE")" && find browser -maxdepth 4 -not -path '*/profile/*/*' | sort | sed 's/^/  /')
}

# Opens beta's menu from its header and selects Clear Browser Data…, the entry above Remove
# Project, the last.
clear_menu() {
  click "$BETA_ROW_X" "$BETA_ROW_Y" right
  settle 1
  press "" End
  settle 1
  press "" Up
  settle 1
}

steps() {
  local alpha beta project
  alpha=$E2E_WORK/alpha
  beta=$E2E_WORK/beta
  project=$(browser_project_dir "$beta")
  settle 12
  # Trusts alpha.
  press "" Return
  settle 2
  echo "== alpha signs in as alpha"
  palette "marley: open browser"
  settle 6
  go_to "$SITE/signin.html?as=alpha" 5
  echo "== beta, handed over, signs in as beta and opens a second tab"
  hand_over "$beta"
  settle 4
  # Trusts beta.
  press "" Return
  settle 3
  new_tab_at "$SITE/signin.html?as=beta" 5
  new_tab_at "$SITE/whoami.html"
  tabs
  expect "beta's two tabs read beta" test "$(tabs_of beta "cookie=beta local=beta idb=beta")" -eq 2
  expect "alpha's tab reads alpha" tab_of alpha "whoami: cookie=alpha local=alpha idb=alpha"

  echo "== Clear Browser Data… from beta's row asks first"
  clear_menu
  shot 581-01-menu
  press "" Return
  settle 1
  shot 581-02-asked
  echo "== Escape changes nothing"
  press "" Escape
  settle 2
  shot 581-03-cancelled
  tabs
  expect "beta's unit still runs" unit_is active "$(browser_unit "$beta")"
  expect "beta's two tabs still read beta" test "$(tabs_of beta "cookie=beta local=beta idb=beta")" -eq 2
  expect "beta's profile is still there" test -d "$(browser_profile "$beta")"

  echo "== Clear: beta's tabs close, its Chromium stops, its profile goes"
  clear_menu
  press "" Return
  settle 1
  # Clear, the prompt's first button.
  press "" Return
  settle 6
  shot 581-04-cleared
  tabs
  listing
  expect "no Browser tab of beta is left" test "$(tabs_of beta "project beta")" -eq 0
  expect "beta's unit stopped" unit_is inactive "$(browser_unit "$beta")"
  expect "beta's profile is gone" test ! -e "$(browser_profile "$beta")"
  expect "beta's project.json is kept" test -f "$project/project.json"
  expect "alpha's unit runs on" unit_is active "$(browser_unit "$alpha")"

  echo "== beta's next tab is signed out; alpha's is not"
  click "$PANE_X" "$PANE_Y"
  settle 1
  new_tab_at "$SITE/whoami.html"
  shot 581-05-beta-signed-out
  tabs
  expect "beta's new tab is signed out" tab_of beta "whoami: cookie=none local=none idb=none"
  expect "alpha's tab still reads alpha" tab_of alpha "whoami: cookie=alpha local=alpha idb=alpha"
  expect "beta has a new profile" test -d "$(browser_profile "$beta")"

  echo "== a clear that cannot remove the profile says why"
  chmod a-w "$project"
  clear_menu
  press "" Return
  settle 1
  press "" Return
  settle 6
  shot 581-06-failed
  listing
  expect "the profile could not be removed" test -d "$(browser_profile "$beta")"
  expect "project.json is kept" test -f "$project/project.json"
  expect "beta's unit stopped all the same" unit_is inactive "$(browser_unit "$beta")"
  chmod u+w "$project"
}
