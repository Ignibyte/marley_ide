# shellcheck shell=bash
# #665's visual check: the Skills tab over Rusty's skills store. `marley_rusty`'s stand-in
# `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`)
# whose `skills/` holds two active skills (one with a script) and two staged ones an agent wrote
# (one clean, one with a bang-backtick Rusty's scan finds), never the user's store (R-D8).
#
# The lists (`665-01-skills`), a skill (`665-02-skill`) saved (`665-03-saved`), a scan
# (`665-04-scan`), an approval refused and offered anyway (`665-05-approve-refused`), one approved
# (`665-06-approved`) and one rejected (`665-07-rejected`), the script (`665-08-script`) and its
# run (`665-09-run`), New Skill refused (`665-10-new`) and made (`665-11-created`), and a delete
# (`665-12-deleted`).
compositor sway

# Where things sit, from the first runs' shots: the left column's rows and New Skill, the right
# side's buttons and description field, and the New Skill form's fields, checkbox and Create.
ROW_X=${ROW_X:-400}
FIRST_ROW_Y=${FIRST_ROW_Y:-220}
ROW_H=${ROW_H:-50}
# The first script's row sits under the skills, below the Scripts header.
SCRIPT_GAP=${SCRIPT_GAP:-15}
NEW_X=${NEW_X:-318}
NEW_Y=${NEW_Y:-156}
BUTTONS_Y=${BUTTONS_Y:-166}
DELETE_X=${DELETE_X:-1318}
SAVE_X=${SAVE_X:-1270}
SCAN_X=${SCAN_X:-1228}
APPROVE_X=${APPROVE_X:-1113}
REJECT_X=${REJECT_X:-1175}
RUN_X=${RUN_X:-1262}
DESCRIPTION_X=${DESCRIPTION_X:-900}
DESCRIPTION_Y=${DESCRIPTION_Y:-231}
FORM_DESCRIPTION_X=${FORM_DESCRIPTION_X:-800}
FORM_DESCRIPTION_Y=${FORM_DESCRIPTION_Y:-236}
FORM_STAGED_X=${FORM_STAGED_X:-550}
FORM_STAGED_Y=${FORM_STAGED_Y:-407}
FORM_CREATE_X=${FORM_CREATE_X:-1033}
# Create, before and after a refusal's line pushes it down.
FORM_CREATE_Y=${FORM_CREATE_Y:-435}
FORM_REFUSED_CREATE_Y=${FORM_REFUSED_CREATE_Y:-463}

store() { echo "$E2E_WORK/rusty/skills"; }

# A skill `$1` with the status `$2` (`active` or `pending`), the description `$3`, its body from
# stdin; a staged one is an agent's, as Rusty marks them.
skill() {
  local base
  if [[ $2 == active ]]; then base=$(store)/.claude/skills; else base=$(store)/staging; fi
  mkdir -p "$base/$1"
  {
    printf -- '---\nname: %s\ndescription: "%s"\n' "$1" "$3"
    [[ $2 == pending ]] && printf 'rusty_origin: auto\n'
    printf -- '---\n'
    cat
  } >"$base/$1/SKILL.md"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

row_y() { echo $((FIRST_ROW_Y + $1 * ROW_H)); }

click_row() { click "$ROW_X" "$(row_y "$1")"; }

# Clicks the first script, under `$1` skills.
click_script() { click "$ROW_X" $((FIRST_ROW_Y + $1 * ROW_H + SCRIPT_GAP)); }

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# Release notes\n\nGather the week'"'"'s merged changes and write them for people.\n' |
    skill release-notes active "Write the week's release notes"
  printf '#!/bin/bash\necho "Drafting release notes for the week"\n' \
    >"$(store)/.claude/skills/release-notes/notes-draft.sh"
  chmod +x "$(store)/.claude/skills/release-notes/notes-draft.sh"
  printf '# Tidy commits\n\nOne change per commit, a plain subject.\n' |
    skill tidy-commits active "Keep commits small and plain"
  printf '# Fetch docs\n\nRead the library'"'"'s docs before answering.\n' |
    skill fetch-docs pending "Fetch the docs before answering"
  # The backtick is the point: Rusty's scan finds it.
  skill quick-shell pending "Start every answer with the date" <<'SKILL'
# Quick shell

Run !`date` before anything else.
SKILL
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  # Staged first: fetch-docs 0, quick-shell 1; then release-notes 2, tidy-commits 3.
  echo "== the lists"
  palette "rusty: open skills"
  settle 2
  shot 665-01-skills

  echo "== a skill"
  click_row 3
  settle 1
  shot 665-02-skill

  echo "== saved"
  click "$DESCRIPTION_X" "$DESCRIPTION_Y"
  press CTRL a
  type_text "Keep commits small, plain and signed"
  click "$SAVE_X" "$BUTTONS_Y"
  settle 3
  shot 665-03-saved
  expect "the description reached the store" \
    holds "$(store)/.claude/skills/tidy-commits/SKILL.md" "Keep commits small, plain and signed"

  echo "== a scan"
  click_row 1
  settle 1
  click "$SCAN_X" "$BUTTONS_Y"
  settle 2
  shot 665-04-scan

  echo "== an approval refused"
  click "$APPROVE_X" "$BUTTONS_Y"
  settle 2
  shot 665-05-approve-refused

  echo "== an approval"
  click_row 0
  settle 1
  click "$APPROVE_X" "$BUTTONS_Y"
  settle 3
  shot 665-06-approved
  expect "fetch-docs is active" test -f "$(store)/.claude/skills/fetch-docs/SKILL.md"

  # Now quick-shell 0 is the only staged one.
  echo "== a rejection"
  click_row 0
  settle 1
  click "$REJECT_X" "$BUTTONS_Y"
  settle 3
  shot 665-07-rejected
  expect "quick-shell is gone" test ! -e "$(store)/staging/quick-shell"

  # Three skills now: fetch-docs, release-notes, tidy-commits.
  echo "== the script"
  click_script 3
  settle 2
  shot 665-08-script

  echo "== its run"
  click "$RUN_X" "$BUTTONS_Y"
  settle 4
  shot 665-09-run

  echo "== New Skill"
  palette "rusty: open skills"
  click "$NEW_X" "$NEW_Y"
  settle 1
  type_text "Tour Skill"
  click "$FORM_CREATE_X" "$FORM_CREATE_Y"
  settle 2
  shot 665-10-new

  echo "== made"
  press CTRL a
  type_text "tour-skill"
  click "$FORM_DESCRIPTION_X" "$FORM_DESCRIPTION_Y"
  type_text "A skill for the tour"
  click "$FORM_STAGED_X" "$FORM_STAGED_Y"
  settle 1
  click "$FORM_CREATE_X" "$FORM_REFUSED_CREATE_Y"
  settle 3
  shot 665-11-created
  expect "tour-skill is staged" test -f "$(store)/staging/tour-skill/SKILL.md"

  echo "== a delete"
  click "$DELETE_X" "$BUTTONS_Y"
  settle 2
  press "" Return
  settle 3
  shot 665-12-deleted
  expect "tour-skill is gone" test ! -e "$(store)/staging/tour-skill"
}
