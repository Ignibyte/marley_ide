# shellcheck shell=bash
# #512's e2e test: Marley started on a seat whose keyboard has gone. The run's Marley opens the
# scratch repository and quits through its palette; the `wtype` keyboards that typed the quit
# exit with it, so the headless sway's seat has no keyboard to hand the next client a keymap.
# A second Marley on the same profile then gets `modifiers` with no usable keymap, which
# panicked gpui's keyboard handler before #512 (F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001).
# It must open its window and keep running, and once a keyboard types it must take the keys:
# Ctrl+Shift+P opens the palette.
compositor sway

SECOND_PID=

setup() {
  git init -q -b keymap "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -f $E2E_WORK/second.log ]]; then
    echo "== the second Marley's stderr"
    grep -E -A2 "panicked|Received keymap|Failed to create keymap" "$E2E_WORK/second.log" || echo "(no panic, no keymap message)"
  fi
  if [[ -n $SECOND_PID ]]; then
    kill -TERM "$SECOND_PID" 2>/dev/null || true
    sleep 1
  fi
}

steps() {
  local second
  second=$(realpath -m "${CARGO_TARGET_DIR:-target}/debug/marley")
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  quit_marley
  settle 1
  echo "== a Marley starts on a seat whose keyboard has gone"
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" SWAYSOCK="$SWAY_SOCK" \
    setsid -f "$second" --user-data-dir "$E2E_PROFILE" "$E2E_WORK/repo" >>"$E2E_WORK/second.log" 2>&1 </dev/null
  for _ in $(seq 90); do
    SECOND_PID=$(sway_marley_pid)
    [[ -n $SECOND_PID ]] && break
    sleep 1
  done
  if [[ -z $SECOND_PID ]]; then
    echo "no window from the second Marley within 90 seconds" >&2
    return 1
  fi
  settle 10
  if ! kill -0 "$SECOND_PID" 2>/dev/null; then
    echo "the second Marley ($SECOND_PID) stopped within ten seconds" >&2
    return 1
  fi
  echo "the second Marley ($SECOND_PID) still runs after ten seconds"
  shot 512-01-no-keyboard
  echo "== a keyboard types"
  press "CTRL SHIFT" p
  settle 2
  shot 512-02-keys-arrive
  press "" Escape
  settle 1
}
