# shellcheck shell=bash
# #543's visual check: a remote terminal whose shell outlives a dropped link. A stand-in `ssh`
# (`MARLEY_SSH`) logs its argv and pid, drops the options and the destination, and runs the remote
# command on this box with a tmux socket folder and a HOME of the run's own, so the run's tmux
# server is its own. The settings save `e2e-host` and a host that starts with `-o`; the picker
# lists the first only (REQ-001). Its terminal runs Marley's tmux session with a counter ticking
# (REQ-002); killing the stand-in ends the task and leaves the tab marked ended (REQ-003); Rerun
# attaches the same session, the counter further on (REQ-004). A stand-in Claude Code in the
# session runs the plugin's real `event.py` and wraps each answer for tmux, as Claude Code does
# under tmux; the frames reach Marley through the passthrough, marking the terminal and posting
# the banner on a private bus (REQ-005). Setup checks the hook's gate (REQ-006), and the argv log
# holds only fixed words and the session name (REQ-007).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks

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

# The hook's answer to a `Stop` under `env` arguments `$@`.
hook_answer() {
  printf '{"hook_event_name": "Stop", "last_assistant_message": "done"}' |
    env -u TERM_PROGRAM -u MARLEY_REMOTE "$@" python3 "$HOOKS/event.py"
}

# Whether the hook answers nothing under a plain tmux and a sequence under Marley's (REQ-006).
gate_holds() {
  local plain marley
  plain=$(hook_answer TERM_PROGRAM=tmux)
  marley=$(hook_answer TERM_PROGRAM=tmux MARLEY_REMOTE=1)
  echo "plain tmux: $plain"
  echo "Marley's tmux: ${marley:0:60}…"
  [[ $plain == "{}" && $marley == *terminalSequence* ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin remote=$E2E_WORK/remote-home tmux_dir
  mkdir -p "$home" "$bin" "$remote/bin"
  expect "the hook's gate: nothing under a plain tmux, a sequence under Marley's" gate_holds
  cat >"$home/.bashrc" <<'RC'
PS1='\$ '
RC
  terminal_env HOME "$home"
  # A tmux socket's path must fit 108 bytes, which the run's folder does not.
  tmux_dir=$(mktemp -d "${TMPDIR:-/tmp}/e2e-tmux.XXXXXX")
  echo "$tmux_dir" >"$E2E_WORK/tmux.dir"
  cat >"$remote/.bashrc" <<RC
PS1='remote\\$ '
export PATH="$remote/bin:\$PATH"
RC
  # tmux starts a login shell.
  echo '. ~/.bashrc' >"$remote/.bash_profile"
  sed -e "s|@LOG@|$E2E_WORK/ssh.log|" -e "s|@TMUX@|$tmux_dir|" -e "s|@HOME@|$remote|" \
    >"$bin/ssh" <<'FAKE'
#!/usr/bin/env bash
# A stand-in ssh: logs its argv and pid, drops the options and the destination, and runs the
# remote command here, joined with spaces for a shell as ssh joins it for the host's.
{
  printf 'argv:'
  printf ' %s' "$@"
  printf '\n'
  echo "pid: $$"
} >>@LOG@
while [[ $# -gt 0 ]]; do
  case $1 in
    -t) shift ;;
    -p) shift 2 ;;
    --) shift; break ;;
    *) break ;;
  esac
done
shift
unset TMUX TMUX_PANE
export TMUX_TMPDIR=@TMUX@ HOME=@HOME@ SHELL=/bin/bash
cd "$HOME" || exit 1
# The host's shell parses the joined words; `exec` keeps the logged pid the client's.
eval "exec $*"
FAKE
  chmod +x "$bin/ssh"
  export MARLEY_SSH=$bin/ssh
  sed -e "s|@HOOKS@|$HOOKS|" >"$remote/bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code on the host: a turn through the plugin's event.py, each answer's sequence
# wrapped for tmux's passthrough when TMUX is set, as Claude Code wraps it.
import json
import os
import subprocess
import sys

COMMON = {"session_id": "e2e-remote", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}
EVENTS = [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Tidy the imports"},
    {"hook_event_name": "Stop", "last_assistant_message": "Tidied the imports on the host."},
]
for event in EVENTS:
    answer = subprocess.run([sys.executable, "@HOOKS@/event.py"],
                            input=json.dumps({**COMMON, **event}),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence and os.environ.get("TMUX"):
        sequence = "\x1bPtmux;" + sequence.replace("\x1b", "\x1b\x1b") + "\x1b\\"
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()
print("claude (stand-in): the turn ended", flush=True)
FAKE
  chmod +x "$remote/bin/claude"
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
  : >"$E2E_WORK/ssh.log"
  set_setting ssh_connections '[{"host": "e2e-host", "nickname": "e2e"}, {"host": "-oProxyCommand=touch pwned", "nickname": "smuggled"}]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid tmux_dir
  for pid in user-bus notifications; do
    [[ -f $E2E_WORK/$pid.pid ]] && kill "$(cat "$E2E_WORK/$pid.pid")" 2>/dev/null
  done
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  if [[ -f $E2E_WORK/tmux.dir ]]; then
    tmux_dir=$(cat "$E2E_WORK/tmux.dir")
    TMUX_TMPDIR=$tmux_dir tmux -L marley kill-server 2>/dev/null
    [[ $tmux_dir == */e2e-tmux.* ]] && rm -rf "$tmux_dir"
  fi
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

# The argv lines the stand-in ssh logged.
argv_lines() { grep '^argv:' "$E2E_WORK/ssh.log"; }

# Whether the first logged argv is Marley's remote command, every word after the destination a
# fixed word or the session name (REQ-002, REQ-007).
argv_is_marleys() {
  python3 - "$(argv_lines | head -1)" <<'PY'
import re
import sys

words = sys.argv[1].split()[1:]
print("argv:", " ".join(words))
fixed = {"-t", "--", "e2e-host", "tmux", "-L", "marley", "-f", "/dev/null", "new-session", "-A",
         "-s", "-e", "MARLEY_REMOTE=1", "\\;", "set-option", "-g", "status", "off", "prefix",
         "None", "mouse", "on", "allow-passthrough", "-as", "terminal-features",
         ",xterm-256color:RGB"}
names = [word for word in words if word not in fixed]
session = words[words.index("-s") + 1] if "-s" in words else ""
ok = (words[:3] == ["-t", "--", "e2e-host"] and re.fullmatch(r"marley-[0-9a-f]{8}", session)
      and names == [session])
sys.exit(0 if ok else 1)
PY
}

steps() {
  local pid
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the picker: the saved host, not the one that starts with -o"
  palette "marley: open remote terminal"
  settle 2
  shot 543-01-picker

  echo "== connected: Marley's tmux session, a counter ticking"
  press "" Return
  settle 4
  type_text "n=0; while :; do n=\$((n+1)); echo \"tick \$n\"; sleep 1; done &"
  press "" Return
  settle 4
  shot 543-02-connected
  cat "$E2E_WORK/ssh.log"
  expect "one ssh ran, with Marley's remote command" argv_is_marleys
  expect "only the saved host was reached" test "$(argv_lines | wc -l)" -eq 1

  echo "== dropped: the stand-in ssh killed"
  pid=$(grep '^pid:' "$E2E_WORK/ssh.log" | tail -1 | cut -d' ' -f2)
  kill -HUP "$pid"
  settle 3
  shot 543-03-dropped

  echo "== reattached: Rerun, ten seconds on"
  settle 7
  palette "terminal: rerun task"
  settle 4
  shot 543-04-reattached
  cat "$E2E_WORK/ssh.log"
  expect "the rerun ran the same argv" test "$(argv_lines | sed -n 2p)" = "$(argv_lines | head -1)"
  TMUX_TMPDIR=$(cat "$E2E_WORK/tmux.dir") tmux -L marley list-sessions

  echo "== Claude Code on the host, the terminal not in front"
  type_text "kill %1; sleep 4; claude"
  press "" Return
  palette "workspace: new terminal"
  settle 8
  shot 543-05-notified
  echo "the host's session:"
  TMUX_TMPDIR=$(cat "$E2E_WORK/tmux.dir") tmux -L marley capture-pane -p | grep -v '^$' | tail -5
  cat "$E2E_WORK/banners.log"
  # The banner names the folder the host's Claude Code runs in.
  expect "the host's turn posted its banner" \
    grep -qxF "Marley|remote-home: Claude finished|Tidied the imports on the host." "$E2E_WORK/banners.log"
  # Other programs post to the user's bus as they will; only Marley's name counts (#535's check).
  expect "no banner reached the user's bus" bash -c "! grep -q 'STRING \"Marley\"' '$E2E_WORK/user-bus.log'"
}
