# shellcheck shell=bash
# #937's visual check: the System One page as the SystemOne board draws it. A scratch repository;
# System One on with the `rules` provider (no key, nothing leaves the machine), never the user's
# account; Stop kind on Suggest, Stall kind on Shadow, Click consequence on Act, as #847's run sets
# them. Today's call file holds three calls, the Click consequence one failed.
# - `marley: open system one calls`: the title and the opening line, the cards, the ten uses with
#   their modes, the calls heading and the three rows, the failed one tinted (`937-01-page`,
#   REQ-001, REQ-002, REQ-004, REQ-005).
# - Act clicked on Stop kind: its Act filled (`937-02-changed`, REQ-003).
# - The Settings window searched for "stop kind": its row on Act (`937-03-settings`, REQ-003);
#   searched for "system one" and scrolled to the section's end: the link's words (`937-04-link`,
#   REQ-006). The search reads a link's title only, so the section's last item is reached by scrolling.
# - The page in Marley Day (`937-05-day`, REQ-001, REQ-002, REQ-004, REQ-005).
compositor sway

STOP_AT=${STOP_AT:-0}
# Stop kind's Act button on the page, from the first run's shot.
ACT_X=${ACT_X:-1298}
ACT_Y=${ACT_Y:-336}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo calls=$E2E_PROFILE/system_one now
  mkdir -p "$home" "$repo" "$calls"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  now=$(date +%Y-%m-%dT%H:%M:%S%:z)
  cat >"$calls/calls-$(date +%F).jsonl" <<JSONL
{"row":"call","id":"e2e-1","time":"$now","use":"stop_kind","set":"stop_kind/1","model":"rules","provider":"rules","project":"repo","mode":"suggest","questions":[],"reading":"done and checked","failed":false,"latency_ms":3}
{"row":"call","id":"e2e-2","time":"$now","use":"stall_kind","set":"stall_kind/1","model":"rules","provider":"rules","project":"repo","mode":"shadow","questions":[],"reading":"working","failed":false,"latency_ms":2}
{"row":"call","id":"e2e-3","time":"$now","use":"click_consequence","set":"click_consequence/1","model":"rules","provider":"rules","project":"repo","mode":"act","questions":[],"reading":"the call failed","failed":true,"error":"the rules answer nothing for this page"}
JSONL
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.system_one '{"enabled": true, "provider": "rules", "uses": {"stop_kind": "suggest", "stall_kind": "shadow", "click_consequence": "act"}}'
  open_path "$repo"
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

  echo "== the System One page"
  palette "marley: open system one calls" 4
  pointer_to 1500 950
  settle 1
  shot 937-01-page
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Act on Stop kind"
  click "$ACT_X" "$ACT_Y"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 937-02-changed
  expect "the user's settings hold stop_kind on act" holds "$E2E_PROFILE/config/settings.json" '"stop_kind": "act"'

  echo "== the Settings window"
  palette "marley: open settings" 5
  press "CTRL" f
  settle 1
  type_text "stop kind"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 937-03-settings
  press "CTRL" a
  type_text "system one"
  settle 3
  pointer_to 1300 600
  scroll 80
  settle 2
  shot 937-04-link

  echo "== Marley Day"
  press "CTRL" w
  settle 2
  palette "marley: open system one calls" 4
  profile_setting theme '"Marley Day"'
  settle 4
  pointer_to 1500 950
  settle 1
  shot 937-05-day
}
