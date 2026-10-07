# shellcheck shell=bash
# #681's visual check: the docs, settings and actions tools an agent reaches through Marley's MCP
# server. A stand-in agent calls each through the Claude Code plugin's bridge, as Claude Code in a
# Marley terminal does, and the scenario checks the answers: `docs_search` finds Zed's docs and
# Marley's guide (REQ-001, REQ-002), `docs_read` reads a section with its key placeholder filled
# (REQ-003) and refuses a page it lacks (REQ-004), `settings_schema` describes a setting
# (REQ-005) and refuses a key it lacks (REQ-006), `settings_read` gives each file's value, the
# project's first (REQ-007), with a secret hidden (REQ-008), and `actions_list` gives an action's
# keys (REQ-009). Every answer stays under 40,000 bytes (REQ-010), and `initialize` names the
# tools (REQ-011). It runs in a headless sway of its own, so a Marley open on the desktop stays out
# of it.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/.zed"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  git init -q -b main "$repo"
  printf '{ "tab_size": 3 }\n' >"$repo/.zed/settings.json"
  # A user value for a setting the schema describes, and a custom agent server, never started,
  # whose environment holds a fake token put together here.
  profile_setting marley.rail_order '"attention"'
  FAKE_TOKEN="e2e-fake-$(date +%s)-value"
  export FAKE_TOKEN
  profile_setting agent_servers.e2e-fake "{\"type\": \"custom\", \"command\": \"true\", \"env\": {\"API_TOKEN\": \"$FAKE_TOKEN\"}}"
  open_path "$repo"
}

# `answer <file> <python expression over a>`: whether the expression holds for the structured
# answer the stand-in printed in the file (its line `  <tool>: <json>`).
answer() {
  python3 - "$1" "$2" <<'PY'
import json, sys
path, expression = sys.argv[1:]
for line in open(path):
    name, _, rest = line.strip().partition(": ")
    if rest.startswith("{") and not name.endswith("refused"):
        a = json.loads(rest)
        ok = bool(eval(expression, {}, {"a": a}))
        print(f"  {expression}: {ok}")
        sys.exit(0 if ok else 1)
print("  no answer in", path)
sys.exit(1)
PY
}

# `size_under <file> <max>`: whether every answer size the stand-in printed is under max bytes.
size_under() {
  local size found=0
  while read -r size; do
    found=1
    ((size < $2)) || return 1
  done < <(grep -oP "answer size: \K[0-9]+" "$1")
  ((found))
}

call() {
  local out=$E2E_WORK/$1.txt
  shift
  mcp_agent tool "$@" | tee "$out" | cut -c1-400
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the instructions name the tools"
  mcp_agent instructions | tee "$E2E_WORK/instructions.txt" | head -3
  expect "they name the five tools" holds "$E2E_WORK/instructions.txt" "docs_search" "docs_read" \
    "settings_schema" "settings_read" "actions_list" "named but not listed: none"
  expect "they stay under 2,048 bytes" answer_size_ok "$E2E_WORK/instructions.txt"

  echo "== docs_search"
  call search-font docs_search '{"query": "terminal font size"}'
  expect "at most 10 sections, a Zed page in the first three" answer "$E2E_WORK/search-font.txt" \
    "len(a['results']) <= 10 and any(r['page'].startswith('zed/') for r in a['results'][:3])"
  call search-rail docs_search '{"query": "rail filter"}'
  expect "the guide in the first three" answer "$E2E_WORK/search-rail.txt" \
    "any(r['page'] == 'marley/guide.md' for r in a['results'][:3])"

  echo "== docs_read"
  call read-section docs_read '{"page": "zed/configuring-zed.md", "heading": "Settings Editor"}'
  expect "the section, its placeholders filled" answer "$E2E_WORK/read-section.txt" \
    "'Settings Editor' in a['text'] and '{#kb' not in a['text'] and '{#action' not in a['text'] and a['first_line'] == 1 and a['next'] is None"
  call read-missing docs_read '{"page": "zed/nope.md"}'
  expect "a page it lacks is no_doc" holds "$E2E_WORK/read-missing.txt" 'no_doc' "docs_search"

  echo "== settings_schema"
  call schema-rail settings_schema '{"key": "marley.rail_order"}'
  expect "rail_order's values and default" answer "$E2E_WORK/schema-rail.txt" \
    "a['default'] == 'window' and 'attention' in str(a['values']) and 'window' in str(a['values']) and a['description']"
  call schema-missing settings_schema '{"key": "terminal.no_such_key"}'
  expect "a key it lacks is no_setting with terminal's keys" holds "$E2E_WORK/schema-missing.txt" \
    'no_setting' "font_size"

  echo "== settings_read"
  call read-tab settings_read '{"key": "tab_size"}'
  expect "the project's value first, and it wins" answer "$E2E_WORK/read-tab.txt" \
    "a['values'][0]['file'].startswith('project') and a['values'][0]['value'] == 3 and a['set_in'] == a['values'][0]['file'] and a['effective'] == 3 and a['values'][-1] == {'file': 'default', 'value': 4}"
  call read-rail settings_read '{"key": "marley.rail_order"}'
  expect "the user's value over the default" answer "$E2E_WORK/read-rail.txt" \
    "a['effective'] == 'attention' and a['values'][0]['file'].startswith('user') and a['values'][-1]['value'] == 'window'"
  call read-secret settings_read '{"key": "agent_servers.e2e-fake"}'
  expect "the token is hidden" holds "$E2E_WORK/read-secret.txt" "[redacted: setting]"
  expect "and appears nowhere" bash -c "! grep -qF '$FAKE_TOKEN' '$E2E_WORK/read-secret.txt'"
  call read-secret-key settings_read '{"key": "agent_servers.e2e-fake.env.API_TOKEN"}'
  expect "read by its own key, too" bash -c "! grep -qF '$FAKE_TOKEN' '$E2E_WORK/read-secret-key.txt'"

  echo "== actions_list"
  call actions-save actions_list '{"query": "save"}'
  expect "workspace::Save with its keys" answer "$E2E_WORK/actions-save.txt" \
    "any(x['name'] == 'workspace::Save' and x['palette'] == 'workspace: save' and any('s' in k['keystrokes'].lower() and 'ctrl' in k['keystrokes'].lower() for k in x['keys']) for x in a['actions'][:3])"

  echo "== every answer fits"
  cat "$E2E_WORK"/search-*.txt "$E2E_WORK"/read-*.txt "$E2E_WORK"/schema-*.txt \
    "$E2E_WORK"/actions-*.txt >"$E2E_WORK/all.txt"
  grep "answer size" "$E2E_WORK/all.txt" | sort -t: -k2 -n | tail -3
  expect "all under 40,000 bytes" size_under "$E2E_WORK/all.txt" 40000
  settle 1
  shot 681-01-after
}

# Whether the instructions the stand-in printed are at most 2,048 bytes.
answer_size_ok() {
  local size
  size=$(grep -oP "instructions: \K[0-9]+" "$1") || return 1
  ((size > 0 && size <= 2048))
}
