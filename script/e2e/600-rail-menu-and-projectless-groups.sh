# shellcheck shell=bash
# #600's e2e test: the rail's right-click menu and projectless groups. A scratch project `repo`.
# A right-click under its rows opens the empty space's menu (`menu`, REQ-001); New Group… named
# Scratch lists a group with its icon, chevron and + (`group`, REQ-002); its + offers no thread,
# worktree or Launch entry (`group-plus`, REQ-009) and its New Terminal starts in the home folder
# (`home`, REQ-003); the empty space's New Terminal makes the Home group with a terminal
# (`home-group`, REQ-004); Scratch's New Browser Tab starts one more `marley-browser-` unit
# (`units.txt`, REQ-005). A second launch hands over `repo-b`, which is then removed so the Add
# Project list offers it; with Scratch shown, adding `repo-b` from that list keeps Scratch
# (`kept`, REQ-006). Rename Group… makes it Tools (`renamed`, REQ-007) and Remove Group takes it
# away (`removed`, REQ-008).
compositor sway

# Points from the first run's shots: the empty space under the rows, Scratch's + and header, the
# rail's Add Project, and repo-b's header once it is listed.
EMPTY_X=${EMPTY_X:-130}
EMPTY_Y=${EMPTY_Y:-760}
GROUP_PLUS_X=${GROUP_PLUS_X:-236}
GROUP_PLUS_Y=${GROUP_PLUS_Y:-188}
GROUP_X=${GROUP_X:-110}
GROUP_Y=${GROUP_Y:-188}
# Scratch's header once repo-b, with its terminal, is listed above it again.
GROUP_BELOW_Y=${GROUP_BELOW_Y:-280}
ADD_PROJECT_X=${ADD_PROJECT_X:-244}
ADD_PROJECT_Y=${ADD_PROJECT_Y:-16}
REPO_B_X=${REPO_B_X:-110}
REPO_B_Y=${REPO_B_Y:-96}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/repo-b"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  printf '# repo-b\n' >"$E2E_WORK/repo-b/README.md"
  open_path "$E2E_WORK/repo"
}

browser_units() {
  systemctl --user list-units 'marley-browser-*' --no-legend --plain 2>/dev/null | awk '{print $1}' | sort
}

# A second launch on this profile with `$1`, which hands it to the running Marley (#513).
hand_off() {
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null
}

# Picks the `$1`th entry of an open context menu with the keys: the first is selected as it opens.
menu_entry() {
  local step
  for ((step = 1; step < $1; step++)); do
    press "" Down
  done
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a right-click on the empty space"
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 1
  shot menu

  echo "== New Group…, named Scratch"
  menu_entry 1
  settle 1
  type_text "Scratch"
  press "" Return
  settle 4
  shot group

  echo "== Scratch's +, then New Terminal and pwd"
  click "$GROUP_PLUS_X" "$GROUP_PLUS_Y"
  settle 1
  shot group-plus
  menu_entry 1
  settle 4
  type_text "pwd"
  press "" Return
  settle 2
  shot home

  echo "== the empty space's New Terminal makes the Home group"
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 1
  menu_entry 2
  settle 6
  shot home-group

  echo "== Scratch's New Browser Tab starts a browser of its own"
  browser_units >"$E2E_WORK/units-before.txt"
  click "$GROUP_PLUS_X" "$GROUP_PLUS_Y"
  settle 1
  menu_entry 2
  settle 12
  browser_units >"$E2E_WORK/units-after.txt"
  comm -13 "$E2E_WORK/units-before.txt" "$E2E_WORK/units-after.txt" | tee "$(shot_file units.txt)"
  expect "one more browser unit for Scratch" \
    test "$(comm -13 "$E2E_WORK/units-before.txt" "$E2E_WORK/units-after.txt" | wc -l)" -eq 1
  shot browser

  echo "== repo-b, handed over, then removed, so Add Project lists it"
  hand_off "$E2E_WORK/repo-b"
  settle 5
  # Trusts repo-b.
  press "" Return
  settle 3
  shot repo-b
  click "$REPO_B_X" "$REPO_B_Y" right
  settle 1
  shot repo-b-menu
  # Move Project Up is disabled at the top, so the menu opens on Move Project Down.
  menu_entry 3
  settle 4
  shot repo-b-removed

  echo "== with Scratch shown, Add Project opens repo-b and Scratch stays"
  click "$GROUP_X" "$GROUP_Y"
  settle 2
  shot scratch-shown
  click "$ADD_PROJECT_X" "$ADD_PROJECT_Y"
  settle 2
  type_text "repo-b"
  settle 1
  shot add-project
  press "" Return
  settle 6
  # Trusts repo-b again, if Zed asks.
  press "" Escape
  settle 2
  shot kept

  echo "== Rename Group…, to Tools"
  click "$GROUP_X" "$GROUP_BELOW_Y" right
  settle 1
  menu_entry 1
  settle 1
  type_text "Tools"
  press "" Return
  settle 2
  shot renamed

  echo "== Remove Group, which stops its browser"
  local scratch_unit
  scratch_unit=$(comm -13 "$E2E_WORK/units-before.txt" "$E2E_WORK/units-after.txt")
  click "$GROUP_X" "$GROUP_BELOW_Y" right
  settle 1
  menu_entry 2
  settle 8
  shot removed
  systemctl --user is-active "$scratch_unit" | tee "$(shot_file removed-unit.txt)" || true
  expect "the removed group's browser stopped" \
    test "$(systemctl --user is-active "$scratch_unit")" != active
}
