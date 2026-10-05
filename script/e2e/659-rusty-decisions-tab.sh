# shellcheck shell=bash
# #659's visual check: System One's log renamed System One calls, and Rusty's Decisions tab over
# `brain_due`. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, serves a scratch
# vault of seven made-up decision pages and one project page, never the user's brain (R-D8), with
# its day fixed at 2026-10-03 by its state folder's `today`, and logs each call with its pid. An
# edit of a page in its vault is announced by its watcher; a `fail` file in its state folder makes
# the tool it names fail. The run's keymap binds Ctrl+Alt+Shift+Y to the old `marley::OpenDecisions`.
#
# `659-01-system-one-calls` to `659-15-off-action`, one per step below.
compositor sway

# Where things sit, from the first runs' shots.
BRAIN_X=${BRAIN_X:-42}
RAIL_HEADER_Y=${RAIL_HEADER_Y:-16}
DECISIONS_ENTRY_X=${DECISIONS_ENTRY_X:-96}
FIXED_ROW_Y=${FIXED_ROW_Y:-49}
SETTINGS_X=${SETTINGS_X:-1300}
SETTINGS_Y=${SETTINGS_Y:-600}
SETTINGS_SCROLL=${SETTINGS_SCROLL:-40}
SETTINGS_SEARCH_X=${SETTINGS_SEARCH_X:-912}
SETTINGS_SEARCH_Y=${SETTINGS_SEARCH_Y:-59}
OPEN_CALLS_X=${OPEN_CALLS_X:-1467}
OPEN_CALLS_Y=${OPEN_CALLS_Y:-882}
USE_A_RAIL_X=${USE_A_RAIL_X:-303}
USE_A_RAIL_Y=${USE_A_RAIL_Y:-386}
READ_AGAIN_X=${READ_AGAIN_X:-464}
READ_AGAIN_Y=${READ_AGAIN_Y:-143}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1`.
asked() { calls | grep -c " tools/call $1 " || true; }

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up decision page: slug `$1`, title `$2`, status `$3`, decided `$4`, follow-up day `$5`
# (none when empty), and any more frontmatter lines in `$6`.
decision_page() {
  local path
  path=$(vault)/decisions/$1.md
  mkdir -p "$(dirname "$path")"
  {
    printf -- '---\ntitle: %s\nstatus: %s\ndecided: %s\n' "$2" "$3" "$4"
    if [[ -n $5 ]]; then
      printf 'follow_up_by: %s\n' "$5"
    fi
    if [[ -n ${6:-} ]]; then
      printf '%s\n' "$6"
    fi
    printf -- '---\n# %s\n\nA made-up decision.\n' "$2"
  } >"$path"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive" "$(vault)/projects"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '2026-10-03\n' >"$E2E_WORK/rusty/today"
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": "marley::OpenDecisions"}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  printf -- '---\ntitle: Atlas\ntype: project\n---\n# Atlas\n\nNot a decision.\n' \
    >"$(vault)/projects/atlas.md"
  decision_page try-the-stand-in "Try the stand-in" decided 2026-09-20 2026-09-27
  decision_page ship-the-rail-switch "Ship the rail switch" decided 2026-09-30 2026-10-03
  decision_page keep-zeds-theme-for-every-marley-view-until-the-design-system-lands \
    "Keep Zed's theme for every Marley view until the design system lands and is tried by hand" \
    decided 2026-09-25 2026-10-20
  decision_page batch-tickets-by-five "Batch tickets by five" revised 2026-09-12 2026-10-10
  decision_page use-a-rail "Use a rail" kept 2026-09-10 ""
  decision_page split-the-crate "Split the crate" decided 2026-09-02 "" \
    "supersedes: decisions/one-big-crate"
  decision_page one-big-crate "One big crate" superseded 2026-09-01 "" \
    "superseded_by: decisions/split-the-crate"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== System One calls"
  palette "marley: open system one calls"
  settle 3
  shot 659-01-system-one-calls

  echo "== the palette"
  press CTRL w
  settle 2
  press "CTRL SHIFT" p
  settle 1
  type_text "open decisions"
  settle 2
  shot 659-02-palette
  press "" Escape
  settle 1

  echo "== the old id"
  press "CTRL ALT SHIFT" y
  settle 3
  shot 659-03-old-id

  echo "== the settings link"
  press CTRL w
  settle 2
  palette "marley: open settings"
  settle 4
  # The settings search narrows the page to the System One section, the link at its end.
  click "$SETTINGS_SEARCH_X" "$SETTINGS_SEARCH_Y"
  settle 1
  type_text "System One Calls"
  settle 3
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll "$SETTINGS_SCROLL"
  settle 2
  shot 659-04-settings-link

  echo "== opened from the settings"
  click "$OPEN_CALLS_X" "$OPEN_CALLS_Y"
  settle 3
  shot 659-05-settings-opened

  echo "== the fixed row"
  press CTRL w
  settle 2
  click "$BRAIN_X" "$RAIL_HEADER_Y"
  settle 1
  # Off the header, whose tooltip would cover the row.
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 2
  shot 659-06-fixed-row

  echo "== the Decisions tab"
  click "$DECISIONS_ENTRY_X" "$FIXED_ROW_Y"
  settle 3
  shot 659-07-decisions
  expect "brain_due was asked" test "$(asked brain_due)" -ge 1

  echo "== a decision opened"
  click "$USE_A_RAIL_X" "$USE_A_RAIL_Y"
  settle 3
  shot 659-08-opened

  echo "== found again, read once shown"
  local reads
  reads=$(asked brain_due)
  decision_page ship-the-rail-switch "Ship the rail switch" kept 2026-09-30 ""
  settle 3
  expect "no brain_due while hidden" test "$(asked brain_due)" = "$reads"
  palette "rusty: open decisions"
  settle 3
  shot 659-09-found
  expect "one more brain_due once shown" test "$(asked brain_due)" = $((reads + 1))

  echo "== live"
  decision_page try-the-stand-in "Try the stand-in" revised 2026-09-20 2026-10-17
  settle 4
  shot 659-10-live

  echo "== a failed read"
  printf 'brain_due\nThe brain is busy; try again.\n' >"$E2E_WORK/rusty/fail"
  date +%s%N >"$(vault)/refresh.txt"
  settle 4
  shot 659-11-failed

  echo "== read again"
  rm -f "$E2E_WORK/rusty/fail"
  click "$READ_AGAIN_X" "$READ_AGAIN_Y"
  settle 3
  shot 659-12-read-again

  echo "== no decisions"
  mv "$(vault)/decisions" "$E2E_WORK/decisions-away"
  settle 4
  shot 659-13-empty

  echo "== Rusty off"
  profile_setting marley.rusty.enabled false
  settle 5
  reads=$(asked brain_due)
  settle 3
  shot 659-14-off
  expect "no brain_due once off" test "$(asked brain_due)" = "$reads"

  echo "== the action while off"
  palette "rusty: open decisions"
  settle 2
  shot 659-15-off-action
  calls | grep -E 'brain_due' | tail -4
}
