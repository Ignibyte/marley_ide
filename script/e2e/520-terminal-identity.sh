# shellcheck shell=bash
# #520's e2e test: each terminal knows its id, and Marley's tools know their caller. Marley starts
# with MARLEY_TERMINAL_ID and MARLEY_PROJECT already set to foreign values, which its terminals
# must not pass on. In the repository's terminal, `ids` prints both variables (and logs them); a
# split gets an id of its own; the stand-in agent, run inside the split through the plugin's
# bridge, sees its own terminal marked `self` and lists that terminal's blocks without naming it;
# a task, spawned from the palette, sees neither variable. From the harness's own shell, which
# is no terminal of Marley's, nothing is `self` and a call that names no terminal is refused with
# its next step.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home"
  write_mcp_agent
  cat >"$home/.bashrc" <<RC
PS1='\$ '
ids() { echo "id=\$MARLEY_TERMINAL_ID project=\$MARLEY_PROJECT" | tee -a "$E2E_WORK/ids.log"; }
agent() {
  BRIDGE="$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \\
    MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \\
    python3 "$E2E_WORK/mcp-agent.py" "\$@" | tee -a "$E2E_WORK/agent.log"
}
RC
  terminal_env HOME "$home"
  git init -q -b identity "$repo"
  mkdir -p "$repo/.zed"
  cat >"$repo/.zed/show-ids.sh" <<SH
echo "task-id=\${MARLEY_TERMINAL_ID:-unset} task-project=\${MARLEY_PROJECT:-unset}" | tee -a "$E2E_WORK/task.log"
SH
  echo '[{"label": "show ids", "command": "sh .zed/show-ids.sh"}]' >"$repo/.zed/tasks.json"
  # What Marley inherits and must not hand its terminals.
  export MARLEY_TERMINAL_ID=inherited MARLEY_PROJECT=/nowhere
  open_path "$repo"
}

# Whether the two `ids` lines hold two different UUIDs and the repository's folder, and neither
# inherited value.
own_ids() {
  cat "$E2E_WORK/ids.log"
  python3 - "$E2E_WORK/ids.log" "$E2E_WORK/repo" <<'PY'
import re
import sys

lines = open(sys.argv[1]).read().splitlines()
shape = r"id=([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}) project=(.*)"
found = [re.fullmatch(shape, line) for line in lines]
ok = (len(found) == 2 and all(found)
      and found[0].group(1) != found[1].group(1)
      and all(match.group(2) == sys.argv[2] for match in found))
sys.exit(0 if ok else 1)
PY
}

# Whether the agent's `terminals` marks the split, and only it, `self`, under the split's id.
split_is_self() {
  local split_id
  split_id=$(sed -n 2p "$E2E_WORK/ids.log" | sed -E 's/^id=([^ ]*) .*/\1/')
  [[ $(grep -c "(self)" "$E2E_WORK/agent.log") -eq 1 ]] &&
    grep "(self)" "$E2E_WORK/agent.log" | grep -qF "id $split_id,"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "ids"
  press "" Return
  settle 1
  palette "pane: split right"
  settle 4
  type_text "ids"
  press "" Return
  settle 1
  shot 520-01-identity
  expect "each terminal has its own id and the project, none inherited" own_ids
  type_text "agent terminals"
  press "" Return
  settle 3
  shot 520-02-self
  expect "the agent's own terminal, and only it, is self" split_is_self
  type_text "agent blocks-here"
  press "" Return
  settle 3
  shot 520-03-own-blocks
  expect "a call that names no terminal reads the caller's" \
    holds "$E2E_WORK/agent.log" "'ids'" "'agent terminals'"
  palette "task: spawn"
  settle 2
  type_text "show ids"
  settle 1
  press "" Return
  settle 4
  shot 520-04-task
  expect "a task sees neither variable" \
    holds "$E2E_WORK/task.log" "task-id=unset task-project=unset"
  echo "== from outside Marley's terminals"
  # Blank for the call: the bridge sends no id and no project it cannot use.
  MARLEY_TERMINAL_ID='' MARLEY_PROJECT='' mcp_agent terminals | tee "$E2E_WORK/outside.txt"
  expect "nothing is self for a caller outside" bash -c "! grep -q '(self)' '$E2E_WORK/outside.txt'"
  MARLEY_TERMINAL_ID='' MARLEY_PROJECT='' mcp_agent blocks-here | tee "$E2E_WORK/outside-blocks.txt"
  expect "naming no terminal from outside is refused with its next step" \
    holds "$E2E_WORK/outside-blocks.txt" "terminal_blocks refused" "terminal_list"
}
