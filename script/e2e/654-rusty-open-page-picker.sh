# shellcheck shell=bash
# #654's visual check: `rusty: open page`, a picker over every page of Rusty's brain. `marley_rusty`'s
# stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder whose `vault/`
# holds seven made-up pages with their files' times set, so the stand-in's `brain_list_pages`
# lists them newest first in a known order; never the user's brain (R-D8). The recent pages' scope
# and the shortcut note's row for `rusty::OpenPage` are deleted from the run's copy of the
# database, so the run starts with nothing opened.
#
# The list (`654-01-listed`), a title match (`654-02-by-title`), a slug match (`654-03-by-slug`),
# an exact slug (`654-04-exact`), Enter (`654-05-opened`), the recent pages first
# (`654-06-recent`), back to a shown tab (`654-07-back`), the create row (`654-08-create-row`),
# made (`654-09-created`), refused (`654-10-refused`), an unresolved link that makes its page
# (`654-11-link-created`, `654-12-link-resolved`) and one refused (`654-13-link-refused`), the
# command palette (`654-14-palette`, `654-15-from-palette`), Rusty not connected
# (`654-16-not-connected`) and off (`654-17-off`).
compositor sway

# Where Atlas's two unresolved links sit, from the first run's shots.
LATER_X=${LATER_X:-397}
LATER_Y=${LATER_Y:-319}
GONE_X=${GONE_X:-405}
GONE_Y=${GONE_Y:-347}

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up page `$1` titled `$2`, its file's time `$3` minutes past noon, its body from stdin.
vault_page() {
  local path
  path=$(vault)/$1.md
  mkdir -p "$(dirname "$path")"
  { printf -- '---\ntitle: %s\n---\n# %s\n\n' "$2" "$2"; cat; } >"$path"
  touch -d "2026-10-01 12:$3:00" "$path"
}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# Whether the stand-in was asked for `brain_new_page` with path `$1`, folder `$2` and name `$3`.
made_with() {
  calls | python3 -c '
import json, sys
path, folder, name = sys.argv[1:]
for line in sys.stdin:
    parts = line.split(" ", 3)
    if len(parts) == 4 and parts[1] == "tools/call" and parts[2] == "brain_new_page":
        arguments = json.loads(parts[3])
        if arguments == {"path": path, "folder": folder, "name": name}:
            sys.exit(0)
sys.exit(1)' "$1" "$2" "$3"
}

# The pid of the stand-in Marley connects to: the one that read the settings last.
marleys_pid() { calls | grep ' tools/call settings_list ' | tail -1 | awk '{print $1}'; }

picker() {
  press "CTRL ALT" u
  settle 3
}

query() {
  press CTRL a
  type_text "$1"
  settle 2
}

setup() {
  local bin=$E2E_WORK/bin db
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  echo 'Where the day starts.' | vault_page home "Home" 59
  echo 'Sam keeps the renderer honest.' | vault_page people/sam "Sam" 58
  echo 'What the quarter brought.' | vault_page notes/2026-q3 "Quarterly review" 57
  echo 'One renderer for every Markdown view.' | vault_page decisions/use-zeds-renderer "Use Zed's renderer" 56
  printf 'The project.\n\n- Later: [[ideas/later]]\n- Gone: [[archive/gone]]\n' | vault_page projects/atlas "Atlas" 55
  echo "Marley's shell." | vault_page projects/marley/shell "Shell" 54
  echo 'A seed of an idea.' | vault_page ideas/seed "Seed" 53
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from scoped_kv_store where namespace = 'marley-rusty-recent-pages'; delete from scoped_kv_store where namespace = 'marley-shortcut-note' and key = 'rusty::OpenPage'" 2>/dev/null || true
  done
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the list"
  picker
  shot 654-01-listed
  expect "the list was read once, every page asked for" test "$(calls | grep -c ' tools/call brain_list_pages {"limit": 100000}')" = 1

  echo "== matching"
  query "quart rev"
  shot 654-02-by-title
  query "peop sam"
  shot 654-03-by-slug
  query "people/sam"
  shot 654-04-exact

  echo "== opening"
  query "quart"
  press "" Return
  settle 3
  shot 654-05-opened

  echo "== the recent pages"
  picker
  query "sam"
  press "" Return
  settle 3
  picker
  query "atlas"
  press "" Return
  settle 3
  picker
  shot 654-06-recent

  echo "== back to a shown page"
  press "" Return
  settle 3
  shot 654-07-back

  echo "== making a page"
  picker
  query "projects/marley/review"
  shot 654-08-create-row
  press "" Return
  settle 3
  shot 654-09-created
  expect "the page was made at its path" made_with projects/marley/review projects/marley review

  echo "== a refused page"
  picker
  query "archive/old"
  press "" Return
  settle 3
  shot 654-10-refused
  expect "nothing was made under archive" test ! -e "$(vault)/archive/old.md"
  press "" Escape
  settle 1

  echo "== an unresolved link"
  picker
  query "atlas"
  press "" Return
  settle 3
  click "$LATER_X" "$LATER_Y"
  settle 3
  shot 654-11-link-created
  expect "the link's page was made" made_with ideas/later ideas later
  press ALT Left
  settle 3
  shot 654-12-link-resolved
  click "$GONE_X" "$GONE_Y"
  settle 3
  shot 654-13-link-refused

  echo "== the command palette"
  press "CTRL SHIFT" p
  settle 1
  type_text "open page"
  settle 2
  shot 654-14-palette
  press "" Return
  settle 3
  shot 654-15-from-palette
  press "" Escape
  settle 1

  echo "== not connected"
  mv "$E2E_WORK/bin/rusty-mcp" "$E2E_WORK/bin/rusty-mcp.away"
  kill "$(marleys_pid)"
  settle 10
  picker
  shot 654-16-not-connected

  echo "== off"
  profile_setting marley.rusty.enabled false
  settle 4
  picker
  shot 654-17-off
  calls | grep -E 'brain_list_pages|brain_new_page'
}
