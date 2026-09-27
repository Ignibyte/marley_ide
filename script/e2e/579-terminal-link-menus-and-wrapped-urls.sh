# shellcheck shell=bash
# #579's e2e test: the menu on a clicked terminal link, the right-click menu's link entries, the
# default offered until chosen, and URLs a program wrapped at the edge or drew in a box, joined.
# A fake `xdg-open`, first on every PATH and exiting 0, logs what the system browser is asked to
# open; `gio`, `google-chrome` and `firefox` log to `leak.log`, which must stay empty. Fakes on
# the terminal's PATH print and wait: `urls` a local URL (the site's, which listens, so #503's
# offer strip shows) and one that is not local; `osc8` an OSC 8 link to the site; `edge` a URL
# longer than the terminal is wide, cut by a newline exactly at the edge; `framed` a URL over
# three rows of a box. Checks: the Browser tabs a stand-in agent lists, the `xdg-open` log, the
# clipboard, and the settings file.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=

# The terminal's geometry, as #503 measured it: rows are counted up from the cursor's while the
# offer's strip shows under the terminal.
TEXT_X=${TEXT_X:-269}
CELL_W=${CELL_W:-9}
ROW_H=${ROW_H:-19.5}
CURSOR_Y_STRIP=${CURSOR_Y_STRIP:-930}
TERMINAL_ROW_X=${TERMINAL_ROW_X:-100}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-136}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name log
  mkdir -p "$home" "$bin" "$E2E_WORK/site"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
case "\$BROWSER" in
  */marley-open-url) ;;
  *) export BROWSER="$bin/xdg-open" ;;
esac
RC
  terminal_env HOME "$home"
  for name in xdg-open gio google-chrome firefox; do
    log=$E2E_WORK/leak.log
    [[ $name == xdg-open ]] && log=$E2E_WORK/xdg-open.log
    cat >"$bin/$name" <<SH
#!/bin/sh
printf '%s\n' "\$*" >>"$log" || true
exit 0
SH
    chmod +x "$bin/$name"
  done
  : >"$E2E_WORK/xdg-open.log"
  : >"$E2E_WORK/leak.log"
  export PATH="$bin:$PATH"
  [[ $(command -v xdg-open) == "$bin/xdg-open" ]] || {
    echo "the fake xdg-open is not first on the PATH; not launching" >&2
    return 1
  }
  printf '<!doctype html><title>Site page</title><p>The site.</p>\n' >"$E2E_WORK/site/index.html"
  printf '<!doctype html><title>OSC 8 page</title>\n' >"$E2E_WORK/site/osc8.html"
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  write_fakes
  git init -q -b menus "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_fakes() {
  local bin=$E2E_WORK/bin
  cat >"$bin/urls" <<SH
#!/bin/sh
echo "local: $SITE/"
echo "remote: https://example.com/docs"
read -r _
SH
  cat >"$bin/osc8" <<SH
#!/bin/sh
printf '\033]8;;%s\033\\\\%s\033]8;;\033\\\\\n' "$SITE/osc8.html" "Open the OSC 8 page"
read -r _
SH
  # A URL one and a half rows long, cut by a newline at the last column.
  cat >"$bin/edge" <<'SH'
#!/bin/bash
cols=$(tput cols)
url="https://example.com/edge/$(printf 'a%.0s' $(seq 1 "$cols"))/end"
printf '%s\n%s\n' "${url:0:cols}" "${url:cols}"
read -r _
SH
  # A URL over three rows of a 44-column box, as a TUI draws one.
  cat >"$bin/framed" <<'SH'
#!/bin/bash
inner=40
url="https://example.com/framed/$(printf 'b%.0s' $(seq 1 60))/end"
line() { printf '│ %-*s │\n' "$inner" "$1"; }
printf '┌%s┐\n' "$(printf '─%.0s' $(seq 1 $((inner + 2))))"
line "${url:0:inner}"
line "${url:inner:inner}"
line "${url:$((inner * 2))}"
printf '└%s┘\n' "$(printf '─%.0s' $(seq 1 $((inner + 2))))"
read -r _
SH
  chmod +x "$bin/urls" "$bin/osc8" "$bin/edge" "$bin/framed"
}

# The y of the row `$1` rows above the cursor's, with the offer's strip showing.
row_y() {
  awk -v base="$CURSOR_Y_STRIP" -v rows="$1" -v height="$ROW_H" 'BEGIN { printf "%d", base - rows * height }'
}

# The x of column `$1` (from 0) of a row.
column_x() {
  awk -v x="$TEXT_X" -v column="$1" -v width="$CELL_W" 'BEGIN { printf "%d", x + (column + 0.5) * width }'
}

first_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
}

run() {
  type_text "$1"
  press "" Return
  settle "${2:-3}"
}

# Chooses the menu's entry `$1`, counting from 1: the menu starts on its first entry.
choose() {
  local downs
  for ((downs = 1; downs < $1; downs++)); do
    press "" Down
  done
  press "" Return
  settle 2
}

tabs() {
  mcp_agent tabs >"$E2E_WORK/tabs.txt"
  cat "$E2E_WORK/tabs.txt"
}

opened() {
  grep -cxF -- "$1" "$E2E_WORK/xdg-open.log" || true
}

clipboard() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline 2>/dev/null || true
}

steps() {
  local framed_url
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== a plain click on a URL shows its menu"
  run urls 4
  click "$(column_x 10)" "$(row_y 2)"
  settle 1
  shot 579-01-link-menu
  echo "== Open in Browser Tab"
  choose 1
  shot 579-02-menu-tab
  tabs
  expect "the menu opened a Browser tab of the project" \
    holds "$E2E_WORK/tabs.txt" "'Site page' at $SITE/, project repo"
  echo "== Open in System Browser"
  first_terminal
  click "$(column_x 12)" "$(row_y 1)"
  settle 1
  shot 579-03-menu-system
  choose 2
  expect "the menu sent the URL to the system browser" \
    test "$(opened "https://example.com/docs")" -eq 1
  echo "== Copy Link"
  click "$(column_x 10)" "$(row_y 2)"
  settle 1
  choose 3
  echo "clipboard: $(clipboard)"
  expect "Copy Link put the URL on the clipboard" test "$(clipboard)" = "$SITE/"
  echo "== a plain click on text that is no link shows nothing"
  click "$(column_x 2)" "$(row_y 2)"
  settle 1
  # With no menu, these reach the waiting program, and Return ends it.
  choose 2
  expect "no menu on plain text" test "$(opened "$SITE/")" -eq 0
  echo "== the right-click menu of an OSC 8 link"
  run osc8 3
  click "$(column_x 4)" "$(row_y 1)" right
  settle 1
  shot 579-04-right-click-osc8
  choose 3
  echo "clipboard: $(clipboard)"
  expect "Copy Link copied the OSC 8 link's target" test "$(clipboard)" = "$SITE/osc8.html"
  press "" Return
  settle 1
  echo "== the default, offered until chosen"
  run urls 4
  click "$(column_x 10)" "$(row_y 2)"
  settle 1
  choose 5
  settle 2
  grep -n '"terminal_links"' "$E2E_PROFILE/config/settings.json" || true
  expect "the choice is in the settings" \
    grep -q '"terminal_links": "system_browser"' "$E2E_PROFILE/config/settings.json"
  click "$(column_x 10)" "$(row_y 2)"
  settle 1
  shot 579-05-default-chosen
  press "" Escape
  settle 1
  press "" Return
  settle 1
  echo "== a URL a program cut at the edge opens whole"
  run edge 3
  click_with CTRL "$(column_x 3)" "$(row_y 1)"
  settle 2
  shot 579-06-edge-wrapped
  tail -1 "$E2E_WORK/xdg-open.log"
  expect "Ctrl+click on the second row opened the whole URL" \
    bash -c "tail -1 '$E2E_WORK/xdg-open.log' | grep -qE '^https://example.com/edge/a+/end$'"
  press "" Return
  settle 1
  echo "== a URL drawn in a box opens whole"
  run framed 3
  framed_url="https://example.com/framed/$(printf 'b%.0s' $(seq 1 60))/end"
  click_with CTRL "$(column_x 6)" "$(row_y 3)"
  settle 2
  shot 579-07-framed
  tail -1 "$E2E_WORK/xdg-open.log"
  expect "Ctrl+click in the box's middle row opened the whole URL" \
    test "$(opened "$framed_url")" -eq 1
  press "" Return
  settle 1
  cat "$E2E_WORK/leak.log"
  expect "no real browser was asked" test ! -s "$E2E_WORK/leak.log"
}
