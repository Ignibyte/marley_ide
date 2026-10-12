# shellcheck shell=bash
# #756's visual check: the browser tools agents lacked. A stand-in agent reaches Marley's MCP server
# through the plugin's bridge (`mcp_agent`) and acts in a Browser tab on `lab/index.html`: a button
# that reads "hovered" under the pointer, a colour select whose choice shows, two lists of
# draggable cards whose order shows, a file input that lists the chosen names, a Confirm button
# whose outcome shows, a Start button after which "Ready" appears two seconds later, the
# viewport's size, and a line reading the page's `prefers-color-scheme`, which starts as the
# system's: the agent forces the other one. The repository holds
# `notes.txt` to upload and a `.env` a secret glob matches.
# Shots: hover (REQ-001), select (REQ-002), drag (REQ-003), upload (REQ-004), the dialog card an
# agent's click opened and its answer (REQ-006), the wait (REQ-007), a device size (REQ-009), the
# scheme forced (REQ-010) and Reset (REQ-011). The refusals (REQ-005, REQ-008) and the tool list
# (REQ-012) are checked in the run's log.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
RESET_X=${RESET_X:-1565}
RESET_Y=${RESET_Y:-96}
BROWSER_TAB_X=${BROWSER_TAB_X:-1031}
BROWSER_TAB_Y=${BROWSER_TAB_Y:-51}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# The ref of the element named `$1` in a full snapshot of the tab.
ref_of() {
  mcp_agent snapshot full | tee "$E2E_WORK/snapshot.txt" | python3 -c '
import re, sys
found = re.search(r"\"%s\"[^\n]*?\[ref=(e\d+)\]" % re.escape(sys.argv[1]), sys.stdin.read())
print(found.group(1) if found else "")' "$1"
}

# Calls the tool `$1` with the JSON `$2`, printing and saving the answer as `$E2E_WORK/<$3>.txt`.
tool() {
  mcp_agent tool "$1" "$2" 2>&1 | tee "$E2E_WORK/$3.txt"
}

# Whether the page shows `$1`, through `browser_wait`.
shows() {
  mcp_agent tool browser_wait "{\"text\": \"$1\", \"timeout_ms\": 3000}" 2>&1 | grep -q '"held"'
}

not_shows() {
  ! shows "$1"
}

said() {
  grep -q -- "$2" "$E2E_WORK/$1.txt"
}

write_lab() {
  mkdir -p "$E2E_WORK/lab"
  cat >"$E2E_WORK/lab/index.html" <<'HTML'
<!doctype html><html><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>Lab</title><style>
body { margin: 0; padding: 12px; font: 16px sans-serif; background: #fff; color: #111 }
@media (prefers-color-scheme: dark) { body { background: #222; color: #eee } }
section { border: 1px solid #888; padding: 6px; margin: 6px 0 }
ul { min-height: 40px; border: 1px dashed #888; padding: 6px; margin: 4px 0; list-style: none }
li { display: inline-block; padding: 4px 10px; margin: 2px; background: #9cf; color: #111 }
</style></head><body>
<section><button id="hover">Hover me</button> <span id="hovered">not hovered</span></section>
<section><label>Colour <select id="colour"><option value="red">Red</option>
<option value="green">Green</option><option value="blue">Blue</option></select></label>
<span id="chose">Chose: red</span></section>
<section>Todo <ul id="todo" aria-label="Todo"><li draggable="true" id="a" aria-label="Card A">Card A</li><li draggable="true" id="b" aria-label="Card B">Card B</li></ul>
Done <ul id="done" aria-label="Done"></ul><span id="order">Done: none</span></section>
<section><input type="file" id="file" aria-label="Attachment" multiple> <span id="files">Files: none</span></section>
<section><button id="confirm">Confirm</button> <span id="outcome">Outcome: none</span></section>
<section><button id="start">Start</button> <span id="ready"></span></section>
<section><span id="size"></span> · <span id="scheme"></span></section>
<script>
const $ = (id) => document.getElementById(id);
$('hover').addEventListener('mouseenter', () => { $('hovered').textContent = 'hovered'; });
$('colour').addEventListener('change', () => { $('chose').textContent = 'Chose: ' + $('colour').value; });
for (const card of document.querySelectorAll('li')) {
  card.addEventListener('dragstart', (event) => event.dataTransfer.setData('text/plain', card.id));
}
for (const list of [$('todo'), $('done')]) {
  list.addEventListener('dragover', (event) => event.preventDefault());
  list.addEventListener('drop', (event) => {
    event.preventDefault();
    const card = $(event.dataTransfer.getData('text/plain'));
    if (card) { list.appendChild(card); }
    const done = Array.from($('done').children).map((item) => item.textContent);
    $('order').textContent = 'Done: ' + (done.join(', ') || 'none');
  });
}
$('file').addEventListener('change', () => {
  $('files').textContent = 'Files: ' + Array.from($('file').files).map((file) => file.name).join(', ');
});
$('confirm').addEventListener('click', () => {
  $('outcome').textContent = 'Outcome: ' + (confirm('Delete the draft?') ? 'confirmed' : 'cancelled');
});
$('start').addEventListener('click', () => { setTimeout(() => { $('ready').textContent = 'Ready'; }, 2000); });
const measure = () => {
  $('size').textContent = 'Size: ' + innerWidth + 'x' + innerHeight;
  $('scheme').textContent = 'Scheme: ' + (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
};
addEventListener('resize', measure);
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', measure);
measure();
</script></body></html>
HTML
}

setup() {
  local repo=$E2E_WORK/repo
  mkdir -p "$repo"
  write_lab
  printf 'Notes for the upload.\n' >"$repo/notes.txt"
  printf 'API_TOKEN=not-a-real-token\n' >"$repo/.env"
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
  mcp_agent snapshot full
  # The page starts in the system's scheme; the agent forces the other one.
  local system=light forced=dark
  if shows "Scheme: dark"; then
    system=dark forced=light
  fi
  echo "the system's scheme: $system"

  echo "== hover"
  tool browser_hover "{\"ref\": \"$(ref_of 'Hover me')\"}" hover
  settle 1
  take 756-01-hover || return 0
  expect "the button reads hovered" shows "hovered"

  echo "== select"
  tool browser_select "{\"ref\": \"$(ref_of 'Colour')\", \"labels\": [\"Blue\"]}" select
  settle 1
  take 756-02-select || return 0
  expect "blue is chosen" shows "Chose: blue"
  mcp_agent refusal browser_select "{\"ref\": \"$(ref_of 'Colour')\", \"labels\": [\"Purple\"]}" | tee "$E2E_WORK/no-option.txt"
  expect "an option not there is refused no_option" said no-option "code no_option"

  echo "== drag"
  tool browser_drag "{\"from_ref\": \"$(ref_of 'Card A')\", \"to_ref\": \"$(ref_of 'Done')\"}" drag
  settle 1
  take 756-03-drag || return 0
  expect "card A is done" shows "Done: Card A"

  echo "== upload"
  local file
  file=$(ref_of 'Attachment')
  tool browser_upload "{\"ref\": \"$file\", \"paths\": [\"$repo/notes.txt\"]}" upload
  settle 1
  take 756-04-upload || return 0
  expect "the input lists notes.txt" shows "Files: notes.txt"
  mcp_agent refusal browser_upload "{\"ref\": \"$file\", \"paths\": [\"/etc/hostname\"]}" | tee "$E2E_WORK/outside.txt"
  expect "a file outside the projects is refused" said outside "code outside_projects"
  mcp_agent refusal browser_upload "{\"ref\": \"$file\", \"paths\": [\"$repo/.env\"]}" | tee "$E2E_WORK/secret.txt"
  expect "a secret file is refused" said secret "code secret_file"

  echo "== a click that opens a dialog, and its answer"
  local started
  started=$(date +%s)
  tool browser_click "{\"ref\": \"$(ref_of 'Confirm')\"}" confirm-click
  echo "the click answered in $(($(date +%s) - started)) s"
  expect "the click answered within 10 s" test "$(($(date +%s) - started))" -lt 10
  expect "the click names the dialog" said confirm-click '"kind": "confirm"'
  settle 1
  take 756-05a-dialog-card || return 0
  tool browser_dialog '{"accept": true}' dialog
  settle 1
  take 756-05-dialog || return 0
  expect "the page reads confirmed" shows "Outcome: confirmed"
  mcp_agent refusal browser_dialog '{"accept": true}' | tee "$E2E_WORK/no-dialog.txt"
  expect "no dialog is refused no_dialog" said no-dialog "code no_dialog"

  echo "== wait"
  tool browser_click "{\"ref\": \"$(ref_of 'Start')\"}" start
  tool browser_wait '{"text": "Ready"}' wait
  take 756-06-wait || return 0
  expect "the wait held" said wait '"held"'
  mcp_agent refusal browser_wait '{"text": "Never", "timeout_ms": 1000}' | tee "$E2E_WORK/not-met.txt"
  expect "a wait that does not hold is refused not_met" said not-met "code not_met"

  echo "== a device size"
  # The Browser tab's pane zoomed, so its toolbar has room for the chip and Reset.
  click "$BROWSER_TAB_X" "$BROWSER_TAB_Y"
  settle 1
  press "SHIFT" Escape
  settle 2
  tool browser_resize '{"preset": "iPhone 15 Pro"}' resize
  settle 3
  take 756-07-resize || return 0
  expect "the page is 393 wide" shows "Size: 393x852"

  echo "== the other scheme"
  tool browser_color_scheme "{\"scheme\": \"$forced\"}" scheme
  settle 2
  take 756-08-scheme || return 0
  expect "the page reads $forced" shows "Scheme: $forced"
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"

  echo "== Reset"
  click "$RESET_X" "$RESET_Y"
  settle 3
  take 756-09-reset || return 0
  expect "the page reads $system again" shows "Scheme: $system"
  expect "the page is no longer 393 wide" not_shows "Size: 393x852"

  echo "== the tools"
  mcp_agent tools | tee "$E2E_WORK/tools.txt"
  local name
  for name in browser_hover browser_select browser_drag browser_upload browser_dialog \
    browser_resize browser_color_scheme browser_wait; do
    expect "$name is listed" grep -q "$name" "$E2E_WORK/tools.txt"
  done
}
