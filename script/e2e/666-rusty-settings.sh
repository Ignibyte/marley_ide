# shellcheck shell=bash
# #666's visual check: Rusty's own settings on the Rusty's Server page. `marley_rusty`'s stand-in
# `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`)
# whose `settings.json` holds made-up values, a credential-looking one among them, which the
# stand-in masks as Rusty does; its `semantic.json` gives the embedding status. Never the user's
# Rusty (R-D8). A run-only key opens the Settings window on the page, as #643's does.
#
# The page (`666-01-settings`), a value saved (`666-02-saved`), a masked one kept
# (`666-03-masked-kept`) and replaced (`666-04-masked-replaced`), a key added (`666-05-added`),
# and the embedding status read again (`666-06-status`).
compositor sway

# Where things sit in the Settings window, from the first runs' shots: a field's left part, the
# rows' fields, the add row's fields and Set.
FIELD_X=${FIELD_X:-1420}
PIN_Y=${PIN_Y:-777}
MODEL_Y=${MODEL_Y:-622}
MASKED_Y=${MASKED_Y:-875}
NEW_KEY_X=${NEW_KEY_X:-1160}
NEW_VALUE_X=${NEW_VALUE_X:-1400}
NEW_Y=${NEW_Y:-920}
SET_X=${SET_X:-1553}
SCROLL_X=${SCROLL_X:-1300}
SCROLL_Y=${SCROLL_Y:-500}

# Whether the stand-in stores `$2` under `$1`.
stored_is() {
  python3 - "$E2E_WORK/rusty/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, sys

settings = json.loads(pathlib.Path(sys.argv[1]).read_text())
sys.exit(0 if settings.get(sys.argv[2]) == sys.argv[3] else 1)
SETTINGS
}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

server_page() {
  press "CTRL ALT SHIFT" r
  settle 4
}

# Clicks a field at height `$1`, empties it and types `$2`, then Enter.
retype() {
  click "$FIELD_X" "$1"
  settle 1
  press CTRL a
  type_text "$2"
  press "" Return
  settle 3
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  cat >"$E2E_WORK/rusty/settings.json" <<'JSON'
{
  "default_agent": "claude",
  "embedding_provider": "ollama",
  "notes_path": "notes",
  "openai_api_key": "made-up-value-for-the-tour",
  "pin_timeout_minutes": "5"
}
JSON
  cat >"$E2E_WORK/rusty/semantic.json" <<'JSON'
{"provider": "ollama:nomic-embed-text", "stats": {"model": "nomic-embed-text", "dims": 768, "pages": 42, "chunks": 130}, "stale": 3}
JSON
  # A key for the Rusty's Server page, which the settings search does not list.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-r": ["zed::OpenSettingsAt", {"path": "marley.rusty.server"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch repository.
  press "" Return
  settle 4

  echo "== the page"
  server_page
  shot 666-01-settings
  expect "the status was read" bash -c "grep -q ' tools/call brain_semantic_status ' '$E2E_WORK/rusty/calls'"

  echo "== a value saved"
  retype "$PIN_Y" 10
  shot 666-02-saved
  expect "the value reached Rusty" stored_is pin_timeout_minutes 10

  echo "== the masked value kept"
  pointer_to "$SCROLL_X" "$SCROLL_Y"
  scroll 10
  settle 1
  click "$FIELD_X" "$MASKED_Y"
  settle 1
  press "" Return
  settle 2
  shot 666-03-masked-kept
  expect "nothing was written over the credential" stored_is openai_api_key made-up-value-for-the-tour

  echo "== the masked value replaced"
  type_text "another-made-up-value"
  press "" Return
  settle 3
  shot 666-04-masked-replaced
  expect "the new value reached Rusty" stored_is openai_api_key another-made-up-value

  echo "== a key added"
  click "$NEW_KEY_X" "$NEW_Y"
  type_text "tour_note"
  click "$NEW_VALUE_X" "$NEW_Y"
  type_text "added on the tour"
  click "$SET_X" "$NEW_Y"
  settle 3
  shot 666-05-added
  expect "the key reached Rusty" stored_is tour_note "added on the tour"

  echo "== the status read again"
  printf '%s\n' '{"provider": "ollama:nomic-embed-text-v2", "stats": {"model": "nomic-embed-text-v2", "dims": 768, "pages": 0, "chunks": 0}, "stale": 42}' \
    >"$E2E_WORK/rusty/semantic.json"
  pointer_to "$SCROLL_X" "$SCROLL_Y"
  scroll -10
  settle 1
  retype "$MODEL_Y" nomic-embed-text-v2
  shot 666-06-status
  calls | grep -E 'setting_set|brain_semantic_status' | tail -8
}
