# shellcheck shell=bash
# #576's e2e test: a Browser tab's saved row survives another workspace's tab of the same item id.
# Three launches, Marley's Chromium stopped after each so a restored tab opens its saved URL: the
# first on repo-a, where the harness's agent opens page A in a Browser tab; the second on repo-b,
# where it opens page B; the third on repo-a again, whose tab must come back on page A. Item ids
# repeat across launches, so the two tabs may be saved under the same one; to make sure of it, the
# run saves repo-b's tab again under repo-a's item id before the third launch, then prints the
# scratch workspaces' saved rows and the table's schema. Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The Browser tab in the split's right pane, measured from the first run.
TAB_X=930
TAB_Y=51

setup() {
  local page
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  for page in a b; do
    cat >"$E2E_WORK/site/$page.html" <<HTML
<!doctype html><html><head><title>Page $page</title></head>
<body style="margin:0;font:28px sans-serif;background:#f3eefb"><h1 style="margin:40px">Page $page</h1></body></html>
HTML
  done
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b tabs "$E2E_WORK/repo-a"
  git init -q -b tabs "$E2E_WORK/repo-b"
  open_path "$E2E_WORK/repo-a"
}

teardown() {
  browser_teardown
}

# The scratch workspaces' saved Browser tab rows, and the table's schema: the profile is a copy of
# the user's, so only this run's workspaces are read.
saved_tabs() {
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 -readonly "$db" "select 'row', t.workspace_id, t.item_id, t.url from marley_browser_tabs t join workspaces w on w.workspace_id = t.workspace_id where w.paths like '%$E2E_WORK%'; select 'schema', sql from sqlite_master where name = 'marley_browser_tabs';" 2>/dev/null || true
  done
}

# Saves repo-b's tab again under repo-a's tab's item id, with the statement the app saves a tab
# with, as a launch whose item ids met would: on the old table it replaced repo-a's row. It runs
# on the run's own copy of the profile while Marley is stopped, and does nothing when the ids met
# already.
collide() {
  local db a_item b_workspace
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    a_item=$(sqlite3 -readonly "$db" "select t.item_id from marley_browser_tabs t join workspaces w on w.workspace_id = t.workspace_id where w.paths like '%$E2E_WORK/repo-a%' limit 1" 2>/dev/null || true)
    b_workspace=$(sqlite3 -readonly "$db" "select t.workspace_id from marley_browser_tabs t join workspaces w on w.workspace_id = t.workspace_id where w.paths like '%$E2E_WORK/repo-b%' limit 1" 2>/dev/null || true)
    [[ -n $a_item && -n $b_workspace ]] || continue
    echo "saving repo-b's tab again under repo-a's item id $a_item"
    sqlite3 "$db" "insert or replace into marley_browser_tabs(item_id, workspace_id, target_id, url, title) select $a_item, workspace_id, target_id, url, title from marley_browser_tabs where workspace_id = $b_workspace limit 1"
  done
}

# Quits Marley and stops its Chromiums, one per project since #507, so the next launch starts
# them with no pages.
quit_all() {
  local repo
  quit_marley
  for repo in repo-a repo-b; do
    systemctl --user stop "$(browser_unit "$E2E_WORK/$repo")" 2>/dev/null || true
  done
  settle 2
}

steps() {
  settle 12
  # Trusts repo-a.
  press "" Return
  settle 2
  echo "== repo-a: a Browser tab on page A"
  mcp_agent navigate "$SITE/a.html"
  settle 3
  shot 576-01-a
  quit_all
  echo "== repo-b: a Browser tab on page B"
  open_path "$E2E_WORK/repo-b"
  launch_marley
  settle 12
  # Trusts repo-b.
  press "" Return
  settle 2
  mcp_agent navigate "$SITE/b.html"
  settle 3
  shot 576-02-b
  quit_all
  collide
  saved_tabs | tee "$E2E_WORK/rows.txt"
  echo "== repo-a again"
  open_path "$E2E_WORK/repo-a"
  launch_marley
  settle 15
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
  # A click on the restored tab, as #494's scenario does, brings its page's frames.
  click "$TAB_X" "$TAB_Y"
  settle 3
  shot 576-03-a-again
  expect "repo-a's Browser tab came back on page A" \
    holds "$E2E_WORK/tabs.txt" "a.html, project repo-a"
  expect "the table keys a row by workspace and item, with no unique item id" \
    bash -c "grep -q '^schema' '$E2E_WORK/rows.txt' && ! grep '^schema' '$E2E_WORK/rows.txt' | grep -q UNIQUE"
}
