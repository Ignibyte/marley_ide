# shellcheck shell=bash
# #810's visual check: the bar an agent's writes leave can be closed. An outside client (the
# browser fixture's stand-in, as Chad's outside client) types into a Python REPL in the repo's
# terminal, with `marley.agent_terminal_writes` at `never_ask`, and clicks in a page.
# - The terminal's bar names the write, with Take Over and a close button (`810-01-bar`); the close
#   button hides it (`810-02-closed`, REQ-001); the next write brings it back (`810-03-back`,
#   REQ-003); Ctrl-I: "You have control", Hand Back, no close button (`810-04-taken`); Ctrl-I again
#   hands back and the bar goes (`810-05-handed-back`, REQ-002).
# - The page's bar, the same way: `810-06-page-bar`, closed (`810-07-page-closed`, REQ-001),
#   back on the next click (`810-08-page-back`, REQ-003), and gone at Hand Back
#   (`810-09-page-handed-back`, REQ-002).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A point in the terminal, the terminal bar's close button, a point in the page and the page bar's
# close button, as the first run's shots found them.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
TERMINAL_CLOSE_X=${TERMINAL_CLOSE_X:-1585}
TERMINAL_CLOSE_Y=${TERMINAL_CLOSE_Y:-954}
PAGE_X=${PAGE_X:-1250}
PAGE_Y=${PAGE_Y:-600}
PAGE_CLOSE_X=${PAGE_CLOSE_X:-1585}
PAGE_CLOSE_Y=${PAGE_CLOSE_Y:-122}
SITE=

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo" "$E2E_WORK/site"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Outside client"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><title>Buttons</title>
<body style="font: 20px sans-serif; margin: 32px"><button>Start</button> <button>Stop</button></body>
HTML
  SITE="http://127.0.0.1:$(serve_site site)"
  echo "$SITE" >"$E2E_WORK/site-url"
  offline_chromium
  profile_setting marley.agent_terminal_writes '"never_ask"'
  git init -q -b main "$repo"
  open_path "$repo"
}

teardown() {
  browser_teardown
}

steps() {
  local site
  site=$(cat "$E2E_WORK/site-url")
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== a write into the terminal's program: the bar"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "python3 -q"
  press "" Return
  settle 3
  mcp_agent terminal-type python "print(6 * 7)" --submit | tee "$E2E_WORK/type1.txt"
  expect "the outside client typed" holds "$E2E_WORK/type1.txt" "bytes into"
  settle 2
  shot 810-01-bar

  echo "== the close button hides it"
  click "$TERMINAL_CLOSE_X" "$TERMINAL_CLOSE_Y"
  settle 1
  shot 810-02-closed

  echo "== the next write brings it back"
  mcp_agent terminal-type python "print('again')" --submit | tee "$E2E_WORK/type2.txt"
  settle 2
  shot 810-03-back

  echo "== Ctrl-I takes over, and again hands back"
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL" i
  settle 1
  shot 810-04-taken
  mcp_agent terminal-type python "print(1)" --submit | tee "$E2E_WORK/type3.txt"
  expect "a write while the user has control is refused" holds "$E2E_WORK/type3.txt" "taken over"
  press "CTRL" i
  settle 1
  shot 810-05-handed-back

  echo "== an agent acts in a page: the page's bar"
  mcp_agent navigate --new-tab "$site/index.html" | tee "$E2E_WORK/navigate.txt"
  settle 3
  mcp_agent click-on button Start | tee "$E2E_WORK/click1.txt"
  settle 2
  shot 810-06-page-bar
  click "$PAGE_CLOSE_X" "$PAGE_CLOSE_Y"
  settle 1
  shot 810-07-page-closed
  mcp_agent click-on button Stop | tee "$E2E_WORK/click2.txt"
  settle 2
  shot 810-08-page-back

  echo "== Ctrl-I in the page takes over, and again hands back"
  click "$PAGE_X" "$PAGE_Y"
  press "CTRL" i
  settle 1
  shot 810-08b-page-taken
  press "CTRL" i
  settle 1
  shot 810-09-page-handed-back
}
