# shellcheck shell=bash
# #799's visual check: the Component Library and the components it draws.
# - `marley: open component library` opens the tab (`799-01-palette`, REQ-001).
# - Scrolled down the tab: every catalog type with its description, its example drawn and the
#   example's JSON, group by group (`799-02-layout` to `799-05-status`, REQ-002).
# - A table pasted into the scratch box is drawn under it (`799-06-scratch`, REQ-003); a tree with an
#   unknown type shows the error with its JSON path and draws nothing (`799-07-error`, REQ-004); a
#   table of 600 rows draws 500 and a line naming the 100 more (`799-08-limit`, REQ-005).
# - With the tab closed, Home's Components button opens it (`799-09-home`, REQ-001); with it
#   closed again, the Settings window's Component Library link opens it (`799-10-settings`,
#   REQ-001).
# - Under a light theme the tones follow the theme (`799-11-theme`, REQ-006).
compositor sway

# The scratch box, the Default group's header (its Home page), Home's Components button and the
# Settings window's link, as the first run's shots found them.
SCRATCH_X=${SCRATCH_X:-700}
SCRATCH_Y=${SCRATCH_Y:-250}
DEFAULT_X=${DEFAULT_X:-70}
DEFAULT_Y=${DEFAULT_Y:-89}
COMPONENTS_X=${COMPONENTS_X:-869}
COMPONENTS_Y=${COMPONENTS_Y:-614}
SETTINGS_LINK_X=${SETTINGS_LINK_X:-1470}
SETTINGS_LINK_Y=${SETTINGS_LINK_Y:-235}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # The three trees the scratch box is given: a table with badges, a tree with an unknown type
  # deep inside, and a table of 600 rows.
  cat >"$E2E_WORK/table.json" <<'JSON'
{"type": "card", "title": "Services", "children": [
  {"type": "table", "columns": ["Service", {"label": "Latency", "align": "right"}, "State"],
   "rows": [["api", "42 ms", {"type": "badge", "text": "up", "tone": "success"}],
            ["worker", "310 ms", {"type": "badge", "text": "slow", "tone": "warning"}]],
   "striped": true}]}
JSON
  cat >"$E2E_WORK/error.json" <<'JSON'
{"type": "column", "children": [{"type": "text", "text": "fine"}, {"type": "gauge", "value": 3}]}
JSON
  cat >"$E2E_WORK/callout.json" <<'JSON'
{"type": "column", "children": [
  {"type": "callout", "tone": "warning", "title": "Migration pending", "text": "Run the migration before the next deploy."},
  {"type": "row", "children": [{"type": "badge", "text": "passed", "tone": "success"},
    {"type": "badge", "text": "failed", "tone": "error"}]},
  {"type": "progress", "label": "Indexing", "value": 64}]}
JSON
  python3 - "$E2E_WORK/big.json" <<'PY'
import json, sys
rows = [[f"row {index}", index] for index in range(600)]
json.dump({"type": "table", "columns": ["Name", "Number"], "rows": rows}, open(sys.argv[1], "w"))
PY
  open_path "$E2E_WORK/repo"
}

# Runs `$1` from the command palette, the name, a pause and Return in one `wtype`.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

# Replaces the scratch box's text with the file `$1`, through the run's clipboard.
paste_into_scratch() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy <"$1"
  click "$SCRATCH_X" "$SCRATCH_Y"
  settle 1
  press CTRL a
  settle 0.5
  press CTRL v
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the palette opens the Component Library"
  palette "marley: open component library"
  settle 3
  shot 799-01-palette

  echo "== the groups, scrolled"
  pointer_to 1000 600
  scroll 8
  settle 1
  shot 799-02-layout
  scroll 10
  settle 1
  shot 799-03-text-data
  scroll 10
  settle 1
  shot 799-04-charts
  scroll 6
  settle 1
  shot 799-05-sparkline
  scroll 8
  settle 1
  shot 799-05-status
  scroll 12
  settle 1
  shot 799-05-status-end
  scroll -100
  settle 1

  echo "== the scratch box"
  paste_into_scratch "$E2E_WORK/table.json"
  shot 799-06-scratch
  paste_into_scratch "$E2E_WORK/error.json"
  shot 799-07-error
  paste_into_scratch "$E2E_WORK/big.json"
  # Scroll distances differ from run to run, so the table's end is found in a series of shots.
  pointer_to 1000 600
  local part
  for part in 01 02 03 04 05 06 07 08 09 10 11 12 13 14; do
    scroll 16
    settle 0.5
    shot "799-08-limit-$part"
  done
  scroll -400
  settle 1

  echo "== a callout, badges and a progress bar in the scratch box"
  paste_into_scratch "$E2E_WORK/callout.json"
  shot 799-06b-callout

  echo "== the tab closed; Home's Components button"
  press CTRL w
  settle 2
  click "$DEFAULT_X" "$DEFAULT_Y"
  settle 2
  shot 799-09a-home
  if ((COMPONENTS_X > 0)); then
    click "$COMPONENTS_X" "$COMPONENTS_Y"
    settle 3
    shot 799-09-home
  fi

  echo "== the tab closed; the Settings window's link"
  press CTRL w
  settle 2
  palette "zed: open settings"
  settle 4
  type_text "Component Library"
  settle 2
  shot 799-10a-settings
  if ((SETTINGS_LINK_X > 0)); then
    click "$SETTINGS_LINK_X" "$SETTINGS_LINK_Y"
    settle 3
    shot 799-10-settings
  fi

  echo "== a light theme"
  palette "theme selector: toggle"
  settle 2
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "One Light" -s 800 -k Return
  settle 3
  paste_into_scratch "$E2E_WORK/table.json"
  shot 799-11-theme
}
