# shellcheck shell=bash
# #567's e2e test: browser_find and terminal_find, an agent's two tools that find an element of a
# page or a line of a block from a query in words. The stand-in agent, through the plugin's
# bridge, calls them against an offline Chromium's page (a Sign in button, two Submit buttons in
# two forms, a Docs link) and a block of 300 numbered lines. The profile turns #565's layer on
# with the `replay` provider and the scratch repository listed, and both uses in `act`; the
# recorded answers, written before Marley starts, stand in for Jev. The query's words alone answer
# "sign in" and "connection refused" with no call (REQ-001, REQ-005), and a click on the answered
# ref signs in (REQ-004); the model ranks the two Submit buttons (REQ-002), reads "coupon code"
# absent (REQ-003) and finds the failed connection among two windows of lines, the block's token
# masked in the state (REQ-005, REQ-009). In `shadow` the words answer and the model's reading is
# only logged (REQ-006); an unlisted project makes no call (REQ-008); `off` takes both tools out
# of the list and refuses them by name (REQ-007). Last, the Marley settings page's two items.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The rail's terminal row, where a click shows the terminal, and a point in the Settings
# window's page to scroll at, from the first run's shots.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-110}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-135}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

# Merges the JSON object `$1` into `marley.system_one` in the profile copy's settings (565's).
system_one_setting() {
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
settings.setdefault("marley", {}).setdefault("system_one", {}).update(changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
}

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site" "$E2E_PROFILE/system_one"
  # shellcheck disable=SC2016 # the prompt is the terminal's to expand
  printf 'PS1=%s\n' "'\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Find</title><style>
body { margin: 0; padding: 20px; font: 18px sans-serif }
button, input { font-size: 18px }
form { border: 1px solid #888; padding: 8px; margin: 10px 0; width: 420px }
</style></head><body>
<h1>The shop</h1>
<p><button id="sign-in">Sign in</button> <span id="status">Not signed in.</span></p>
<form id="newsletter" onsubmit="event.preventDefault()"><h2>Newsletter</h2>
  <input aria-label="Email" id="email"> <button>Submit</button></form>
<form id="contact" onsubmit="event.preventDefault()"><h2>Contact</h2>
  <input aria-label="Message" id="message"> <button>Submit</button></form>
<p><a href="#docs">Docs</a></p>
<script>
document.getElementById('sign-in').addEventListener('click', () =>
  document.getElementById('status').textContent = 'Signed in.');
</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b find "$E2E_WORK/repo"
  # A token put together here, so no file of the repository holds one.
  local token
  token=$(printf 'gh%s_%s' p "$(printf 'Fake%.0s' {1..9})")
  cat >"$E2E_WORK/repo/print-lines.sh" <<SH
#!/usr/bin/env bash
for number in \$(seq 1 300); do
  case \$number in
    212) echo "line 212: connect: connection refused" ;;
    250) echo "line 250: token $token" ;;
    *) echo "line \$number: ok" ;;
  esac
done
SH
  chmod +x "$E2E_WORK/repo/print-lines.sh"
  write_replay
  system_one_setting "{\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"browser_find\": \"act\", \"terminal_find\": \"act\"}}"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

# The recorded answers, keyed by the find set of the window's number of items and a match on the
# state's query line. "coupon code" asks over every ref of the page, whose number the rows cover.
write_replay() {
  python3 - "$E2E_PROFILE/system_one/replay.jsonl" <<'PY'
import json, sys

def choice(option, confidence, rest):
    probabilities = {option: confidence, **rest}
    return {"type": "choice", "choice": option, "confidence": confidence, "probabilities": probabilities}

rows = [
    # The two Submit buttons, asked twice: in act, then in shadow.
    {"set": "find_2/1", "match": "query: submit", "answers": {
        "which": choice("2", 0.8, {"1": 0.15, "none": 0.05}), "present": {"type": "noul", "noul": 0.9}}},
    {"set": "find_2/1", "match": "query: submit", "answers": {
        "which": choice("2", 0.8, {"1": 0.15, "none": 0.05}), "present": {"type": "noul", "noul": 0.9}}},
    # The failed connection, in the first window of the block's lines.
    {"set": "find_254/1", "match": "query: where did the server fail to connect", "answers": {
        "which": choice("212", 0.85, {"211": 0.05, "none": 0.1}), "present": {"type": "noul", "noul": 0.92}}},
]
rows += [
    {"set": f"find_{count}/1", "match": "query: coupon code", "answers": {
        "which": choice("none", 0.9, {"1": 0.1}), "present": {"type": "noul", "noul": 0.05}}}
    for count in range(1, 21)
]
with open(sys.argv[1], "w") as file:
    for row in rows:
        file.write(json.dumps(row) + "\n")
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Today's call rows.
calls() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' || true
}

# The answer the stand-in printed into $E2E_WORK/<label>.txt, and a field of it.
field() {
  python3 - "$E2E_WORK/$1.txt" "$2" <<'PY'
import json, sys

answer = json.loads(open(sys.argv[1]).read().strip())
value = answer
for key in sys.argv[2].split("."):
    value = value[int(key)] if isinstance(value, list) else value.get(key)
print(value)
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the tools listed, with both uses on"
  mcp_agent tools | tee "$E2E_WORK/tools-on.txt"
  expect "both find tools are listed while their uses are on" holds "$E2E_WORK/tools-on.txt" \
    browser_find terminal_find

  echo "== a page"
  mcp_agent navigate "$SITE/index.html" | tee "$E2E_WORK/navigate.txt"
  settle 3
  mcp_agent snapshot | tee "$E2E_WORK/snapshot.txt"

  echo "== \"sign in\", by the words"
  mcp_agent find "sign in" | tee "$E2E_WORK/sign-in.txt"
  expect "the words alone answer the Sign in button" holds "$E2E_WORK/sign-in.txt" \
    '"source": "rules"' '"sure": true' '"role": "button", "name": "Sign in"'
  expect "and no call was made" test "$(calls | wc -l)" -eq 0
  mcp_agent click-ref "$(field sign-in ref)" | tee "$E2E_WORK/clicked.txt"
  settle 2
  shot 567-01-clicked
  mcp_agent snapshot full >"$E2E_WORK/after-click.txt"
  expect "the click on the answered ref signed in" holds "$E2E_WORK/after-click.txt" "Signed in."

  echo "== \"submit\", ranked by the model"
  mcp_agent find "submit" | tee "$E2E_WORK/submit.txt"
  expect "the model ranked the two Submit buttons and chose one" holds "$E2E_WORK/submit.txt" \
    '"source": "model"' '"sure": true' '"present": "found"' '"probability": 0.8'
  local second
  second=$(python3 -c 'import re,sys; print(re.findall(r"- button \"Submit\".*\[ref=(e\d+)\]", open(sys.argv[1]).read())[1])' \
    "$E2E_WORK/snapshot.txt")
  echo "  the second Submit is $second; the answer's ref is $(field submit ref)"
  expect "the answer is the second Submit, as the recorded answer chose" \
    test "$(field submit ref)" = "$second"

  echo "== \"coupon code\", absent"
  mcp_agent find "coupon code" | tee "$E2E_WORK/coupon.txt"
  expect "the model reads nothing there, and the answer says to read the page" \
    holds "$E2E_WORK/coupon.txt" '"sure": false' '"present": "absent"' '"next": "browser_snapshot"'

  echo "== a block of lines"
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "./print-lines.sh"
  press "" Return
  settle 3
  mcp_agent tfind print-lines "connection refused" | tee "$E2E_WORK/refused.txt"
  expect "the words alone answer line 212" holds "$E2E_WORK/refused.txt" \
    '"source": "rules"' '"sure": true' '"line": 212'
  mcp_agent tfind print-lines "where did the server fail to connect" | tee "$E2E_WORK/fail.txt"
  expect "the model found line 212 among the windows" holds "$E2E_WORK/fail.txt" \
    '"source": "model"' '"sure": true' '"line": 212' '"present": "found"' \
    '"text": "line 212: connect: connection refused"'
  calls | grep '"set":"find_254/1"' | tail -n 1 >"$E2E_WORK/window.json"
  expect "the window's state holds the token masked" holds "$E2E_WORK/window.json" '[redacted: '
  expect "and not as printed" bash -c "! grep -q 'FakeFake' '$E2E_WORK/window.json'"

  echo "== shadow"
  system_one_setting '{"uses": {"browser_find": "shadow", "terminal_find": "act"}}'
  settle 3
  local before
  before=$(calls | wc -l)
  mcp_agent find "submit" | tee "$E2E_WORK/shadow.txt"
  settle 1
  expect "shadow answers by the words" holds "$E2E_WORK/shadow.txt" '"source": "rules"' \
    '"sure": false'
  expect "and logs the model's reading" bash -c \
    "test \$(cat '$E2E_PROFILE'/system_one/calls-*.jsonl | grep -c '\"mode\":\"shadow\"') -ge 1"
  expect "one call, for the one window" test "$(calls | wc -l)" -eq $((before + 1))

  echo "== an unlisted project"
  system_one_setting '{"projects": [], "uses": {"browser_find": "act", "terminal_find": "act"}}'
  settle 3
  before=$(calls | wc -l)
  mcp_agent find "submit" | tee "$E2E_WORK/unlisted.txt"
  expect "an unlisted project answers by the words with the reason" holds "$E2E_WORK/unlisted.txt" \
    '"source": "rules"' 'project not listed'
  expect "and makes no call" test "$(calls | wc -l)" -eq "$before"

  echo "== off"
  system_one_setting "{\"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"browser_find\": \"off\", \"terminal_find\": \"off\"}}"
  settle 3
  mcp_agent tools | tee "$E2E_WORK/tools-off.txt"
  expect "neither find tool is listed while its use is off" bash -c \
    "! grep -q 'browser_find\|terminal_find' '$E2E_WORK/tools-off.txt'"
  mcp_agent find "submit" | tee "$E2E_WORK/off.txt"
  expect "a call by name is refused with the setting named" holds "$E2E_WORK/off.txt" \
    "browser_find is off" "marley.system_one.uses.browser_find"
  shot 567-02-off-not-listed

  echo "== the Marley settings page"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  # Past the System One section's head to its last items.
  scroll 28
  settle 2
  shot 567-03-settings
}
