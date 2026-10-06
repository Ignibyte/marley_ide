# shellcheck shell=bash
# #667's visual check: the Secrets tab over Rusty's vault, and no secret in Marley's log.
# `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder
# (`RUSTY_STAND_IN_STATE`) whose `secrets.json` holds two made-up secrets and no PIN, never the
# user's vault or Rusty (R-D8). The run logs Zed's MCP client at trace (`ZED_LOG`), and the checks
# read Marley's log for Rusty's messages by size and for none of the PIN, the values or the tokens.
#
# No PIN (`667-01-no-pin`), a PIN set (`667-02-pin-set`), a wrong PIN (`667-03-wrong-pin`),
# unlocked (`667-04-unlocked`), a value shown (`667-05-revealed`), a secret added
# (`667-06-added`) and replaced (`667-07-replaced`), a delete asked and done
# (`667-08-delete-prompt`, `667-09-deleted`), locked when the window lost the focus
# (`667-10-locked-on-blur`) and when the unlock ran out (`667-11-expired`).
compositor sway

# Where things sit, from the first runs' shots: the PIN fields, the unlock field, the set row's
# fields, and the names' buttons.
PIN_X=${PIN_X:-400}
AGAIN_X=${AGAIN_X:-670}
PIN_Y=${PIN_Y:-200}
UNLOCK_X=${UNLOCK_X:-400}
UNLOCK_Y=${UNLOCK_Y:-172}
KEY_X=${KEY_X:-400}
VALUE_X=${VALUE_X:-800}
SET_Y=${SET_Y:-207}
REVEAL_X=${REVEAL_X:-1193}
REPLACE_X=${REPLACE_X:-1256}
DELETE_X=${DELETE_X:-1318}
FIRST_NAME_Y=${FIRST_NAME_Y:-286}
NAME_H=${NAME_H:-43}
# The notice line a write leaves above the names.
NOTICE_H=${NOTICE_H:-31}

PIN=tour-pin-42

log() { cat "$E2E_PROFILE/logs/Marley.log" 2>/dev/null; }

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# Whether Marley's log holds none of the PIN, the values the run used, or a token Rusty issued.
log_holds_no_secret() {
  local needle
  for needle in "$PIN" wrong-pin made-up-one made-up-two made-up-tour-value replaced-tour-value \
    $(calls | grep -o '"token": "[0-9a-f]*"' | grep -o '[0-9a-f]\{16,\}' | sort -u); do
    if log | grep -qF "$needle"; then
      echo "found in the log: a secret the run used" >&2
      return 1
    fi
  done
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

name_y() { echo $((FIRST_NAME_Y + $1 * NAME_H)); }

# Name `$1`'s height once a write's notice shows above the names.
noticed_y() { echo $((FIRST_NAME_Y + NOTICE_H + $1 * NAME_H)); }

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  export ZED_LOG=info,context_server=trace
  printf '{"github_token": "made-up-one", "openai_api_key": "made-up-two"}\n' \
    >"$E2E_WORK/rusty/secrets.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  # The names in order: github_token 0, openai_api_key 1, and tour_token 2 once added.
  echo "== no PIN"
  palette "rusty: open secrets"
  settle 2
  shot 667-01-no-pin

  echo "== a PIN set"
  click "$PIN_X" "$PIN_Y"
  type_text "$PIN"
  click "$AGAIN_X" "$PIN_Y"
  type_text "$PIN"
  press "" Return
  settle 3
  shot 667-02-pin-set
  expect "the PIN is set" test -s "$E2E_WORK/rusty/pin"

  echo "== a wrong PIN"
  click "$UNLOCK_X" "$UNLOCK_Y"
  type_text wrong-pin
  press "" Return
  settle 2
  shot 667-03-wrong-pin

  echo "== unlocked"
  type_text "$PIN"
  press "" Return
  settle 3
  shot 667-04-unlocked

  echo "== a value shown"
  click "$REVEAL_X" "$(name_y 0)"
  settle 2
  shot 667-05-revealed

  echo "== a secret added"
  click "$KEY_X" "$SET_Y"
  type_text tour_token
  click "$VALUE_X" "$SET_Y"
  type_text made-up-tour-value
  press "" Return
  settle 3
  shot 667-06-added
  expect "the secret reached the vault" grep -q '"tour_token": "made-up-tour-value"' "$E2E_WORK/rusty/secrets.json"

  echo "== a value replaced"
  click "$REPLACE_X" "$(noticed_y 0)"
  settle 1
  type_text replaced-tour-value
  press "" Return
  settle 3
  click "$REVEAL_X" "$(noticed_y 0)"
  settle 2
  shot 667-07-replaced
  expect "the new value reached the vault" grep -q '"github_token": "replaced-tour-value"' "$E2E_WORK/rusty/secrets.json"

  echo "== a delete"
  click "$DELETE_X" "$(noticed_y 2)"
  settle 2
  shot 667-08-delete-prompt
  press "" Return
  settle 3
  shot 667-09-deleted
  expect "tour_token is gone" bash -c "! grep -q tour_token '$E2E_WORK/rusty/secrets.json'"

  echo "== locked when the window loses the focus"
  palette "marley: open settings"
  settle 3
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
  shot 667-10-locked-on-blur

  echo "== locked when the unlock runs out"
  python3 - "$E2E_WORK/rusty/settings.json" <<'PY'
import json, pathlib, sys

path = pathlib.Path(sys.argv[1])
settings = json.loads(path.read_text())
settings["pin_timeout_seconds"] = "4"
path.write_text(json.dumps(settings, indent=2, sort_keys=True))
PY
  click "$UNLOCK_X" "$UNLOCK_Y"
  type_text "$PIN"
  press "" Return
  settle 8
  shot 667-11-expired

  echo "== the log"
  expect "Rusty's messages were traced by size" bash -c "grep -qE 'recv: [0-9]+ bytes' '$E2E_PROFILE/logs/Marley.log'"
  expect "the log holds no PIN, value or token" log_holds_no_secret
}
