# shellcheck shell=bash
# #649's visual check: rich input through the agent's own editor key. With
# `marley.agent_editor_in_tab` on, a terminal Marley opens for an agent gets `marley-edit` as
# `VISUAL` and `EDITOR`, over the `.bashrc`'s own exports, and Ctrl-G there sends the agent its key.
# A stand-in `claude`, a Python script first on the terminal's PATH and named in `MARLEY_CLAUDE`,
# does what Claude Code does on its editor key and nothing else: it prints `VISUAL` and `EDITOR`,
# reads its terminal a byte at a time and logs each byte in hex to `keys.log`; Ctrl-G writes its
# draft to a `claude-prompt-<n>.md`, runs `$VISUAL` split on spaces with the file, prints how the
# editor exited and what it read back, and keeps that as its draft; Enter prints `submitted:` and
# the draft. It mirrors what it prints to `stand-in.log`, and never reaches a model. The run's
# keymap binds Ctrl+Alt+Shift+K to the setting's place in the Settings window.
#
# `649-01-agent-env`, `649-02-tab-open`, `649-03-saved`, `649-04-read-back`, `649-05-submitted`,
# `649-06-button`, `649-07-plain-env`, `649-08-hand-run-overlay`, `649-09-refused`, `649-10-off`,
# `649-11-setting`.
compositor sway

# The agent bar's Rich Input button, from the shots of #648's runs.
RICH_X=${RICH_X:-401}
RICH_Y=${RICH_Y:-954}

log_holds() { grep -qF -- "$1" "$E2E_WORK/stand-in.log"; }
log_lacks() { ! grep -qF -- "$1" "$E2E_WORK/stand-in.log"; }

# How many times the stand-in read byte `$1` (hex) from its terminal.
key_count() { grep -cx -- "$1" "$E2E_WORK/keys.log" 2>/dev/null || true; }

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# A Claude Code from the New Agent picker.
picker_claude() {
  press "CTRL ALT" n
  settle 2
  type_text "Claude"
  settle 1
  press "" Return
  settle 6
}

close_settings() {
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/tmp"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
export VISUAL=user-visual EDITOR=user-editor
RC
  terminal_env HOME "$home"
  sed -e "s|@WORK@|$E2E_WORK|g" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: its editor key, its draft, and Enter, as Claude Code handles them.
import os
import subprocess
import sys
import termios
import tty

if sys.argv[1:2] in (["plugin"], ["--version"]):
    sys.exit(0)
WORK = "@WORK@"


def say(line):
    sys.stdout.write(line + "\r\n")
    sys.stdout.flush()
    with open(f"{WORK}/stand-in.log", "a", encoding="utf-8") as log:
        log.write(line + "\n")


say(f"VISUAL={os.environ.get('VISUAL', '')}")
say(f"EDITOR={os.environ.get('EDITOR', '')}")
draft = "from the agent"
say(f"draft: {draft}")
fd = sys.stdin.fileno()
cooked = termios.tcgetattr(fd)
tty.setraw(fd)
count = 0
try:
    while True:
        byte = os.read(fd, 1)
        if not byte:
            break
        with open(f"{WORK}/keys.log", "a", encoding="utf-8") as keys:
            keys.write(byte.hex() + "\n")
        if byte == b"\x07":
            count += 1
            path = f"{WORK}/tmp/claude-prompt-{count}.md"
            with open(path, "w", encoding="utf-8") as prompt:
                prompt.write(draft + "\n")
            editor = os.environ.get("VISUAL") or os.environ.get("EDITOR") or "vi"
            termios.tcsetattr(fd, termios.TCSADRAIN, cooked)
            status = subprocess.run(editor.split(" ") + [path], check=False).returncode
            tty.setraw(fd)
            say(f"editor exited {status}")
            if status == 0:
                with open(path, encoding="utf-8") as prompt:
                    draft = prompt.read().removesuffix("\n")
            say("read back: " + draft.replace("\n", " / "))
        elif byte in (b"\r", b"\n"):
            say("submitted: " + draft.replace("\n", " / "))
        elif byte in (b"\x03", b"\x04"):
            break
finally:
    termios.tcsetattr(fd, termios.TCSADRAIN, cooked)
FAKE
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  profile_setting marley.agent_editor_in_tab true
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-k": ["zed::OpenSettingsAt", {"path": "marley.agent_editor_in_tab"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  printf 'the note\n' >"$E2E_WORK/note.md"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== an agent terminal's editor"
  picker_claude
  shot 649-01-agent-env
  expect "the agent's VISUAL is Marley's editor" log_holds "VISUAL=$E2E_PROFILE/mcp/marley-edit"
  expect "the agent's EDITOR is Marley's editor" log_holds "EDITOR=$E2E_PROFILE/mcp/marley-edit"

  echo "== Ctrl-G"
  press "CTRL" g
  settle 3
  shot 649-02-tab-open
  expect "the agent got its key" test "$(key_count 07)" = 1

  echo "== a save"
  press "CTRL" a
  type_text "edited in Marley"
  press "" Return
  type_text "second line"
  press "CTRL" s
  settle 2
  shot 649-03-saved
  expect "the save did not end the edit" log_lacks "editor exited"

  echo "== the tab closed"
  press "CTRL" w
  settle 3
  shot 649-04-read-back
  expect "the editor exited 0" log_holds "editor exited 0"
  expect "the agent read both lines back" log_holds "read back: edited in Marley / second line"
  expect "nothing was submitted" log_lacks "submitted:"
  expect "Marley sent no other key" test "$(wc -l <"$E2E_WORK/keys.log")" = 1

  echo "== Enter, the terminal in front"
  press "" Return
  settle 2
  shot 649-05-submitted
  expect "the edited draft was submitted" log_holds "submitted: edited in Marley / second line"

  echo "== the bar's button"
  click "$RICH_X" "$RICH_Y"
  settle 3
  shot 649-06-button
  expect "the button sent the key" test "$(key_count 07)" = 2
  press "CTRL" w
  settle 3

  echo "== a New Terminal"
  palette "workspace: new terminal"
  settle 3
  type_text "echo \"\$VISUAL \$EDITOR\""
  press "" Return
  settle 2
  shot 649-07-plain-env

  echo "== the agent run by hand"
  type_text "claude"
  press "" Return
  settle 3
  press "CTRL" g
  settle 2
  shot 649-08-hand-run-overlay
  expect "Ctrl-G there reached no agent" test "$(key_count 07)" = 2
  press "" Escape
  settle 1
  press "CTRL" c
  settle 2

  echo "== refused"
  type_text "MARLEY_TERMINAL_ID= $E2E_PROFILE/mcp/marley-edit $E2E_WORK/note.md; echo \"exit \$?\""
  press "" Return
  settle 4
  shot 649-09-refused
  expect "the note is as it was" holds "$E2E_WORK/note.md" "the note"

  echo "== the switch off"
  profile_setting marley.agent_editor_in_tab false
  settle 3
  picker_claude
  press "CTRL" g
  settle 2
  shot 649-10-off
  expect "the new agent kept the user's editor" log_holds "VISUAL=user-visual"
  press "" Escape
  settle 1

  echo "== the setting"
  press "CTRL ALT SHIFT" k
  settle 4
  shot 649-11-setting
  close_settings
}
