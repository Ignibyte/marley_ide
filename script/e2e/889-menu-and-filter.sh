# shellcheck shell=bash
# #889's visual check: the Launch section of a project's + menu and the block filter, as the
# Workflows board draws them. A scratch repository whose `.zed/marley.json` names `Dev` and
# `Docs preview`, and a terminal that prints forty lines.
# - The project's +: the menu with its Launch section (`889-01-menu`, REQ-001).
# - The block filtered for "FAILED": the field, the count beside it, Aa, .* and Not
#   (`889-02-filter`, REQ-002).
# - Marley Day (`889-03-day`, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-0}
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/.zed"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$repo/.zed/marley.json" <<'JSON'
{
  "launch": {
    "Dev": { "items": [{ "terminal": "echo dev", "title": "dev server" }] },
    "Docs preview": { "items": [{ "terminal": "echo docs", "title": "docs" }] }
  }
}
JSON
  for index in $(seq 1 40); do
    if ((index % 13 == 0)); then
      printf 'test walk::case_%s ... FAILED\n' "$index"
    else
      printf 'test walk::case_%s ... ok\n' "$index"
    fi
  done >"$repo/tests.log"
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the project's + menu"
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  pointer_to 900 700
  settle 1
  shot 889-01-menu
  press "" Escape
  settle 1
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== the block filtered"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "cat tests.log"
  press "" Return
  settle 2
  press "ALT SHIFT" f
  settle 2
  type_text "FAILED"
  settle 2
  pointer_to 1500 950
  settle 1
  shot 889-02-filter

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 889-03-day
}
