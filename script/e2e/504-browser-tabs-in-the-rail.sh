# shellcheck shell=bash
# #504's e2e test: Browser tabs as rows of their project in the rail. The site serves `index.html`
# ("Checkout", which declares a red PNG icon and has a Save button), `plain.html` ("Plain page",
# no icon, and the server has no `/favicon.ico`) and the fixture's `/slow`, which answers after
# three seconds. Tabs open through `marley: new browser tab` and a typed URL. Checks: no browser
# unit before the first tab; the icon's red on the Checkout row and none on the Plain row; the tab
# a stand-in agent sees focused after a row's click and after Down, Down and Enter in the rail;
# the closed tab's page gone; nothing reaching the system browser. The shots show the rest: the
# spinner, the counts, the agent's mark and its clearing, the selected row and the filter.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=

# Where things are, in the headless output's pixels, measured from the first run's shots: the
# rail's rows under the project (the terminal, then the Browser tabs in the order they opened),
# a row's icon, and the Save button in the Checkout page.
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-136}
ROW_X=${ROW_X:-120}
ICON_X=${ICON_X:-38}
BROWSER_ROW_1_Y=${BROWSER_ROW_1_Y:-182}
BROWSER_ROW_2_Y=${BROWSER_ROW_2_Y:-228}
BROWSER_ROW_3_Y=${BROWSER_ROW_3_Y:-274}
CLOSE_X=${CLOSE_X:-233}
SAVE_X=${SAVE_X:-360}
SAVE_Y=${SAVE_Y:-209}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin" "$E2E_WORK/site"
  echo "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  # Nothing here opens the system browser. Should a click land on a link, these log it and
  # succeed, so neither `open`'s later commands nor gpui's desktop-portal fallback, which would
  # reach the user's own browser, runs.
  for name in xdg-open gio google-chrome firefox; do
    cat >"$bin/$name" <<SH
#!/bin/sh
printf '%s %s\n' "$name" "\$*" >>"$E2E_WORK/leak.log" || true
exit 0
SH
    chmod +x "$bin/$name"
  done
  : >"$E2E_WORK/leak.log"
  export PATH="$bin:$PATH"
  [[ $(command -v xdg-open) == "$bin/xdg-open" ]] || {
    echo "the fake xdg-open is not first on the PATH; not launching" >&2
    return 1
  }
  python3 - "$E2E_WORK/site" <<'PY'
import pathlib, struct, sys, zlib

site = pathlib.Path(sys.argv[1])


def png(width, height, rgb):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    raw = b"".join(b"\0" + bytes(rgb) * width for _ in range(height))
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")


(site / "icon.png").write_bytes(png(32, 32, (230, 20, 20)))
(site / "index.html").write_text(
    '<!doctype html><title>Checkout</title><link rel="icon" href="icon.png">'
    '<body style="margin:0"><button style="position:absolute;left:40px;top:80px;'
    'width:120px;height:40px">Save</button></body>\n'
)
(site / "plain.html").write_text("<!doctype html><title>Plain page</title><p>No icon.</p>\n")
PY
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b rail "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Opens `$1` in a new Browser tab, whose address bar has the focus.
new_tab() {
  palette "marley: new browser tab"
  settle 3
  type_text "$1"
  press "" Return
  settle "${2:-4}"
}

tabs() {
  mcp_agent tabs >"$E2E_WORK/tabs.txt"
  cat "$E2E_WORK/tabs.txt"
}

# The id of the tab whose title is `$1`.
tab_id() {
  sed -n "s/^  tab \([0-9A-F]*\): '$1' .*/\1/p" "$E2E_WORK/tabs.txt" | head -1
}

# Whether the shot `$1` has a pixel of the icon's red in the 16-pixel square around (`$2`, `$3`):
# nothing else in the rail is red.
red_near() {
  local x=$(($2 - 8)) y=$(($3 - 8))
  magick "$(shot_file "$1.png")" -crop "16x16+$x+$y" +repage -depth 8 txt:- |
    awk -F'[(),]' 'NR > 1 && $3 > 180 && $4 < 90 && $5 < 90 { found = 1 } END { exit !found }'
}

no_red_near() {
  ! red_near "$@"
}

steps() {
  local checkout
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== no browser before the first Browser tab"
  expect "no browser unit runs yet" bash -c "! systemctl --user is-active --quiet '$(browser_unit)'"
  echo "== two Browser tabs: rows under the project, after its terminal"
  new_tab "$SITE/index.html" 5
  new_tab "$SITE/plain.html" 4
  shot 504-01-rows
  tabs
  expect "the page's icon shows on the Checkout row" red_near 504-01-rows "$ICON_X" "$BROWSER_ROW_1_Y"
  expect "the Plain row shows the globe, not an icon" no_red_near 504-01-rows "$ICON_X" "$BROWSER_ROW_2_Y"
  echo "== a page that loads slowly: the spinner"
  new_tab "$SITE/slow" 1
  shot 504-02-loading
  settle 4
  echo "== a pick and an annotation in the Checkout tab"
  click "$ROW_X" "$BROWSER_ROW_1_Y"
  settle 2
  palette "marley: pick element"
  settle 1
  click "$SAVE_X" "$SAVE_Y"
  settle 3
  tabs
  checkout=$(tab_id Checkout)
  mcp_agent --tab "$checkout" annotate button Save "The agent checked this"
  settle 2
  shot 504-03-counts
  echo "== the agent acts in the Checkout page while the Plain tab is in front"
  click "$ROW_X" "$BROWSER_ROW_2_Y"
  settle 2
  mcp_agent --tab "$checkout" click-on button Save
  settle 2
  shot 504-04-agent-mark
  echo "== the Checkout row, clicked"
  click "$ROW_X" "$BROWSER_ROW_1_Y"
  settle 2
  shot 504-05-shown
  tabs
  expect "the Checkout tab is focused" holds "$E2E_WORK/tabs.txt" "'Checkout' at $SITE/index.html, project repo, focused"
  echo "== the keyboard in the rail: from the terminal's row, Down twice and Enter"
  click "$ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  press "CTRL ALT" semicolon
  settle 1
  press "" Down
  press "" Down
  settle 1
  press "" Return
  settle 2
  shot 504-06-keyboard
  tabs
  expect "Enter on the second Browser row showed its tab" \
    holds "$E2E_WORK/tabs.txt" "'Plain page' at $SITE/plain.html, project repo, focused"
  echo "== the filter"
  press "CTRL ALT" semicolon
  settle 1
  press CTRL f
  settle 1
  type_text "plain"
  settle 2
  shot 504-07-filter
  press "" Escape
  settle 1
  echo "== the Slow row's close button"
  click "$ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  pointer_to "$ROW_X" "$BROWSER_ROW_3_Y"
  settle 1
  click "$CLOSE_X" "$BROWSER_ROW_3_Y"
  settle 3
  shot 504-08-closed
  tabs
  expect "the Slow tab and its page are gone" bash -c "! grep -q \"'Slow page'\" '$E2E_WORK/tabs.txt'"
  cat "$E2E_WORK/leak.log"
  expect "nothing reached the system browser" test ! -s "$E2E_WORK/leak.log"
}
