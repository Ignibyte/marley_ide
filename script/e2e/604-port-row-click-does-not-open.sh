# shellcheck shell=bash
# #604's e2e test: a click on a port row opens nothing. The scratch project `repo`, and a
# `python3 -m http.server` started in its terminal, which gives it a port row. One click on the
# row marks it and opens no Browser tab (`single`, REQ-001); Enter then opens the URL in a Browser
# tab (`enter`, REQ-002). With the terminal shown again (`terminal`), a double-click on the row
# brings the tab back (`double`, REQ-003).
compositor sway

ROW_X=${ROW_X:-110}
# The terminal's row and the port's row under it, and the port's row once the Browser tab's row
# sits between them, as the first run's shots found them.
TERMINAL_Y=${TERMINAL_Y:-136}
PORT_Y=${PORT_Y:-182}
PORT_BELOW_Y=${PORT_BELOW_Y:-228}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '<!doctype html><title>Port page</title><h1>Served from repo</h1>\n' \
    >"$E2E_WORK/repo/index.html"
  open_path "$E2E_WORK/repo"
}

# Two clicks at `$1`,`$2`, close enough together to count as one double-click.
double_click() {
  pointer_to "$1" "$2"
  sleep 0.05
  pointer_down
  sleep 0.05
  pointer_up
  sleep 0.08
  pointer_down
  sleep 0.05
  pointer_up
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a server in the project's terminal"
  click "$ROW_X" "$TERMINAL_Y"
  settle 1
  type_text "python3 -m http.server 38605 --bind 127.0.0.1"
  press "" Return
  settle 6

  echo "== one click on its row"
  click "$ROW_X" "$PORT_Y"
  settle 3
  shot single

  echo "== Enter"
  press "" Return
  settle 10
  shot enter

  echo "== the terminal again, then a double-click on the port's row"
  click "$ROW_X" "$TERMINAL_Y"
  settle 2
  shot terminal
  double_click "$ROW_X" "$PORT_BELOW_Y"
  settle 4
  shot double
}
