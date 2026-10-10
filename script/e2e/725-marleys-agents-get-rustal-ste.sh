# shellcheck shell=bash
# #725's visual check: Marley's agents get the rustal-ste skill. The runner's copy turns
# `marley.rustal_ste_skill` off and gives the run its own CODEX_HOME. This scenario turns the
# setting on.
# - Zed's agent lists `rustal-ste` in its `/` menu (`725-01-zed-skills`, REQ-001).
# - A new terminal's CLAUDE_CODE_PLUGIN_DIRS names the skill's plugin folder (`725-02-terminal`,
#   REQ-002).
# - Codex's skills folder holds Marley's marked copy (REQ-003).
# - Turned off from outside, the copy goes, and a new terminal names no folder (`725-03-off`,
#   REQ-004).
# - A `rustal-ste` folder Marley didn't write survives turning it on again (REQ-003).
compositor sway

setup() {
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  profile_setting marley.rustal_ste_skill true
  # A key that opens a thread on Zed's own agent, whose `/` menu lists its skills.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-z": ["agent::NewExternalAgentThread", {"agent": "Zed Agent"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether the file `$1` doesn't hold `$2`.
lacks() { ! grep -qF -- "$2" "$1"; }

codex_copy() { [[ -f $CODEX_HOME/skills/rustal-ste/SKILL.md && -f $CODEX_HOME/skills/rustal-ste/.marley-owned ]]; }
no_codex_copy() { [[ ! -e $CODEX_HOME/skills/rustal-ste ]]; }
user_folder_kept() { [[ -f $CODEX_HOME/skills/rustal-ste/USER.md && ! -e $CODEX_HOME/skills/rustal-ste/.marley-owned ]]; }

# A new terminal in the project, and what its CLAUDE_CODE_PLUGIN_DIRS says, into `$1`.
plugin_dirs_into() {
  palette "workspace: new terminal"
  settle 3
  type_text "echo \"dirs=\$CLAUDE_CODE_PLUGIN_DIRS\" > $1"
  press "" Return
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== Zed's agent"
  press "CTRL ALT SHIFT" z
  settle 5
  type_text "/rustal"
  settle 2
  shot 725-01-zed-skills
  press "" Escape
  press "CTRL" a
  press "" BackSpace
  settle 1

  echo "== a new terminal loads the plugin folder"
  plugin_dirs_into "$E2E_WORK/dirs-on.txt"
  shot 725-02-terminal
  cat "$E2E_WORK/dirs-on.txt"
  expect "the terminal names the skill's plugin folder" holds "$E2E_WORK/dirs-on.txt" "claude-code/rustal-ste/"
  expect "Codex holds Marley's marked copy" codex_copy

  echo "== off"
  profile_setting marley.rustal_ste_skill false
  settle 4
  plugin_dirs_into "$E2E_WORK/dirs-off.txt"
  shot 725-03-off
  cat "$E2E_WORK/dirs-off.txt"
  expect "a new terminal names no skill folder" lacks "$E2E_WORK/dirs-off.txt" "rustal-ste"
  expect "Marley removed its Codex copy" no_codex_copy

  echo "== a folder Marley didn't write"
  mkdir -p "$CODEX_HOME/skills/rustal-ste"
  printf 'the user put this here\n' >"$CODEX_HOME/skills/rustal-ste/USER.md"
  profile_setting marley.rustal_ste_skill true
  settle 4
  expect "Marley left the user's folder alone" user_folder_kept
}
