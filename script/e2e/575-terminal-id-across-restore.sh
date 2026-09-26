# shellcheck shell=bash
# #575's e2e test: a terminal Marley restores keeps its MARLEY_TERMINAL_ID. Marley starts with a
# foreign, well-formed id under the restore's private key, which no terminal may take or show.
# The repository's terminal and a split of it each print their id (`ids`); after a quit and a
# launch, each restored terminal prints the id it had, and the stand-in agent, run in the right
# one through the plugin's bridge, sees it `self` under that id; after a second quit and launch,
# the ids hold again. A split of a restored terminal and a new terminal each get an id no other
# terminal has.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A point in the left and the right pane of the split, measured from the first run.
LEFT_X=500
LEFT_Y=500
RIGHT_X=1100
RIGHT_Y=500
# What Marley inherits under the private key, well formed, and must neither show nor take.
FOREIGN=00000000-0000-4000-8000-000000000575

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  write_mcp_agent
  cat >"$home/.bashrc" <<RC
PS1='\$ '
ids() {
  echo "\$1 id=\$MARLEY_TERMINAL_ID project=\$MARLEY_PROJECT restored=[\${MARLEY_RESTORED_TERMINAL_ID-unset}]" | tee -a "$E2E_WORK/ids.log"
}
agent() {
  BRIDGE="$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \\
    MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \\
    python3 "$E2E_WORK/mcp-agent.py" "\$@" 2>&1 | tee -a "$E2E_WORK/agent.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/ids.log"
  : >"$E2E_WORK/agent.log"
  git init -q -b restore "$E2E_WORK/repo"
  export MARLEY_RESTORED_TERMINAL_ID=$FOREIGN
  open_path "$E2E_WORK/repo"
}

# The id on the newest `ids` line of label `$1`.
id_of() {
  grep -E "^$1 id=" "$E2E_WORK/ids.log" | tail -1 | sed -E 's/^[^ ]* id=([^ ]*) .*/\1/' || true
}

# Whether label `$1` has an id of a UUID's shape, the same as label `$2`'s.
same_id() {
  local first second
  first=$(id_of "$1")
  second=$(id_of "$2")
  echo "$1 $first, $2 $second"
  [[ $first =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ && $first == "$second" ]]
}

# Whether label `$1`'s id is well formed and none of the other labels'.
new_id() {
  local own label
  own=$(id_of "$1")
  shift
  [[ $own =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] || return 1
  for label in "$@"; do
    [[ $own != "$(id_of "$label")" ]] || return 1
  done
}

# Whether every `ids` line shows the private key empty and no line holds the foreign id.
no_key_seen() {
  ! grep -qF "$FOREIGN" "$E2E_WORK/ids.log" && ! grep -v 'restored=\[\]' "$E2E_WORK/ids.log" | grep -q .
}

# Whether the agent's newest `terminals` answer marks the terminal of label `$1` self.
self_is() {
  local own
  own=$(id_of "$1")
  grep "(self)" "$E2E_WORK/agent.log" | tail -1 | grep -qF "id $own,"
}

# Types a command into the pane at x `$1`, y `$2`.
run_at() {
  click "$1" "$2"
  settle 1
  type_text "$3"
  press "" Return
  settle 2
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
  type_text "ids left-0"
  press "" Return
  settle 1
  palette "pane: split right"
  settle 4
  type_text "ids right-0"
  press "" Return
  settle 2
  shot 575-01-before
  cat "$E2E_WORK/ids.log"
  expect "the two terminals have ids of their own" new_id right-0 left-0
  echo "== a quit and a launch"
  quit_marley
  launch_marley
  settle 12
  run_at "$LEFT_X" "$LEFT_Y" "ids left-1"
  run_at "$RIGHT_X" "$RIGHT_Y" "ids right-1"
  run_at "$RIGHT_X" "$RIGHT_Y" "agent terminals"
  settle 2
  shot 575-02-restored
  cat "$E2E_WORK/ids.log"
  expect "the left terminal came back with its id" same_id left-0 left-1
  expect "the right terminal came back with its id" same_id right-0 right-1
  expect "the tools see the restored terminal self under its id" self_is right-0
  echo "== a second quit and launch"
  quit_marley
  launch_marley
  settle 12
  run_at "$LEFT_X" "$LEFT_Y" "ids left-2"
  run_at "$RIGHT_X" "$RIGHT_Y" "ids right-2"
  shot 575-03-restored-again
  expect "the left terminal's id held" same_id left-0 left-2
  expect "the right terminal's id held" same_id right-0 right-2
  echo "== a split of a restored terminal, and a new terminal"
  click "$LEFT_X" "$LEFT_Y"
  settle 1
  palette "pane: split right"
  settle 4
  type_text "ids split"
  press "" Return
  settle 2
  palette "workspace: new terminal"
  settle 4
  type_text "ids new"
  press "" Return
  settle 2
  shot 575-04-split-and-new
  cat "$E2E_WORK/ids.log"
  expect "the split got an id of its own" new_id split left-0 right-0
  expect "the new terminal got an id of its own" new_id new left-0 right-0 split
  expect "no terminal saw or took the private key's value" no_key_seen
}
