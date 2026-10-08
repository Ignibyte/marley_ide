# shellcheck shell=bash
# #690's visual check: a harness session watched in a terminal. A scratch harness (its built
# `bin/rh`, as #534 and #689 use it) serves one actor, `worker`, which prints three lines and
# sleeps. `marley.harness` is `rh --state <root> mcp --grant write` and `marley.harness_writes` is
# on. In worker's tab, Views lists native and tmux, each with Open and Copy (`690-01-views`,
# REQ-001); Open on tmux starts `rh attach` in a new terminal, which shows the actor's pane
# (`690-02-terminal`, REQ-002).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
# Where the row and the controls sit, from the first runs' shots; 0 stops the run after the shot
# that measures them.
WORKER_Y=${WORKER_Y:-303}
VIEWS_X=${VIEWS_X:-1553}
VIEWS_Y=${VIEWS_Y:-120}
OPEN_X=${OPEN_X:-353}
OPEN_Y=${OPEN_Y:-207}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

# `$1` as the JSON array of its bytes, for an actor's `output` step.
bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

setup() {
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh690-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  local tries=50
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$ROOT/worker.json" <<JSON
{"version": 1, "steps": [
  {"step": "publish", "state": "running", "value": {"note": "busy"}},
  {"step": "output", "bytes": $(bytes $'one from the pane\ntwo from the pane\nthree from the pane\n')},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000}]}
JSON
  rh actor new worker --script "$ROOT/worker.json" --cwd "$ROOT" >/dev/null
  profile_setting marley.harness "{\"command\": \"$RH\", \"args\": [\"--state\", \"$ROOT\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harness_writes true
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  [[ -f $E2E_WORK/root ]] || return 0
  ROOT=$(cat "$E2E_WORK/root")
  chmod 0700 "$ROOT" 2>/dev/null
  local ws
  for ws in $(rh list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
    rh stop "$ws" >/dev/null 2>&1
  done
  rh shutdown --stop-backend >/dev/null 2>&1
  rm -rf "$ROOT"
  return 0
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4
  pointer_to 900 400
  settle 1
  shot 690-00-rail
  if ((WORKER_Y == 0)); then
    echo "the row's place is not set yet: the rail shot only"
    return 0
  fi

  echo "== the views, each with Open"
  click 120 "$WORKER_Y"
  settle 3
  click "$VIEWS_X" "$VIEWS_Y"
  settle 3
  shot 690-01-views
  if ((OPEN_X == 0)); then
    echo "Open's place is not set yet: the views shot only"
    return 0
  fi

  echo "== Open on tmux"
  click "$OPEN_X" "$OPEN_Y"
  settle 6
  shot 690-02-terminal
}
