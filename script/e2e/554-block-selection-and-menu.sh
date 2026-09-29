# shellcheck shell=bash
# #554's visual check: a selected block and the block menu. Three commands at a plain bash
# prompt; `ctrl-up` outlines the newest block, `up` moves the outline, Escape and a typed key end
# it (REQ-001 to REQ-003). A right-click on a block selects it and shows the Block section of
# Zed's terminal menu (REQ-004); its copies put the command, both, or the Markdown form on the
# headless sway's clipboard (REQ-005); Reinput and Reinput with sudo type the command at the
# prompt, unrun (REQ-006), as `ctrl-shift-i` does for the selected block (REQ-008); a block whose
# frame did not carry the terminal's nonce offers neither (REQ-007). The menu is driven by its
# keys from its end: Reinput with sudo is last, Copy Command five above it.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The rows of `$ echo two` and `$ false`, and the unverified block's output, from the shots; the
# content sits on the terminal's bottom edge.
ECHO_TWO_Y=${ECHO_TWO_Y:-896}
FALSE_Y=${FALSE_Y:-935}
FAKE_Y=${FAKE_Y:-935}
ROW_X=${ROW_X:-420}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  open_path "$repo"
}

# The headless sway's clipboard, which Marley's copies write.
clipboard() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline 2>/dev/null
}

# Right-clicks the row at `$1` and chooses the menu's item `$2` rows above its last.
menu_item() {
  local steps
  click "$ROW_X" "$1" right
  settle 1
  press "" End
  for ((steps = 0; steps < $2; steps++)); do
    press "" Up
  done
  press "" Return
  settle 1
}

# Whether the prompt's row shows `$1`.
prompt_shows() {
  mcp_agent terminal-screen repo >"$E2E_WORK/prompt.txt"
  tail -1 "$E2E_WORK/prompt.txt"
  tail -1 "$E2E_WORK/prompt.txt" | grep -qE "^  \| \\$ $1 *\$"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  local command
  for command in "echo one" "echo two" "false"; do
    type_text "$command"
    press "" Return
    settle 1
  done
  settle 1

  echo "== ctrl-up selects the newest block, up moves, Escape and typing end it"
  press "CTRL" Up
  settle 1
  shot 554-01-selected
  press "" Up
  settle 1
  shot 554-02-moved
  press "" Escape
  settle 1
  shot 554-03-cleared
  press "CTRL" Up
  settle 1
  type_text "x"
  settle 1
  shot 554-04-typed-clears
  press "CTRL" u
  settle 1

  echo "== the block menu"
  click "$ROW_X" "$ECHO_TWO_Y" right
  settle 1
  shot 554-05-menu
  press "" Escape
  settle 1
  menu_item "$ECHO_TWO_Y" 5
  clipboard | tee "$E2E_WORK/copy-command.txt"
  echo
  expect "Copy Command copied the command" test "$(cat "$E2E_WORK/copy-command.txt")" = "echo two"
  menu_item "$ECHO_TWO_Y" 3
  clipboard | tee "$E2E_WORK/copy-both.txt"
  echo
  expect "Copy Both copied the command and the output" holds "$E2E_WORK/copy-both.txt" "echo two" "two"
  menu_item "$ECHO_TWO_Y" 2
  clipboard | tee "$E2E_WORK/copy-markdown.txt"
  echo
  expect "Copy as Markdown copied the fence" holds "$E2E_WORK/copy-markdown.txt" '```' "\$ echo two" "exit 0"
  shot 554-06-copied

  echo "== Reinput and Reinput with sudo"
  menu_item "$FALSE_Y" 1
  shot 554-09-reinput
  expect "Reinput typed false at the prompt" prompt_shows false
  press "CTRL" u
  settle 1
  menu_item "$FALSE_Y" 0
  shot 554-10-reinput-sudo
  expect "Reinput with sudo typed sudo false" prompt_shows "sudo false"
  press "CTRL" u
  settle 1
  press "" Escape
  settle 1

  echo "== ctrl-shift-i reinputs the selected block"
  press "CTRL" Up
  settle 1
  press "CTRL SHIFT" i
  settle 1
  shot 554-12-reinput-key
  expect "ctrl-shift-i typed false" prompt_shows false
  press "CTRL" u
  settle 1

  echo "== a block whose frame carried no nonce offers no Reinput"
  type_text "printf '\\033Pppreexec;command=fake cmd\\033\\\\'; echo fake-output"
  press "" Return
  settle 2
  shot 554-11a-unverified-block
  click "$ROW_X" "$FAKE_Y" right
  settle 1
  shot 554-11-unverified
  press "" Escape
}
