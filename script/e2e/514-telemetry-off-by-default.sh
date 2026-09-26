# shellcheck shell=bash
# #514's e2e test: telemetry off by default. The profile copy's settings point `server_url` at a
# dead local port first, so nothing this run records can leave the box. Part one, with no
# telemetry setting: Marley opens and is used, and its `telemetry.log` holds no event; the
# Settings window, searched for "telemetry", shows both toggles off. Part two: with
# `telemetry.metrics` set true, a relaunched Marley records events again (its posts then fail
# against the dead port).
compositor sway

LOG=

# Adds `"<key>": <json>` as the first entry of the profile copy's settings. Zed's settings file
# is JSON with comments, so the entry goes in as text after the opening brace.
settings_prepend() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import sys, pathlib
path, key, value = sys.argv[1], sys.argv[2], sys.argv[3]
p = pathlib.Path(path)
t = p.read_text()
i = t.index("{")
p.write_text(t[: i + 1] + f"\n  \"{key}\": {value}," + t[i + 1 :])
PY
}

# Prints the telemetry events in the profile's log, one type per line.
events() {
  if [[ -s $LOG ]]; then
    python3 -c 'import json,sys
for line in open(sys.argv[1]):
    line = line.strip()
    if line:
        print(json.loads(line).get("event_type"))' "$LOG"
  fi
}

setup() {
  LOG=$E2E_PROFILE/logs/telemetry.log
  settings_prepend server_url '"http://127.0.0.1:9"'
  git init -q -b telemetry "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  local pid count
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== part one: no telemetry setting"
  press "CTRL SHIFT" p
  settle 1
  press "" Escape
  settle 4
  count=$(events | grep -c . || true)
  echo "telemetry events recorded: $count"
  if [[ $count -ne 0 ]]; then
    echo "telemetry recorded events with no setting on:" >&2
    events >&2
    return 1
  fi
  press "CTRL" comma
  settle 3
  type_text "telemetry"
  settle 2
  shot 514-01-privacy-off
  echo "== part two: telemetry.metrics set true"
  pid=$(marley_pid)
  kill -TERM "$pid"
  for _ in $(seq 30); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 1
  done
  settings_prepend telemetry '{"metrics": true}'
  launch_marley
  settle 10
  press "CTRL SHIFT" p
  settle 1
  press "" Escape
  settle 4
  count=$(events | grep -c . || true)
  echo "telemetry events recorded: $count ($(events | sort -u | paste -sd ',' -))"
  if [[ $count -eq 0 ]]; then
    echo "telemetry.metrics true recorded no event" >&2
    return 1
  fi
}
