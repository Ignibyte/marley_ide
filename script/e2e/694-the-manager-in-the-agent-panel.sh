# shellcheck shell=bash
# #694's visual check: the manager in the Agent Panel. A scratch harness (its built `bin/rh`) runs
# one actor, `boss`, designated the root's manager with `rh manager`, which waits for a message.
# `marley.harness` is `rh --state <root> mcp --grant write` and `marley.harness_writes` is on, so
# Marley adds a Manager entry running `rh --state <root> acp`. New Agent Thread's submenu lists it
# (`694-01-menu`, REQ-001); its thread, once `rh acp` is checked to be what runs, takes "hello
# manager", which lands in the thread and reaches boss (`694-02-thread`, REQ-002).
compositor sway

# The project's +, and how many steps down New Agent Thread's submenu Manager sits, from the first
# run's shot; 0 stops after the menu shot.
PLUS_X=${PLUS_X:-224}
PLUS_Y=${PLUS_Y:-123}
MANAGER_STEPS=${MANAGER_STEPS:-2}

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

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
  ROOT=$XDG_RUNTIME_DIR/rh694-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  local tries=50
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$ROOT/boss.json" <<JSON
{"version": 1, "steps": [
  {"step": "publish", "state": "idle", "value": {}},
  {"step": "wait", "key": "message"},
  {"step": "output", "bytes": $(bytes $'got it\n')},
  {"step": "publish", "state": "idle", "value": {}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  rh actor new boss --script "$ROOT/boss.json" --cwd "$ROOT" >/dev/null
  local id tries=20
  until [[ -n ${id:-} ]] || ((tries-- == 0)); do
    id=$(rh fleet | python3 -c 'import json, sys
for seat in json.load(sys.stdin).get("seats", []):
    if seat.get("title") == "boss":
        print(seat["id"])')
    [[ -n $id ]] || sleep 0.5
  done
  rh manager "$id" >/dev/null
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

# Whether `rh acp` runs on the scratch root: nothing is typed into a thread before (PR-687).
acp_runs() {
  pgrep -af -- "--state $(cat "$E2E_WORK/root") acp"
}

# Whether the thread holds the person's "hello manager", and boss took it.
message_reached_the_manager() {
  rh thread show | tee "$E2E_WORK/thread.json" | python3 -c 'import json, sys
thread = json.load(sys.stdin)
records = thread.get("records", [])
for record in records:
    print("  ", record.get("author"), record.get("kind"), repr(record.get("text")), "handoff" if record.get("handoff") else "")
sys.exit(0 if any(r.get("author") == "person" and r.get("text") == "hello manager" for r in records) else 1)'
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the submenu lists Manager"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" Down
  press "" Down
  press "" Right
  settle 1
  shot 694-01-menu
  if ((MANAGER_STEPS == 0)); then
    echo "Manager's place is not set yet: the menu shot only"
    press "" Escape
    return 0
  fi

  echo "== the Manager thread takes a message"
  for ((step = 0; step < MANAGER_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  expect "the thread runs rh acp on the scratch root" acp_runs
  type_text "hello manager"
  press "" Return
  settle 8
  shot 694-02-thread
  expect "the message is in the thread" message_reached_the_manager
}
