# shellcheck shell=bash
# #662's visual check: Rusty's bookmarks as the Brain view's Favourites, a Page tab's star, and the
# page picker's favourite pages. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`,
# over a scratch state folder (`RUSTY_STAND_IN_STATE`) whose `vault/` holds made-up pages and a
# `.rusty/bookmarks.json` with a folder, a search and a heading, never the user's brain (R-D8). The
# recent pages' scope is deleted from the run's copy of the database, so the picker starts with
# nothing opened. Every change to the list goes through Marley but one: `662-09-outside`'s, written
# from outside as Rusty's app would, which the stand-in's watcher announces.
#
# The group (`662-01-favourites`), a page starred (`662-02-starred`), the picker
# (`662-03-picker`), the folder, search and heading favourites clicked (`662-04-folder`,
# `662-05-search`, `662-06-heading`), a rename (`662-07-renamed`), Ctrl+D (`662-08-ctrl-d`), a
# change from outside (`662-09-outside`) and a removal (`662-10-removed`).
compositor sway

# Where things sit, from the first runs' shots: the rail's Brain button, the Favourites group's
# first row and a row's height, the tree's first row, and the Page tab's star.
RAIL_HEADER_Y=${RAIL_HEADER_Y:-16}
BRAIN_X=${BRAIN_X:-42}
ROW_X=${ROW_X:-110}
ROW_H=${ROW_H:-23}
FIRST_FAVOURITE_Y=${FIRST_FAVOURITE_Y:-141}
FIRST_ROW_Y=${FIRST_ROW_Y:-215}
STAR_X=${STAR_X:-1296}
PAGE_HEADER_Y=${PAGE_HEADER_Y:-82}

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up page `$1` titled `$2` with the tags `$3` (a YAML list), its body from stdin.
vault_page() {
  local path
  path=$(vault)/$1.md
  mkdir -p "$(dirname "$path")"
  { printf -- '---\ntitle: %s\ntags: %s\n---\n# %s\n\n' "$2" "${3:-[]}" "$2"; cat; } >"$path"
}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# Whether the stand-in was asked to run tool `$1` with exactly the arguments `$2`.
called_with() { calls | grep -qF " tools/call $1 $2"; }

# Whether the stand-in's list holds, in order, the bookmark keys `$@`.
bookmarks_are() {
  python3 - "$(vault)/.rusty/bookmarks.json" "$@" <<'BOOKMARKS'
import json, pathlib, sys

def key(bookmark):
    kind = bookmark.get("kind", "")
    if kind == "search":
        return f"search:{bookmark.get('query', '')}"
    if kind == "heading":
        return f"heading:{bookmark.get('path', '')}#{bookmark.get('heading', '')}"
    return f"{kind}:{bookmark.get('path', '')}"

held = [key(bookmark) for bookmark in json.loads(pathlib.Path(sys.argv[1]).read_text())]
print("held:", held)
sys.exit(0 if held == sys.argv[2:] else 1)
BOOKMARKS
}

# The title the stand-in holds for the bookmark keyed `$1`.
bookmark_titled() {
  python3 - "$(vault)/.rusty/bookmarks.json" "$1" "$2" <<'BOOKMARKS'
import json, pathlib, sys

for bookmark in json.loads(pathlib.Path(sys.argv[1]).read_text()):
    if bookmark.get("kind") == "file" and f"file:{bookmark.get('path')}" == sys.argv[2]:
        sys.exit(0 if bookmark.get("title") == sys.argv[3] else 1)
sys.exit(1)
BOOKMARKS
}

favourite_y() { echo $((FIRST_FAVOURITE_Y + $1 * ROW_H)); }

click_favourite() { click "$ROW_X" "$(favourite_y "$1")"; }

right_click_favourite() { click "$ROW_X" "$(favourite_y "$1")" right; }

click_row() { click "$ROW_X" $((FIRST_ROW_Y + $1 * ROW_H)); }

picker() {
  press "CTRL ALT" u
  settle 3
}

# Opens the page `$1` through the picker.
open_page() {
  picker
  type_text "$1"
  settle 2
  press "" Return
  settle 3
}

setup() {
  local bin=$E2E_WORK/bin db
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive" "$(vault)/.rusty"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  echo 'Where the day starts.' | vault_page home "Home"
  echo 'The first note.' | vault_page notes/alpha "Alpha"
  {
    printf '## First\n\n'
    for line in $(seq 1 40); do
      printf 'Line %s of the first part, which runs long enough to push the second off the page.\n\n' "$line"
    done
    printf '## Second\n\nThe part the heading favourite opens at.\n\n'
    for line in $(seq 1 40); do
      printf 'Line %s of the second part, long enough for its heading to reach the top.\n\n' "$line"
    done
    printf '## Third\n\nThe end.\n'
  } | vault_page notes/beta "Beta"
  echo 'The project.' | vault_page projects/atlas "Atlas" "[idea]"
  echo 'A seed of an idea.' | vault_page ideas/seed "Seed" "[idea]"
  echo 'Not an idea.' | vault_page ideas/plain "Plain"
  cat >"$(vault)/.rusty/bookmarks.json" <<'LIST'
[
  {"kind": "folder", "title": "Projects", "path": "projects"},
  {"kind": "search", "title": "Ideas", "query": "tag:idea"},
  {"kind": "heading", "title": "", "path": "notes/beta", "heading": "Second"}
]
LIST
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from scoped_kv_store where namespace = 'marley-rusty-recent-pages'" 2>/dev/null || true
  done
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the group"
  click "$BRAIN_X" "$RAIL_HEADER_Y"
  settle 3
  shot 662-01-favourites
  expect "the list was read" bash -c "grep -q ' tools/call bookmark_list ' '$E2E_WORK/rusty/calls'"

  # The tree: ideas 0, notes 1, projects 2, home 3; notes open: alpha 2, beta 3.
  echo "== a page starred"
  click_row 1
  settle 1
  click_row 2
  settle 3
  click "$STAR_X" "$PAGE_HEADER_Y"
  settle 2
  shot 662-02-starred
  expect "alpha was added" called_with bookmark_add '{"kind": "file", "path": "notes/alpha"}'

  echo "== the picker"
  open_page atlas
  open_page home
  picker
  shot 662-03-picker
  press "" Escape
  settle 1

  echo "== the folder favourite"
  click_favourite 0
  settle 2
  shot 662-04-folder

  echo "== the search favourite"
  click_favourite 1
  settle 2
  shot 662-05-search
  expect "the query was searched" bash -c "grep -q ' tools/call brain_search .*tag:idea' '$E2E_WORK/rusty/calls'"

  echo "== the heading favourite"
  click_favourite 2
  settle 3
  shot 662-06-heading

  echo "== a rename"
  right_click_favourite 3
  settle 1
  press "" Return
  settle 1
  type_text "Alpha notes"
  press "" Return
  settle 2
  shot 662-07-renamed
  expect "the list was sent with the new title" bookmark_titled file:notes/alpha "Alpha notes"
  expect "the order held" bookmarks_are folder:projects search:tag:idea 'heading:notes/beta#Second' file:notes/alpha

  echo "== Ctrl+D"
  open_page alpha
  press CTRL d
  settle 2
  shot 662-08-ctrl-d
  expect "alpha was removed" bookmarks_are folder:projects search:tag:idea 'heading:notes/beta#Second'

  echo "== a change from outside"
  cat >"$(vault)/.rusty/bookmarks.json" <<'LIST'
[
  {"kind": "file", "title": "Alpha from outside", "path": "notes/alpha"},
  {"kind": "search", "title": "Ideas", "query": "tag:idea"},
  {"kind": "folder", "title": "Projects", "path": "projects"}
]
LIST
  settle 4
  shot 662-09-outside

  # Alpha 0, the search 1, the folder 2.
  echo "== a removal"
  right_click_favourite 1
  settle 1
  press "" Down
  press "" Return
  settle 2
  shot 662-10-removed
  expect "the search was removed" bookmarks_are file:notes/alpha folder:projects
  calls | grep -E 'bookmark_'
}
