# shellcheck shell=bash
# #818's visual check: a thread's row dragged onto another group's header moves the thread there.
# #807's set-up (sourced): the Marley entry on a scripted agent, which answers "Noted: <prompt>"
# and logs what it reads, and Rusty on through its stand-in. `repo` opens; a Rusty and a Marley
# conversation start there; `repo-b` is handed to the same window (#513), as #613's scenario does.
# - The Rusty conversation's row under `repo` (`818-01-rusty-in-repo`).
# - Dragged onto Default's header: its row under Default, the window on Default with its tab
#   (`818-02-rusty-in-default`, REQ-001, REQ-003); a prompt there is answered (`818-03-answers`,
#   REQ-004).
# - The Marley conversation's row dragged onto `repo-b`'s header: under `repo-b`
#   (`818-04-marley-in-repo-b`, REQ-002).
# - Marley quit and started again: both rows where they were moved (`818-05-restored`, REQ-005).
. script/e2e/807-marley-rusty-and-home-icons.sh

# The rail's points on the 1600 x 1000 output, from the first run's shots: Default's header and the
# Rusty row in `818-00-layout`; `repo-b`'s header and the Marley row once Default holds Home and
# the Rusty row (`818-02-rusty-in-default`).
ROW_X=${ROW_X:-130}
DEFAULT_Y=${DEFAULT_Y:-88}
RUSTY_Y=${RUSTY_Y:-385}
MARLEY_Y=${MARLEY_Y:-431}
REPO_B_Y=${REPO_B_Y:-228}
# The moved thread's message field, in Default's center.
FIELD_X=${FIELD_X:-930}
FIELD_Y=${FIELD_Y:-930}

# A second launch on this profile with `$1`, which hands it to the running Marley (#513).
hand_off() {
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null
}

# Drags the row at y `$1` onto the header at y `$2`, in steps, as a hand does (#613).
drag_row() {
  local from=$1 to=$2 step
  pointer_to "$ROW_X" "$from"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6; do
    pointer_to "$ROW_X" "$((from + (to - from) * step / 6))"
    sleep 0.1
  done
  settle 0.5
  pointer_up
  settle 3
}

# Whether the scripted agent read the prompt `$1`.
read_prompt() {
  grep -q "\"text\": \"$1\"" "$E2E_WORK/agent.log"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== a Rusty and a Marley conversation in repo"
  palette "marley: new rusty conversation"
  settle 6
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "hello rusty" -s 300 -k Return
  settle 4
  palette "marley: new marley conversation"
  settle 6
  shot 818-01-rusty-in-repo

  echo "== repo-b in the same window"
  mkdir -p "$E2E_WORK/repo-b"
  printf '# repo-b\n' >"$E2E_WORK/repo-b/README.md"
  hand_off "$E2E_WORK/repo-b"
  settle 6
  press "" Return
  settle 3
  shot 818-00-layout

  echo "== the Rusty row onto Default's header"
  drag_row "$RUSTY_Y" "$DEFAULT_Y"
  shot 818-02-rusty-in-default
  click "$FIELD_X" "$FIELD_Y"
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "still here" -s 300 -k Return
  settle 4
  shot 818-03-answers
  expect "the moved conversation's agent read the prompt" read_prompt "still here"

  echo "== the Marley row onto repo-b's header"
  drag_row "$MARLEY_Y" "$REPO_B_Y"
  shot 818-04-marley-in-repo-b

  echo "== a restart"
  quit_marley
  # No path, so Marley restores the window it had.
  open_path ""
  launch_marley
  settle 15
  shot 818-05-restored
}
