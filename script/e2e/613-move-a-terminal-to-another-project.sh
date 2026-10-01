# shellcheck shell=bash
# #613's e2e test: moving a terminal to another project in the rail. `repo` opens with its first
# terminal, which prints `marker-613` and runs `sleep 913`; `repo-b` is handed to the same window
# (#513). The terminal's row dragged onto `repo-b`'s header moves the terminal there, with its
# scrollback and `sleep 913` still running (`dragged`, `pid.txt`, REQ-001, REQ-002). Its menu's
# Move to Project brings it back (`menu`, `moved`, REQ-003). Moved once more and Marley started
# again, it comes back under `repo-b` (`restored`, REQ-005).
compositor sway

# The rail's rows on the 1600 x 1000 output, as the first run drew them: `repo-b` (shown, so
# first) at the top, `repo`'s header and its `sleep 913` row under it.
ROW_X=130
REPO_B_Y=96
SLEEP_Y=229
# The moved row under `repo-b`, and Move to Project's entry below the pointer in its menu.
MOVED_Y=188
MOVE_ENTRY_DY=${MOVE_ENTRY_DY:-34}
# The submenu's `repo` entry, beside Move to Project.
SUBMENU_X=355
SUBMENU_Y=232

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in repo repo-b; do
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

# Drags `repo`'s `sleep 913` row onto `repo-b`'s header, shooting `$1` on the way when given.
drag_to_repo_b() {
  local step
  pointer_to "$ROW_X" "$SLEEP_Y"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6; do
    pointer_to "$ROW_X" "$((SLEEP_Y + (REPO_B_Y - SLEEP_Y) * step / 6))"
    sleep 0.1
  done
  settle 0.5
  if [[ -n ${1:-} ]]; then
    shot "$1"
  fi
  pointer_up
}

# Whether `sleep 913` still runs, into `$1`.
sleep_alive() {
  if pgrep -f 'sleep 913' >/dev/null; then
    echo "sleep 913 runs: pid $(pgrep -f 'sleep 913' | head -n 1)"
  else
    echo "sleep 913 is gone"
  fi | tee "$(shot_file "$1")"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 1
  type_text "echo marker-613"
  press "" Return
  settle 1
  type_text "sleep 913"
  press "" Return
  settle 2

  echo "== repo-b in the same window"
  hand_off "$E2E_WORK/repo-b"
  settle 6
  press "" Return
  settle 3
  shot layout

  echo "== the row dragged onto repo-b's header"
  drag_to_repo_b over
  settle 3
  shot dragged
  sleep_alive pid.txt

  echo "== Move to Project from the row's menu"
  click "$ROW_X" "$MOVED_Y" right
  settle 1
  pointer_to "$((ROW_X + 40))" "$((MOVED_Y + MOVE_ENTRY_DY))"
  settle 1.5
  shot menu
  click "$SUBMENU_X" "$SUBMENU_Y"
  settle 3
  shot moved
  sleep_alive pid-moved.txt

  echo "== moved to repo-b again, then Marley started again"
  drag_to_repo_b
  settle 3
  quit_marley
  open_path ""
  launch_marley
  settle 15
  pointer_to 700 600
  settle 1
  shot restored
}

