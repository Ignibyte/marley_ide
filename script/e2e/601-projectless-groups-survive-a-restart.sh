# shellcheck shell=bash
# #601's e2e test: projectless groups survive a restart. A scratch project `repo`. The rail's empty
# space makes a group Web, whose Browser tab shows a local page, and a group Scratch with two
# terminals, one moved to /tmp (`before`). Marley quits through its palette and starts again on
# the same profile with no path, as the app menu starts it: both groups are back, in their
# order, with their names (`after`, REQ-001),
# Scratch's terminals in `~` and /tmp and Web's tab on its page (`after`, REQ-002). Web's tab
# comes back on Web's own browser, the unit that ran before the quit, with no new one
# (`units.txt`).
compositor sway

EMPTY_X=${EMPTY_X:-130}
EMPTY_Y=${EMPTY_Y:-760}
# The first group's + under `repo` and its terminal, as #600's run found it, and the second
# group's + once the first holds its Browser tab.
FIRST_PLUS_X=${FIRST_PLUS_X:-236}
FIRST_PLUS_Y=${FIRST_PLUS_Y:-188}
SECOND_PLUS_X=${SECOND_PLUS_X:-236}
SECOND_PLUS_Y=${SECOND_PLUS_Y:-280}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$E2E_WORK/page.html" <<'HTML'
<!doctype html><title>Group page</title><h1>A page in the Web group</h1>
HTML
  open_path "$E2E_WORK/repo"
}

browser_units() {
  systemctl --user list-units 'marley-browser-*' --no-legend --plain 2>/dev/null | awk '{print $1}' | sort
}

# Picks the `$1`th entry of an open context menu with the keys: the first is selected as it opens.
menu_entry() {
  local step
  for ((step = 1; step < $1; step++)); do
    press "" Down
  done
  press "" Return
}

# Makes a group named `$1` from the empty space's menu.
new_group() {
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 1
  menu_entry 1
  settle 1
  type_text "$1"
  press "" Return
  settle 4
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== Web, with a Browser tab on a local page"
  browser_units >"$E2E_WORK/units-start.txt"
  new_group "Web"
  click "$FIRST_PLUS_X" "$FIRST_PLUS_Y"
  settle 1
  menu_entry 2
  settle 10
  type_text "file://$E2E_WORK/page.html"
  press "" Return
  settle 4
  browser_units >"$E2E_WORK/units-before.txt"
  comm -13 "$E2E_WORK/units-start.txt" "$E2E_WORK/units-before.txt" >"$E2E_WORK/web-unit.txt"
  cat "$E2E_WORK/web-unit.txt"

  echo "== Scratch, with a terminal in /tmp and one at home"
  new_group "Scratch"
  click "$SECOND_PLUS_X" "$SECOND_PLUS_Y"
  settle 1
  menu_entry 1
  settle 4
  type_text "cd /tmp"
  press "" Return
  settle 2
  click "$SECOND_PLUS_X" "$SECOND_PLUS_Y"
  settle 1
  menu_entry 1
  settle 4
  shot before

  echo "== quit, and start again on the same profile, as the app menu starts it"
  quit_marley
  # With no path, as a launch from the menu: a path is an open request, which Zed answers
  # instead of restoring the last session.
  open_path ""
  launch_marley
  settle 15
  shot after
  browser_units >"$E2E_WORK/units-after.txt"
  {
    echo "Web's unit before the quit:"
    cat "$E2E_WORK/web-unit.txt"
    echo "units new after the restart:"
    comm -13 "$E2E_WORK/units-before.txt" "$E2E_WORK/units-after.txt"
  } | tee "$(shot_file units.txt)"
  expect "Web had a browser of its own" test -s "$E2E_WORK/web-unit.txt"
  expect "no new browser after the restart" \
    test -z "$(comm -13 "$E2E_WORK/units-before.txt" "$E2E_WORK/units-after.txt")"
}
