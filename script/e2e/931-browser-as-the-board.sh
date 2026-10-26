# shellcheck shell=bash
# #931's visual check: the Browser tab as the Browser board draws it. Marley's Chromium reaches no
# host but this machine (`offline_chromium`); a served page has a `div.card` with a Save button and
# a `console.error`, and another asks `confirm()`. The terminal is used first, so picks go to it;
# its `.bashrc` sets the prompt only. `mcp_agent click-on` acts as an agent in the page. No agent
# CLI runs.
# - `931-01-picked`: the card picked: the accent mark and its `div.card · 1` badge on the page, the
#   drawer open with "Console 1" in red and the error row tinted, the project chip in the address
#   box (REQ-001 to REQ-004).
# - `931-02-card`: the picks panel: the card's head (chip, place, text), the three-row Note and To
#   as a dropdown (REQ-005 to REQ-007).
# - `931-03-to-open`: the dropdown open: the terminal used last and A new agent (REQ-007).
# - `931-02b-sent`: a note and Enter: the card says it was sent (REQ-006).
# - `931-04-driving`: after `mcp_agent click-on`: "… is driving this page", the dot, Take Over and
#   its key (REQ-008).
# - `931-05-asks`: the page's `confirm()`: "<origin> asks" (REQ-009).
# - `931-06-clear`: `marley: clear project browser data`: the project, the tab count, "Other
#   projects keep theirs" (REQ-010).
# - `931-07-script`: the scripts tray's one field, its placeholder and its Ctrl Enter hint, then a
#   name and Ctrl Enter: kept for every project (REQ-011).
# - `931-08-note`: annotate, a box dragged: the note's placeholder and "Enter keeps it · Esc drops
#   it" (REQ-012).
# - `931-09-day`: Marley Day, the card picked again and the panel (REQ-001 to REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-700}
# From the probe runs' shots.
TO_X=${TO_X:-1440}
TO_Y=${TO_Y:-501}
NOTE_X=${NOTE_X:-1440}
NOTE_Y=${NOTE_Y:-265}
CARD_TAB_X=${CARD_TAB_X:-545}
CARD_TAB_Y=${CARD_TAB_Y:-50}
SCRIPT_FIELD_X=${SCRIPT_FIELD_X:-700}
SCRIPT_FIELD_Y=${SCRIPT_FIELD_Y:-190}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  cat >"$E2E_WORK/site/card.html" <<'HTML'
<!doctype html><html><head><title>Card</title><style>
body { margin: 0; font: 18px sans-serif; background: #f4f1ea }
.card { position: absolute; left: 40px; top: 40px; width: 360px; height: 140px; background: #fff; border: 1px solid #ccc; border-radius: 8px }
.card h2 { position: absolute; left: 20px; top: 14px; margin: 0; font-size: 24px }
#save { position: absolute; left: 20px; top: 70px; width: 200px; height: 44px; font-size: 18px }
</style></head><body>
<div class="card wide">
  <h2>Checkout</h2>
  <button id="save">Save changes</button>
</div>
<script>console.error("price is missing on the Checkout card")</script>
</body></html>
HTML
  cat >"$E2E_WORK/site/confirm.html" <<'HTML'
<!doctype html><html><head><title>Confirm</title></head><body>
<script>setTimeout(() => confirm('Reset every verdict to Not tried?'), 300);</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  profile_setting marley.tmux_hosts '[]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

go_to() {
  press CTRL l
  settle 1
  type_text "$1"
  press "" Return
  settle 4
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4
  echo "== the terminal is used first"
  click 800 500
  settle 1

  echo "== the page, the card picked"
  palette "marley: open browser" 5
  go_to "$SITE/card.html"
  press "CTRL SHIFT" c
  settle 1
  click $((PAGE_X + 320)) $((PAGE_Y + 60))
  settle 3
  away
  take 931-01-picked || return 0
  take 931-02-card || return 0

  echo "== To opened"
  click "$TO_X" "$TO_Y"
  settle 2
  take 931-03-to-open || return 0
  press "" Escape
  settle 1

  echo "== a note and Enter"
  click "$NOTE_X" "$NOTE_Y"
  settle 1
  type_text "Make the card wider"
  press "" Return
  settle 3
  away
  take 931-02b-sent || return 0
  mcp_agent terminal-screen bash | tail -4
  # The send brought the terminal forward; the page's tab comes back.
  click "$CARD_TAB_X" "$CARD_TAB_Y"
  settle 2

  echo "== an agent drives the page"
  mcp_agent click-on button "Save changes"
  settle 3
  away
  take 931-04-driving || return 0

  echo "== the page asks"
  go_to "$SITE/confirm.html"
  settle 2
  away
  take 931-05-asks || return 0
  press "" Escape
  settle 1

  echo "== clear the project's browser data"
  palette "marley: clear project browser data" 3
  away
  take 931-06-clear || return 0
  press "" Escape
  settle 1

  echo "== the scripts tray"
  palette "marley: playwright scripts" 3
  click "$SCRIPT_FIELD_X" "$SCRIPT_FIELD_Y"
  settle 1
  away
  take 931-07-script || return 0
  type_text "smoke"
  press CTRL Return
  settle 3
  ls "$E2E_PROFILE"/config/playwright-scripts 2>/dev/null || find "$E2E_PROFILE/config" -name 'smoke*' | head -3
  # The new script opens in an editor tab; the Browser's tab comes back and its tray closes.
  click "$CARD_TAB_X" "$CARD_TAB_Y"
  settle 2
  palette "marley: playwright scripts" 2

  echo "== annotate, a box"
  go_to "$SITE/card.html"
  palette "marley: annotate" 1
  pointer_to $((PAGE_X + 30)) $((PAGE_Y + 30))
  pointer_down
  pointer_to $((PAGE_X + 200)) $((PAGE_Y + 100))
  pointer_to $((PAGE_X + 420)) $((PAGE_Y + 200))
  settle 1
  pointer_up
  settle 2
  take 931-08-note || return 0
  press "" Escape
  settle 1
  palette "marley: annotate" 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  press "CTRL SHIFT" c
  settle 1
  click $((PAGE_X + 320)) $((PAGE_Y + 60))
  settle 3
  away
  shot 931-09-day
}
