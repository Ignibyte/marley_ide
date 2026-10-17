# shellcheck shell=bash
# #802's visual check: `browser_show_page` shows an agent's HTML page in a Browser tab of its
# project. A stand-in agent calls the tool through the Claude Code plugin's bridge from the
# repository's folder, as an agent CLI there does:
# - `html` and `name: status`, a page whose colors are the stylesheet's variables: a Browser tab
#   of the project shows it in the theme's colors (`802-01-scratch`, REQ-001, REQ-003).
# - The same `name` with changed text: the same tab shows the change, and no second tab opened
#   (`802-02-reload`, REQ-002).
# - The theme switched to One Light: the page loads again in the light colors (`802-04-theme`,
#   REQ-003).
# - `path` to the project's `report.html`, then `sed -i` on the file: the tab shows the new text,
#   and the tool wrote nothing into the project (`802-03-path`, REQ-002, REQ-004).
# - From the project's terminal, `path` to a page outside the project and to `README.md`:
#   `opened: false` with the reason, and no new tab (`802-05-outside`, REQ-005).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The rail's row of the project's terminal, as the first run's shots found it.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-100}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-172}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  write_page_agent
  cat >"$home/.bashrc" <<RC
PS1='\$ '
show_page() {
  BRIDGE="$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \\
    MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \\
    python3 "$E2E_WORK/page-agent.py" "\$@" 2>&1 | tee -a "$E2E_WORK/client.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/client.log"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$E2E_WORK/repo/report.html" <<'HTML'
<!doctype html><title>Report</title>
<body style="font: 20px sans-serif; margin: 32px"><h1>Report</h1><p>The first draft.</p></body>
HTML
  cat >"$E2E_WORK/outside.html" <<'HTML'
<!doctype html><title>Outside</title><body>Outside the project.</body>
HTML
  offline_chromium
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_page_agent() {
  cat >"$E2E_WORK/page-agent.py" <<'PY'
# A stand-in agent CLI for #802's e2e test: it runs the plugin's bridge, as Claude Code does,
# calls browser_show_page, and prints the answer.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params=None):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method,
                                   "params": params or {}}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        if line.strip():
            message = json.loads(line)
            if message.get("id") == next_id:
                return message
    sys.exit(f"{method}: the bridge closed")


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": {"name": "e2e", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()

PAGE = """<!doctype html><html><head><title>Build status</title></head>
<body style="margin: 0; padding: 32px; background: var(--marley-background);
  color: var(--marley-text); font: 20px var(--marley-font)">
<div style="background: var(--marley-surface); border: 1px solid var(--marley-border);
  border-radius: 8px; padding: 24px">
<h1 style="color: var(--marley-accent)">Build status</h1>
<p>Run %(run)s: <span style="color: var(--marley-success)">%(passing)s passing</span>,
<span style="color: var(--marley-error)">%(failing)s failing</span>,
<span style="color: var(--marley-warning)">1 skipped</span></p>
<p style="color: var(--marley-text-muted); font-family: var(--marley-mono-font)">cargo test --workspace</p>
</div></body></html>"""

verb = sys.argv[1]
if verb == "scratch":
    run = sys.argv[2]
    passing, failing = ("3", "1") if run == "1" else ("4", "0")
    arguments = {"html": PAGE % {"run": run, "passing": passing, "failing": failing},
                 "name": "status"}
else:
    arguments = {"path": sys.argv[2]}
reply = call("tools/call", {"name": "browser_show_page", "arguments": arguments})
print("answer:", json.dumps(reply.get("result", {}).get("structuredContent", reply)))
bridge.stdin.close()
PY
}

# Calls the tool from the project's folder, as an agent CLI started there does.
page_agent() {
  local bridge=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge
  (
    cd "$E2E_WORK/repo" &&
      BRIDGE=$bridge MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \
        python3 "$E2E_WORK/page-agent.py" "$@"
  ) 2>&1 | tee -a "$E2E_WORK/client.log"
}

tabs() {
  mcp_agent tabs >"$E2E_WORK/tabs.txt"
  cat "$E2E_WORK/tabs.txt"
}

tab_count() {
  grep -c "^  tab " "$E2E_WORK/tabs.txt" || true
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

# Types a command at the terminal's prompt and runs it, in one `wtype`.
run_in_terminal() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 300 -k Return
}

steps() {
  local stylesheet
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== html and a name: a Browser tab of the project, in the theme's colors"
  page_agent scratch 1
  settle 8
  shot 802-01-scratch
  expect "the page opened, with its tab" holds "$E2E_WORK/client.log" '"opened": true' '"tab": "' '"project": "repo"'
  stylesheet=$(find "$E2E_PROFILE/pages" -name marley-theme.css -print -quit)
  echo "stylesheet: $stylesheet"
  cat "$stylesheet"
  expect "the scratch page links the stylesheet" \
    test "$(grep -rl --include=status.html "marley-theme.css" "$E2E_PROFILE/pages" | wc -l)" -eq 1
  expect "the stylesheet holds the theme's variables" holds "$stylesheet" "color-scheme: dark" "--marley-surface:" "--marley-mono-font:"
  expect "Zed's font names are the fonts they stand for" holds "$stylesheet" '"IBM Plex Sans"' '"Lilex"'

  echo "== the same name again, changed: the same tab shows it"
  page_agent scratch 2
  settle 4
  shot 802-02-reload
  tabs
  expect "one Browser tab, on the page" test "$(tab_count)" -eq 1

  echo "== a light theme: the page loads again in its colors"
  palette "theme selector: toggle"
  settle 2
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "One Light" -s 800 -k Return
  settle 4
  shot 802-04-theme
  cat "$stylesheet"
  expect "the stylesheet follows the theme" holds "$stylesheet" "color-scheme: light"

  echo "== a project file by its path, then changed on disk"
  ls -A "$E2E_WORK/repo" >"$E2E_WORK/repo-before.txt"
  page_agent path report.html
  settle 4
  sed -i 's/The first draft\./The second draft, written by sed./' "$E2E_WORK/repo/report.html"
  settle 4
  shot 802-03-path
  tabs
  ls -A "$E2E_WORK/repo" >"$E2E_WORK/repo-after.txt"
  expect "the report opened" holds "$E2E_WORK/client.log" "report.html"
  expect "two Browser tabs" test "$(tab_count)" -eq 2
  expect "nothing written into the project" cmp -s "$E2E_WORK/repo-before.txt" "$E2E_WORK/repo-after.txt"

  echo "== from the terminal: a page outside the project, and a file that is not HTML"
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 2
  run_in_terminal "show_page path $E2E_WORK/outside.html"
  settle 4
  run_in_terminal "show_page path README.md"
  settle 4
  shot 802-05-outside
  tabs
  expect "the outside page was declined" holds "$E2E_WORK/client.log" '"opened": false' "outside your project" "not an .html or .htm file"
  expect "and no new tab" test "$(tab_count)" -eq 2
}
