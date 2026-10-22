# shellcheck shell=bash
# #899's visual check: Ask an agent to fix this, and the inline rename note. #856's scratch crate,
# which rust-analyzer reads (`rail.wid` is an error at line 17), and in the window's terminal the
# stand-in Claude Code, idle through Marley's plugin hook, so it is the window's one agent.
# - Ctrl+. on the error: the actions, then "Ask an agent to fix this…" (`899-01-actions`,
#   REQ-001).
# - The entry clicked: the terminal shows "Fix this:", the line and the error, typed with no Enter
#   (`899-02-sent`, REQ-001).
# - F2 on `Rail`, "Board" typed: "Renaming: Rail → Board · Enter" (`899-03-rename`, REQ-002), and
#   in Marley Day (`899-04-day`, REQ-002).
compositor sway
HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)

STOP_AT=${STOP_AT:-}
# The menu's Ask an agent to fix this, and main.rs's tab, from the first run's shots.
ASK_X=${ASK_X:-553}
ASK_Y=${ASK_Y:-587}
TAB_X=${TAB_X:-656}
TAB_Y=${TAB_Y:-50}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo/src"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  printf '[package]\nname = "rail"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  cat >"$repo/src/main.rs" <<'RS'
/// How wide the rail draws.
pub struct Rail {
    width: u32,
    shrunk: bool,
}

impl Rail {
    /// The rail's width, shrunk or not.
    pub fn width_for(&self, shrunk: bool) -> u32 {
        if shrunk { 48 } else { self.width }
    }
}

fn main() {
    let rail = Rail { width: 260, shrunk: false };
    let rows = rail.width_for(rail.shrunk);
    rail.wid
}
RS
  git init -q -b main "$repo"
  sed -e "s|@HOOK@|$HOOK|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
print("stand-in Claude Code, idle", flush=True)
event = {"hook_event_name": "SessionStart", "source": "startup", "cwd": os.getcwd(),
         "session_id": "s-" + str(os.getpid()), "permission_mode": "default"}
answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                        capture_output=True, text=True, check=False).stdout
sequence = json.loads(answer or "{}").get("terminalSequence")
if sequence:
    sys.stdout.write(sequence)
    sys.stdout.flush()
for line in sys.stdin:
    print("got: " + line.rstrip("\n"), flush=True)
FAKE
  chmod +x "$bin/claude"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 2
  press "" Return
  settle 3
}

# Moves the caret to `$1`, line:column.
goto() {
  palette "go to line"
  type_text "$1"
  press "" Return
  settle 2
}

# Opens main.rs.
main_rs() {
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  settle 2
}

# Renames `Rail` inline to Board, and shoots it as `$1`.
rename() {
  goto 2:12
  press "" F2
  settle 2
  type_text "Board"
  settle 2
  pointer_to 1500 950
  shot "$1"
  press "" Escape
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch crate.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 3
  main_rs
  # rust-analyzer reads the crate and std.
  settle 40

  echo "== code actions on the error"
  goto 17:12
  press CTRL period
  settle 4
  take 899-01-actions || return 0

  echo "== Ask an agent to fix this"
  if ((ASK_X > 0)); then
    click "$ASK_X" "$ASK_Y"
    settle 3
  fi
  pointer_to 1500 950
  settle 1
  take 899-02-sent || return 0

  echo "== inline rename"
  click "$TAB_X" "$TAB_Y"
  settle 2
  rename 899-03-rename
  [[ $STOP_AT != 899-03-rename ]] || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  rename 899-04-day
}
