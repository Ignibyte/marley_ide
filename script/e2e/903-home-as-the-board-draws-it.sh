# shellcheck shell=bash
# #903's visual check: Home as the Home board draws it. A fresh profile opens `repo` with Rusty on
# through `marley_rusty`'s stand-in `rusty-mcp` (#782's: a scratch vault with a page and a decision
# due yesterday, made-up task lists), and stand-in `claude` and `codex` on the PATH so the prompt
# has agent chips. Home opens with `marley home`.
# - `903-02-rusty`: Home with Rusty on: the heading, the prompt, the start pills, the cards in a
#   wrapping grid with Rusty's among them, the links at the foot (REQ-001, REQ-002).
# - `903-03-day`: the same in Marley Day (REQ-001, REQ-002).
# - `903-01-night`: Rusty off, Marley Night: Home's own cards (REQ-001).
# - `903-04-started`: a task typed in the two-line box, Enter: a terminal runs the stand-in
#   `claude` with the task, so Enter still starts the agent (REQ-001).
compositor sway

AWAY_X=${AWAY_X:-1580}
AWAY_Y=${AWAY_Y:-120}
# Home's prompt box, from the first run's shots.
BOX_X=${BOX_X:-900}
BOX_Y=${BOX_Y:-218}

setup() {
  local bin=$E2E_WORK/bin vault=$E2E_WORK/rusty/vault name
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$vault/notes" "$vault/decisions"
  # The stand-ins first on the terminals' PATH, so no real agent CLI runs (PR-687).
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  for name in claude codex; do
    cat >"$bin/$name" <<'SH'
#!/bin/sh
# A stand-in agent CLI: it says what it was asked, and waits.
printf 'stand-in %s was asked: %s\n' "$(basename "$0")" "$*"
exec sleep 600
SH
    chmod +x "$bin/$name"
  done
  export PATH=$bin:$PATH
  rm -rf "$E2E_PROFILE/db"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.tmux_hosts '[]'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# A note\n\nA page in the scratch vault.\n' >"$vault/notes/a-note.md"
  printf -- '---\ntitle: Ship the home page\nstatus: decided\ndecided: %s\nfollow_up_by: %s\n---\n# Ship the home page\n\nA made-up decision.\n' \
    "$(date -d '-10 days' +%F)" "$(date -d '-1 day' +%F)" >"$vault/decisions/ship-the-home-page.md"
  cat >"$E2E_WORK/rusty/tasks.json" <<'JSON'
{"groups": [
  {"id": 1, "name": "Home", "tasks": [
    {"id": 11, "title": "Water the plants"},
    {"id": 13, "title": "Return the library books"}]},
  {"id": 2, "name": "Work", "tasks": [
    {"id": 21, "title": "Draft the release notes"},
    {"id": 22, "title": "Review the rail change"}]}
]}
JSON
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts repo.
  press "" Return
  settle 4

  echo "== Home with Rusty on"
  launch_marley_with home
  settle 6
  away
  shot 903-02-rusty

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 903-03-day

  echo "== Rusty off, Marley Night"
  profile_setting theme '"Marley Night"'
  profile_setting marley.rusty.enabled false
  settle 4
  shot 903-01-night

  echo "== a task and Enter"
  click "$BOX_X" "$BOX_Y"
  settle 1
  type_text "write the README"
  settle 1
  press "" Return
  settle 6
  away
  shot 903-04-started
}
