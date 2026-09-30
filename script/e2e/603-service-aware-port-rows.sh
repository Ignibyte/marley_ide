# shellcheck shell=bash
# #603's e2e test: port rows that know a service. The scratch project `repo` and two transient user
# services in its folder, each a `python3 -m http.server` with `Restart=always`:
# `marley-e2e-603` on 38603 and `marley-e2e-603-refused` on 38604. A `systemctl` first on Marley's
# PATH refuses to stop the second, as polkit does, and hands every other call to the real one. Each
# row's second line ends with its unit (`row`, REQ-001), and its tooltip names a user service
# (`row-tooltip`). Stop's tooltip says it stops the user service (`tooltip`, REQ-005). Stop on the
# first takes its row away for good (`stopped`, `state.txt`, REQ-002); Stop on the second shows the
# refusal's toast (`refused`, REQ-003), whose Copy Command copies the command (`clipboard.txt`).
compositor sway

ROW_X=${ROW_X:-110}
# The first port row, two lines tall, under `repo`'s header and its terminal, and the second under
# it; Stop at the row's end; the toast's Copy Command. As the first run's shots found them.
FIRST_Y=${FIRST_Y:-188}
SECOND_Y=${SECOND_Y:-246}
STOP_X=${STOP_X:-234}
COPY_X=${COPY_X:-1199}
COPY_Y=${COPY_Y:-933}

UNITS=(marley-e2e-603 marley-e2e-603-refused)

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # polkit's refusal for the second unit; everything else to the real systemctl.
  cat >"$E2E_WORK/bin/systemctl" <<'FAKE'
#!/usr/bin/env bash
if [[ " $* " == *" stop "* && " $* " == *" marley-e2e-603-refused"* ]]; then
  echo "Failed to stop marley-e2e-603-refused.service: Interactive authentication required." >&2
  echo "See system logs and 'systemctl status marley-e2e-603-refused.service' for details." >&2
  exit 1
fi
exec /usr/bin/systemctl "$@"
FAKE
  chmod +x "$E2E_WORK/bin/systemctl"
  export PATH="$E2E_WORK/bin:$PATH"
  local port=38603 unit
  for unit in "${UNITS[@]}"; do
    /usr/bin/systemd-run --user --quiet --unit="$unit" --working-directory="$E2E_WORK/repo" \
      --property=Restart=always --property=RestartSec=2 \
      python3 -m http.server "$port" --bind 127.0.0.1
    port=$((port + 1))
  done
  open_path "$E2E_WORK/repo"
}

teardown() {
  /usr/bin/systemctl --user stop "${UNITS[@]}" 2>/dev/null || true
  /usr/bin/systemctl --user reset-failed "${UNITS[@]}" 2>/dev/null || true
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== the rows, with their units"
  pointer_to "$ROW_X" "$FIRST_Y"
  settle 2
  shot row
  settle 2
  shot row-tooltip

  echo "== Stop's tooltip"
  pointer_to "$STOP_X" "$FIRST_Y"
  settle 2
  shot tooltip

  echo "== Stop on marley-e2e-603, then past its restart delay"
  click "$STOP_X" "$FIRST_Y"
  settle 8
  pointer_to 700 500
  settle 2
  shot stopped
  /usr/bin/systemctl --user is-active marley-e2e-603 | tee "$(shot_file state.txt)" || true
  expect "marley-e2e-603 stays stopped" \
    test "$(/usr/bin/systemctl --user is-active marley-e2e-603)" = inactive

  echo "== Stop on marley-e2e-603-refused, refused"
  # The first row went, so the second is where the first was.
  pointer_to "$ROW_X" "$FIRST_Y"
  settle 1
  click "$STOP_X" "$FIRST_Y"
  settle 3
  shot refused

  echo "== Copy Command"
  click "$COPY_X" "$COPY_Y"
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline | tee "$(shot_file clipboard.txt)" || true
  echo
  expect "the command is on the clipboard" \
    test "$(WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline)" = \
    "systemctl --user stop marley-e2e-603-refused.service"
  /usr/bin/systemctl --user is-active marley-e2e-603-refused | tee "$(shot_file refused-state.txt)" || true
}
