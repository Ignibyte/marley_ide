# shellcheck shell=bash
# #639's e2e test: Marley's blocks follow Zed's screen clears, and Zed's startup check is no block.
# In bash with a one-row prompt, `echo one` and `echo two`, then Zed's Clear (Escape gives the keys
# back to the shell, Ctrl-Shift-L clears): no header stays over the cleared rows; `echo three`
# then is the one block. Claude Code from the New Agent picker, a stand-in first on the PATH, opens
# a terminal whose first block is `claude` with the stand-in's line, and none for Zed's
# `printf … __zed_init_command_ready_ …` check.
compositor sway

GRID_X=800
GRID_Y=500

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$bin/claude" <<'STAND_IN'
#!/usr/bin/env python3
# A stand-in Claude Code: one line, then it waits.
import sys

print("the stand-in Claude Code runs", flush=True)
for line in sys.stdin:
    pass
STAND_IN
  chmod +x "$bin/claude"
  git init -q -b main "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click "$GRID_X" "$GRID_Y"
  settle 1.5

  echo "== Zed's Clear"
  type_text "echo one"
  press "" Return
  settle 1.5
  type_text "echo two"
  press "" Return
  settle 1.5
  # The prompt editor gives the keys back to the shell, where Ctrl-Shift-L is the terminal's Clear.
  press "" Escape
  settle 1
  press "CTRL SHIFT" l
  settle 2
  shot 639-01-cleared
  click "$GRID_X" "$GRID_Y"
  settle 1
  type_text "echo three"
  press "" Return
  settle 2
  shot 639-02-after-clear

  echo "== Claude Code from the New Agent picker"
  press "CTRL ALT" n
  settle 2
  type_text "Claude Code"
  settle 1
  press "" Return
  settle 8
  shot 639-03-agent
}
