# shellcheck shell=bash
# #709's visual check: the shared Claude Code plugin loaded in Marley's terminals. A stand-in
# `claude`, first on the terminal's PATH and named by `MARLEY_CLAUDE` for Marley's version check,
# answers `--version` with 2.1.287. Run in a terminal, it prints `CLAUDE_CODE_PLUGIN_DIRS`, the first
# folder's plugin name, version and file modes to `stand-in.log`, then reports `waiting` on
# "Bash: ls" through `$MARLEY_BIN`, as the plugin's mod does while a permission dialog stands, and
# waits for `q`. Never the user's Claude Code (PR-687).
#
# `709-01-on`: with `marley.claude_code_shared_plugin` on, a new terminal's stand-in finds
# rustal-harness 0.2.0 first, read-only, and the rail's row shows it waiting (REQ-001, REQ-002).
# `709-02-off`: with it off, a new terminal's stand-in finds no Marley plugin folder (REQ-003).
compositor sway

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home"
  cat >"$bin/claude" <<'PY'
#!/usr/bin/env python3
# A stand-in claude for #709's e2e test.
import json, os, subprocess, sys
if sys.argv[1:] == ["--version"]:
    print("2.1.287 (Claude Code)")
    sys.exit(0)
log = open(os.environ["STAND_IN_LOG"], "a")
def say(line):
    print(line)
    log.write(line + "\n")
    log.flush()
dirs = os.environ.get("CLAUDE_CODE_PLUGIN_DIRS", "")
say(f"CLAUDE_CODE_PLUGIN_DIRS={dirs}")
first = dirs.split(":")[0] if dirs else ""
manifest = os.path.join(first, ".claude-plugin", "plugin.json") if first else ""
if manifest and os.path.exists(manifest):
    plugin = json.load(open(manifest))
    say(f"plugin: {plugin['name']} {plugin['version']}")
    say(f"folder: {os.path.basename(first)}")
    say("modes: " + " ".join(f"{name}={os.stat(os.path.join(first, name)).st_mode & 0o777:o}" for name in ["hooks/register.js", "LICENSE-MIT", "hooks"]))
    bin_ = os.environ.get("MARLEY_BIN", "")
    done = subprocess.run([bin_, "report", "--source", "mod:claude-code", "--seq", "1", "waiting", "--activity", "Bash: ls"], capture_output=True, text=True)
    say(f"report waiting: exit {done.returncode} {done.stderr.strip()}")
else:
    say("no plugin")
for line in sys.stdin:
    if line.strip() == "q":
        break
PY
  chmod +x "$bin/claude"
  printf 'PS1=%q\nexport PATH=%q:%s\n' '$ ' "$bin" "\$PATH" >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  terminal_env STAND_IN_LOG "$E2E_WORK/stand-in.log"
  export MARLEY_CLAUDE=$bin/claude
  profile_setting marley.claude_code_shared_plugin true
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# Whether the stand-in printed `$1`.
printed() { grep -qF -- "$1" "$E2E_WORK/stand-in.log"; }

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the setting on: a new terminal's claude"
  palette "workspace: new center terminal"
  settle 2
  type_text "claude"
  press "" Return
  settle 4
  shot 709-01-on
  expect "the plugin is first, rustal-harness 0.2.0" printed "plugin: rustal-harness 0.2.0"
  expect "its folder is the digest" printed "folder: 83d0bb8f5cd3041483c73b92a9d09ca01f4c5bd8763f8cf65b8433a713b10303"
  expect "its files are read-only" printed "modes: hooks/register.js=400 LICENSE-MIT=400 hooks=700"
  expect "the report reached Marley" printed "report waiting: exit 0"
  type_text "q"
  press "" Return
  settle 2

  echo "== the setting off: a new terminal's claude"
  profile_setting marley.claude_code_shared_plugin false
  settle 3
  palette "workspace: new center terminal"
  settle 2
  type_text "claude"
  press "" Return
  settle 3
  shot 709-02-off
  expect "no plugin when off" printed "no plugin"
}
