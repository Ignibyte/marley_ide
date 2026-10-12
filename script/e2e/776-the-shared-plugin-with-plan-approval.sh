# shellcheck shell=bash
# #776's visual check: the shared Claude Code plugin at rustal-harness TICKET-119 (0.4.0) hands a
# real Claude Code's plan to the Plan tab. `marley.claude_code_shared_plugin` and
# `marley.plan_review` are on. In a new terminal, the harness's pinned Claude Code (2.1.287) runs in
# plan mode against the harness's scripted Messages API (`scripts/fixtures/claude_api.py`), in a
# user and network namespace of its own, as the harness's own suite runs it: no model and no key. It
# keeps Marley's variables and `CLAUDE_CODE_PLUGIN_DIRS`, and a scratch config that has finished
# onboarding. Typed markers pick the model's replies: `PLAN_WRITE` writes the plan file the
# plan-mode reminder names, with one `:::question`; `PLAN_SUBMIT` calls `ExitPlanMode`;
# `PLAN_REWRITE` writes the plan with the question settled.
#
# Shots: the plan in the Plan tab with its question as a card (`776-01-plan`, REQ-002); after Accept
# recommended and Send Answers, the terminal: the call refused with the answers, still in plan mode
# (`776-02-answers`, REQ-003); the revised plan approved, Claude Code out of plan mode
# (`776-03-approved`, REQ-004), after the revised plan with its question settled
# (`776-03a-revised`). The folder's digest and the skill's listing to the model are read
# from the run's files (REQ-001).
compositor sway

HARNESS=${HARNESS:-/srv/stacks/rustal-harness}
API=$HARNESS/scripts/fixtures/claude_api.py
DIGEST=b3b5ee99611a175712af94d53dabd907e9e34056380839e2f6c7476fede7e26a
# Points from the first run's shots; 0 stops after the shot before the click.
ACCEPT_X=${ACCEPT_X:-1083}
ACCEPT_Y=${ACCEPT_Y:-195}
SEND_X=${SEND_X:-1259}
SEND_Y=${SEND_Y:-301}
APPROVE_X=${APPROVE_X:-1259}
APPROVE_Y=${APPROVE_Y:-301}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-607}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
AWAY_X=${AWAY_X:-1700}
AWAY_Y=${AWAY_Y:-1000}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# The pinned Claude Code the harness's suite runs, from its record.
pinned_claude() {
  python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["binary"]["observed_path"])' \
    "$HARNESS/dependencies/claude-code.json"
}

# The model's replies, by the marker in the last user message (the harness's rules, TICKET-119).
write_rules() {
  python3 - "$1" <<'PY'
import json
import sys

plan = ('# Plan\n\nKeep the run records in the journal.\n\n:::question\nWhich store keeps the run records?\n'
        'The runtime keeps its journal in SQLite now.\n- [ ] SQLite — the runtime has it already\n'
        '- [ ] Postgres — each host needs a server for it\nRecommended: SQLite\n:::\n')
revised = plan.replace('- [ ] SQLite', '- [x] SQLite')
plan_file = r'(?:create your plan at|already exists at) (\S+?\.md)'


def write(text):
    return {'tool_use': {'name': 'Write', 'input': {'file_path': '{captured}', 'content': text}}}


rules = [
    {'watch': 'Use in plan mode before you call ExitPlanMode'},
    {'watch': 'User has approved your plan.'},
    {'watch': 'Answers to your questions'},
    {'marker': 'PLAN_WRITE', 'capture': plan_file, 'reply': [write(plan)]},
    {'marker': 'PLAN_REWRITE', 'capture': plan_file, 'reply': [write(revised)]},
    {'marker': 'PLAN_SUBMIT', 'reply': [{'tool_use': {'name': 'ExitPlanMode', 'input': {}}}]},
    {'after_tool': True, 'reply': [{'text': 'Tool turn done.'}]},
]
json.dump(rules, open(sys.argv[1], 'w'))
PY
}

setup() {
  local bin=$E2E_WORK/bin repo=$E2E_WORK/repo session=$E2E_WORK/session claude key
  claude=$(pinned_claude)
  [[ -x $claude && -f $API ]] || {
    echo "no pinned Claude Code or scripted API: set up rustal-harness's Claude lane first" >&2
    return 1
  }
  mkdir -p "$bin" "$repo" "$E2E_WORK/home" "$session/home" "$session/tmp" "$session/config"
  chmod 0700 "$session" "$session/home" "$session/tmp" "$session/config"
  # The fixture's placeholder, which the scripted API takes and no service would.
  key=$(printf '%s-%s-%s' sk ant fixture-not-a-key)
  python3 - "$session/config/.claude.json" "$key" "$repo" <<'PY'
import json
import os
import sys

path, key, repo = sys.argv[1:4]
settings = {'hasCompletedOnboarding': True,
            'customApiKeyResponses': {'approved': [key[-20:]], 'rejected': []},
            'projects': {repo: {'hasTrustDialogAccepted': True}}}
with open(os.open(path, os.O_WRONLY | os.O_CREAT, 0o600), 'w') as file:
    json.dump(settings, file)
PY
  write_rules "$E2E_WORK/api-script.json"
  : >"$E2E_WORK/api.jsonl"
  # Inside the namespace: Claude Code with a cleared environment but for Marley's variables, the
  # plugin folders and the runtime directory, against the API the launcher names.
  cat >"$bin/plan-claude-inner" <<SH
#!/bin/bash
keep=()
for name in \$(compgen -e); do
  case \$name in MARLEY_* | CLAUDE_CODE_PLUGIN_DIRS | XDG_RUNTIME_DIR) keep+=("\$name=\${!name}") ;; esac
done
exec /usr/bin/env -i PATH=/usr/bin:/bin LANG=C.UTF-8 TERM="\${TERM:-xterm-256color}" \\
  HOME="$session/home" TMPDIR="$session/tmp" CLAUDE_CONFIG_DIR="$session/config" \\
  ANTHROPIC_BASE_URL="\$RH_TEST_CLAUDE_API" ANTHROPIC_API_KEY="$key" \\
  CLAUDE_CODE_DISABLE_AUTO_MEMORY=1 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \\
  DISABLE_AUTOUPDATER=1 DISABLE_UPDATES=1 "\${keep[@]}" \\
  "$claude" --setting-sources '' --permission-mode plan --strict-mcp-config
SH
  cat >"$bin/plan-claude" <<SH
#!/bin/bash
exec /usr/bin/unshare --user --map-root-user --net /usr/bin/python3 -B "$API" \\
  --script "$E2E_WORK/api-script.json" --trace "$E2E_WORK/api.jsonl" -- "$bin/plan-claude-inner"
SH
  chmod +x "$bin/plan-claude" "$bin/plan-claude-inner"
  printf 'PS1=%q\nexport PATH=%q:%s\n' '$ ' "$bin" "\$PATH" >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  # The Claude Code whose version Marley checks against the plugin's tested range.
  export MARLEY_CLAUDE=$claude
  profile_setting marley.claude_code_shared_plugin true
  profile_setting marley.plan_review true
  printf '# repo\n' >"$repo/README.md"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# Types a marker at Claude Code's prompt and sends it.
say() {
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-6}"
}

# Whether the scripted API's record saw `$1` in a request.
watched() {
  python3 - "$E2E_WORK/api.jsonl" "$1" <<'PY'
import json
import sys

path, text = sys.argv[1:3]
records = [json.loads(line) for line in open(path) if line.strip()]
sys.exit(0 if any(text in record.get('watched', []) for record in records) else 1)
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the shared plugin's folder, at the harness's digest"
  find "$E2E_PROFILE" -maxdepth 6 -type d -path "*claude-code/shared/$DIGEST" >"$E2E_WORK/folder.txt"
  cat "$E2E_WORK/folder.txt"
  expect "the plugin's folder is named by the harness's digest" [ -s "$E2E_WORK/folder.txt" ]

  echo "== Claude Code in plan mode in a new terminal"
  palette "workspace: new center terminal"
  settle 2
  type_text "plan-claude"
  press "" Return
  settle 15
  say PLAN_WRITE 10
  say PLAN_SUBMIT 10
  pointer_to "$AWAY_X" "$AWAY_Y"
  take 776-01-plan || return 0
  expect "the model was told of the plan-questions skill" watched 'Use in plan mode before you call ExitPlanMode'
  if ((ACCEPT_X == 0)); then
    echo "the card's Accept is not placed yet: 776-01 is the last shot"
    return 0
  fi

  echo "== Accept recommended, Send Answers: the call refused with the answers"
  click "$ACCEPT_X" "$ACCEPT_Y"
  settle 1
  click "$SEND_X" "$SEND_Y"
  settle 8
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  # The tab's tooltip goes once the pointer leaves it.
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  take 776-02-answers || return 0
  expect "the model read the answers in the refusal" watched 'Answers to your questions'
  if ((APPROVE_X == 0)); then
    echo "Approve is not placed yet: 776-02 is the last shot"
    return 0
  fi

  echo "== the revised plan, approved"
  say PLAN_REWRITE 10
  say PLAN_SUBMIT 10
  pointer_to "$AWAY_X" "$AWAY_Y"
  take 776-03a-revised || return 0
  click "$APPROVE_X" "$APPROVE_Y"
  settle 8
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  # The tab's tooltip goes once the pointer leaves it.
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  take 776-03-approved || return 0
  expect "Claude Code told the model the plan was approved" watched 'User has approved your plan.'
}
