# shellcheck shell=bash
# #771's visual check: Codex 0.162.0, the box's own, starts from Marley with its App Server. A
# scratch repo; `MARLEY_CODEX` and the terminals' `codex` name the box's 0.162.0 (mise's), with
# the runner's scratch `CODEX_HOME`, so Codex opens on its sign-in screen and no turn runs;
# `marley.codex_app_server` on; `marley` on the terminals' PATH (#749).
# - `marley agent-cli --provider=codex` in the repo's terminal: Codex's terminal and its agent bar
#   with no chip saying the App Server is off; an `app-server --listen` process runs on the run's
#   socket, and the run's log says no "so Codex starts without one" (`codex`, REQ-001).
compositor sway

# The box's Codex 0.162.0, and the repo's terminal row in the rail.
CODEX_BIN=${CODEX_BIN:-$HOME/.local/share/mise/installs/codex/0.162.0/bin/codex}
TERMINAL_ROW_X=${TERMINAL_ROW_X:-95}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-176}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  if [[ ! -x $CODEX_BIN ]]; then
    echo "no Codex 0.162.0 at $CODEX_BIN" >&2
    return 1
  fi
  # The runner names a scratch CODEX_HOME; Codex refuses one that does not exist.
  mkdir -p "$home" "$bin" "$repo" "$CODEX_HOME"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  printf '# repo\n' >"$repo/README.md"
  ln -s "$(marley_binary)" "$bin/marley"
  ln -s "$CODEX_BIN" "$bin/codex"
  terminal_env HOME "$home"
  terminal_env PATH "$bin:$PATH"
  export MARLEY_CODEX=$CODEX_BIN
  profile_setting marley.codex_app_server true
  open_path "$repo"
}

# Types `$1` into the repo's terminal, which the rail's row brings forward, and runs it.
run_in_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

# Whether an App Server runs on the run's Marley's socket folder.
app_server_runs() {
  local servers
  servers=$(pgrep -af "app-server --listen unix://.*/marley-$(marley_pid)/codex/" || true)
  echo "app servers: ${servers:-none}"
  [[ -n $servers ]]
}

# Whether the run's log has no App Server refusal.
no_refusal() {
  ! grep "so Codex starts without one" "$E2E_PROFILE/logs/Marley.log"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Codex 0.162.0 from the terminal"
  run_in_terminal "marley agent-cli --provider=codex"
  settle 12
  shot 771-01-codex
  expect "an App Server runs for Codex" app_server_runs
  expect "Marley did not start Codex without one" no_refusal
}
