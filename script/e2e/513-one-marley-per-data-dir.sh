# shellcheck shell=bash
# #513's e2e test: one Marley per data directory. The run's Marley opens repo-a. Then, as
# Omarchy's menu or a terminal would, the harness starts the same binary on the same profile:
# - with repo-b: it hands repo-b to the running Marley and exits within 15 seconds, one Marley
#   runs on the profile, and a terminal opens at repo-b's root, as a folder opened for the
#   first time gets (#455), seen through Marley's MCP server (REQ-001);
# - with no path, while the Settings window has the focus: it exits, and the running Marley asks
#   the compositor to activate its window: an `xdg_activation_v1.activate` request in its Wayland
#   trace, which the run turns on (REQ-002). Whether the compositor then focuses it is the
#   compositor's policy: Omarchy's Hyprland does (`focus_on_activate`), and this sway, which
#   checks the token against the seat's focus, did not; the tree goes to the log;
# - on a second profile with repo-c: it starts beside the first (REQ-003);
# - after the first is killed with SIGKILL, its socket file left behind: a launch on the same
#   profile starts (REQ-004).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local repo home=$E2E_WORK/home
  # The run's Marley prints its Wayland requests, so the log shows its activation requests.
  export WAYLAND_DEBUG=client
  mkdir -p "$home"
  echo "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  for repo in repo-a repo-b repo-c; do
    git init -q -b "$repo" "$E2E_WORK/$repo"
  done
  open_path "$E2E_WORK/repo-a"
}

teardown() {
  local pid
  for pid in $(marleys_on "$E2E_WORK/profile-2") $(marleys_on "$E2E_PROFILE"); do
    kill -TERM "$pid" 2>/dev/null || true
  done
}

# The pids of the Marleys whose command line names `--user-data-dir <profile>`.
marleys_on() {
  local pid
  for pid in $(pgrep -x marley); do
    if tr '\0' '\n' <"/proc/$pid/cmdline" 2>/dev/null | grep -qxF -- "$1"; then
      echo "$pid"
    fi
  done
}

# Starts the run's binary on profile `$1` with the rest as its arguments, in the run's sway, and
# waits up to 15 seconds for it to end: prints "exited <status>" and its output, or "still runs"
# with its pid in LATE.
LATE=
second_launch() {
  local profile=$1 pid status
  shift
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$profile" "$@" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null &
  pid=$!
  LATE=
  for _ in $(seq 30); do
    if ! kill -0 "$pid" 2>/dev/null; then
      if wait "$pid"; then status=0; else status=$?; fi
      echo "exited $status"
      # What it printed, less the Wayland trace the run turns on.
      grep -v '^\[\|^\s*$' "$E2E_WORK/second.log" | tail -3
      return 0
    fi
    sleep 0.5
  done
  echo "still runs after 15 s"
  LATE=$pid
}

# The Marley windows in sway's tree: pid, focused, urgent, title.
windows() {
  sway_msg -t get_tree | jq -r --arg class "$E2E_CLASS" \
    '.. | objects | select(.app_id? == $class) | "\(.pid) focused=\(.focused) urgent=\(.urgent) \(.name)"'
}

# Whether the terminal listing has a terminal at the root of each folder named.
in_one_listing() {
  local folder
  for folder in "$@"; do
    grep -q "in $E2E_WORK/$folder\$" "$E2E_WORK/terminals.txt" || return 1
  done
}

# How many xdg-activation requests the run's Marley has sent.
activations() {
  grep -c 'xdg_activation_v1#[0-9]*\.activate(' "$E2E_LOG" || true
}

# Stops the Marleys on profile `$1` and waits up to 30 seconds for them to end.
stop_on() {
  local pid
  for pid in $(marleys_on "$1"); do
    kill -TERM "$pid" 2>/dev/null || true
  done
  for _ in $(seq 30); do
    [[ -z $(marleys_on "$1") ]] && return 0
    sleep 1
  done
  echo "Marley on $1 still runs 30 seconds after TERM" >&2
  return 1
}

steps() {
  local first before
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  sway_msg focus_on_window_activation focus >/dev/null ||
    echo "sway: focus_on_window_activation could not be set"
  first=$(marleys_on "$E2E_PROFILE")
  echo "the run's Marley: $first"
  echo "== a second launch on the same profile, with repo-b"
  second_launch "$E2E_PROFILE" "$E2E_WORK/repo-b"
  settle 4
  # Trusts repo-b, new to the profile, wherever it opened.
  press "" Return
  settle 3
  shot 513-01-handed-off
  windows
  mcp_agent terminals | tee "$E2E_WORK/terminals.txt"
  if [[ -n $LATE ]]; then
    kill -TERM "$LATE" 2>/dev/null || true
  fi
  expect "the second launch exits" test -z "$LATE"
  expect "one Marley on the profile" test "$(marleys_on "$E2E_PROFILE" | wc -l)" -eq 1
  # The first Marley's list holds both folders; a second app's would hold repo-b alone.
  expect "repo-b opened in the first Marley" in_one_listing repo-a repo-b
  echo "== a second launch with no path, while the Settings window has the focus"
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: open settings"
  settle 1
  press "" Return
  settle 3
  # A click on the Settings page's heading gives gpui a pointer press to ask for its token with.
  click 1082 118
  settle 1
  windows
  before=$(activations)
  second_launch "$E2E_PROFILE"
  settle 3
  shot 513-02-asked-to-come-forward
  echo "sway's tree after the hand-off (whether sway acted on the request):"
  windows
  expect "the no-path launch exits" test -z "$LATE"
  expect "the running Marley asked to activate its window" test "$(activations)" -gt "$before"
  echo "== a Marley on a second profile"
  # The profile copy's settings, with the scratch HOME the runner wrote for its terminals.
  mkdir -p "$E2E_WORK/profile-2/config"
  cp "$E2E_PROFILE/config/settings.json" "$E2E_WORK/profile-2/config/"
  second_launch "$E2E_WORK/profile-2" "$E2E_WORK/repo-c"
  settle 8
  shot 513-03-two-profiles
  windows
  expect "the second profile's Marley runs" test "$(marleys_on "$E2E_WORK/profile-2" | wc -l)" -eq 1
  stop_on "$E2E_WORK/profile-2"
  echo "== the first Marley killed, its socket left behind"
  kill -KILL "$first"
  settle 2
  ls -l "$E2E_PROFILE"/zed-*.sock 2>&1 || true
  launch_marley
  settle 8
  expect "a relaunch after the kill starts" test "$(marleys_on "$E2E_PROFILE" | wc -l)" -eq 1
}
