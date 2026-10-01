# shellcheck shell=bash
# #534's visual check: the harness's sessions in the rail, read side. A scratch harness (its
# built `bin/rh`, never built here) serves three actors: `worker` works and goes quiet, `asker`
# waits on "Which base branch?" with `main` and `release`, `crasher` fails. The run's settings
# name `rh --state <root> mcp` as `marley.harness`, and a working session is quiet after a minute.
#
# The Harness section lists the three, the asker's question on its row (`534-01-group`, REQ-001
# to REQ-003); a click on worker opens its tab with its three lines (`534-02-view`, REQ-006); the
# asker's question is in the inbox (`534-04-inbox`, REQ-008); `rh actor send` answers it, and its
# row and open tab move on (`534-03-answered`, REQ-004, REQ-007); worker reads as quiet
# (`534-05-stale`, REQ-005); with `rh mcp` killed and its root refused, the header says not
# running and the rows stay, marked stale (`534-06-down`), and with the root allowed again the
# retry reconnects (`534-07-back`, REQ-009).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
# Where the rows sit in the rail, from the shots.
WORKER_Y=${WORKER_Y:-446}
ASKER_Y=${ASKER_Y:-492}

# Sets the settings key path `$1` (dot-separated) to the JSON value `$2` in the run's copy of the
# settings.
set_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

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
  ROOT=$XDG_RUNTIME_DIR/rh534-$$
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
  {"step": "output", "bytes": $(bytes $'one\ntwo\nthree\n')},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000}]}
JSON
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
  cat >"$ROOT/crasher.json" <<JSON
{"version": 1, "steps": [
  {"step": "output", "bytes": $(bytes $'giving up\n')},
  {"step": "exit", "code": 3}]}
JSON
  for actor in worker asker crasher; do
    rh actor new "$actor" --script "$ROOT/$actor.json" --cwd "$ROOT" >/dev/null
  done
  set_setting marley.harness "{\"command\": \"$RH\", \"args\": [\"--state\", \"$ROOT\", \"mcp\"]}"
  set_setting marley.no_update_after_minutes 1
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
  shot 534-01-group
  if ((WORKER_Y == 0)); then
    echo "the rows' places are not set yet: the group shot only"
    return 0
  fi

  echo "== worker's tab"
  click 120 "$WORKER_Y"
  settle 3
  shot 534-02-view

  echo "== the inbox"
  shot 534-04-inbox

  echo "== the asker answered"
  click 120 "$ASKER_Y"
  settle 2
  local inc
  inc=$(rh actor inspect asker | python3 -c 'import json, sys; print(json.load(sys.stdin)["actors"][0]["incarnations"][-1]["id"])')
  rh actor send asker --incarnation "$inc" --key base --delivery "$(python3 -c 'import uuid; print(uuid.uuid4())')" main >/dev/null
  settle 5
  shot 534-03-answered

  echo "== quiet"
  settle 50
  pointer_to 900 420
  settle 2
  shot 534-05-stale

  echo "== down"
  chmod 0755 "$ROOT"
  pkill -f "state $ROOT mcp"
  settle 9
  shot 534-06-down

  echo "== back"
  chmod 0700 "$ROOT"
  settle 20
  shot 534-07-back
}
