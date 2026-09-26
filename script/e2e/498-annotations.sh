# shellcheck shell=bash
# #498's e2e test: boxes and notes drawn over a page, by the user and by an agent. A loopback page
# is tall, with headings down it. Annotate mode, from the palette, turns a drag around the
# second heading into a box, and its note field keeps "Check this heading" on Enter; two wheel
# detents down, the box is still around its heading; the stand-in agent annotates the third
# heading by its ref, and lists both boxes with their page coordinates and makers; back at the
# top, a click on the user's note and Delete remove the user's box; a navigation to another page
# leaves no box. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/long.html" <<'HTML'
<!doctype html><html><head><title>Long page</title><style>
body { margin: 0; font: 18px sans-serif; background: #fbf8f1; height: 3000px; position: relative }
h2 { position: absolute; left: 40px; width: 400px; height: 40px; margin: 0; font-size: 28px; line-height: 40px }
#first { top: 40px } #second { top: 240px } #third { top: 640px } #fourth { top: 1040px }
</style></head><body>
<h2 id="first">First heading</h2>
<h2 id="second">Second heading</h2>
<h2 id="third">Third heading</h2>
<h2 id="fourth">Fourth heading</h2>
</body></html>
HTML
  cat >"$E2E_WORK/site/other.html" <<'HTML'
<!doctype html><html><head><title>Another page</title></head>
<body style="margin:0;font:18px sans-serif;background:#eef"><h2 style="margin:40px">Another page</h2></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b browser "$E2E_WORK/repo"
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

# The window's point for (x, y) in the page's viewport.
page_point() {
  echo "$((PAGE_X + $1)) $((PAGE_Y + $2))"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 5
  press CTRL l
  settle 1
  type_text "$SITE/long.html"
  press "" Return
  settle 4
  echo "== annotate mode, a drag around the second heading, a note, Enter"
  palette "marley: annotate"
  settle 1
  # shellcheck disable=SC2046
  pointer_to $(page_point 30 230)
  pointer_down
  # shellcheck disable=SC2046
  pointer_to $(page_point 240 260)
  # shellcheck disable=SC2046
  pointer_to $(page_point 450 290)
  settle 1
  pointer_up
  settle 1
  type_text "Check this heading"
  press "" Return
  settle 2
  shot 498-01-drawn
  echo "== two wheel detents down: the box stays around its heading"
  # shellcheck disable=SC2046
  pointer_to $(page_point 700 500)
  scroll 2
  settle 2
  shot 498-02-scrolled
  echo "== the agent annotates the third heading by its ref"
  mcp_agent annotate heading "Third heading" "The agent looked here"
  settle 2
  shot 498-03-agent
  mcp_agent annotations
  echo "== back at the top, a click on the user's note and Delete remove the user's box"
  # shellcheck disable=SC2046
  pointer_to $(page_point 700 500)
  scroll -2
  settle 2
  # shellcheck disable=SC2046
  click $(page_point 60 220)
  settle 1
  press "" Delete
  settle 2
  shot 498-03b-deleted
  mcp_agent annotations
  echo "== a navigation to another page leaves no box"
  press CTRL l
  settle 1
  type_text "$SITE/other.html"
  press "" Return
  settle 4
  shot 498-04-gone
  mcp_agent annotations
}
