# shellcheck shell=bash
# #724's visual check: the shared Claude Code plugin at harness TICKET-115 (0.3.0, with the
# `rustal-ste` skill). It is #709's scenario: a stand-in `claude`, first on the terminal's PATH,
# prints what CLAUDE_CODE_PLUGIN_DIRS gives it. Both `marley.claude_code_shared_plugin` and #725's
# `marley.rustal_ste_skill` are on.
# `724-01-on`: a new terminal's stand-in finds rustal-harness 0.3.0 first, at the harness's digest,
# with `skills/rustal-ste/SKILL.md` (REQ-001), and no second rustal-ste folder (REQ-002).
compositor sway

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home"
  cat >"$bin/claude" <<'PY'
#!/usr/bin/env python3
# A stand-in claude for #724's e2e test.
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
    say("skill: " + ("present" if os.path.exists(os.path.join(first, "skills", "rustal-ste", "SKILL.md")) else "missing"))
    say(f"rustal-ste folders: {sum(1 for d in dirs.split(':') if 'rustal-ste' in d)}")
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
  profile_setting marley.rustal_ste_skill true
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

  echo "== a new terminal's claude, with both the shared plugin and the skill on"
  palette "workspace: new center terminal"
  settle 2
  type_text "claude"
  press "" Return
  settle 4
  shot 724-01-on
  cat "$E2E_WORK/stand-in.log"
  expect "the plugin is first, rustal-harness 0.3.0" printed "plugin: rustal-harness 0.3.0"
  expect "its folder is the harness's digest" printed "folder: eebd8a515080bb5c301f40121fe9ba0de41cd4b5afb11eb8c7adabecb213b1d0"
  expect "it carries the rustal-ste skill" printed "skill: present"
  expect "no second rustal-ste folder is loaded" printed "rustal-ste folders: 0"
  type_text "q"
  press "" Return
}
