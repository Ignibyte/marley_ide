# shellcheck shell=bash
# #574's e2e test: a browser tool that names no tab acts in the caller's project. Marley opens
# repo-a; a second launch hands it repo-b (#513), which opens in the same window. A stand-in agent,
# run in each project's terminal through the Claude Code plugin's bridge, calls the browser tools
# with no tab:
# - in repo-b's terminal, `navigate` opens page one in a new tab of repo-b, which the user then
#   focuses: the tab the user focused last;
# - in repo-a's terminal, `look` is refused with its next step while repo-a has no tab (REQ-003);
#   `navigate` opens page two in a new tab of repo-a and leaves repo-b's tab on page one
#   (REQ-002); `look` then reads repo-a's tab, not the one the user focused last (REQ-001); and
#   `tabs` gives each tab's project, repo-a's marked `default`, repo-b's `focused` (REQ-004);
# - from repo-b's folder with no terminal id and no project, as Zed's agents' bridge runs,
#   `navigate` moves repo-b's tab to page two (REQ-005);
# - from the repository's root, in no project, `navigate` moves the tab the user focused last,
#   repo-b's, to page three, as before this ticket (REQ-006).
# Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The rail's row of each project's terminal, a point in the terminal's pane and one in repo-b's
# page, measured from the first run: the rail lists repo-b, handed over last, above repo-a.
RAIL_A_X=100
RAIL_A_Y=229
RAIL_B_X=100
RAIL_B_Y=136
TERMINAL_X=500
TERMINAL_Y=500
PAGE_X=1250
PAGE_Y=500

setup() {
  local home=$E2E_WORK/home page
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  for page in one two three; do
    cat >"$E2E_WORK/site/$page.html" <<HTML
<!doctype html><html><head><title>Page $page</title></head>
<body style="margin:0;font:28px sans-serif;background:#eef3fb"><h1 style="margin:40px">Page $page</h1></body></html>
HTML
  done
  SITE=http://127.0.0.1:$(serve_site site)
  write_mcp_agent
  cat >"$home/.bashrc" <<RC
PS1='\$ '
agent() {
  BRIDGE="$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \\
    MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \\
    python3 "$E2E_WORK/mcp-agent.py" "\$@" 2>&1 | tee -a "$E2E_WORK/agent.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/agent.log"
  git init -q -b one "$E2E_WORK/repo-a"
  git init -q -b two "$E2E_WORK/repo-b"
  open_path "$E2E_WORK/repo-a"
}

teardown() {
  browser_teardown
}

# The stand-in from the harness, in folder `$1`, with no terminal id and no project: the bridge
# sends only the folder.
from_folder() {
  local folder=$1
  shift
  (
    cd "$folder" || exit 1
    MARLEY_TERMINAL_ID='' MARLEY_PROJECT='' \
      BRIDGE="$OLDPWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \
      MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \
      python3 "$E2E_WORK/mcp-agent.py" "$@"
  )
}

# Types a command into the focused terminal, waits, and prints what the agent printed since.
run_in_terminal() {
  local before
  before=$(wc -l <"$E2E_WORK/agent.log" 2>/dev/null || echo 0)
  type_text "$1"
  press "" Return
  settle "${2:-4}"
  echo "\$ $1"
  tail -n +"$((before + 1))" "$E2E_WORK/agent.log" 2>/dev/null || true
}

# The id in the newest `navigate` answer of the agent's log.
navigated_tab() {
  grep -F '"did": "went to' "$E2E_WORK/agent.log" | tail -1 |
    sed -E 's/.*"tab": "([^"]*)".*/\1/' || true
}

# Whether the newest line of `$1` about tab `$2` holds each of the rest.
tab_line() {
  local file=$1 tab=$2 line text
  shift 2
  line=$(grep -F "  tab $tab:" "$file" | tail -1 || true)
  [[ -n $line ]] || return 1
  for text in "$@"; do
    [[ $line == *"$text"* ]] || return 1
  done
}

# Whether the newest `look` answer in the agent's log is of tab `$1` and holds `$2`.
looked_at() {
  local line
  line=$(grep -F '"viewport"' "$E2E_WORK/agent.log" | tail -1 || true)
  [[ $line == *"\"tab\": \"$1\""* && $line == *"$2"* ]]
}

steps() {
  local tab_a tab_b
  settle 12
  # Trusts repo-a.
  press "" Return
  settle 2
  echo "== repo-b, handed to the running Marley"
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$E2E_WORK/repo-b" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null || true
  settle 4
  # Trusts repo-b.
  press "" Return
  settle 3
  shot 574-00-handed-off
  echo "== in repo-b's terminal, navigate with no tab: a new tab in repo-b"
  run_in_terminal "agent navigate $SITE/one.html" 5
  tab_b=$(navigated_tab)
  echo "repo-b's tab: $tab_b"
  shot 574-00b-b-navigated
  # The user focuses repo-b's page: the tab the user focused last.
  click "$PAGE_X" "$PAGE_Y"
  settle 1
  shot 574-01-two-projects
  echo "== in repo-a's terminal"
  click "$RAIL_A_X" "$RAIL_A_Y"
  settle 2
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  shot 574-01b-in-a
  run_in_terminal "agent look" 4
  expect "a look with no tab from repo-a, which has none, is refused with its next step" \
    holds "$E2E_WORK/agent.log" "browser_look refused" \
    "no Browser tab of the project repo-a shows a page; browser_navigate opens one there"
  run_in_terminal "agent navigate $SITE/two.html" 5
  tab_a=$(navigated_tab)
  echo "repo-a's tab: $tab_a"
  shot 574-02-scoped-navigate
  expect "repo-a's navigate opened a tab of its own" test -n "$tab_a" -a "$tab_a" != "$tab_b"
  run_in_terminal "agent look" 4
  expect "a look from repo-a reads repo-a's tab, not the one the user focused last" \
    looked_at "$tab_a" "two.html"
  run_in_terminal "agent tabs" 4
  shot 574-03-tabs
  expect "tabs: repo-a's tab is its project's and the default" \
    tab_line "$E2E_WORK/agent.log" "$tab_a" "two.html, project repo-a" ", default"
  expect "tabs: repo-b's tab is its project's, still on page one, the one focused last" \
    tab_line "$E2E_WORK/agent.log" "$tab_b" "one.html, project repo-b, focused"
  echo "== from repo-b's folder, no terminal id, no project"
  from_folder "$E2E_WORK/repo-b" navigate "$SITE/two.html" | tee "$E2E_WORK/cwd.txt"
  expect "the folder's navigate went to repo-b's tab" holds "$E2E_WORK/cwd.txt" "\"tab\": \"$tab_b\""
  click "$RAIL_B_X" "$RAIL_B_Y"
  settle 2
  shot 574-04-cwd
  echo "== from the repository's root, in no project"
  MARLEY_TERMINAL_ID='' MARLEY_PROJECT='' mcp_agent navigate "$SITE/three.html" | tee "$E2E_WORK/outside.txt"
  MARLEY_TERMINAL_ID='' MARLEY_PROJECT='' mcp_agent tabs | tee "$E2E_WORK/outside-tabs.txt"
  expect "a caller in no project drives the tab the user focused last, as before" \
    holds "$E2E_WORK/outside.txt" "\"tab\": \"$tab_b\""
  expect "and nothing else moved" tab_line "$E2E_WORK/outside-tabs.txt" "$tab_a" "two.html"
  expect "with no project, the default is the tab focused last" \
    tab_line "$E2E_WORK/outside-tabs.txt" "$tab_b" "three.html" "focused, default"
}
