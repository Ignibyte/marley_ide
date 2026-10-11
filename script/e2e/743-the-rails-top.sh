# shellcheck shell=bash
# #743's visual check: the Rusty group leaves the rail, and Home becomes Default. The installed
# build (#700's Rusty group) runs first on the run's profile with Rusty on through its stand-in,
# shows its Rusty group and quits; the debug build then restores that session with no path.
# - The installed build's rail: Home, then Rusty, on top (`old`).
# - The debug build's restored rail: Default first, no Rusty row, Default shown (`restored`,
#   REQ-001, REQ-003, REQ-004).
# - A Rusty screen from the palette while the repo shows opens as a repo tab (`tasks-here`,
#   REQ-002).
# - Rusty's home page carries Open Page… and the three captures (`rusty-home`, REQ-005), and Open
#   Page… opens the page picker (`open-page`).
compositor sway

# The restored repo's row, closed under Default, and the home page's Open Page… button, as the
# first run's shots found them.
REPO_ROW_X=${REPO_ROW_X:-51}
REPO_ROW_Y=${REPO_ROW_Y:-182}
OPEN_PAGE_X=${OPEN_PAGE_X:-340}
OPEN_PAGE_Y=${OPEN_PAGE_Y:-164}

# The installed release build, which still makes a Rusty group.
INSTALLED=${INSTALLED:-$HOME/.local/lib/marley/marley}
binary "$INSTALLED"

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
settings.setdefault("marley", {})["rusty"] = {"enabled": True}
settings["restore_on_startup"] = "last_session"
json.dump(settings, open(path, "w"), indent=2)
PY
  open_path "$E2E_WORK/repo"
}

palette() {
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
  # #700 makes Home and Rusty once nothing restored can still bring them back.
  settle 6

  echo "== the installed build shows its Rusty group"
  palette "rusty: open home"
  settle 4
  shot 743-01-old

  echo "== it quits, and the debug build restores that session"
  quit_marley
  # The debug build, on the same profile, with no path, so the last session comes back.
  use_binary "${CARGO_TARGET_DIR:-target}/debug/marley"
  open_path ""
  launch_marley
  settle 20
  shot 743-02-restored

  echo "== a Rusty screen opens where the user is"
  click "$REPO_ROW_X" "$REPO_ROW_Y"
  settle 6
  palette "rusty: open tasks"
  settle 4
  shot 743-03-tasks-here

  echo "== Rusty's home page holds the forms"
  palette "rusty: open home"
  settle 4
  shot 743-04-rusty-home
  click "$OPEN_PAGE_X" "$OPEN_PAGE_Y"
  settle 3
  shot 743-05-open-page
}
