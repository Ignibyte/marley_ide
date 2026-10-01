# shellcheck shell=bash
# #632's visual check: Marley runs the harness's runtime itself. `MARLEY_RH` names the harness's
# built `bin/rh` (never built here) and `marley.embedded_harness` is on; Marley serves a root in
# its own data folder, which is the run's profile.
#
# The Harness section reads connected with no sessions (`632-01-connected`, REQ-001); an actor
# started on that root gets a row (`632-02-session`, REQ-002); with the root refused and `serve`
# killed, the header says the runtime stopped and why, the row kept and stale (`632-03-stopped`);
# with the root allowed again, Marley's next `serve` brings it back with the actor's row, which its
# tmux kept (`632-04-back`, REQ-003).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

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

# The harness's CLI on the root Marley serves, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$E2E_PROFILE/harness" "$@"; }

setup() {
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # Marley's runtime is started from Marley's own environment.
  export MARLEY_RH=$RH
  unset TMUX TMUX_PANE
  set_setting marley.embedded_harness true
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '{"version": 1, "steps": [{"step": "publish", "state": "running", "value": {"note": "busy"}}, {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000}]}\n' \
    >"$E2E_WORK/worker.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local root=$E2E_PROFILE/harness ws
  [[ -d $root ]] || return 0
  chmod 0700 "$root"
  for ws in $(rh list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
    rh stop "$ws" >/dev/null 2>&1
  done
  rh shutdown --stop-backend >/dev/null 2>&1
  # Marley may have served the root again after the shutdown; its tmux goes with the profile.
  tmux -S "$root/tmux.sock" kill-server >/dev/null 2>&1
  return 0
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4
  pointer_to 900 400
  settle 1
  shot 632-01-connected

  echo "== a session on Marley's root"
  rh actor new worker --script "$E2E_WORK/worker.json" --cwd "$E2E_WORK" >/dev/null
  settle 4
  shot 632-02-session

  echo "== the runtime stopped"
  chmod 0755 "$E2E_PROFILE/harness"
  pkill -f "state $E2E_PROFILE/harness serve"
  settle 4
  shot 632-03-stopped

  echo "== the runtime back"
  chmod 0700 "$E2E_PROFILE/harness"
  settle 20
  shot 632-04-back
}
