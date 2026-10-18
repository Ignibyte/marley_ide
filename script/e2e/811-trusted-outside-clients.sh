# shellcheck shell=bash
# #811's visual check: a program outside Marley's terminals pairs before it acts. The scenario runs
# Marley's bridge from the harness, outside every Marley terminal, against a Python REPL in the
# repository's terminal, with `marley.agent_terminal_writes` at `never_ask`.
# - With the shared endpoint file's bearer alone, `terminal_list` answers and `terminal_type` is
#   refused `not_paired` (REQ-001).
# - `marley-mcp-bridge pair laptop`: the notification names `laptop` with the code the command
#   printed and four buttons (`811-01-prompt`, REQ-002). Read and Write: the command prints that
#   `laptop` may read and write, with its endpoint line (REQ-003).
# - As `laptop`, `terminal_type` is refused `not_granted` (REQ-004). Outside Clients lists `laptop`
#   with Write pressed (`811-02-clients`, REQ-003); Run there, and `laptop` types into the REPL,
#   whose bar names it (`811-03-run`, REQ-004, REQ-005).
# - Cut Off: `laptop`'s next call is refused (REQ-006). The stand-in on Marley's own endpoint still
#   types (REQ-007).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A point in the terminal, the notification's Read and Write button, and Outside Clients' Run and
# Cut Off on the first row, as the first run's shots found them.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
WRITE_X=${WRITE_X:-1250}
WRITE_Y=${WRITE_Y:-933}
RUN_X=${RUN_X:-998}
RUN_Y=${RUN_Y:-259}
CUT_X=${CUT_X:-1048}
CUT_Y=${CUT_Y:-259}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  profile_setting marley.agent_terminal_writes '"never_ask"'
  git init -q -b main "$repo"
  open_path "$repo"
}

# Runs `$1` from the command palette, the name, a pause and Return in one `wtype`.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

steps() {
  local shared=$E2E_PROFILE/mcp-endpoint.json laptop=$E2E_PROFILE/mcp/clients/laptop.json
  local bridge=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge pairer
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "python3 -q"
  press "" Return
  settle 3

  echo "== a program that has not paired reads, and nothing more"
  mcp_agent --endpoint "$shared" tool terminal_list | tee "$E2E_WORK/unpaired-read.txt"
  expect "an unpaired program reads" holds "$E2E_WORK/unpaired-read.txt" "terminal_list:"
  mcp_agent --endpoint "$shared" refusal terminal_type '{}' | tee "$E2E_WORK/unpaired-type.txt"
  expect "an unpaired program's write is refused" \
    holds "$E2E_WORK/unpaired-type.txt" "code not_paired" "marley-mcp-bridge pair NAME"

  echo "== the program pairs as laptop"
  MARLEY_MCP_ENDPOINT=$shared python3 "$bridge" pair laptop >"$E2E_WORK/pair.txt" 2>&1 &
  pairer=$!
  settle 3
  cat "$E2E_WORK/pair.txt"
  shot 811-01-prompt
  expect "the command printed a code" grep -Eq "code [0-9]{3} [0-9]{3}" "$E2E_WORK/pair.txt"
  click "$WRITE_X" "$WRITE_Y"
  wait "$pairer" || echo "the pair command failed"
  cat "$E2E_WORK/pair.txt"
  expect "laptop is paired to read and write" \
    holds "$E2E_WORK/pair.txt" "Paired: laptop may read and write." "clients/laptop.json"

  echo "== laptop writes, and does not run"
  mcp_agent --endpoint "$laptop" refusal terminal_type '{}' | tee "$E2E_WORK/laptop-type.txt"
  expect "laptop's run is refused" holds "$E2E_WORK/laptop-type.txt" "code not_granted"

  echo "== Outside Clients: laptop may run"
  palette "marley: outside clients"
  settle 2
  shot 811-02-clients
  click "$RUN_X" "$RUN_Y"
  settle 1
  press "" Escape
  settle 1
  mcp_agent --endpoint "$laptop" terminal-type python "print(6 * 7)" --submit |
    tee "$E2E_WORK/laptop-run.txt"
  expect "laptop types once it may run" holds "$E2E_WORK/laptop-run.txt" "bytes into"
  settle 2
  shot 811-03-run

  echo "== Cut Off"
  palette "marley: outside clients"
  settle 2
  click "$CUT_X" "$CUT_Y"
  settle 1
  shot 811-04-cut-off
  press "" Escape
  settle 1
  mcp_agent --endpoint "$laptop" tool terminal_list | tee "$E2E_WORK/laptop-cut.txt"
  expect "laptop is refused after Cut Off" holds "$E2E_WORK/laptop-cut.txt" "does not allow this client"

  echo "== Marley's own agent still types"
  mcp_agent terminal-type python "print('own')" --submit | tee "$E2E_WORK/own-type.txt"
  expect "Marley's own agent types" holds "$E2E_WORK/own-type.txt" "bytes into"
}
