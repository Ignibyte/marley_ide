# shellcheck shell=bash
# #618's e2e test: a port row's lines at the default rail width, and a closed header's tooltip
# against its menu. A user unit with a long name serves the scratch project `repo` (#615's way).
# Its row shows the host and port and the unit's name's end (`row`, REQ-001); with the pointer on
# it, Open, Copy and Stop lie over its end and the text stays (`hover`, REQ-002); its tooltip
# starts with the whole URL (`tooltip`, REQ-001). A second launch hands Marley `repo-b` (#513);
# with `repo` shown, Marley starts again with no path, so `repo-b` is closed. Its header's tooltip
# shows (`header-tooltip`), and a right-click on it, the pointer still, opens its menu with the
# tooltip gone (`menu`, REQ-003).
compositor sway

UNIT=""
PORT=""

# The rail on the 1600 x 1000 output: the service's port row (#615's), and after the restart
# `repo-b`'s closed header.
ROW_X=${ROW_X:-130}
ROW_Y=${ROW_Y:-188}
REPO_B_Y=${REPO_B_Y:-96}

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in repo repo-b; do
    mkdir -p "$E2E_WORK/$name"
    printf '# %s\n' "$name" >"$E2E_WORK/$name/README.md"
  done
  PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  UNIT=marley-e2e-618-a-service-with-a-long-name-$PORT
  /usr/bin/systemd-run --user --quiet --unit="$UNIT" --working-directory="$E2E_WORK/repo" \
    python3 -m http.server "$PORT" --bind 127.0.0.1
  echo "the unit $UNIT serves port $PORT"
  open_path "$E2E_WORK/repo"
}

teardown() {
  /usr/bin/systemctl --user stop "$UNIT" 2>/dev/null || true
  /usr/bin/systemctl --user reset-failed "$UNIT" 2>/dev/null || true
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
  settle 6
  pointer_to 700 600
  settle 1
  shot row

  echo "== the pointer on the port row"
  pointer_to "$ROW_X" "$ROW_Y"
  settle 0.3
  shot hover
  settle 2
  shot tooltip

  echo "== repo-b handed over, then Marley started again with repo shown"
  hand_off "$E2E_WORK/repo-b"
  settle 6
  press "" Return
  settle 3
  hand_off "$E2E_WORK/repo"
  settle 5
  quit_marley
  open_path ""
  launch_marley
  settle 15
  pointer_to 700 600
  settle 1
  shot restart

  echo "== repo-b's tooltip, then its menu"
  pointer_to "$ROW_X" "$REPO_B_Y"
  settle 2
  shot header-tooltip
  click "$ROW_X" "$REPO_B_Y" right
  settle 1.5
  shot menu
}
