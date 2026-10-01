# shellcheck shell=bash
# #622's e2e test: a task block's Rerun Task, and no summary line in the Marley layout.
# `repo/.zed/tasks.json` has `fails` (`echo building $RUN; exit 3`, where `run.sh` counts its runs);
# run from `task: spawn`, its block ends the output with no `Task ... finished` line under it, the
# command line kept (`finished`, REQ-002). The pointer on the block shows Rerun Task among its
# buttons (`hover`); its click runs the task again in the tab, which then says `building 2`
# (`rerun`, REQ-001).
compositor sway

# As the first run's shots found them: the block's first row, and its Rerun Task button.
BLOCK_X=${BLOCK_X:-500}
BLOCK_Y=${BLOCK_Y:-897}
RERUN_X=${RERUN_X:-1290}
RERUN_Y=${RERUN_Y:-897}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/.zed"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$E2E_WORK/repo/run.sh" <<'SH'
count=$(( $(cat .runs 2>/dev/null || echo 0) + 1 ))
echo "$count" >.runs
echo "building $count"
exit 3
SH
  cat >"$E2E_WORK/repo/.zed/tasks.json" <<'JSON'
[
  { "label": "fails", "command": "sh run.sh" }
]
JSON
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== fails"
  press "CTRL SHIFT" p
  settle 1
  type_text "task: spawn"
  settle 1
  press "" Return
  settle 1.5
  type_text "fails"
  settle 1
  press "" Return
  settle 4
  pointer_to 900 400
  settle 1
  shot finished

  echo "== the block's Rerun Task"
  pointer_to "$BLOCK_X" "$BLOCK_Y"
  settle 1.5
  shot hover
  click "$RERUN_X" "$RERUN_Y"
  settle 5
  pointer_to 900 400
  settle 1
  shot rerun
  echo "runs counted: $(cat "$E2E_WORK/repo/.runs")"
}
