# shellcheck shell=bash
# #532's e2e test: agent permission modes. Fake `claude` and `codex`, first on the terminal's
# PATH, write the arguments they were started with to a log and print them; the fake `claude`
# also sends #519's hook events through Marley's plugin hook, reporting `bypassPermissions` when
# its arguments ask for bypass or its environment says so (as Claude Code's own settings would),
# else `default`, and `default` again at each Enter (as after Shift+Tab). The scratch
# repository's own `.zed/settings.json` asks for bypass, which changes nothing (REQ-005). The
# rail's + starts Claude Code with no flag (REQ-001); with `agent_permissions_by_project` for the
# repository, with `--dangerously-skip-permissions` and the row's bypass chip (REQ-002,
# REQ-004); with `codex_permissions: full_access`, Codex with full access and its chip (REQ-003);
# a bypass typed by hand, with the fake's events off, is marked from its arguments (REQ-004); the
# reported mode wins both ways (REQ-008); and the Marley page lists both settings (REQ-006).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The project's +, the rows' chips and the settings page, from the first run's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
# Down steps from New Terminal in the + menu: New Browser Tab, New Agent Thread, then the CLIs.
# One more since New Agent in Worktree (#510) sits above the agent CLIs.
CLAUDE_STEPS=${CLAUDE_STEPS:-4}
CODEX_STEPS=${CODEX_STEPS:-5}
CHIP_X=${CHIP_X:-192}
CHIP_Y=${CHIP_Y:-229}
BYPASS_ROW_X=${BYPASS_ROW_X:-110}
BYPASS_ROW_Y=${BYPASS_ROW_Y:-228}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  local repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo/.zed"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # The plugin listed at the version Marley ships, so the agent bar offers no update.
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  git init -q -b permissions "$repo"
  # A repository that asks for bypass itself: Marley must not take it.
  cat >"$repo/.zed/settings.json" <<'JSON'
{"marley": {"claude_code_permissions": "bypass", "codex_permissions": "full_access"}}
JSON
  write_fakes "$bin"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

# The fakes. Each writes `<program>: <its arguments>` to the launches log and prints it.
write_fakes() {
  sed -e "s|@HOOK@|$HOOK|" -e "s|@LOG@|$E2E_WORK/launches.log|" -e "s|@READ@|$E2E_WORK/stdin.log|" \
    >"$1/claude" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)

ARGS = sys.argv[1:]
with open("@LOG@", "a", encoding="utf-8") as log:
    log.write("claude: " + " ".join(ARGS) + "\n")
print("started with: " + " ".join(ARGS), flush=True)
BYPASS = "--dangerously-skip-permissions" in ARGS or "--permission-mode=bypassPermissions" in ARGS
MODE = "bypassPermissions" if BYPASS else os.environ.get("STAND_IN_MODE", "default")
EVENTS = os.environ.get("STAND_IN_EVENTS") != "0"
COMMON = {"transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd(),
          "session_id": "s-" + str(os.getpid()), "prompt_id": "p1"}


def send(event, mode):
    if not EVENTS:
        return
    answer = subprocess.run([sys.executable, "@HOOK@"],
                            input=json.dumps({**COMMON, "permission_mode": mode, **event}),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


send({"hook_event_name": "SessionStart", "source": "startup"}, MODE)
for line in sys.stdin:
    with open("@READ@", "a", encoding="utf-8") as read:
        read.write(json.dumps({"args": ARGS, "events": EVENTS, "line": line}) + "\n")
    # As after Shift+Tab out of bypass: the next event reports the new mode.
    send({"hook_event_name": "UserPromptSubmit", "prompt": "Carry on"}, "default")
    print("mode now: default", flush=True)
FAKE
  sed -e "s|@LOG@|$E2E_WORK/launches.log|" >"$1/codex" <<'FAKE'
#!/usr/bin/env python3
import sys

ARGS = sys.argv[1:]
with open("@LOG@", "a", encoding="utf-8") as log:
    log.write("codex: " + " ".join(ARGS) + "\n")
print("started with: " + " ".join(ARGS), flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1/claude" "$1/codex"
}

# Merges the JSON object `$1` into `marley` in the profile copy's settings (570's, one level).
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, re, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
# The profile copy may hold comments and trailing commas.
text = re.sub(r"^\s*//.*$", "", text, flags=re.MULTILINE)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {}).update(changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
  settle 3
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The project's +, then `$1` steps down from New Terminal, then Enter.
plus_entry() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  local step
  for ((step = 0; step < $1; step++)); do
    press "" Down
  done
  settle 1
}

last_launch() {
  tail -n 1 "$E2E_WORK/launches.log" 2>/dev/null
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the terminal's claude and codex are the fakes"
  type_text "command -v claude codex > $E2E_WORK/which.txt"
  press "" Return
  settle 2
  cat "$E2E_WORK/which.txt"
  if ! grep -qx "$E2E_WORK/bin/claude" "$E2E_WORK/which.txt" ||
    ! grep -qx "$E2E_WORK/bin/codex" "$E2E_WORK/which.txt"; then
    echo "check the fakes come first on the terminal's PATH: FAIL (stopping before any launch)"
    return 1
  fi

  echo "== nothing set: Claude Code starts with its prompts"
  plus_entry "$CLAUDE_STEPS"
  shot 532-00-menu
  press "" Return
  settle 6
  shot 532-01-default
  expect "Claude Code started with no flag" test "$(last_launch)" = "claude: "

  echo "== bypass for the repository: Claude Code starts with it, marked"
  marley_setting "{\"agent_permissions_by_project\": {\"$E2E_WORK/repo\": {\"claude_code\": \"bypass\"}}}"
  plus_entry "$CLAUDE_STEPS"
  press "" Return
  settle 6
  shot 532-02-bypass
  expect "Claude Code started with --dangerously-skip-permissions" \
    test "$(last_launch)" = "claude: --dangerously-skip-permissions"
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  shot 532-02b-tooltip

  echo "== full access for Codex"
  marley_setting '{"codex_permissions": "full_access"}'
  plus_entry "$CODEX_STEPS"
  press "" Return
  settle 6
  shot 532-03-full-access
  expect "Codex started with full access" \
    test "$(last_launch)" = "codex: --sandbox danger-full-access --ask-for-approval never"

  echo "== a bypass typed by hand, with no events: marked from its arguments"
  palette "workspace: new terminal"
  settle 3
  type_text "STAND_IN_EVENTS=0 claude --dangerously-skip-permissions"
  press "" Return
  settle 5
  shot 532-04-typed

  echo "== the reported mode wins: default clears, bypassPermissions marks"
  palette "workspace: new terminal"
  settle 3
  type_text "STAND_IN_MODE=bypassPermissions claude"
  press "" Return
  settle 5
  click "$BYPASS_ROW_X" "$BYPASS_ROW_Y"
  settle 2
  press "" Return
  settle 4
  shot 532-05-reported
  cat "$E2E_WORK/stdin.log"
  expect "the one Enter reached the stand-in the + started with bypass" python3 - "$E2E_WORK/stdin.log" <<'PY'
import json, sys
reads = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
sys.exit(0 if len(reads) == 1 and reads[0]["args"] == ["--dangerously-skip-permissions"]
         and reads[0]["events"] else 1)
PY

  echo "== the Marley settings page"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 8
  settle 2
  shot 532-06-settings-page
  cat "$E2E_WORK/launches.log"
}
