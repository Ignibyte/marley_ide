# shellcheck shell=bash
# #621's e2e test: a task's run is a block. `repo/.zed/tasks.json` has `fails` (`echo building;
# exit 3`) and `passes` (`echo ok`), each run from `task: spawn`. `fails` leaves one block with its
# command, its output and the failed pill with 3 (`failed`, REQ-001, REQ-002); a right-click on it
# opens its block menu (`menu`, REQ-003); `passes` leaves a block with the finished pill (`passed`,
# REQ-001).
compositor sway

# The task's output row, for its menu: the content sits against the bottom edge (#476).
OUTPUT_X=${OUTPUT_X:-500}
OUTPUT_Y=${OUTPUT_Y:-858}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/.zed"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$E2E_WORK/repo/.zed/tasks.json" <<'JSON'
[
  { "label": "fails", "command": "echo building; exit 3" },
  { "label": "passes", "command": "echo ok" }
]
JSON
  open_path "$E2E_WORK/repo"
}

# Runs the task `$1` from `task: spawn`.
spawn_task() {
  press "CTRL SHIFT" p
  settle 1
  type_text "task: spawn"
  settle 1
  press "" Return
  settle 1.5
  type_text "$1"
  settle 1
  press "" Return
  settle 4
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== fails"
  spawn_task fails
  pointer_to 900 600
  settle 1
  shot failed
  click "$OUTPUT_X" "$OUTPUT_Y" right
  settle 1.5
  shot menu
  press "" Escape
  settle 1

  echo "== passes"
  spawn_task passes
  pointer_to 900 600
  settle 1
  shot passed
}
