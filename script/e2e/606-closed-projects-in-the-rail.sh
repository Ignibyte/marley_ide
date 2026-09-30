# shellcheck shell=bash
# #606's e2e test: a window's closed projects stay in the rail. Marley opens `repo`, and second
# launches hand it `repo-b` and `repo-c` (#513); `repo-b`'s terminal moves into its folder `sub`,
# and `repo` is shown when Marley quits (`before`). Started again with no path, Zed reopens only
# `repo`: the rail lists `repo-b` and `repo-c` dimmed, header only (`restart`, REQ-001), and a
# dimmed header's tooltip says it is not open (`tooltip`, REQ-002). A click on `repo-b` opens it
# with its terminal back in `sub` (`opened`, REQ-003). `repo-c`'s right-click menu (`menu`)
# removes it (`removed`, REQ-005).
compositor sway

ROW_X=${ROW_X:-110}
# As the first run's shots found them: the top project's terminal row once a hand-off put it
# first, `repo`'s header before the quit, the two dimmed headers after the restart, and Remove
# Project in `repo-c`'s menu.
TOP_TERMINAL_Y=${TOP_TERMINAL_Y:-136}
REPO_Y=${REPO_Y:-330}
REPO_B_Y=${REPO_B_Y:-143}
REPO_C_Y=${REPO_C_Y:-96}
REMOVE_X=${REMOVE_X:-170}
REMOVE_Y=${REMOVE_Y:-195}

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home" "$E2E_WORK/repo-b/sub"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in repo repo-b repo-c; do
    mkdir -p "$E2E_WORK/$name"
    printf '# %s\n' "$name" >"$E2E_WORK/$name/README.md"
  done
  open_path "$E2E_WORK/repo"
}

# A second launch on this profile with `$1`, which hands it to the running Marley (#513).
hand_off() {
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== repo-b, with its terminal in sub"
  hand_off "$E2E_WORK/repo-b"
  settle 5
  press "" Return
  settle 3
  click "$ROW_X" "$TOP_TERMINAL_Y"
  settle 1
  type_text "cd sub"
  press "" Return
  settle 2

  echo "== repo-c"
  hand_off "$E2E_WORK/repo-c"
  settle 5
  press "" Return
  settle 3
  shot three

  echo "== repo shown, then quit and start again with no path"
  click "$ROW_X" "$REPO_Y"
  settle 3
  shot before
  quit_marley
  open_path ""
  launch_marley
  settle 15
  pointer_to 700 600
  settle 1
  shot restart

  echo "== a dimmed header's tooltip"
  pointer_to "$ROW_X" "$REPO_B_Y"
  settle 2
  shot tooltip

  echo "== a click opens repo-b"
  click "$ROW_X" "$REPO_B_Y"
  settle 8
  pointer_to 700 600
  settle 1
  shot opened

  echo "== repo-c's menu, and Remove Project"
  click "$ROW_X" "$REPO_C_Y" right
  settle 1
  shot menu
  click "$REMOVE_X" "$REMOVE_Y"
  settle 4
  shot removed
}
