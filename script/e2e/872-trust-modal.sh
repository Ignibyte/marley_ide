# shellcheck shell=bash
# #872's visual check: Zed's trust modal as the Dialogs board draws it. Two scratch repositories,
# neither trusted; the first opens at start, the second is handed over later (#513).
# - At start: "Trust repo?" over the board's line, Zed's restricted-mode list and its scope
#   choice, Not Now quiet and Trust primary at the right (`872-01-night`, REQ-001).
# - Enter: the card gone, the project trusted (`872-02-trusted`, REQ-002).
# - Marley Day, the second folder handed over: its card (`872-03-day`, REQ-001).
compositor sway

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in repo second; do
    mkdir -p "$E2E_WORK/$name"
    printf '# %s\n' "$name" >"$E2E_WORK/$name/README.md"
    git init -q -b main "$E2E_WORK/$name"
  done
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
}

# Hands `path` to the running Marley, as a second launch does (#513).
hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

steps() {
  settle 15
  pointer_to 700 150
  settle 1
  shot 872-01-night

  echo "== Enter"
  press "" Return
  settle 4
  shot 872-02-trusted

  echo "== Marley Day, a second folder"
  profile_setting theme '"Marley Day"'
  settle 3
  hand_over "$E2E_WORK/second"
  settle 6
  pointer_to 700 150
  settle 1
  shot 872-03-day
}
