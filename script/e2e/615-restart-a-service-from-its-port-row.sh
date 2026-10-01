# shellcheck shell=bash
# #615's e2e test: a service's port row: its menu, Restart, its state and its logs. A user unit
# of the scenario's own serves the scratch project on a free port from `serve.sh`, which waits
# four seconds before it listens, and exits 1 at once when its flag file is gone.
#
# The row's menu has Restart Service, Stop Service and Show Logs (`menu`, REQ-001). Restart keeps
# the row while the port is quiet, then the new process listens (`restarting`, `restarted`,
# REQ-002). With the flag gone, a restart fails, and the row says so (`failed`, REQ-003). Show
# Logs opens the unit's journal in a terminal (`logs`, REQ-004).
compositor sway

UNIT=""
PORT=""

# The rail on the 1600 x 1000 output: the service's port row, and its menu's entries.
ROW_X=130
ROW_Y=188
ENTRY_X=190
RESTART_Y=264
LOGS_Y=310

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  UNIT=marley-e2e-615-$PORT
  touch "$E2E_WORK/serve-flag"
  cat >"$E2E_WORK/serve.sh" <<SERVE
#!/bin/sh
# #615's service: no flag, no server.
[ -e "$E2E_WORK/serve-flag" ] || exit 1
sleep 4
exec python3 -m http.server $PORT --bind 127.0.0.1
SERVE
  chmod +x "$E2E_WORK/serve.sh"
  /usr/bin/systemd-run --user --quiet --unit="$UNIT" --working-directory="$E2E_WORK/repo" \
    "$E2E_WORK/serve.sh"
  echo "the unit $UNIT serves port $PORT"
  open_path "$E2E_WORK/repo"
}

teardown() {
  /usr/bin/systemctl --user stop "$UNIT" 2>/dev/null || true
  /usr/bin/systemctl --user reset-failed "$UNIT" 2>/dev/null || true
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 6
  shot row
  click "$ROW_X" "$ROW_Y" right
  settle 1.5
  shot menu

  echo "== Restart Service"
  click "$ENTRY_X" "$RESTART_Y"
  settle 2
  shot restarting
  settle 7
  pointer_to "$ROW_X" "$ROW_Y"
  settle 2
  shot restarted

  echo "== a restart that fails"
  rm -f "$E2E_WORK/serve-flag"
  click "$ROW_X" "$ROW_Y" right
  settle 1.5
  click "$ENTRY_X" "$RESTART_Y"
  settle 6
  pointer_to 700 600
  settle 1
  shot failed
  # `is-active` ends non-zero for a failed unit, which is what this step expects.
  { /usr/bin/systemctl --user is-active "$UNIT" || true; } | tee "$(shot_file state.txt)"

  echo "== Show Logs"
  click "$ROW_X" "$ROW_Y" right
  settle 1.5
  click "$ENTRY_X" "$LOGS_Y"
  settle 5
  shot logs
}
