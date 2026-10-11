# shellcheck shell=bash
# #713's visual check: the rail refreshes on row changes, Home's page follows a new rail, and a
# window saves its groups only once its restore is done. A scratch project `repo`; `ZED_LOG` puts
# the rail's refresh lines in the run's log; a fake `claude` prints for three seconds, then waits.
# - A terminal's one process prints 300 lines over six seconds, then it rings: its row while it
#   runs and once it is done (`printing`, `finished`, REQ-003, REQ-004); two seconds mid-print
#   make three refreshes at most (the port scan's timer among them).
# - The fake `claude` while it prints, then once it is quiet (`agent-working`, `agent-waiting`,
#   REQ-004).
# - Zed's layout, then the Marley layout again, and a second fake agent: Home's page lists it, and
#   every refresh after the swap names one rail (`home-after-swap`, REQ-001, REQ-002).
# - Two groups with no folder, Alpha and Beta; Marley quits, starts with no path and is killed
#   as soon as its window maps; it starts again: both groups are back, in their order, and the
#   last launch let its window save once its restore was done (`restored`, REQ-006).
compositor sway

# The rail's empty space (#601), the repo's terminal row and the status bar's Home button, as the
# first run's shots found them; `STOP_AT=<shot>` ends the run after that shot.
EMPTY_X=${EMPTY_X:-130}
EMPTY_Y=${EMPTY_Y:-760}
TERMINAL_ROW_X=${TERMINAL_ROW_X:-95}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-176}
HOME_X=${HOME_X:-1422}
BAR_Y=${BAR_Y:-985}
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  printf '# repo\n' >"$repo/README.md"
  # Named `claude` in its own argv, as the rail reads a foreground program's name.
  cat >"$bin/claude" <<'SH'
#!/bin/bash
exec -a claude python3 -c "import time
for line in range(30):
    print('working', line, flush=True)
    time.sleep(0.1)
time.sleep(600)"
SH
  chmod +x "$bin/claude"
  terminal_env HOME "$home"
  terminal_env PATH "$bin:$PATH"
  export ZED_LOG="info,marley_workbench=debug"
  open_path "$repo"
}

# The run's log, where the rail writes a line per refresh.
log_file() {
  printf '%s\n' "$E2E_PROFILE/logs/Marley.log"
}

# How many refresh lines the log holds.
refreshes() {
  grep -c "rail EntityId(.*) refresh" "$(log_file)" 2>/dev/null || echo 0
}

# Whether the refreshes after line `$1` of the log name one rail.
one_rail_after() {
  local rails
  rails=$(tail -n "+$1" "$(log_file)" | grep -o "rail EntityId([^)]*) refresh" | sort -u)
  echo "rails refreshing after the swap: ${rails//$'\n'/, }"
  [[ $(grep -c . <<<"$rails") -eq 1 ]]
}

# Whether at most `$1` refreshes came between the counts `$2` and `$3`.
at_most() {
  echo "refreshes during the print: $(($3 - $2))"
  (($3 - $2 <= $1))
}

# Makes a group named `$1` from the empty space's menu (#601).
new_group() {
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 1
  # New Group…, the menu's first entry: no entry is selected as it opens.
  click "$((EMPTY_X + 50))" "$((EMPTY_Y + 17))"
  settle 1
  type_text "$1"
  press "" Return
  settle 4
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Types `$1` into the repo's terminal, which the rail's row brings forward, and runs it.
run_in_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

# Shoots `$1`, and ends the run there when `STOP_AT` names it.
take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

steps() {
  local before after swapped
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== a terminal printing for six seconds, then ringing"
  run_in_terminal "clear"
  settle 2
  # One process prints, so the terminal's title stays: a shell loop retitles the tab for each
  # command it runs, which is a change the row shows.
  run_in_terminal "python3 -c \"import time; [(print('line', i, flush=True), time.sleep(0.02)) for i in range(300)]\"; printf '\\a'"
  # Counted mid-print, once the command's start has redrawn its row and before its end does.
  settle 1.5
  before=$(refreshes)
  settle 2
  after=$(refreshes)
  take 713-01-printing || return 0
  expect "three refreshes or fewer while it prints" at_most 3 "$before" "$after"
  settle 5
  take 713-02-finished || return 0

  echo "== the fake claude, working then waiting"
  run_in_terminal "claude"
  settle 1.5
  take 713-03-agent-working || return 0
  settle 5
  take 713-04-agent-waiting || return 0

  echo "== Home's page, a layout swap and back, and a second agent"
  click "$HOME_X" "$BAR_Y"
  settle 2
  palette "marley: use zed layout"
  settle 4
  palette "marley: use marley layout"
  settle 4
  swapped=$(($(wc -l <"$(log_file)") + 1))
  palette "workspace: new center terminal"
  settle 3
  type_text "claude"
  press "" Return
  settle 2
  click "$HOME_X" "$BAR_Y"
  settle 2
  take 713-05-home-after-swap || return 0
  expect "every refresh after the swap names one rail" one_rail_after "$swapped"

  echo "== two groups with no folder, a launch killed during its restore, and another"
  new_group "Alpha"
  new_group "Beta"
  take 713-06a-groups || return 0
  quit_marley
  open_path ""
  launch_marley
  kill -9 "$(marley_pid)"
  settle 2
  launch_marley
  settle 15
  take 713-06-restored || return 0
  expect "the last launch let its window save once restored" \
    grep -q "lets its window save: its restore is done" "$(log_file)"
}
