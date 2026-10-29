# shellcheck shell=bash
# #947's visual check: an outside program's pairing card as the AgentAccess board draws it. Marley's
# bridge runs from the harness, outside every Marley terminal, as #811's does.
# - `marley-mcp-bridge pair laptop`: the card's title, "From <folder>. Check the code …", the
#   printed code in six boxes, the three grants with Read picked, Refuse and Pair, the two-minute
#   line (`947-01-local`, REQ-001, REQ-002, REQ-004).
# - Read and Write clicked: its dot filled (`947-02-picked`, REQ-003); Pair: the command prints
#   that `laptop` may read and write (REQ-003).
# - `pair laptop2` with `SSH_CONNECTION` set, as a bridge run over SSH: "… over SSH."
#   (`947-03-ssh`, REQ-002); Refuse.
# - Marley Day, `pair laptop3` (`947-04-day`, REQ-001); Refuse.
compositor sway

# Places on the card, as the first run's shots found them; `STOP_AT=1` ends the run after the
# first shot.
WRITE_X=${WRITE_X:-1250}
WRITE_Y=${WRITE_Y:-810}
PAIR_X=${PAIR_X:-1510}
PAIR_Y=${PAIR_Y:-929}
REFUSE_X=${REFUSE_X:-1447}
REFUSE_Y=${REFUSE_Y:-929}
STOP_AT=${STOP_AT:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  open_path "$repo"
}

away() {
  pointer_to 300 600
  settle 1
}

# Starts `pair $1` from the repository with the rest of the arguments as its environment, in the
# background, its output in `$E2E_WORK/pair-$1.txt`.
pair_as() {
  local name=$1
  shift
  (cd "$E2E_WORK/repo" &&
    env "$@" MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \
      python3 "$BRIDGE" pair "$name" >"$E2E_WORK/pair-$name.txt" 2>&1) &
  PAIRER=$!
  settle 3
  cat "$E2E_WORK/pair-$name.txt"
}

steps() {
  BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== laptop asks to pair, from this machine"
  pair_as laptop -u SSH_CONNECTION
  away
  shot 947-01-local
  expect "the command printed a code" grep -Eq "code [0-9]{3} [0-9]{3}" "$E2E_WORK/pair-laptop.txt"
  if [[ $STOP_AT == 1 ]]; then
    kill "$PAIRER" 2>/dev/null || true
    return
  fi
  echo "== Read and Write picked, then Pair"
  click "$WRITE_X" "$WRITE_Y"
  settle 1
  away
  shot 947-02-picked
  click "$PAIR_X" "$PAIR_Y"
  wait "$PAIRER" || echo "the pair command failed"
  cat "$E2E_WORK/pair-laptop.txt"
  expect "laptop is paired to read and write" \
    holds "$E2E_WORK/pair-laptop.txt" "Paired: laptop may read and write."
  echo "== laptop2 asks over SSH"
  pair_as laptop2 SSH_CONNECTION="127.0.0.1 50000 127.0.0.1 22"
  away
  shot 947-03-ssh
  click "$REFUSE_X" "$REFUSE_Y"
  wait "$PAIRER" || true
  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  pair_as laptop3 -u SSH_CONNECTION
  away
  shot 947-04-day
  click "$REFUSE_X" "$REFUSE_Y"
  wait "$PAIRER" || true
}
