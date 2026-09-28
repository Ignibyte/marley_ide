# shellcheck shell=bash
# #571's e2e test: the pause before a consequential click. The stand-in agent, through the
# plugin's bridge, clicks in an offline Chromium's shop page: "Place order" in a cart form that
# posts to /checkout, "Remove item", a "Next" link, a "Continue" under a charge, a second one under
# text about a password, and "Delete account"; each click is written into the page's log. The
# profile turns #565's layer on with the `replay` provider and the scratch repository listed; the
# recorded answers stand in for Jev. A consequential click waits under the toolbar with Refuse and
# Allow and a toast (REQ-001); Allow clicks (REQ-002), Refuse and Escape refuse (REQ-003), 25
# seconds refuse (REQ-004); the model's reading pauses an open click in `act` (REQ-005); a plain
# click and a reading in the band go at once (REQ-006, REQ-007); a page changed under the pause
# clicks nothing (REQ-008); the tab refuses other writes while it waits (REQ-009); an unknown
# caller waits under `agents_without_prompts` (REQ-010), and `off` pauses nothing (REQ-011). The
# flight recorder keeps the pause (REQ-012). #566's stand-in `claude` calls from inside Marley's
# terminal, so its prompting session goes at once and its session in `bypassPermissions` waits
# (REQ-016); a caller named `Zed` waits only while Zed's tool permissions allow the tool unasked
# (REQ-013, REQ-017). Last, the settings page's two items (REQ-014).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The card's Refuse and Allow, a point on its sentence, the page's Swap button, a point in the
# page, the rail's terminal row, and a point in the Settings window's page, from the first run's
# shots.
REFUSE_X=${REFUSE_X:-1273}
REFUSE_Y=${REFUSE_Y:-136}
ALLOW_X=${ALLOW_X:-1329}
ALLOW_Y=${ALLOW_Y:-136}
CARD_X=${CARD_X:-1000}
CARD_Y=${CARD_Y:-136}
SWAP_X=${SWAP_X:-1125}
SWAP_Y=${SWAP_Y:-380}
PAGE_X=${PAGE_X:-1250}
PAGE_Y=${PAGE_Y:-230}
TERMINAL_ROW_X=${TERMINAL_ROW_X:-110}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-135}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}
# The toast's Show, from the first run's shots.
SHOW_X=${SHOW_X:-1171}
SHOW_Y=${SHOW_Y:-933}

# Merges the JSON object `$1` into the profile copy's settings, object by object.
setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
def skip_blank(index):
    while index < len(text):
        if text[index] in " \t\r\n":
            index += 1
        elif text.startswith("//", index):
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = len(text) if end < 0 else end + 2
        else:
            break
    return index


out, index, in_string = [], 0, False
while index < len(text):
    character = text[index]
    if in_string:
        out.append(character)
        if character == "\\" and index + 1 < len(text):
            out.append(text[index + 1])
            index += 2
            continue
        if character == '"':
            in_string = False
        index += 1
        continue
    if character == '"':
        in_string = True
    elif text.startswith("//", index) or text.startswith("/*", index):
        index = skip_blank(index)
        continue
    elif character == ",":
        ahead = skip_blank(index + 1)
        if ahead < len(text) and text[ahead] in "}]":
            index += 1
            continue
    out.append(character)
    index += 1
cleaned = "".join(out).strip()
settings = json.loads(cleaned) if cleaned else {}


def merge(into, changes):
    for key, value in changes.items():
        if isinstance(value, dict) and isinstance(into.get(key), dict):
            merge(into[key], value)
        else:
            into[key] = value


merge(settings, changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
  settle 2
}

# The click consequence's mode, and whose clicks it pauses.
pause_mode() {
  setting "{\"marley\": {\"browser_click_pause_agents\": \"${2:-all_agents}\", \"system_one\": {\"uses\": {\"click_consequence\": \"$1\"}}}}"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  offline_chromium
  mkdir -p "$home" "$bin" "$config/plugins" "$E2E_WORK/shop" "$E2E_PROFILE/system_one"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # The plugin listed at the version Marley ships, so the agent bar offers no update.
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  write_shop
  SITE=http://127.0.0.1:$(serve_site shop)
  git init -q -b shop "$E2E_WORK/repo"
  write_replay
  write_mcp_agent
  write_stand_in "$bin/claude"
  # The `claude` Marley runs for the plugin's commands: the stand-in, never the real one.
  export MARLEY_CLAUDE=$bin/claude
  setting "{\"marley\": {\"browser_click_pause_agents\": \"all_agents\", \"system_one\": {\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"click_consequence\": \"shadow\"}}}, \"agent\": {\"tool_permissions\": {\"default\": \"confirm\"}}}"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_shop() {
  cat >"$E2E_WORK/shop/index.html" <<'HTML'
<!doctype html><html><head><title>Shop</title><style>
body { margin: 0; padding: 16px; font: 18px sans-serif }
button { font-size: 18px; margin: 4px }
form, section { border: 1px solid #888; padding: 8px; margin: 8px 0; width: 520px }
h2 { margin: 4px 0; font-size: 20px }
</style></head><body>
<h1>The shop</h1>
<form id="cart" action="/checkout" method="post" onsubmit="event.preventDefault()">
  <h2>Your cart</h2>
  <p>One mug.</p>
  <button id="place" type="submit">Place order</button>
  <button id="remove" type="button">Remove item</button>
  <button id="swap" type="button">Swap</button>
</form>
<p><a id="next" href="#next">Next</a></p>
<section id="pay"><h2>Payment</h2><p>You will be charged $12.00.</p>
  <button id="continue-pay">Continue</button></section>
<section id="keep"><h2>Security</h2><p>Your password stays as it is.</p>
  <button id="continue-keep">Continue</button></section>
<section id="settings"><h2>Settings</h2><button id="delete">Delete account</button></section>
<p>Log: <span id="log"></span></p>
<script>
for (const element of document.querySelectorAll('button, a')) {
  element.addEventListener('click', (event) => {
    event.preventDefault();
    const log = document.getElementById('log');
    log.textContent += (log.textContent ? ', ' : '') + element.textContent;
    document.title = 'Log: ' + log.textContent;
    if (element.id === 'swap') document.getElementById('place').textContent = 'Cancel order';
  });
}
</script>
</body></html>
HTML
}

# The recorded answers: the first Continue pays, and the second is in the band.
write_replay() {
  cat >"$E2E_PROFILE/system_one/replay.jsonl" <<'JSONL'
{"set": "click_consequence/1", "match": "You will be charged", "answers": {"pays": {"type": "noul", "noul": 0.88}, "deletes": {"type": "noul", "noul": 0.05}, "sends": {"type": "noul", "noul": 0.02}, "changes_account": {"type": "noul", "noul": 0.03}}}
{"set": "click_consequence/1", "match": "Your password stays", "answers": {"pays": {"type": "noul", "noul": 0.5}, "deletes": {"type": "noul", "noul": 0.5}, "sends": {"type": "noul", "noul": 0.5}, "changes_account": {"type": "noul", "noul": 0.5}}}
JSONL
}

# #566's stand-in `claude`: at each line it reads, it sends its step's hook events through the
# plugin's hook in one session, with the step's permission mode, then runs the stand-in agent's
# click from inside Marley's terminal, so the click carries the terminal's id (#520), and keeps
# what it printed.
write_stand_in() {
  python3 - "$E2E_WORK/steps.json" <<'PY'
import json, sys

steps = [
    {"label": "a session that asks first", "prompt_id": "p1", "mode": "default",
     "prompt": "Place the order", "out": "claude-prompts.txt"},
    {"label": "a session that asks for nothing", "prompt_id": "p2", "mode": "bypassPermissions",
     "prompt": "Place the order again", "out": "claude-bypass.txt"},
]
json.dump(steps, open(sys.argv[1], "w"), indent=1)
PY
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@WORK@|$E2E_WORK|" \
    -e "s|@BRIDGE@|$BRIDGE|" -e "s|@ENDPOINT@|$E2E_PROFILE/mcp-endpoint.json|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code (#566's): `claude plugin ...` does nothing. Otherwise, at each line it
# reads, it acts out the next step: a prompt through the plugin's hook, in the step's permission
# mode, then the stand-in agent's click on "Place order", run as Claude Code runs its MCP server's
# bridge, from inside the terminal.
import json
import os
import subprocess
import sys
import time

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
print("Claude Code (stand-in): press Enter for each step", flush=True)
for number, step in enumerate(STEPS, 1):
    if not sys.stdin.readline():
        break
    payload = {"hook_event_name": "UserPromptSubmit", "prompt": step["prompt"], "session_id": "s1",
               "prompt_id": step["prompt_id"], "permission_mode": step["mode"],
               "transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd()}
    answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(payload),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()
    print(f"step {number}: {step['label']}", flush=True)
    # Time for Marley to fold the prompt's event before the click asks who is calling.
    time.sleep(1.5)
    environment = {**os.environ, "BRIDGE": "@BRIDGE@", "MARLEY_MCP_ENDPOINT": "@ENDPOINT@"}
    with open(os.path.join("@WORK@", step["out"]), "w", encoding="utf-8") as out:
        subprocess.run([sys.executable, "@WORK@/mcp-agent.py", "click-on", "button", "Place order"],
                       stdout=out, stderr=subprocess.STDOUT, env=environment, check=False)
    print(f"step {number}: clicked or refused", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Today's rows of the click consequence; whether one holds a text; and the page's log, which its
# script keeps in its title, as the agent reads it.
calls() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep '"use":"click_consequence"' || true
}
row_holds() {
  calls | grep -qF -- "$1"
}
page_log() {
  mcp_agent tabs | grep -o "'Log: [^']*'" | head -1 || true
}

# Starts the stand-in agent's `click-on button "$1"` in the background, its output in
# $E2E_WORK/<label>.txt; `finish` waits for it.
click_later() {
  mcp_agent click-on button "$1" >"$E2E_WORK/$2.txt" 2>&1 &
  CLICKING=$!
  settle 3
}
finish() {
  wait "$CLICKING" || true
  cat "$E2E_WORK/$1.txt"
}

# Whether the stand-in's output $E2E_WORK/<label>.txt holds each text.
said() {
  local label=$1
  shift
  holds "$E2E_WORK/$label.txt" "$@"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  mcp_agent navigate "$SITE/index.html"
  settle 3

  echo "== a consequential click waits, in shadow, on the rules"
  click_later "Place order" placed
  shot 571-01-paused
  page_log | tee "$E2E_WORK/log-paused.txt"
  expect "nothing was clicked while it waits" bash -c "! grep -q 'Place order' '$E2E_WORK/log-paused.txt'"
  expect "the rules' row is logged" row_holds 'rules found: its name'

  echo "== Allow"
  click "$ALLOW_X" "$ALLOW_Y"
  settle 2
  finish placed
  expect "the tool answers as a click" said placed 'clicked button'
  page_log | tee "$E2E_WORK/log-allowed.txt"
  expect "the page took the click" holds "$E2E_WORK/log-allowed.txt" 'Place order'
  shot 571-02-allowed
  click "$PAGE_X" "$PAGE_Y"
  settle 1
  palette "marley: record this"
  settle 3
  local listing id
  listing=$(mcp_agent recordings)
  id=$(printf '%s\n' "$listing" | sed -n 's/^  recording \([0-9A-Za-z-]*\):.*/\1/p' | tail -1)
  mcp_agent recording "$id" | tee "$E2E_WORK/recording.txt"
  expect "the recorder kept the pause and its end" holds "$E2E_WORK/recording.txt" \
    '"did": "paused: ' '"did": "allowed: '

  echo "== Escape on the focused card refuses"
  click_later "Delete account" deleted
  click "$CARD_X" "$CARD_Y"
  settle 1
  press "" Escape
  settle 2
  finish deleted
  expect "the agent reads that the user refused" said deleted 'Marley paused this click' \
    'The user refused it'
  shot 571-03-refused
  page_log | tee "$E2E_WORK/log-refused.txt"
  expect "Delete account was not clicked" bash -c "! grep -q 'Delete account' '$E2E_WORK/log-refused.txt'"

  echo "== act: the model's reading pauses an open click"
  pause_mode act
  click_later "Continue" continued
  shot 571-04-open-case
  expect "the model was asked about the charge" row_holds 'You will be charged'
  click "$REFUSE_X" "$REFUSE_Y"
  settle 2
  finish continued
  expect "the open click was held and refused" said continued 'as the model reads it (0.88)' \
    'The user refused it'

  echo "== a plain click goes at once"
  local before
  before=$(calls | wc -l)
  mcp_agent click-on link "Next" | tee "$E2E_WORK/next.txt"
  settle 1
  shot 571-05-plain
  expect "the link was clicked at once" said next 'clicked link'
  expect "and nothing was asked" test "$(calls | wc -l)" -eq "$before"

  echo "== 25 seconds unanswered"
  click_later "Remove item" removed
  settle 24
  finish removed
  shot 571-06-expired
  expect "the agent reads the expiry" said removed 'did not answer within 25 seconds'
  page_log | tee "$E2E_WORK/log-expired.txt"
  expect "Remove item was not clicked" bash -c "! grep -q 'Remove item' '$E2E_WORK/log-expired.txt'"

  echo "== a reading in the band goes at once"
  local second
  second=$(mcp_agent snapshot | python3 -c 'import re,sys; print(re.findall(r"- button \"Continue\".*\[ref=(e\d+)\]", sys.stdin.read())[1])')
  mcp_agent click-ref "$second" | tee "$E2E_WORK/second.txt"
  settle 1
  shot 571-07-no-signal
  expect "the second Continue was clicked at once" said second '"did": "clicked button'
  expect "and its reading is logged" row_holds 'Your password stays'

  echo "== a page changed under the pause"
  click_later "Place order" changed
  click "$SWAP_X" "$SWAP_Y"
  settle 1
  click "$CARD_X" "$CARD_Y"
  settle 1
  press "" Return
  settle 2
  finish changed
  shot 571-08-changed
  expect "Allow on a changed page clicks nothing" said changed 'The page changed while it waited'
  page_log | tee "$E2E_WORK/log-changed.txt"
  expect "the renamed button was not clicked" bash -c "! grep -q 'Cancel order' '$E2E_WORK/log-changed.txt'"

  echo "== the tab refuses other writes while it waits"
  mcp_agent navigate "$SITE/index.html"
  settle 3
  click_later "Place order" blocked
  mcp_agent scroll 100 | tee "$E2E_WORK/scroll.txt"
  shot 571-09-blocked-writes
  expect "a scroll waits for the user too" said scroll 'a click is paused in this tab'
  click "$REFUSE_X" "$REFUSE_Y"
  settle 2
  finish blocked

  echo "== agents_without_prompts: an unknown caller waits"
  pause_mode shadow agents_without_prompts
  click_later "Place order" unknown
  shot 571-10-unknown-caller
  click "$REFUSE_X" "$REFUSE_Y"
  settle 2
  finish unknown
  expect "a caller Marley cannot name waits" said unknown 'e2e wants to click' \
    'The user refused it'

  echo "== a caller named Zed, while Zed asks first"
  export MCP_CLIENT_NAME=Zed
  mcp_agent click-on button "Place order" | tee "$E2E_WORK/zed-asks.txt"
  settle 1
  shot 571-14a-zed-asks
  expect "Zed's agent, which asks first, clicks at once" said zed-asks 'clicked button'

  echo "== a caller named Zed, while Zed allows the tool unasked"
  setting '{"agent": {"tool_permissions": {"default": "allow"}}}'
  mcp_agent navigate "$SITE/index.html"
  settle 3
  click_later "Place order" zed-allows
  shot 571-14b-zed-allows
  click "$REFUSE_X" "$REFUSE_Y"
  settle 2
  finish zed-allows
  unset MCP_CLIENT_NAME
  expect "Zed's agent that asks nothing waits" said zed-allows "Zed's agent wants to click"

  echo "== Claude Code in Marley's terminal"
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "claude"
  press "" Return
  settle 3
  press "" Return
  settle 6
  shot 571-12-claude-prompts
  cat "$E2E_WORK/claude-prompts.txt"
  expect "a session that asks first clicks at once" said claude-prompts 'clicked button'
  mcp_agent navigate "$SITE/index.html"
  settle 3
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  press "" Return
  settle 4
  shot 571-13-claude-bypass
  # The terminal is in front: the toast's Show brings the tab with the focus on the card.
  click "$SHOW_X" "$SHOW_Y"
  settle 2
  press "" Escape
  settle 3
  cat "$E2E_WORK/claude-bypass.txt"
  expect "a session that asks for nothing waits" said claude-bypass 'Claude Code wants to click' \
    'The user refused it'

  echo "== off"
  pause_mode off all_agents
  before=$(calls | wc -l)
  mcp_agent click-on button "Place order" | tee "$E2E_WORK/off.txt"
  settle 1
  shot 571-11-off
  expect "off clicks at once" said off 'clicked button'
  expect "and asks nothing" test "$(calls | wc -l)" -eq "$before"

  echo "== the Marley settings page"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 8
  settle 2
  shot 571-15a-agents
  scroll 32
  settle 2
  shot 571-15b-system-one
}
