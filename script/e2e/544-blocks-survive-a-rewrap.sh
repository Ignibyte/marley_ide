# shellcheck shell=bash
# #544's e2e test: blocks keep their rows when a resize rewraps the terminal. The terminal runs
# three commands, the middle one printing lines longer than a narrowed terminal is wide. Each
# block is read through Marley's MCP server. Then the Settings window opens, tiled beside the main
# window by sway, which narrows the terminal from about 120 columns to 30 so the long lines
# rewrap: the blocks read the same (REQ-001), and the shot shows each block's bar and pill on its
# rows (REQ-002). The Settings window closes, the terminal widens again, and the reads still match.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

COMMANDS=("seq 1 3" "long" "echo done")

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
long() {
  local n
  for n in 1 2 3 4 5; do
    printf 'long line %d:' "$n"
    printf ' word%02d' $(seq 1 14)
    printf '\n'
  done
}
RC
  terminal_env HOME "$home"
  git init -q -b rewrap "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

# Reads each command's block into $E2E_WORK/<label>.txt, through the stand-in agent.
read_all() {
  local command
  : >"$E2E_WORK/$1.txt"
  for command in "${COMMANDS[@]}"; do
    mcp_agent terminal-read "$command" >>"$E2E_WORK/$1.txt"
  done
  cat "$E2E_WORK/$1.txt"
}

# Whether two reads are the same text.
same() {
  cmp -s "$E2E_WORK/$1.txt" "$E2E_WORK/$2.txt" || {
    diff "$E2E_WORK/$1.txt" "$E2E_WORK/$2.txt" || true
    return 1
  }
}

steps() {
  local command
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  for command in "${COMMANDS[@]}"; do
    type_text "$command"
    press "" Return
    settle 1
  done
  settle 1
  shot 544-01-wide
  echo "== the reads at the full width"
  read_all wide
  echo "== the Settings window, tiled beside the main window, narrows the terminal"
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: open settings"
  settle 1
  press "" Return
  settle 4
  shot 544-02-narrowed
  read_all narrowed
  expect "the blocks read the same after the rewrap" same wide narrowed
  echo "== the Settings window closes, and the terminal widens again"
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 3
  shot 544-03-wide-again
  read_all wide-again
  expect "the blocks read the same at the width they started at" same wide wide-again
}
