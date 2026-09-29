# shellcheck shell=bash
# #545's visual check: a second launch hands its launcher's activation token to the running
# Marley. A GTK4 stand-in launcher, on the headless sway with `focus_on_window_activation smart`,
# takes the focus and requests an xdg-activation token as a launcher does for its click. A
# hand-off with no token leaves the focus on the launcher, as sway refuses the token gpui asks for
# itself (REQ-002); a hand-off carrying the launcher's token brings Marley's window forward
# (REQ-001), and the token opens nothing (REQ-003). sway's tree says which window has the focus.
compositor sway

# The launcher's window, tiled to the right of Marley's, from the shots.
LAUNCHER_X=${LAUNCHER_X:-1200}
LAUNCHER_Y=${LAUNCHER_Y:-500}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  printf "PS1='\\\\$ '\n" >"$home/.bashrc"
  terminal_env HOME "$home"
  write_launcher
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  [[ -f $E2E_WORK/launcher.pid ]] && kill "$(cat "$E2E_WORK/launcher.pid")" 2>/dev/null
  return 0
}

# A launcher's part: a window that takes the focus, and, once `$1.go` appears, a token from GTK's
# launch context, which asks the compositor with its latest input serial, written to `$1`.
write_launcher() {
  cat >"$E2E_WORK/launcher.py" <<'PY'
import os
import sys

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gdk, Gio, GLib, Gtk

out = sys.argv[1]
app = Gtk.Application(application_id="dev.e2e.Launcher")


def asked():
    if not os.path.exists(out + ".go"):
        return True
    context = Gdk.Display.get_default().get_app_launch_context()
    info = Gio.AppInfo.create_from_commandline("marley", "Marley", Gio.AppInfoCreateFlags.NONE)
    token = context.get_startup_notify_id(info, [])
    with open(out, "w", encoding="utf-8") as file:
        file.write(token or "")
    return False


def activate(app):
    window = Gtk.ApplicationWindow(application=app, title="e2e launcher")
    window.set_child(Gtk.Label(label="launcher"))
    window.present()
    GLib.timeout_add(200, asked)


app.connect("activate", activate)
app.run([])
PY
}

# Hands the running Marley no paths, as a second launch does, with `$1` as its activation token or
# with none.
hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE -u XDG_ACTIVATION_TOKEN \
    WAYLAND_DISPLAY="$SWAY_DISPLAY" SWAYSOCK="$SWAY_SOCK" ${1:+XDG_ACTIVATION_TOKEN="$1"} \
    "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

# The app id of the window with sway's focus.
focused() {
  swaymsg -s "$SWAY_SOCK" -t get_tree |
    jq -r '[.. | objects | select(.focused? == true and .type == "con")][0].app_id // "none"'
}

steps() {
  local token
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  swaymsg -s "$SWAY_SOCK" focus_on_window_activation smart
  WAYLAND_DISPLAY=$SWAY_DISPLAY GDK_BACKEND=wayland python3 "$E2E_WORK/launcher.py" \
    "$E2E_WORK/token" >"$E2E_WORK/launcher.log" 2>&1 &
  echo "$!" >"$E2E_WORK/launcher.pid"
  settle 4
  echo "focused: $(focused)"
  expect "the launcher has the focus" test "$(focused)" = dev.e2e.Launcher

  echo "== a hand-off with no token"
  hand_over
  settle 4
  shot 545-01-no-token
  echo "focused: $(focused)"
  expect "without a token the focus stays on the launcher" test "$(focused)" = dev.e2e.Launcher

  echo "== the launcher's token, then a hand-off with it"
  # A launcher asks with the serial of the click that launches: GTK sends its last press's.
  click "$LAUNCHER_X" "$LAUNCHER_Y"
  settle 1
  touch "$E2E_WORK/token.go"
  for _ in $(seq 20); do
    [[ -s $E2E_WORK/token ]] && break
    sleep 0.5
  done
  token=$(cat "$E2E_WORK/token" 2>/dev/null)
  echo "token: ${token:0:8}… (${#token} characters)"
  expect "the launcher got a token" test -n "$token"
  hand_over "$token"
  settle 4
  shot 545-02-token
  echo "focused: $(focused)"
  expect "with the token Marley has the focus" test "$(focused)" = "$E2E_CLASS"
  cat "$E2E_WORK/second.log"
  expect "the token opened nothing" \
    bash -c "! grep -ai 'marley-activation-token' '$E2E_PROFILE/logs/Marley.log'"
}
