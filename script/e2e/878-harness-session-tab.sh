# shellcheck shell=bash
# #878's visual check: a harness session's tab as the MoreTabs board draws it. A scratch harness
# (its built `bin/rh`, never built here) on a short root serves `worker`, which works, and `asker`,
# which waits on "Which base branch?" with `main` and `release`; the run's settings name `rh
# --state <root> mcp --grant write` as `marley.harness`, with writes on.
# - The asker's row clicked: its tab's header with "waiting", the question card with main and
#   release, the Send row (`878-01-asker`, REQ-001).
# - Views asked for: the views as chips in the header (`878-02-views`, REQ-001).
# - Marley Day (`878-03-day`, REQ-001).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
STOP_AT=${STOP_AT:-0}
# The asker's row in the rail and the tab's Views button, from the shots.
ASKER_Y=${ASKER_Y:-427}
VIEWS_X=${VIEWS_X:-1505}
VIEWS_Y=${VIEWS_Y:-945}

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
  ROOT=$XDG_RUNTIME_DIR/rh878-$$
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
  for actor in worker asker; do
    rh actor new "$actor" --script "$ROOT/$actor.json" --cwd "$ROOT" >/dev/null
  done
  set_setting marley.harness "{\"command\": \"$RH\", \"args\": [\"--state\", \"$ROOT\", \"mcp\", \"--grant\", \"write\"]}"
  set_setting marley.harness_writes true
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

  echo "== the asker's tab"
  click 120 "$ASKER_Y"
  settle 4
  pointer_to 900 500
  settle 1
  shot 878-01-asker
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== its views"
  click "$VIEWS_X" "$VIEWS_Y"
  settle 4
  pointer_to 900 500
  settle 1
  shot 878-02-views

  echo "== Marley Day"
  set_setting theme '"Marley Day"'
  settle 4
  shot 878-03-day
}
