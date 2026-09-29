# shellcheck shell=bash
# #586's e2e test: a project's local HTML pages open in its Browser tab. On #561's fixture: Marley
# gives the terminal its opener as `BROWSER`, and fakes first on every PATH log instead of opening
# a browser (`xdg-open` to `xdg-open.log`; `gio`, `google-chrome` and `firefox`, which Python's
# `webbrowser` tries after a failed one, to `leak.log`, which must stay empty). A `file://` URL of
# a page (REQ-001) and a plain path to one, as `cargo doc --open` hands its `BROWSER` (REQ-002),
# open in a Browser tab of the project; the page's link loads the next page (REQ-003); a text
# file, a folder and a missing page go to the system's handler (REQ-004); an agent's navigation to
# the page is refused, and the tool declines the text file (REQ-005); with `terminal_links` on
# `system_browser` the page goes to the system browser (REQ-006).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The rail's row of the first terminal, and the page's "Next" link, from the first run's shots.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-100}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-136}
LINK_X=${LINK_X:-304}
LINK_Y=${LINK_Y:-219}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin" "$E2E_WORK/pages" "$E2E_WORK/docs"
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
    local log=$E2E_WORK/leak.log
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
  cat >"$E2E_WORK/pages/index.html" <<'HTML'
<!doctype html><title>A local page</title>
<body style="font: 20px sans-serif; margin: 24px"><h1>A local page</h1><p><a href="other.html">Next</a></p></body>
HTML
  cat >"$E2E_WORK/pages/other.html" <<'HTML'
<!doctype html><title>Another page</title>
<body style="font: 20px sans-serif; margin: 24px"><h1>Another page</h1></body>
HTML
  printf 'notes\n' >"$E2E_WORK/notes.txt"
  offline_chromium
  git init -q -b pages "$E2E_WORK/repo"
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

# Brings the first terminal to the front from the rail.
first_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
}

# Runs `$1` in the terminal in front and waits `$2` seconds.
run() {
  type_text "$1"
  press "" Return
  settle "${2:-3}"
}

# Has Python open `$1` the way a program does, through `BROWSER`.
python_open() {
  run "python3 -c 'import webbrowser; webbrowser.open(\"$1\")'" "${2:-4}"
}

tabs() {
  mcp_agent tabs >"$E2E_WORK/tabs.txt"
  cat "$E2E_WORK/tabs.txt"
}

tab_count() {
  grep -c "^  tab " "$E2E_WORK/tabs.txt" || true
}

logged() {
  grep -cxF -- "$2" "$E2E_WORK/$1" || true
}

# `marley.<key>` set to the JSON value in the profile copy's settings, as 561 sets it.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import pathlib, re, sys
path, key, value = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
text = path.read_text()
existing = re.search(r'"%s"\s*:\s*(true|false|"[^"\n]*"|-?[0-9.]+)' % re.escape(key), text)
if existing:
    text = text[: existing.start(1)] + value + text[existing.end(1) :]
else:
    block = re.search(r'"marley"\s*:\s*\{', text)
    if block:
        text = text[: block.end()] + f'\n    "{key}": {value},' + text[block.end() :]
    else:
        brace = text.index("{")
        text = text[: brace + 1] + f'\n  "marley": {{ "{key}": {value} }},' + text[brace + 1 :]
path.write_text(text)
PY
}

steps() {
  local index="file://$E2E_WORK/pages/index.html" other="file://$E2E_WORK/pages/other.html"
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== a program opens a file:// page: a Browser tab of the project"
  python_open "$index"
  shot 586-01-file-url
  tabs
  expect "a Browser tab of the project on the page, focused" \
    holds "$E2E_WORK/tabs.txt" "'A local page' at $index, project repo, focused"

  echo "== a program hands its opener a path, as cargo doc --open does"
  first_terminal
  run "sh -c '\"\$BROWSER\" \"\$1\"' opener $E2E_WORK/pages/other.html" 4
  shot 586-02-path
  tabs
  expect "the path's page in a Browser tab of the project, focused" \
    holds "$E2E_WORK/tabs.txt" "'Another page' at $other, project repo, focused"

  echo "== the first page again, and its link"
  first_terminal
  python_open "$index"
  click "$LINK_X" "$LINK_Y"
  settle 3
  shot 586-03-link
  tabs
  expect "the link loaded the next page in its tab" \
    test "$(grep -cF "'Another page' at $other" "$E2E_WORK/tabs.txt")" -eq 2

  echo "== a text file, a folder and a missing page: the system's handler"
  first_terminal
  python_open "file://$E2E_WORK/notes.txt" 3
  python_open "file://$E2E_WORK/docs/" 3
  python_open "file://$E2E_WORK/missing.html" 3
  shot 586-04-declined
  tabs
  cat "$E2E_WORK/xdg-open.log"
  expect "the text file reached xdg-open" test "$(logged xdg-open.log "file://$E2E_WORK/notes.txt")" -eq 1
  expect "the folder reached xdg-open" test "$(logged xdg-open.log "file://$E2E_WORK/docs/")" -eq 1
  expect "the missing page reached xdg-open" test "$(logged xdg-open.log "file://$E2E_WORK/missing.html")" -eq 1
  expect "and no new tab" test "$(tab_count)" -eq 2

  echo "== an agent: its navigation to the page is refused, and the tool declines the text file"
  mcp_agent navigate "$index" | tee "$E2E_WORK/navigate.txt"
  mcp_agent open-url "file://$E2E_WORK/notes.txt" "$E2E_WORK/repo" | tee "$E2E_WORK/declined.txt"
  shot 586-05-agent
  expect "browser_navigate refused the file: URL" \
    holds "$E2E_WORK/navigate.txt" "refused" "http and https URLs only"
  expect "browser_open_url declined the text file" holds "$E2E_WORK/declined.txt" '"opened": false'

  echo "== marley.terminal_links set to system_browser"
  marley_setting terminal_links '"system_browser"'
  settle 3
  palette "workspace: new terminal"
  settle 3
  python_open "$index" 3
  mcp_agent open-url "$index" "$E2E_WORK/repo" | tee "$E2E_WORK/off.txt"
  shot 586-06-system
  tabs
  expect "the page reached the system browser" test "$(logged xdg-open.log "$index")" -eq 1
  expect "the tool declined it too" holds "$E2E_WORK/off.txt" '"opened": false' "system browser"
  expect "and no new tab" test "$(tab_count)" -eq 2
  cat "$E2E_WORK/leak.log"
  expect "no real browser was asked" test ! -s "$E2E_WORK/leak.log"
}
