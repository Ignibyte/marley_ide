# shellcheck shell=bash
# #767's visual check: file choosers, downloads and Take Over in the Browser tab. The page holds a
# file input that lists the chosen names, a link that downloads `report.csv`, and a Ping button
# that counts its clicks. Zed's own path prompt answers the person (`use_system_path_prompts` off),
# downloads land in the run's own folder (`XDG_DOWNLOAD_DIR`), and a stand-in agent reaches
# Marley's MCP server through the plugin's bridge (`mcp_agent`).
# Shots: the person's chooser opens the path prompt and fills the input (REQ-001); an agent's
# chooser waits in a bar and `browser_upload` answers it (REQ-002); a download's notification
# (REQ-003); the tab taken over, an agent refused (REQ-004); handed back, an agent acting again
# (REQ-005).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
BROWSER_TAB_X=${BROWSER_TAB_X:-1031}
BROWSER_TAB_Y=${BROWSER_TAB_Y:-51}
CHOOSE_X=${CHOOSE_X:-330}
CHOOSE_Y=${CHOOSE_Y:-181}
PAGE_EMPTY_X=${PAGE_EMPTY_X:-1200}
PAGE_EMPTY_Y=${PAGE_EMPTY_Y:-800}
HAND_BACK_X=${HAND_BACK_X:-1550}
HAND_BACK_Y=${HAND_BACK_Y:-131}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# The ref of the element named `$1` in a full snapshot of the tab.
ref_of() {
  mcp_agent snapshot full | python3 -c '
import re, sys
found = re.search(r"\"%s\"[^\n]*?\[ref=(e\d+)\]" % re.escape(sys.argv[1]), sys.stdin.read())
print(found.group(1) if found else "")' "$1"
}

tool() {
  mcp_agent tool "$1" "$2" 2>&1 | tee "$E2E_WORK/$3.txt"
}

# Whether the page shows `$1`, through `browser_wait`.
shows() {
  mcp_agent tool browser_wait "{\"text\": \"$1\", \"timeout_ms\": 3000}" 2>&1 | grep -q '"held"'
}

said() {
  grep -q -- "$2" "$E2E_WORK/$1.txt"
}

write_lab() {
  mkdir -p "$E2E_WORK/lab"
  printf 'name,total\nmugs,12\n' >"$E2E_WORK/lab/report.csv"
  cat >"$E2E_WORK/lab/index.html" <<'HTML'
<!doctype html><html><head><meta charset="utf-8"><title>Files</title><style>
body { margin: 0; padding: 16px; font: 16px sans-serif; background: #fff; color: #111 }
p { margin: 10px 0 }
</style></head><body>
<p><input type="file" id="file" aria-label="Attachment"> <span id="files">Files: none</span></p>
<p><a id="report" href="report.csv" download>Download the report</a></p>
<p><button id="ping">Ping</button> <span id="pings">Pings: 0</span></p>
<script>
const $ = (id) => document.getElementById(id);
$('file').addEventListener('change', () => {
  $('files').textContent = 'Files: ' + Array.from($('file').files).map((file) => file.name).join(', ');
});
let pings = 0;
$('ping').addEventListener('click', () => { pings += 1; $('pings').textContent = 'Pings: ' + pings; });
</script></body></html>
HTML
}

setup() {
  local repo=$E2E_WORK/repo
  mkdir -p "$repo" "$E2E_WORK/downloads"
  printf 'Notes for the person.\n' >"$repo/notes.txt"
  printf 'More for the agent.\n' >"$repo/more.txt"
  write_lab
  export XDG_DOWNLOAD_DIR=$E2E_WORK/downloads
  profile_setting use_system_path_prompts false
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site lab)
  git init -q -b main "$repo"
  write_mcp_agent
  open_path "$repo"
}

teardown() {
  browser_teardown
}

steps() {
  local repo=$E2E_WORK/repo
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  mcp_agent navigate "$SITE/index.html"
  settle 3
  # The Browser tab's pane zoomed, so the page and its bars have the room.
  click "$BROWSER_TAB_X" "$BROWSER_TAB_Y"
  settle 1
  press "SHIFT" Escape
  settle 2

  echo "== the person's file chooser"
  click "$CHOOSE_X" "$CHOOSE_Y"
  settle 2
  take 767-01-person-prompt || return 0
  press "CTRL" a
  type_text "$repo/notes.txt"
  settle 1
  press "" Return
  settle 2
  take 767-02-person-chosen || return 0
  expect "the input lists notes.txt" shows "Files: notes.txt"

  echo "== an agent's file chooser"
  tool browser_click "{\"ref\": \"$(ref_of 'Attachment')\"}" agent-click
  settle 2
  take 767-03-agent-chooser || return 0
  mcp_agent tabs | tee "$E2E_WORK/tabs-chooser.txt"
  tool browser_upload "{\"paths\": [\"$repo/more.txt\"]}" upload
  settle 2
  take 767-04-agent-upload || return 0
  expect "the input lists more.txt" shows "Files: more.txt"
  mcp_agent refusal browser_upload "{\"paths\": [\"$repo/more.txt\"]}" | tee "$E2E_WORK/no-chooser.txt"
  expect "no chooser and no ref is refused no_chooser" said no-chooser "code no_chooser"

  echo "== a download"
  tool browser_click "{\"ref\": \"$(ref_of 'Download the report')\"}" download
  settle 4
  take 767-05-download || return 0
  ls -la "$E2E_WORK/downloads"
  expect "report.csv is in the Downloads folder" test -s "$E2E_WORK/downloads/report.csv"
  mcp_agent tabs | tee "$E2E_WORK/tabs-download.txt"

  echo "== Take Over"
  click "$PAGE_EMPTY_X" "$PAGE_EMPTY_Y"
  settle 1
  press "CTRL" i
  settle 1
  take 767-06-taken || return 0
  mcp_agent refusal browser_click "{\"ref\": \"$(ref_of 'Ping')\"}" | tee "$E2E_WORK/taken.txt"
  expect "an agent's click is refused taken_over" said taken "code taken_over"

  echo "== Hand Back"
  click "$HAND_BACK_X" "$HAND_BACK_Y"
  settle 1
  take 767-07-handed-back || return 0
  tool browser_click "{\"ref\": \"$(ref_of 'Ping')\"}" ping
  expect "an agent's click works again" shows "Pings: 1"
}
