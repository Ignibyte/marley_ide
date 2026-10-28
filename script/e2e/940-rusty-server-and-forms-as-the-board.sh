# shellcheck shell=bash
# #940's visual check: Rusty's Server page, Secrets and Skills as the boards draw them. #894's
# wrapper runs `marley_rusty`'s stand-in `rusty-mcp`, which fails while `$E2E_WORK/rusty-down`
# exists. The stand-in's state holds #666's settings and embedding status, #876's skills (one
# staged with a backtick its scan finds, one active with a script) and made-up secrets behind a
# made-up PIN, never the user's Rusty.
# - The Server page connected: the header line, the status card, the brain search bar and the ten
#   rows with plain names (`940-01-server`, REQ-001, REQ-002, REQ-004, REQ-005).
# - `rusty: open secrets`: the locked card (`940-04-locked`, REQ-007); the PIN typed and Unlock:
#   "Unlocked · locks in 5 m" (`940-05-unlocked`, REQ-006).
# - `rusty: open skills`: the staged skill's row with "staged by an agent", its scan finding,
#   Reject and Approve Anyway, and the active script's row with Run (`940-06-skills`, REQ-008,
#   REQ-009); New Skill: "Stage it for approval" and Make (`940-07-new-skill`, REQ-010).
# - Marley Day: Skills and the Server page (`940-08-day`, `940-08-day-server`).
# - The flag made and the stand-in ended: the Secrets tab shows #894's card, its reason with
#   "Trying again in N s." (`940-09-dropped-tab`, REQ-011); a Rusty action is refused while
#   Rusty is down, so the tab is one already open; the Server page shows the reason, the
#   countdown and Try Now (`940-02-dropped`, REQ-003); the flag gone and Try Now clicked:
#   Connected, checked a moment ago (`940-03-back`, REQ-003).
compositor sway

STOP_AT=${STOP_AT:-0}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-850}
# From the first runs' shots: the Secrets tab's PIN field, Skills' New Skill, and the Server
# page's Try Now.
PIN_X=${PIN_X:-500}
PIN_Y=${PIN_Y:-200}
NEW_X=${NEW_X:-320}
NEW_Y=${NEW_Y:-156}
TRY_X=${TRY_X:-1203}
TRY_Y=${TRY_Y:-315}
# The Secrets tab in the tab bar.
SECRETS_TAB_X=${SECRETS_TAB_X:-550}
SECRETS_TAB_Y=${SECRETS_TAB_Y:-51}
PIN=tour-pin-42

store() { echo "$E2E_WORK/rusty/skills"; }

# A skill `$1` with the status `$2` (`active` or `pending`), the description `$3`, its body from
# stdin (#876's).
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

# This run's stand-in processes, by the state folder in their environment.
stand_in_pids() {
  local pid
  for pid in $(pgrep -f 'stand_in/rusty-mcp'); do
    if grep -qz "RUSTY_STAND_IN_STATE=$E2E_WORK/rusty" "/proc/$pid/environ" 2>/dev/null; then
      echo "$pid"
    fi
  done
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/rusty-mcp" <<SH
#!/bin/sh
# Fails while the flag exists, so Marley's Rusty reads as dropped (#894's).
if [ -e "$E2E_WORK/rusty-down" ]; then
  echo "rusty-mcp: down for the scenario" >&2
  exit 1
fi
exec "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "\$@"
SH
  chmod +x "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  profile_setting marley.rusty '{"enabled": true, "connection": "embedded", "agent_tools": true}'
  profile_setting marley.tmux_hosts '[]'
  cat >"$E2E_WORK/rusty/settings.json" <<'JSON'
{
  "embedding_provider": "ollama",
  "notes_path": "notes",
  "openai_api_key": "made-up-value-for-the-tour",
  "pin_timeout_minutes": "5"
}
JSON
  cat >"$E2E_WORK/rusty/semantic.json" <<'JSON'
{"provider": "ollama:nomic-embed-text", "stats": {"model": "nomic-embed-text", "dims": 768, "pages": 42, "chunks": 130}, "stale": 3}
JSON
  printf '{"github_token": "made-up-one", "openai_api_key": "made-up-two"}\n' \
    >"$E2E_WORK/rusty/secrets.json"
  printf '%s' "$PIN" >"$E2E_WORK/rusty/pin"
  printf '# Release notes\n\nGather the week'"'"'s merged changes and write them for people.\n' |
    skill release-notes active "Write the week's release notes"
  printf '#!/bin/bash\necho "Drafting release notes for the week"\n' \
    >"$(store)/.claude/skills/release-notes/notes-draft.sh"
  chmod +x "$(store)/.claude/skills/release-notes/notes-draft.sh"
  # The backtick is the point: Rusty's scan finds it.
  skill quick-shell pending "Start every answer with the date" <<'SKILL'
# Quick shell

Run !`date` before anything else.
SKILL
  # A key for the Rusty's Server page, which the settings search does not list (#666's).
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-r": ["zed::OpenSettingsAt", {"path": "marley.rusty.server"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
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

  echo "== the Server page"
  press "CTRL ALT SHIFT" r
  settle 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-01-server
  press "CTRL" w
  settle 2
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Secrets"
  palette "rusty: open secrets" 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-04-locked
  click "$PIN_X" "$PIN_Y"
  settle 1
  type_text "$PIN"
  press "" Return
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-05-unlocked

  echo "== Skills"
  palette "rusty: open skills" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-06-skills
  if [[ $STOP_AT == 2 ]]; then
    return
  fi
  click "$NEW_X" "$NEW_Y"
  settle 2
  shot 940-07-new-skill
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-08-day
  press "CTRL ALT SHIFT" r
  settle 4
  shot 940-08-day-server
  press "CTRL" w
  settle 2
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== dropped"
  touch "$E2E_WORK/rusty-down"
  for pid in $(stand_in_pids); do
    kill "$pid"
  done
  # Marley sees the loss at its next ping, a few seconds on.
  settle 12
  click "$SECRETS_TAB_X" "$SECRETS_TAB_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-09-dropped-tab
  press "CTRL ALT SHIFT" r
  settle 3
  shot 940-02-dropped
  if [[ $STOP_AT == 3 ]]; then
    return
  fi
  rm -f "$E2E_WORK/rusty-down"
  click "$TRY_X" "$TRY_Y"
  # A try already running when Try Now is clicked takes up to 5 s to fail; the next one connects.
  settle 12
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 940-03-back
}
