# shellcheck shell=bash
# #689's visual check: the harness's write verbs from Marley. A scratch harness (its built
# `bin/rh`, as #534 uses it) serves two actors: `asker` waits on "Which base branch?" with `main`
# and `release`, `listener` waits for a message. The run's settings name `rh --state <root> mcp
# --grant write` as `marley.harness` and turn `marley.harness_writes` on; a 0600 profile
# `greeter` declares a third actor.
#
# The asker's tab shows its question with a button per option (`689-01-question`, REQ-001); main
# answers it (`689-02-answered`, REQ-002); text sent from the listener's tab is its wait's message
# (`689-03-sent`, REQ-003); Views lists the commands that watch it (`689-04-views`, REQ-004); and
# `marley: open harness session` opens greeter (`689-05-picker`, `689-06-opened`, REQ-005).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
# Where the rows sit in the rail, and the tab's controls, from the first run's shots; 0 stops the
# run after the shot that measures them.
ASKER_Y=${ASKER_Y:-424}
# The listener's row once the asker's tab has added its group to the rail.
LISTENER_Y=${LISTENER_Y:-631}
MAIN_X=${MAIN_X:-502}
MAIN_Y=${MAIN_Y:-120}
# The send line in a tab with no question above it, and the asker's tab in the tab bar.
EDITOR_X=${EDITOR_X:-850}
EDITOR_Y=${EDITOR_Y:-120}
ASKER_TAB_X=${ASKER_TAB_X:-401}
ASKER_TAB_Y=${ASKER_TAB_Y:-66}
VIEWS_X=${VIEWS_X:-1553}
VIEWS_Y=${VIEWS_Y:-120}

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
  ROOT=$XDG_RUNTIME_DIR/rh689-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  local tries=50
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$ROOT/asker.json" <<JSON
{"version": 1, "steps": [
  {"step": "publish", "state": "blocked", "value": {"question": {"key": "base",
    "prompt": "Which base branch?", "options": ["main", "release"]}}},
  {"step": "wait", "key": "base"},
  {"step": "output", "bytes": $(bytes $'chose it\n')},
  {"step": "publish", "state": "running", "value": {"note": "chosen"}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  cat >"$ROOT/listener.json" <<JSON
{"version": 1, "steps": [
  {"step": "output", "bytes": $(bytes $'listening\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "wait", "key": "message"},
  {"step": "output", "bytes": $(bytes $'got it\n')},
  {"step": "exit", "code": 0}]}
JSON
  cat >"$ROOT/greeter.json" <<JSON
{"version": 1, "steps": [
  {"step": "output", "bytes": $(bytes $'hello from greeter\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  mkdir -m 0700 "$ROOT/profiles"
  printf '{"version": 1, "kind": "actor", "script": "%s", "cwd": "%s"}\n' "$ROOT/greeter.json" \
    "$ROOT" >"$ROOT/profiles/greeter.json"
  chmod 0600 "$ROOT/profiles/greeter.json"
  for actor in asker listener; do
    rh actor new "$actor" --script "$ROOT/$actor.json" --cwd "$ROOT" >/dev/null
  done
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

# The fleet's session titled `$1`, as one line of JSON.
seat_of() {
  rh fleet | python3 -c 'import json, sys
fleet = json.load(sys.stdin)
seats = fleet.get("seats", fleet if isinstance(fleet, list) else [])
for seat in seats:
    if seat.get("title", "").startswith(sys.argv[1]):
        print(json.dumps({key: seat.get(key) for key in ("id", "title", "state", "question")}))
        break' "$1"
}

# Whether the asker no longer waits on its question.
asker_answered() {
  local seat
  seat=$(seat_of asker)
  echo "  asker: $seat"
  [[ -n $seat ]] && python3 -c 'import json, sys; sys.exit(0 if json.loads(sys.argv[1]).get("question") is None else 1)' "$seat"
}

# Whether the listener took the message and ended.
listener_heard() {
  local seat
  seat=$(seat_of listener)
  echo "  listener: $seat"
  [[ -n $seat ]] && python3 -c 'import json, sys; sys.exit(0 if json.loads(sys.argv[1]).get("state") == "done" else 1)' "$seat"
}

# Whether the fleet holds a session opened from greeter.
greeter_opened() {
  local seat
  seat=$(seat_of greeter)
  echo "  greeter: $seat"
  [[ -n $seat ]]
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4
  pointer_to 900 400
  settle 1
  shot 689-00-rail
  if ((ASKER_Y == 0)); then
    echo "the rows' places are not set yet: the rail shot only"
    return 0
  fi

  echo "== the asker's question"
  click 120 "$ASKER_Y"
  settle 3
  shot 689-01-question

  # The rail's rows move once a question is answered, so the listener's tab opens first and the
  # asker's comes back from the tab bar.
  echo "== text sent to the listener"
  click 120 "$LISTENER_Y"
  settle 3
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  type_text "hello harness"
  press "" Return
  settle 4
  shot 689-03-sent
  expect "the listener took the message" listener_heard

  echo "== main answers it"
  click "$ASKER_TAB_X" "$ASKER_TAB_Y"
  settle 2
  click "$MAIN_X" "$MAIN_Y"
  settle 4
  shot 689-02-answered
  expect "the asker's question is answered" asker_answered

  echo "== views"
  click "$VIEWS_X" "$VIEWS_Y"
  settle 3
  shot 689-04-views

  echo "== a session opened from a profile"
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: open harness session"
  settle 1
  press "" Return
  settle 2
  type_text "greeter"
  settle 2
  shot 689-05-picker
  press "" Return
  settle 5
  shot 689-06-opened
  expect "a greeter session is in the fleet" greeter_opened
}
