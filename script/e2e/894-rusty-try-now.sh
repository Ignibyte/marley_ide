# shellcheck shell=bash
# #894's visual check: Rusty dropped, with Try Now. Rusty is on through a wrapper that fails while
# `$E2E_WORK/rusty-down` exists and otherwise runs `marley_rusty`'s stand-in `rusty-mcp`.
# - `rusty: open home` while it fails: Home's Rusty card, the reason and Try Now (`894-01-dropped`,
#   REQ-001), and in Marley Day (`894-03-day`, REQ-001).
# - Seventy seconds on, Marley waits 32 s before its next try (each try takes up to 5 s, the waits
#   double from 1 s); the flag gone and Try Now clicked: Rusty connected 3 s later, which only Try
#   Now explains (`894-02-back`, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-0}
# Try Now on Home's Rusty card, from the first run's shot.
TRY_X=${TRY_X:-468}
TRY_Y=${TRY_Y:-606}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  touch "$E2E_WORK/rusty-down"
  cat >"$bin/rusty-mcp" <<SH
#!/bin/sh
# Fails while the flag exists, so Marley's Rusty reads as dropped.
if [ -e "$E2E_WORK/rusty-down" ]; then
  echo "rusty-mcp: down for the scenario" >&2
  exit 1
fi
exec "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "\$@"
SH
  chmod +x "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  profile_setting marley.rusty '{"enabled": true}'
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Rusty dropped: Home's card"
  palette "rusty: open home" 4
  pointer_to 1500 950
  settle 1
  shot 894-01-dropped
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 894-03-day

  echo "== seventy seconds on, Rusty answering, Try Now"
  settle 70
  rm -f "$E2E_WORK/rusty-down"
  click "$TRY_X" "$TRY_Y"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 894-02-back
}
