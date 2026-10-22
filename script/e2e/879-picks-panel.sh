# shellcheck shell=bash
# #879's visual check: the Browser's picks as a panel of cards, as the Browser board draws them.
# Marley's Chromium reaches no host but this machine (`offline_chromium`); a page served in the run
# has a card with a Save button. The terminal is used first, so a sent pick goes to it.
# - Ctrl+Shift+C and a click on Save: the panel at the page's right with the pick's card, its Note
#   field and Send (`879-01-picked`, REQ-001).
# - A note typed and Enter, which types the pick into the terminal; back on the Browser's tab, the
#   card says what was sent (`879-02-sent`, REQ-001).
# - Marley Day (`879-03-day`, REQ-001).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/site/card.html" <<'HTML'
<!doctype html><html><head><title>Card</title><style>
body { margin: 0; font: 18px sans-serif; background: #f4f1ea }
.card { position: absolute; left: 40px; top: 40px; width: 360px; height: 140px; background: #fff; border: 1px solid #ccc; border-radius: 8px }
.card h2 { position: absolute; left: 20px; top: 14px; margin: 0; font-size: 24px }
#save { position: absolute; left: 20px; top: 70px; width: 200px; height: 44px; font-size: 18px }
</style></head><body>
<div class="card">
  <h2>Checkout</h2>
  <button id="save" data-testid="save-button"><span class="label">Save changes</span></button>
</div>
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

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4
  echo "== the terminal is used first"
  click 800 500
  settle 1

  echo "== the page, and Save picked"
  palette "marley: open browser" 5
  press CTRL l
  settle 1
  type_text "$SITE/card.html"
  press "" Return
  settle 5
  press "CTRL SHIFT" c
  settle 1
  click $((PAGE_X + 160)) $((PAGE_Y + 132))
  settle 3
  pointer_to 900 700
  settle 1
  shot 879-01-picked

  echo "== a note, sent"
  type_text "Make this button green"
  press "" Return
  settle 3
  # Send types the pick into the terminal and shows it; the Browser's tab again for the card.
  click 518 50
  settle 3
  pointer_to 900 700
  settle 1
  shot 879-02-sent

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 879-03-day
}
