# shellcheck shell=bash
# #876's visual check: Rusty's Memory and Skills chips as the RustyTabs board draws them; Secrets and
# Capture shot as they are, the board's shape already. `marley_rusty`'s stand-in `rusty-mcp` serves #664's made-up memories, #665's made-up
# skills (two active, two staged by an agent, one with a script) and #667's two made-up secrets,
# never the user's Rusty.
# - `rusty: open memory`: each importance a chip in its colour (`876-01-memory`, REQ-001).
# - `rusty: open skills`: active green, pending yellow (`876-02-skills`, REQ-002).
# - `rusty: open secrets`, no PIN (`876-03-secrets`).
# - `rusty: capture to today` (`876-04-capture`).
# - Marley Day, Memory (`876-05-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}

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

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.tmux_hosts '[]'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  cat >"$E2E_WORK/rusty/memories.json" <<'JSON'
[
  {"id": "m1", "category": "preference", "importance": "high", "content": "Prefers plain words in docs and commit messages", "type": "memory", "source": "app", "created_at": 1790000000, "updated_at": 1790000000},
  {"id": "m2", "category": "fact", "importance": "normal", "content": "The dev box builds with one cargo command at a time", "type": "memory", "source": "mcp", "created_at": 1790086400, "updated_at": 1790086400},
  {"id": "m3", "category": "context", "importance": "normal", "content": "Working on the Marley shell this month", "type": "memory", "source": "mcp", "created_at": 1790172800, "updated_at": 1790172800},
  {"id": "m4", "category": "fact", "importance": "low", "content": "The old app lived in a Qt window", "type": "memory", "source": "app", "created_at": 1790259200, "updated_at": 1790259200}
]
JSON
  printf '# Release notes\n\nGather the week'"'"'s merged changes and write them for people.\n' |
    skill release-notes active "Write the week's release notes"
  printf '#!/bin/bash\necho "Drafting release notes for the week"\n' \
    >"$(store)/.claude/skills/release-notes/notes-draft.sh"
  chmod +x "$(store)/.claude/skills/release-notes/notes-draft.sh"
  printf '# Tidy commits\n\nOne change per commit, a plain subject.\n' |
    skill tidy-commits active "Keep commits small and plain"
  # The backtick is the point: Rusty's scan finds it.
  skill quick-shell pending "Start every answer with the date" <<'SKILL'
# Quick shell

Run !`date` before anything else.
SKILL
  printf '{"github_token": "made-up-one", "openai_api_key": "made-up-two"}\n' \
    >"$E2E_WORK/rusty/secrets.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Memory"
  palette "rusty: open memory" 4
  pointer_to 1500 900
  settle 1
  shot 876-01-memory

  echo "== Skills"
  palette "rusty: open skills" 4
  shot 876-02-skills

  echo "== Secrets"
  palette "rusty: open secrets" 4
  shot 876-03-secrets

  echo "== Capture"
  palette "rusty: capture to today" 3
  shot 876-04-capture
  press "" Escape
  settle 1
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  palette "rusty: open memory" 3
  profile_setting theme '"Marley Day"'
  settle 4
  shot 876-05-day
}
