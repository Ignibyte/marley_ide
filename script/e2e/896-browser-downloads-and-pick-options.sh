# shellcheck shell=bash
# #896's visual check: the Browser's download bar and the picks panel's options. Marley's Chromium
# reaches no host but this machine (`offline_chromium`); a page served in the run has a card with a
# Save button, a link that downloads a file and a console error. Downloads land in the run's own
# folder. The terminal is used first, so the picks go to it.
# - The link clicked: the bar under the page with the file, its folder, Open and Show, and no toast
#   (`896-01-download`, REQ-001).
# - Ctrl+Shift+C and a click on Save: the panel's foot, Goes with it, To and Send 1 pick
#   (`896-02-options`, REQ-002).
# - The screenshot unticked, the console error ticked, Send 1 pick: the terminal shows the line
#   without the screenshot and with the console (`896-03-sent`, REQ-003).
# - Marley Day, on the Browser's tab, Save picked again (`896-04-day`, REQ-001, REQ-002).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-0}
# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
# The foot's screenshot and console ticks and Send, and the Browser's tab, from the first run's
# shots.
SHOT_X=${SHOT_X:-1308}
SHOT_Y=${SHOT_Y:-320}
CONSOLE_X=${CONSOLE_X:-1308}
CONSOLE_Y=${CONSOLE_Y:-372}
SEND_X=${SEND_X:-1440}
SEND_Y=${SEND_Y:-503}
TAB_X=${TAB_X:-518}
TAB_Y=${TAB_Y:-50}

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site" "$E2E_WORK/downloads"
  export XDG_DOWNLOAD_DIR=$E2E_WORK/downloads
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'Quarterly report\n' >"$E2E_WORK/site/report.txt"
  cat >"$E2E_WORK/site/card.html" <<'HTML'
<!doctype html><html><head><title>Card</title><style>
body { margin: 0; font: 18px sans-serif; background: #f4f1ea }
.card { position: absolute; left: 40px; top: 40px; width: 360px; height: 140px; background: #fff; border: 1px solid #ccc; border-radius: 8px }
.card h2 { position: absolute; left: 20px; top: 14px; margin: 0; font-size: 24px }
#save { position: absolute; left: 20px; top: 70px; width: 200px; height: 44px; font-size: 18px }
#report { position: absolute; left: 40px; top: 220px; font-size: 20px }
</style></head><body>
<div class="card">
  <h2>Checkout</h2>
  <button id="save" data-testid="save-button"><span class="label">Save changes</span></button>
</div>
<a id="report" href="report.txt" download="report.txt">Download the report</a>
<script>console.error("price is missing on the Checkout card")</script>
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

  echo "== the page, and its download"
  palette "marley: open browser" 5
  press CTRL l
  settle 1
  type_text "$SITE/card.html"
  press "" Return
  settle 5
  click $((PAGE_X + 120)) $((PAGE_Y + 232))
  settle 4
  ls "$E2E_WORK/downloads"
  pointer_to 900 700
  settle 1
  shot 896-01-download
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Save picked"
  press "CTRL SHIFT" c
  settle 1
  click $((PAGE_X + 160)) $((PAGE_Y + 132))
  settle 3
  pointer_to 900 700
  settle 1
  shot 896-02-options
  if [[ $STOP_AT == 2 ]]; then
    return
  fi

  echo "== no screenshot, the console, Send"
  click "$SHOT_X" "$SHOT_Y"
  settle 1
  click "$CONSOLE_X" "$CONSOLE_Y"
  settle 1
  click "$SEND_X" "$SEND_Y"
  settle 3
  pointer_to 900 700
  settle 1
  shot 896-03-sent

  echo "== Marley Day"
  click "$TAB_X" "$TAB_Y"
  settle 3
  profile_setting theme '"Marley Day"'
  settle 4
  # A second pick, so the foot shows again.
  press "CTRL SHIFT" c
  settle 1
  click $((PAGE_X + 160)) $((PAGE_Y + 132))
  settle 3
  pointer_to 900 700
  settle 1
  shot 896-04-day
}
