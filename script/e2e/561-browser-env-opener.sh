# shellcheck shell=bash
# #561's e2e test: programs that open a browser through `BROWSER` land in a Browser tab of their
# project. Marley gives its local terminals its opener as `BROWSER`, and Python's `webbrowser` is
# the program that reads it here. Fakes first on every PATH log instead of opening a browser:
# `xdg-open`, the system browser, to `xdg-open.log`; `user-browser`, a shell's own choice, to
# `user-browser.log`; and `gio`, `google-chrome` and `firefox`, which Python's `webbrowser` tries
# after a failed one, to `leak.log`, which must stay empty. The scenario's `.bashrc` sets
# `BROWSER` to the fake `xdg-open` whenever it is not Marley's opener, so with the opener turned
# off Python's own fallback reaches the fake and never the user's browser. Checks: the opener's
# path, the Browser tabs a stand-in agent lists, the logs, the opener run with no Marley, timed,
# and the tool's own refusal.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=

# The rail's row of the first terminal.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-100}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-136}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin" "$E2E_WORK/site" "$E2E_WORK/elsewhere"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
case "\$BROWSER" in
  */marley-open-url) ;;
  *) export BROWSER="$bin/xdg-open" ;;
esac
RC
  terminal_env HOME "$home"
  # Each fake logs its argument and succeeds, so neither the programs' next choice nor gpui's
  # desktop-portal fallback, which would reach the user's own browser, runs.
  for name in xdg-open user-browser gio google-chrome firefox; do
    local log=$E2E_WORK/leak.log
    case $name in
      xdg-open) log=$E2E_WORK/xdg-open.log ;;
      user-browser) log=$E2E_WORK/user-browser.log ;;
    esac
    cat >"$bin/$name" <<SH
#!/bin/sh
printf '%s\n' "\$*" >>"$log" || true
exit 0
SH
    chmod +x "$bin/$name"
  done
  : >"$E2E_WORK/xdg-open.log"
  : >"$E2E_WORK/user-browser.log"
  : >"$E2E_WORK/leak.log"
  export PATH="$bin:$PATH"
  [[ $(command -v xdg-open) == "$bin/xdg-open" ]] || {
    echo "the fake xdg-open is not first on the PATH; not launching" >&2
    return 1
  }
  printf '<!doctype html><title>Opened page</title><p>Opened through BROWSER.</p>\n' \
    >"$E2E_WORK/site/index.html"
  # A file that is no HTML page: #586 opens a local page in a Browser tab, and the rest still goes
  # to the system browser.
  printf 'A file\n' >"$E2E_WORK/doc.txt"
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b opener "$E2E_WORK/repo"
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
  grep -cF -- " at $1, " "$E2E_WORK/tabs.txt" || true
}

logged() {
  grep -cxF -- "$2" "$E2E_WORK/$1" || true
}

# What `echo "$BROWSER"` printed last, as a stand-in agent reads the block.
printed_browser() {
  mcp_agent terminal-read "echo \"\$BROWSER\"" | tail -1
}

# `marley.<key>` set to the JSON value in the profile copy's settings, as #516's scenario sets
# its settings.
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
  local opener started elapsed
  opener=$(realpath "$E2E_PROFILE")/mcp/marley-open-url
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== a local terminal carries the opener"
  run "echo \"\$BROWSER\"" 2
  shot 561-01-env
  echo "BROWSER: $(printed_browser)"
  expect "BROWSER is Marley's opener" test "$(realpath "$(printed_browser)")" = "$opener"
  echo "== a program opens a local URL: a Browser tab of the project"
  python_open "$SITE/"
  shot 561-02-tab
  tabs
  expect "a Browser tab of the project on the page, focused" \
    holds "$E2E_WORK/tabs.txt" "'Opened page' at $SITE/, project repo, focused"
  echo "== again: the same tab comes forward"
  first_terminal
  python_open "$SITE/"
  shot 561-03-same-tab
  tabs
  expect "still one tab on the page" test "$(tab_count "$SITE/")" -eq 1
  echo "== a URL that is not local, and a file that is no page, go to the system browser"
  first_terminal
  python_open "https://example.com/docs" 3
  python_open "file://$E2E_WORK/doc.txt" 3
  shot 561-04-system
  tabs
  expect "the docs URL reached xdg-open" test "$(logged xdg-open.log "https://example.com/docs")" -eq 1
  expect "the file reached xdg-open" test "$(logged xdg-open.log "file://$E2E_WORK/doc.txt")" -eq 1
  expect "and no new tab" test "$(grep -c "^  tab " "$E2E_WORK/tabs.txt")" -eq 1
  echo "== a program outside every project: the system browser"
  run "cd $E2E_WORK/elsewhere" 1
  python_open "$SITE/" 3
  shot 561-05-outside
  expect "the local URL from outside reached xdg-open" test "$(logged xdg-open.log "$SITE/")" -eq 1
  echo "== a shell's own BROWSER wins"
  run "cd $E2E_WORK/repo" 1
  run "BROWSER=$E2E_WORK/bin/user-browser python3 -c 'import webbrowser; webbrowser.open(\"$SITE/mine\")'" 3
  shot 561-06-user-browser
  expect "the shell's own browser got the URL" \
    test "$(logged user-browser.log "$SITE/mine")" -eq 1
  echo "== the opener with no Marley answering falls back at once"
  started=$(date +%s%N)
  MARLEY_MCP_ENDPOINT=$E2E_WORK/missing.json "$opener" "$SITE/nobody"
  elapsed=$((($(date +%s%N) - started) / 1000000))
  echo "fallback took ${elapsed} ms"
  expect "the URL reached xdg-open" test "$(logged xdg-open.log "$SITE/nobody")" -eq 1
  expect "within a second" test "$elapsed" -lt 1000
  echo "== the tool declines what is not local, and opens nothing"
  mcp_agent open-url "https://example.com/" "$E2E_WORK/repo" | tee "$E2E_WORK/declined.txt"
  expect "browser_open_url declined" holds "$E2E_WORK/declined.txt" '"opened": false'
  expect "and sent nothing to the system browser" \
    test "$(logged xdg-open.log "https://example.com/")" -eq 0
  echo "== marley.terminal_links set to system_browser"
  marley_setting terminal_links '"system_browser"'
  settle 3
  palette "workspace: new terminal"
  settle 3
  run "echo \"\$BROWSER\"" 2
  echo "BROWSER: $(printed_browser)"
  expect "a new terminal carries no opener" \
    bash -c "[[ \$(realpath '$(printed_browser)') != '$opener' ]]"
  python_open "$SITE/off" 3
  shot 561-07-off
  expect "its local URL reached the system browser" test "$(logged xdg-open.log "$SITE/off")" -eq 1
  cat "$E2E_WORK/leak.log"
  expect "no real browser was asked" test ! -s "$E2E_WORK/leak.log"
}
