# shellcheck shell=bash
# #660's visual check: a decision's follow-up recorded from the Decisions tab, and the rows'
# last follow-up and successor. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`,
# serves a scratch vault of five made-up decision pages, never the user's brain (R-D8), with its
# day fixed at 2026-10-03 by its state folder's `today`, and logs each call with its pid. Its
# `brain_follow_up` checks and writes as Rusty's does, and its watcher announces the page it wrote.
#
# `660-01-rows` to `660-13-superseded`, one per step below.
compositor sway

# Where things sit, from the first runs' shots.
BRAIN_X=${BRAIN_X:-42}
RAIL_HEADER_Y=${RAIL_HEADER_Y:-16}
DECISIONS_ENTRY_X=${DECISIONS_ENTRY_X:-96}
FIXED_ROW_Y=${FIXED_ROW_Y:-49}
AWAY_X=${AWAY_X:-800}
AWAY_Y=${AWAY_Y:-700}
# The rows with Due above them: Due's two rows, then every decision's five.
FIRST_DUE_Y=${FIRST_DUE_Y:-176}
FOLLOW_UP_X=${FOLLOW_UP_X:-1310}
SUCCESSOR_X=${SUCCESSOR_X:-1280}
ONE_BIG_CRATE_Y=${ONE_BIG_CRATE_Y:-386}
# The rows once nothing is due: every decision's five under the count.
ROW_X=${ROW_X:-400}
BATCH_Y=${BATCH_Y:-238}
SPLIT_Y=${SPLIT_Y:-269}
LAST_ROW_Y=${LAST_ROW_Y:-300}
# The form.
STATUS_Y=${STATUS_Y:-160}
REVISED_X=${REVISED_X:-800}
SUPERSEDED_X=${SUPERSEDED_X:-990}
DAY_X=${DAY_X:-800}
DAY_Y=${DAY_Y:-330}
KEPT_X=${KEPT_X:-626}
RECORD_X=${RECORD_X:-1032}
# Record under a Kept form, which has no day field.
KEPT_RECORD_Y=${KEPT_RECORD_Y:-306}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1`.
asked() { calls | grep -c " tools/call $1 " || true; }

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up decision page: slug `$1`, title `$2`, status `$3`, decided `$4`, and any more
# frontmatter lines in `$5`.
decision_page() {
  local path
  path=$(vault)/decisions/$1.md
  mkdir -p "$(dirname "$path")"
  {
    printf -- '---\ntitle: %s\nstatus: %s\ndecided: %s\n' "$2" "$3" "$4"
    if [[ -n ${5:-} ]]; then
      printf '%s\n' "$5"
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
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '2026-10-03\n' >"$E2E_WORK/rusty/today"
  decision_page try-the-stand-in "Try the stand-in" decided 2026-09-20 "follow_up_by: 2026-09-27"
  decision_page ship-the-rail-switch "Ship the rail switch" decided 2026-09-30 \
    "follow_up_by: 2026-10-03"
  decision_page batch-tickets-by-five "Batch tickets by five" revised 2026-09-12 \
    "$(printf 'follow_up_by: 2026-10-10\nfollowed_up: 2026-09-26')"
  decision_page split-the-crate "Split the crate" decided 2026-09-02
  decision_page one-big-crate "One big crate" superseded 2026-09-01 \
    "superseded_by: decisions/split-the-crate"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the rows"
  click "$BRAIN_X" "$RAIL_HEADER_Y"
  settle 3
  click "$DECISIONS_ENTRY_X" "$FIXED_ROW_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 660-01-rows

  echo "== the successor opened"
  click "$SUCCESSOR_X" "$ONE_BIG_CRATE_Y"
  settle 3
  shot 660-02-successor-opened

  echo "== the form"
  palette "rusty: open decisions"
  settle 3
  click "$FOLLOW_UP_X" "$FIRST_DUE_Y"
  settle 1
  # Off the row under the form, whose tooltip would cover it.
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 660-03-form

  echo "== revised, two lines"
  click "$REVISED_X" "$STATUS_Y"
  settle 1
  type_text "It held, but the stand-in needed a today file."
  press SHIFT Return
  type_text "Look again after the next batch."
  settle 1
  shot 660-04-revised

  echo "== refused"
  local page before
  page=$(vault)/decisions/try-the-stand-in.md
  before=$(cksum <"$page")
  click "$DAY_X" "$DAY_Y"
  settle 1
  type_text "soon"
  press "" Return
  settle 2
  shot 660-05-refused
  expect "the refused follow-up wrote nothing" test "$(cksum <"$page")" = "$before"

  echo "== recorded"
  press CTRL a
  type_text "2026-10-24"
  press "" Return
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 660-06-recorded
  expect "one brain_follow_up revised to 2026-10-24 with two lines" \
    test "$(calls | grep ' tools/call brain_follow_up ' | grep -c '"follow_up_by": "2026-10-24"' | tr -d ' ')" = 1
  expect "the outcome kept its line break" \
    grep -q 'today file.\\nLook again' "$E2E_WORK/rusty/calls"

  echo "== escape"
  local sent
  sent=$(asked brain_follow_up)
  click "$FOLLOW_UP_X" "$FIRST_DUE_Y"
  settle 2
  click "$KEPT_X" "$STATUS_Y"
  settle 1
  type_text "Not sent."
  press "" Escape
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 660-07-escape
  expect "no brain_follow_up after Escape" test "$(asked brain_follow_up)" = "$sent"

  echo "== kept"
  click "$FOLLOW_UP_X" "$FIRST_DUE_Y"
  settle 2
  click "$KEPT_X" "$STATUS_Y"
  settle 1
  type_text "Shipped; the switch works."
  settle 1
  click "$RECORD_X" "$KEPT_RECORD_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 660-08-kept

  expect "Ship the rail switch kept, with no day" \
    test "$(calls | grep ' tools/call brain_follow_up ' | grep 'ship-the-rail-switch' | grep '"status": "kept"' | grep -vc follow_up_by | tr -d ' ')" = 1

  echo "== the menu"
  click "$ROW_X" "$SPLIT_Y" right
  settle 1
  # Off the row, whose tooltip would cover the menu; a move does not close it.
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 660-09-menu
  press "" Escape
  settle 1

  echo "== a superseded row's menu"
  click "$ROW_X" "$LAST_ROW_Y" right
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 660-10-superseded-menu
  press "" Escape
  settle 1

  echo "== the successor picker"
  click "$ROW_X" "$BATCH_Y" right
  settle 2
  press "" Return
  settle 2
  click "$SUPERSEDED_X" "$STATUS_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  type_text "split"
  settle 2
  shot 660-11-successor-picker

  echo "== the successor chosen"
  press "" Return
  settle 2
  shot 660-12-successor-chosen

  echo "== superseded"
  type_text "Split the work instead."
  press "" Return
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 660-13-superseded
  expect "the successor was sent for Batch tickets by five" \
    test "$(calls | grep ' tools/call brain_follow_up ' | grep 'batch-tickets-by-five' | grep -c '"successor": "decisions/split-the-crate"' | tr -d ' ')" = 1
  calls | grep brain_follow_up
}
