# shellcheck shell=bash
# #507's e2e test: a Chromium and a profile per project. Setup leaves the unit of an earlier build
# running on the one profile every build before #507 used, signed in as `legacy` on a loopback
# site that keeps a login three ways: a cookie, `localStorage` and an IndexedDB record. Marley opens
# alpha, whose first Browser tab stops that unit and takes its profile (REQ-001, REQ-008). beta,
# handed to the running Marley (#513), gets a Chromium of its own, signed out until it signs in as
# `beta`, while alpha stays `legacy` (REQ-002). alpha-wt, a linked worktree of alpha, joins
# alpha's project and its browser (REQ-003). The stand-in agent lists every project's tabs and
# looks at beta's by its id (REQ-009). A quit leaves both units running (REQ-004); with both closed,
# as Marley closes one, a launch brings each project's tab back signed in (REQ-005). Removing beta
# from the rail stops its Chromium and leaves alpha's running (REQ-007). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The rail's first project row, measured from the first run: beta, handed over last, is listed
# above alpha, its terminal's row under it at y 136.
TOP_ROW_X=100
TOP_ROW_Y=95
# A point in the page of the pane's active Browser tab, clear of the page's text.
PAGE_X=1300
PAGE_Y=850

setup() {
  local repo
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  write_login_site site
  SITE=http://127.0.0.1:$(serve_site site)
  write_mcp_agent
  for repo in alpha beta; do
    git init -q -b main "$E2E_WORK/$repo"
    git -C "$E2E_WORK/$repo" -c user.name=e2e -c user.email=e2e@localhost -c commit.gpgsign=false \
      -c core.hooksPath=/dev/null commit -q --allow-empty -m start
  done
  git -C "$E2E_WORK/alpha" worktree add -q -b wt "$E2E_WORK/alpha-wt"
  sign_in_earlier_build
  open_path "$E2E_WORK/alpha"
}

teardown() {
  browser_teardown
}

# The unit an earlier build left running: the same name, flags and profile as Marley gave it
# before #507, signed in as `legacy` through the page. It keeps running, as it outlived that Marley.
sign_in_earlier_build() {
  local data legacy unit port
  data=$(realpath "$E2E_PROFILE")
  legacy=$data/browser/profile
  unit=$(browser_unit_of "$legacy")
  mkdir -p "$data/browser"
  systemd-run --user --quiet --collect --unit="$unit" --description="Marley's browser" \
    --service-type=exec --property=KillMode=mixed --property=TimeoutStopSec=10 -- \
    "$MARLEY_CHROMIUM" --headless --remote-debugging-port=0 --user-data-dir="$legacy" \
    --no-first-run --no-default-browser-check --password-store=basic \
    "$SITE/signin.html?as=legacy"
  for _ in $(seq 150); do
    port=$(head -1 "$legacy/DevToolsActivePort" 2>/dev/null || true)
    if [[ -n $port ]] && curl -s "http://127.0.0.1:$port/json/list" |
      grep -q 'cookie=legacy local=legacy idb=legacy'; then
      echo "the profile of earlier builds is signed in as legacy; $unit is $(systemctl --user is-active "$unit")"
      return 0
    fi
    sleep 0.2
  done
  echo "sign_in_earlier_build: the page did not sign in as legacy" >&2
  return 1
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

unit_is() {
  [[ $(systemctl --user is-active "$2" 2>/dev/null || true) == "$1" ]]
}

# The run's project folders and each one's unit, and the unit of the profile of earlier builds.
units() {
  local data project
  data=$(realpath "$E2E_PROFILE")
  echo "== units, $1"
  for project in "$data"/browser/projects/*/; do
    [[ -d $project ]] || continue
    echo "  project $(basename "$project"): $(browser_unit_of "${project%/}/profile") is $(systemctl --user is-active "$(browser_unit_of "${project%/}/profile")" || true)"
  done
  echo "  earlier builds' $(browser_unit_of "$data/browser/profile") is $(systemctl --user is-active "$(browser_unit_of "$data/browser/profile")" || true)"
}

# The run's browser folders, without the profiles' contents, and each project's project.json.
listing() {
  local file
  echo "== the browser folder"
  (cd "$(realpath "$E2E_PROFILE")" && find browser -maxdepth 3 -not -path '*/profile/*' | sort | sed 's/^/  /')
  for file in "$(realpath "$E2E_PROFILE")"/browser/projects/*/project.json; do
    [[ -f $file ]] || continue
    echo "  $(basename "$(dirname "$file")")/project.json: $(tr -d '\n' <"$file" | tr -s ' ')"
  done
}

project_count() {
  local data
  data=$(realpath "$E2E_PROFILE")
  [[ $(find "$data/browser/projects" -mindepth 1 -maxdepth 1 -type d | wc -l) -eq $1 ]]
}

# The tabs as the agent tools list them, kept in `$E2E_WORK/tabs.txt`.
tabs() {
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
}

# Whether a tab of project `$1` in the newest listing shows `$2`.
tab_of() {
  grep -F "project $1" "$E2E_WORK/tabs.txt" | grep -qF -- "$2"
}

steps() {
  local data alpha beta wt tab_beta
  data=$(realpath "$E2E_PROFILE")
  alpha=$E2E_WORK/alpha
  beta=$E2E_WORK/beta
  wt=$E2E_WORK/alpha-wt
  settle 12
  # Trusts alpha.
  press "" Return
  settle 2
  echo "== alpha's first Browser tab: its Chromium starts on the profile of earlier builds"
  palette "marley: open browser"
  settle 6
  go_to "$SITE/whoami.html"
  shot 507-01-migrated
  units "after alpha's first tab"
  listing
  tabs
  expect "the unit of earlier builds was stopped" unit_is inactive "$(browser_unit_of "$data/browser/profile")"
  expect "the profile of earlier builds moved: browser/profile is gone" test ! -e "$data/browser/profile"
  expect "alpha's folder holds project.json" test -f "$(browser_project_dir "$alpha")/project.json"
  expect "alpha's folder holds the profile" test -d "$(browser_profile "$alpha")"
  expect "project.json names alpha's folder" holds "$(browser_project_dir "$alpha")/project.json" "$(realpath "$alpha")"
  expect "alpha's unit runs" unit_is active "$(browser_unit "$alpha")"
  expect "alpha's tab is signed in as legacy: cookie, localStorage and IndexedDB" \
    tab_of alpha "whoami: cookie=legacy local=legacy idb=legacy"

  echo "== beta, handed to the running Marley: a Chromium and a profile of its own"
  hand_over "$beta"
  settle 4
  # Trusts beta.
  press "" Return
  settle 3
  palette "marley: new browser tab"
  settle 5
  type_text "$SITE/whoami.html"
  press "" Return
  settle 4
  shot 507-02a-beta-signed-out
  tabs
  expect "beta's new tab is signed out: alpha's login is not beta's" \
    tab_of beta "whoami: cookie=none local=none idb=none"
  go_to "$SITE/signin.html?as=beta" 5
  shot 507-02-beta-signed-in
  tabs
  expect "beta's tab signed in as beta" tab_of beta "whoami: cookie=beta local=beta idb=beta"
  expect "beta has a unit of its own" unit_is active "$(browser_unit "$beta")"
  expect "two project folders" project_count 2

  echo "== back in alpha, its tab reloads: still legacy"
  hand_over "$alpha"
  settle 3
  click "$PAGE_X" "$PAGE_Y"
  settle 1
  press "" F5
  settle 4
  shot 507-03-alpha-kept
  tabs
  expect "alpha's tab still reads legacy after beta signed in" \
    tab_of alpha "whoami: cookie=legacy local=legacy idb=legacy"

  echo "== alpha-wt, a linked worktree of alpha: alpha's Chromium, signed in as alpha is"
  hand_over "$wt"
  settle 4
  shot 507-04a-worktree-handed-over
  # Trusts the worktree's folder.
  press "" Return
  settle 3
  palette "marley: new browser tab"
  settle 5
  type_text "$SITE/whoami.html"
  press "" Return
  settle 4
  shot 507-04-worktree-shares
  units "after the worktree's tab"
  listing
  tabs
  expect "the worktree's project folder was never made" test ! -e "$(browser_project_dir "$wt")"
  expect "still two project folders" project_count 2
  expect "alpha's unit and beta's still run" unit_is active "$(browser_unit "$alpha")"
  expect "and beta's" unit_is active "$(browser_unit "$beta")"
  expect "two of alpha's tabs read legacy, the worktree's among them" \
    test "$(grep -F "project alpha" "$E2E_WORK/tabs.txt" | grep -cF "cookie=legacy local=legacy idb=legacy")" -eq 2

  echo "== the agent tools see every project's tabs, and act in a named one's browser"
  tab_beta=$(grep -F "project beta" "$E2E_WORK/tabs.txt" | head -1 | sed -E 's/^  tab ([^:]*):.*/\1/')
  echo "beta's tab: $tab_beta"
  expect "browser_tabs lists beta's tab" test -n "$tab_beta"
  mcp_agent --tab "$tab_beta" look | tee "$E2E_WORK/look.txt"
  expect "a look at beta's tab by its id reads beta's page" \
    holds "$E2E_WORK/look.txt" "\"tab\": \"$tab_beta\"" "cookie=beta local=beta idb=beta"

  echo "== Marley quits: both projects' Chromiums run on"
  quit_marley
  settle 5
  units "after the quit"
  expect "alpha's unit runs after the quit" unit_is active "$(browser_unit "$alpha")"
  expect "beta's unit runs after the quit" unit_is active "$(browser_unit "$beta")"

  echo "== both Chromiums close, as Marley closes one; Marley starts again on alpha, then beta"
  expect "alpha's Chromium closed" browser_close "$alpha"
  expect "beta's Chromium closed" browser_close "$beta"
  units "closed"
  launch_marley
  settle 15
  shot 507-05-alpha-after-restart
  tabs
  expect "alpha's restored tab reopened its page in a new Chromium, still legacy" \
    tab_of alpha "whoami: cookie=legacy local=legacy idb=legacy"
  hand_over "$beta"
  settle 12
  shot 507-06-beta-after-restart
  tabs
  units "after the restart"
  expect "beta's restored tab is signed in as beta in its own new Chromium" \
    tab_of beta "whoami: cookie=beta local=beta idb=beta"

  echo "== beta removed from the rail: its Chromium stops, alpha's runs on"
  shot 507-07a-rail
  click "$TOP_ROW_X" "$TOP_ROW_Y" right
  settle 1
  press "" End
  settle 1
  shot 507-07b-remove-project
  press "" Return
  settle 5
  shot 507-07-removed
  units "five seconds after the removal"
  expect "beta's unit stopped" unit_is inactive "$(browser_unit "$beta")"
  expect "alpha's unit runs on" unit_is active "$(browser_unit "$alpha")"
  listing
}
