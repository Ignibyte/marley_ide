# shellcheck shell=bash
# #552's visual check: the agent bar sets up Codex's and OpenCode's notifications in a click.
# Marley runs with `CODEX_HOME` and `XDG_CONFIG_HOME` in the run's own folders and on a private
# session bus whose notification server logs each banner, so none reaches the user's desktop.
# Stand-in `codex` and `opencode` run in the terminal. Codex's bar offers "Turn on Codex
# notifications" (REQ-001), whose click writes the three keys under `[tui]` and keeps the rest
# (REQ-002); OpenCode's offers "Connect OpenCode to Marley" (REQ-003), whose click writes the
# plugin (REQ-004), and "Update" for an older one (REQ-005). The written plugin, handed a
# `session.idle` in a terminal not in front, posts one banner, and none without `TERM_PROGRAM`
# (REQ-006, REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The bar's chip, from the shots.
CHIP_X=${CHIP_X:-460}
CHIP_Y=${CHIP_Y:-954}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo" "$E2E_WORK/codex" "$E2E_WORK/xdg"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  export CODEX_HOME=$E2E_WORK/codex XDG_CONFIG_HOME=$E2E_WORK/xdg
  cat >"$CODEX_HOME/config.toml" <<'TOML'
# The run's Codex config: a comment, a model, and a [tui] key of its own.
model = "gpt-5"

[tui]
theme = "dark"
TOML
  # Stand-ins: each prints a line and reads its input until Ctrl-D.
  local agent
  for agent in codex opencode; do
    printf '#!/usr/bin/env bash\nexec -a %s python3 -u -c %q\n' "$agent" \
      "import sys; print('stand-in $agent'); sys.stdin.read()" >"$bin/$agent"
    chmod +x "$bin/$agent"
  done
  # Loads the plugin Marley wrote and hands its hook a finished session, after a moment in which
  # the scenario moves the focus away.
  cat >"$bin/fire-plugin" <<'SH'
#!/usr/bin/env bash
sleep 2
exec node --input-type=module -e '
const plugin = await import("file://" + process.env.XDG_CONFIG_HOME + "/opencode/plugins/marley.js");
const hooks = await plugin.MarleyNotify({ directory: process.cwd() });
await hooks.event({ event: { type: "session.idle" } });
'
SH
  chmod +x "$bin/fire-plugin"
  # The user's own bus, watched for a banner that should not reach it.
  stdbuf -oL busctl --address="unix:path=$XDG_RUNTIME_DIR/bus" monitor org.freedesktop.Notifications \
    >"$E2E_WORK/user-bus.log" 2>&1 &
  echo "$!" >"$E2E_WORK/user-bus.pid"
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  git init -q -b main "$repo"
  open_path "$repo"
}

teardown() {
  local pid
  for pid in user-bus notifications; do
    [[ -f $E2E_WORK/$pid.pid ]] && kill "$(cat "$E2E_WORK/$pid.pid")" 2>/dev/null
  done
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

write_notification_server() {
  cat >"$E2E_WORK/notifications.py" <<'PY'
# A desktop notification server on the scenario's private bus: logs each banner and answers as a
# real server does.
import sys

import dbus
import dbus.mainloop.glib
import dbus.service
from gi.repository import GLib

LOG = sys.argv[1]
INTERFACE = "org.freedesktop.Notifications"
dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)


class Notifications(dbus.service.Object):
    def __init__(self, bus):
        super().__init__(bus, "/org/freedesktop/Notifications")
        self.last = 0

    @dbus.service.method(INTERFACE, in_signature="susssasa{sv}i", out_signature="u")
    def Notify(self, app, replaces, icon, summary, body, actions, hints, timeout):
        self.last += 1
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(f"{app}|{summary}|{body}\n")
        return self.last

    @dbus.service.method(INTERFACE, in_signature="", out_signature="as")
    def GetCapabilities(self):
        return ["body", "actions"]

    @dbus.service.method(INTERFACE, in_signature="u", out_signature="")
    def CloseNotification(self, id):
        pass

    @dbus.service.method(INTERFACE, in_signature="", out_signature="ssss")
    def GetServerInformation(self):
        return ("e2e", "marley", "1", "1.2")


bus = dbus.SessionBus()
name = dbus.service.BusName(INTERFACE, bus)
server = Notifications(bus)
print("ready", flush=True)
GLib.MainLoop().run()
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

banners() { wc -l <"$E2E_WORK/banners.log"; }

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== Codex: the chip, and its click"
  type_text "codex"
  press "" Return
  settle 4
  shot 552-01-codex-chip
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  shot 552-02-codex-written
  cat "$CODEX_HOME/config.toml"
  expect "the three keys are under [tui], the rest kept" holds "$CODEX_HOME/config.toml" \
    '# The run'"'"'s Codex config' 'model = "gpt-5"' 'theme = "dark"' 'notifications = true' \
    'notification_condition = "always"' 'notification_method = "osc9"'
  press "CTRL" d
  settle 2

  echo "== OpenCode: the chip, and its click"
  type_text "opencode"
  press "" Return
  settle 4
  shot 552-03-opencode-chip
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  shot 552-04-opencode-written
  local plugin=$XDG_CONFIG_HOME/opencode/plugins/marley.js
  head -1 "$plugin"
  expect "the plugin was written with its version" grep -qx '// marley-opencode-plugin 1' "$plugin"
  press "CTRL" d
  settle 2

  echo "== an older plugin: the update"
  sed -i '1s/.*/\/\/ marley-opencode-plugin 0/' "$plugin"
  type_text "opencode"
  press "" Return
  settle 5
  shot 552-05-opencode-update
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  expect "the update rewrote it" grep -qx '// marley-opencode-plugin 1' "$plugin"
  press "CTRL" d
  settle 2

  echo "== the plugin's notification, from a terminal not in front"
  palette "workspace: new terminal"
  settle 3
  press ALT 1
  settle 1
  type_text "fire-plugin"
  press "" Return
  press ALT 2
  settle 5
  shot 552-06-notified
  cat "$E2E_WORK/banners.log"
  expect "one banner came" grep -qxF "Marley|OpenCode|repo finished" "$E2E_WORK/banners.log"

  echo "== without TERM_PROGRAM, none"
  local before
  before=$(banners)
  press ALT 1
  settle 1
  type_text "env -u TERM_PROGRAM fire-plugin"
  press "" Return
  press ALT 2
  settle 5
  expect "no banner without TERM_PROGRAM" test "$(banners)" = "$before"
  # Other programs post to the user's bus as they will; only Marley's name counts (#535's check).
  expect "no banner reached the user's bus" bash -c "! grep -q 'STRING \"Marley\"' '$E2E_WORK/user-bus.log'"
}
