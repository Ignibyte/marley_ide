# shellcheck shell=bash
# #909's visual check: the bottom row for Marley's own screens. A fresh profile opens `repo`;
# Rusty is on through `marley_rusty`'s stand-in `rusty-mcp` (a scratch vault with today's note
# bookmarked, a decision whose follow-up was due yesterday, a list of three tasks); a broken
# template sits in the profile's `components` folder; Codex's folder is left off Marley's PATH; a
# stand-in `claude` sits first on the terminals' PATH (PR-687, F-claude-903); a local page logs
# one console error.
# - `909-01-threads`: the Threads page (REQ-001).
# - `909-02-browser`: the page in a Browser tab: title, address, "1 console error", recording
#   (REQ-002).
# - `909-03-mcp`: MCP Servers: "Codex is not on the PATH", "marley · N tools" (REQ-003).
# - `909-04-components`: the Component Library: the broken template and its error, the counts
#   (REQ-004).
# - `909-05-tasks`: Rusty's Tasks on the Home list: "1 decision overdue", "3 open" (REQ-005).
# - `909-06-graph`: Rusty's Graph: scope, depth, pages, the embedding (REQ-006).
# - `909-07-brain-page`: today's note: its slug, "★ favourite", "Rusty connected" (REQ-007).
# - `909-08-day`: the Browser tab's row in Marley Day (REQ-002, REQ-008).
compositor sway

. script/e2e/browser-fixture.sh

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
TODAY=2026-10-25

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo vault=$E2E_WORK/rusty/vault
  mkdir -p "$home" "$bin" "$repo" "$vault/notes" "$vault/decisions" "$vault/.rusty" "$E2E_WORK/site"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/claude" <<'FAKE'
#!/bin/sh
# A stand-in agent CLI: it says what it was asked, and waits.
printf 'stand-in claude was asked: %s\n' "$*"
exec sleep 600
FAKE
  chmod +x "$bin/claude"
  # Codex's own folder off Marley's PATH, so MCP Servers finds it missing.
  PATH=$(printf '%s' "$bin:$PATH" | tr ':' '\n' | grep -v '/installs/codex/' | paste -sd:)
  export PATH
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' "$TODAY" >"$E2E_WORK/rusty/today"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.tmux_hosts '[]'
  printf '# A note\n\nA page in the scratch vault.\n' >"$vault/notes/a-note.md"
  printf -- '---\ntitle: Ship the bottom row\nstatus: decided\ndecided: 2026-10-15\nfollow_up_by: 2026-10-24\n---\n# Ship the bottom row\n\nA made-up decision.\n' \
    >"$vault/decisions/ship-the-bottom-row.md"
  printf '[{"kind": "file", "path": "daily/%s", "title": "Today"}]\n' "$TODAY" \
    >"$vault/.rusty/bookmarks.json"
  cat >"$E2E_WORK/rusty/tasks.json" <<'JSON'
{"groups": [
  {"id": 1, "name": "Home", "tasks": [
    {"id": 11, "title": "Water the plants"},
    {"id": 12, "title": "Call the plumber"},
    {"id": 13, "title": "Return the library books"}]}
]}
JSON
  mkdir -p "$E2E_PROFILE/config/components"
  printf '{"name": "quota",\n "kind": \n' >"$E2E_PROFILE/config/components/quota.json"
  printf '<!doctype html><title>Console fixture</title><body><h1>A page</h1>\n<script>console.error("the fixture failed");</script></body>\n' \
    >"$E2E_WORK/site/index.html"
  SITE_PORT=$(serve_site site)
  echo "$SITE_PORT" >"$E2E_WORK/site.port"
  offline_chromium
  git init -q -b main "$repo"
  open_path "$repo"
}

teardown() {
  browser_teardown
  return 0
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 4

  echo "== the Threads page"
  palette "marley: open threads" 4
  away
  shot 909-01-threads

  echo "== a Browser tab"
  palette "marley: open browser" 6
  agent navigate "http://127.0.0.1:$(cat "$E2E_WORK/site.port")/"
  settle 4
  away
  shot 909-02-browser

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 909-08-day
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== MCP Servers"
  palette "marley: open mcp servers" 4
  away
  shot 909-03-mcp

  echo "== the Component Library"
  palette "marley: open component library" 4
  away
  shot 909-04-components

  echo "== Rusty's Tasks"
  palette "rusty: open tasks" 6
  away
  shot 909-05-tasks

  echo "== Rusty's Graph"
  palette "rusty: open graph" 6
  away
  shot 909-06-graph

  echo "== today's note"
  launch_marley_with rusty today
  settle 6
  away
  shot 909-07-brain-page
}
