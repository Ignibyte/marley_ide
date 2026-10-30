# shellcheck shell=bash
# #951's visual check: Open File… on the rail's `+`. A scratch project `repo`, and a folder `notes`
# beside it, outside the project, with `plan.md` (headings and a list), `todo.txt` and `ideas.md`.
# Zed's system path prompts are off, so the chooser is Zed's own in the run's sway, never the
# desktop portal on the user's session.
# - `951-01-plus-menu`: `repo`'s `+` menu, Open File… highlighted (REQ-001).
# - `951-02a-prompt`: the chooser with `notes/plan.md` typed.
# - `951-02-markdown`: `plan.md` as a formatted page in the pane, its row under Files naming the
#   `notes` folder (REQ-002, REQ-004).
# - `951-03-text`: `todo.txt` in the editor, its row under Files (REQ-003, REQ-004).
# - `951-04-empty-menu`: the empty space's menu, Open File in Default… highlighted (REQ-005).
# - `951-05-home`: `ideas.md` opened from it, formatted, under the Default group (REQ-005).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
EMPTY_X=${EMPTY_X:-120}
EMPTY_Y=${EMPTY_Y:-700}
# How many Downs reach Open File… in the `+` menu (New Terminal, New tmux Terminal, New Browser
# Tab, Open File…) and in the empty space's (New Terminal, New Browser Tab, Open File, after New
# Group… and its separator).
PLUS_STEPS=${PLUS_STEPS:-4}
EMPTY_STEPS=${EMPTY_STEPS:-4}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/notes"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$E2E_WORK/notes/plan.md" <<'MD'
# The plan

Read outside any project.

## This week

- Open a file from the rail
- **Bold** and `code` drawn as such
MD
  printf 'buy milk\ncall the vet\n' >"$E2E_WORK/notes/todo.txt"
  printf '# Ideas\n\n- one for Home\n' >"$E2E_WORK/notes/ideas.md"
  profile_setting use_system_path_prompts false
  open_path "$E2E_WORK/repo"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

# Presses Down `$1` times in the open menu.
down() {
  local step
  for ((step = 0; step < $1; step++)); do
    press "" Down
  done
  settle 1
}

# Types `$1` over what Zed's chooser starts with, shoots it as `$2` when given, and takes it.
choose() {
  press CTRL a
  type_text "$1"
  settle 2
  [ -n "${2:-}" ] && shot "$2"
  press "" Return
  settle 4
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4
  [ -n "${STOP_AT_SETUP:-}" ] && { away; shot 951-00-probe; return; }

  echo "== repo's + → Open File… → notes/plan.md"
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  down "$PLUS_STEPS"
  shot 951-01-plus-menu
  press "" Return
  settle 2
  choose "$E2E_WORK/notes/plan.md" 951-02a-prompt
  away
  shot 951-02-markdown

  echo "== repo's + → Open File… → notes/todo.txt"
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  down "$PLUS_STEPS"
  press "" Return
  settle 2
  choose "$E2E_WORK/notes/todo.txt"
  away
  shot 951-03-text

  echo "== the empty space → Open File in Default… → notes/ideas.md"
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 2
  down "$EMPTY_STEPS"
  shot 951-04-empty-menu
  press "" Return
  settle 2
  choose "$E2E_WORK/notes/ideas.md"
  away
  shot 951-05-home
}
