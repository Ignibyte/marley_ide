# shellcheck shell=bash
# #635's visual check of two prompt editor fixes. A command sent from the shell's prompt editor
# (#627) rings no bell, so the terminal's tab carries no dirty dot (`635-01-two-terminals`); and
# Ctrl-Shift-W from the editor closes its terminal, with no "save all changes" dialog and the
# window kept (`635-02-closed`), where before it asked to save and then closed the window.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether Marley's MCP server lists `$1` terminals.
terminal_count() {
  mcp_agent terminals | tee "$E2E_WORK/terminals.txt"
  [[ $(grep -c "^  terminal " "$E2E_WORK/terminals.txt" || true) -eq $1 ]]
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b closes "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "echo one"
  press "" Return
  settle 2
  palette "workspace: new terminal"
  settle 4
  type_text "echo two"
  press "" Return
  settle 2
  shot 635-01-two-terminals
  expect "two terminals" terminal_count 2

  echo "== ctrl-shift-w from the prompt editor"
  press "CTRL SHIFT" w
  settle 2
  shot 635-02-closed
  expect "Marley still runs" test -n "$(marley_pid)"
  expect "the terminal closed and the other stayed" terminal_count 1
}
